use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, State};
use walkdir::{DirEntry, WalkDir};

use crate::commands::off_thread;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_packs_changed;
use crate::fsutil::safe_join;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeEntry {
    pub path: String,
    pub name: String,
    pub directory: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorFileStat {
    pub file_type: u8,
    pub size: u64,
    pub ctime: u64,
    pub mtime: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDirectoryEntry {
    pub name: String,
    pub file_type: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDocument {
    pub content: String,
    pub modified_ms: u64,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMatch {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub preview: String,
}

fn document_at(path: &std::path::Path) -> CommandResult<EditorDocument> {
    let bytes = fs::read(path)?;
    let content = String::from_utf8(bytes.clone()).map_err(|_| {
        SerializableError::new(
            "binary_file",
            format!("{} is binary and cannot be opened as text", path.display()),
        )
    })?;
    let metadata = fs::metadata(path)?;
    Ok(EditorDocument {
        content,
        modified_ms: timestamp(metadata.modified()),
        size: metadata.len(),
        hash: format!("{:x}", Sha256::digest(&bytes)),
    })
}

#[tauri::command]
pub async fn editor_document_read(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<EditorDocument> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || document_at(&safe_join(&root, &path)?)).await
}

#[tauri::command]
pub async fn editor_document_write(
    id: String,
    path: String,
    content: String,
    expected_hash: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<EditorDocument> {
    let root = pack_root(&state.workspace()?, &id)?;
    let document = off_thread(move || {
        let target = safe_join(&root, &path)?;
        let current = document_at(&target)?;
        if current.hash != expected_hash {
            return Err(SerializableError::new(
                "file_conflict",
                format!("{} changed outside Packwand", target.display()),
            ));
        }
        crate::fsutil::atomic_write(&target, content.as_bytes())?;
        document_at(&target)
    })
    .await?;
    // Emitted after the write lands, on the caller's task, so the event
    // ordering the frontend sees is unchanged by the thread hop.
    emit_packs_changed(&app)?;
    Ok(document)
}

#[tauri::command]
pub async fn editor_search(
    id: String,
    query: String,
    case_sensitive: bool,
    regex: bool,
    state: State<'_, AppState>,
) -> CommandResult<Vec<SearchMatch>> {
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || search_inner(&root, query, case_sensitive, regex)).await
}

/// Most matches a search returns. Beyond this the result list stops being
/// useful and starts being a memory problem.
const SEARCH_MATCH_LIMIT: usize = 1000;

/// Files scanned per round. Bounds peak memory to one round's matches rather
/// than the whole pack's, and lets a search that fills up stop early.
const SEARCH_BATCH: usize = 256;

/// Skip anything larger than this — a jar or a texture atlas is not something
/// the user is text-searching.
const SEARCH_MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

fn search_inner(
    root: &std::path::Path,
    query: String,
    case_sensitive: bool,
    regex: bool,
) -> CommandResult<Vec<SearchMatch>> {
    let pattern = if regex {
        query
    } else {
        regex_lite::escape(&query)
    };
    let expression = regex_lite::RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .build()
        .map_err(|error| SerializableError::new("search_pattern", error.to_string()))?;

    // Discovery runs on `ignore`'s parallel walker — ripgrep's own — so the
    // directory traversal and the `stat` of every entry overlap instead of
    // running one at a time.
    let candidates = std::sync::Mutex::new(Vec::new());
    ignore::WalkBuilder::new(root)
        .hidden(false)
        .follow_links(false)
        .build_parallel()
        .run(|| {
            Box::new(|entry| {
                if let Ok(entry) = entry
                    && entry.file_type().is_some_and(|kind| kind.is_file())
                    && entry
                        .metadata()
                        .is_ok_and(|metadata| metadata.len() <= SEARCH_MAX_FILE_BYTES)
                {
                    candidates
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(entry.into_path());
                }
                ignore::WalkState::Continue
            })
        });
    let mut candidates = candidates
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Workers finish in arbitrary order, so the file list is sorted before
    // anything is scanned. Everything downstream is a pure function of this
    // order, which is what makes a capped search reproducible: the old
    // sequential version returned whichever 1000 matches the walk reached
    // first, and a parallel walk would have made that vary run to run.
    candidates.sort();

    let jobs = packwand_parallel::configured();
    let mut matches = Vec::new();
    for batch in candidates.chunks(SEARCH_BATCH) {
        let found = packwand_parallel::map(batch, jobs, |path| scan_file(root, path, &expression));
        for file_matches in found {
            matches.extend(file_matches);
        }
        // Stop as soon as the cap is reachable; the remaining files cannot
        // affect the first `SEARCH_MATCH_LIMIT` results in this order.
        if matches.len() >= SEARCH_MATCH_LIMIT {
            break;
        }
    }
    matches.truncate(SEARCH_MATCH_LIMIT);
    Ok(matches)
}

/// Every match in one file, in line then column order. Unreadable or non-UTF-8
/// files yield nothing rather than failing the whole search.
fn scan_file(
    root: &std::path::Path,
    path: &std::path::Path,
    expression: &regex_lite::Regex,
) -> Vec<SearchMatch> {
    let Ok(source) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let mut matches = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        for found in expression.find_iter(line) {
            matches.push(SearchMatch {
                path: relative.clone(),
                line: line_index + 1,
                column: found.start() + 1,
                preview: line.trim().chars().take(240).collect(),
            });
        }
    }
    matches
}

fn timestamp(value: std::io::Result<SystemTime>) -> u64 {
    value
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn file_type(metadata: &fs::Metadata) -> u8 {
    if metadata.is_dir() {
        2
    } else if metadata.is_file() {
        1
    } else {
        0
    }
}

fn io_error(operation: &str, error: std::io::Error) -> SerializableError {
    let kind = match error.kind() {
        std::io::ErrorKind::NotFound => "not_found",
        std::io::ErrorKind::AlreadyExists => "already_exists",
        std::io::ErrorKind::PermissionDenied => "permission_denied",
        std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData => "invalid_data",
        _ => "io",
    };
    SerializableError::new(kind, format!("{operation}: {error}"))
}

fn remove_path(path: &std::path::Path, recursive: bool) -> CommandResult<()> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error("delete", error))?;
    if metadata.is_dir() {
        if recursive {
            fs::remove_dir_all(path).map_err(|error| io_error("delete", error))
        } else {
            fs::remove_dir(path).map_err(|error| io_error("delete", error))
        }
    } else {
        fs::remove_file(path).map_err(|error| io_error("delete", error))
    }
}

fn should_descend(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || !entry.file_type().is_dir()
        || !matches!(entry.file_name().to_str(), Some(".git" | "target"))
}

#[tauri::command]
pub async fn editor_tree(id: String, state: State<'_, AppState>) -> CommandResult<Vec<TreeEntry>> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || tree_inner(&root)).await
}

fn tree_inner(root: &std::path::Path) -> CommandResult<Vec<TreeEntry>> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(root)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_descend)
    {
        let entry = entry.map_err(|error| SerializableError::new("walk", error.to_string()))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| SerializableError::new("unsafe_path", error.to_string()))?
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        entries.push(TreeEntry {
            path: relative,
            name: entry.file_name().to_string_lossy().into_owned(),
            directory: entry.file_type().is_dir(),
            size: entry.metadata().map(|metadata| metadata.len()).unwrap_or(0),
        });
    }
    entries.sort_by(|left, right| {
        right
            .directory
            .cmp(&left.directory)
            .then(left.path.cmp(&right.path))
    });
    Ok(entries)
}

#[tauri::command]
pub async fn editor_file_read(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || {
        let target = safe_join(&root, &path)?;
        let bytes = fs::read(&target)?;
        String::from_utf8(bytes).map_err(|_| {
            SerializableError::new(
                "binary_file",
                format!(
                    "{} is binary and cannot be opened as text",
                    target.display()
                ),
            )
        })
    })
    .await
}

#[tauri::command]
pub async fn editor_file_write(
    id: String,
    path: String,
    content: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || {
        let target = safe_join(&root, &path)?;
        if !target.is_file() {
            return Err(SerializableError::new(
                "not_found",
                format!("{} is not an existing file", target.display()),
            ));
        }
        crate::fsutil::atomic_write(&target, content.as_bytes())
    })
    .await?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_create(
    id: String,
    path: String,
    directory: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    if target.exists() {
        return Err(SerializableError::new(
            "already_exists",
            format!("{} already exists", target.display()),
        ));
    }
    if directory {
        fs::create_dir_all(target)?;
    } else {
        crate::fsutil::atomic_write(&target, b"")?;
    }
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_fs_stat(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<EditorFileStat> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    let metadata = fs::metadata(&target).map_err(|error| io_error("stat", error))?;
    Ok(EditorFileStat {
        file_type: file_type(&metadata),
        size: metadata.len(),
        ctime: timestamp(metadata.created()),
        mtime: timestamp(metadata.modified()),
    })
}

#[tauri::command]
pub fn editor_fs_read_dir(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<EditorDirectoryEntry>> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    let mut entries = fs::read_dir(&target)
        .map_err(|error| io_error("read directory", error))?
        .map(|entry| {
            let entry = entry.map_err(|error| io_error("read directory", error))?;
            let metadata = entry
                .metadata()
                .map_err(|error| io_error("read directory", error))?;
            Ok(EditorDirectoryEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                file_type: file_type(&metadata),
            })
        })
        .collect::<CommandResult<Vec<_>>>()?;
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

#[tauri::command]
pub async fn editor_fs_read_file(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<u8>> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || {
        let target = safe_join(&root, &path)?;
        fs::read(target).map_err(|error| io_error("read file", error))
    })
    .await
}

#[tauri::command]
pub async fn editor_fs_write_file(
    id: String,
    path: String,
    content: Vec<u8>,
    create: bool,
    overwrite: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || {
        let target = safe_join(&root, &path)?;
        let exists = target.exists();
        if exists && target.is_dir() {
            return Err(SerializableError::new(
                "is_directory",
                format!("{} is a directory", target.display()),
            ));
        }
        if !exists && !create {
            return Err(SerializableError::new(
                "not_found",
                format!("{} does not exist", target.display()),
            ));
        }
        if exists && !overwrite {
            return Err(SerializableError::new(
                "already_exists",
                format!("{} already exists", target.display()),
            ));
        }
        crate::fsutil::atomic_write(&target, &content)
    })
    .await?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_fs_create_dir(
    id: String,
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    fs::create_dir(&target).map_err(|error| io_error("create directory", error))?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub async fn editor_fs_delete(
    id: String,
    path: String,
    recursive: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    if path.is_empty() {
        return Err(SerializableError::new(
            "unsafe_path",
            "the pack root cannot be deleted",
        ));
    }
    let root = pack_root(&state.workspace()?, &id)?;
    // A recursive delete of a mods/ or config/ tree is thousands of unlink
    // syscalls, which is exactly the kind of work that must not sit on the
    // window's thread.
    off_thread(move || {
        let target = safe_join(&root, &path)?;
        remove_path(&target, recursive)
    })
    .await?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_fs_rename(
    id: String,
    from: String,
    to: String,
    overwrite: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    if from.is_empty() || to.is_empty() {
        return Err(SerializableError::new(
            "unsafe_path",
            "the pack root cannot be renamed or replaced",
        ));
    }
    let root = pack_root(&state.workspace()?, &id)?;
    let source = safe_join(&root, &from)?;
    let target = safe_join(&root, &to)?;
    if target.exists() {
        if !overwrite {
            return Err(SerializableError::new(
                "already_exists",
                format!("{} already exists", target.display()),
            ));
        }
        remove_path(&target, true)?;
    }
    fs::rename(&source, &target).map_err(|error| io_error("rename", error))?;
    emit_packs_changed(&app)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes `count` files each containing `per_file` matching lines.
    fn seeded_pack(count: usize, per_file: usize) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for file in 0..count {
            let nested = dir.path().join(format!("ns{}", file % 7));
            fs::create_dir_all(&nested).unwrap();
            let body = (0..per_file)
                .map(|line| format!("needle occurrence {file}-{line}\n"))
                .collect::<String>();
            fs::write(nested.join(format!("f{file:03}.txt")), body).unwrap();
        }
        dir
    }

    /// The property the parallel walk had to preserve. A capped search used to
    /// return whichever matches the sequential walk reached first; sorting the
    /// file list before scanning is what keeps that reproducible now that
    /// discovery order is nondeterministic.
    #[test]
    fn an_over_cap_search_returns_the_same_matches_every_run() {
        let pack = seeded_pack(80, 40); // 3200 matches against a 1000 cap
        let first = search_inner(pack.path(), "needle".into(), false, false).unwrap();
        assert_eq!(first.len(), SEARCH_MATCH_LIMIT, "the cap must be reached");
        for _ in 0..6 {
            let again = search_inner(pack.path(), "needle".into(), false, false).unwrap();
            assert_eq!(first, again, "results varied between runs");
        }
    }

    /// Determinism must not depend on how many workers happen to be available.
    #[test]
    fn results_do_not_depend_on_the_worker_count() {
        let pack = seeded_pack(40, 30);
        let expected = search_inner(pack.path(), "needle".into(), false, false).unwrap();
        // `search_inner` reads the process-wide job count, so this exercises
        // the batching path rather than re-configuring it: a single-threaded
        // scan of the same sorted list must agree.
        let mut sequential = Vec::new();
        let mut files: Vec<_> = walkdir::WalkDir::new(pack.path())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .collect();
        files.sort();
        let expression = regex_lite::Regex::new("needle").unwrap();
        for path in &files {
            sequential.extend(scan_file(pack.path(), path, &expression));
        }
        sequential.truncate(SEARCH_MATCH_LIMIT);
        assert_eq!(expected, sequential);
    }

    #[test]
    fn an_under_cap_search_finds_every_match_in_path_order() {
        let pack = seeded_pack(5, 3);
        let found = search_inner(pack.path(), "needle".into(), false, false).unwrap();
        assert_eq!(found.len(), 15);
        let mut sorted = found.clone();
        sorted.sort_by(|left, right| (&left.path, left.line).cmp(&(&right.path, right.line)));
        assert_eq!(found, sorted, "matches must come back in a stable order");
    }

    #[test]
    fn a_regex_search_reports_the_matching_column() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "alpha beta gamma\n").unwrap();
        let found = search_inner(dir.path(), "b[et]+a".into(), false, true).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, 1);
        assert_eq!(found[0].column, 7);
    }
}
