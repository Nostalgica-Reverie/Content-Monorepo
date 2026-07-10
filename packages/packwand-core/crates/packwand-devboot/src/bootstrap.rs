//! `instance bootstrap`: fetch version metadata, plan and execute the
//! installation, and store a launchable instance record.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use packwand_auth::Session;
use packwand_instance::{
    FsInstanceRepository, InstanceError, InstanceRecord, InstanceRepository, InstanceSpec,
    MemoryLimits,
};
use packwand_minecraft::args::{has_unresolved_placeholder, resolve_launch_args, LaunchContext};
use packwand_minecraft::meta::InstallerProfile;
use packwand_minecraft::model::VersionDoc;
use packwand_minecraft::plan::{
    build_asset_plan, build_library_downloads, build_version_plan, merge_plans, InstallLayout,
};
use packwand_minecraft::{
    Host, InstallProgress, Installer, MetadataClient, MetadataEndpoints, UreqClient,
};
use packwand_runtime::{discover, select_compatible, DiscoveryConfig};

pub struct BootstrapRequest {
    pub root: PathBuf,
    pub id: String,
    /// Version id, or the `latest-release` / `latest-snapshot` aliases.
    pub minecraft: String,
    /// Optional loader overlay: `fabric`, `quilt`, `forge`, or `neoforge`.
    pub loader: Option<String>,
    pub loader_version: Option<String>,
    /// The account identity to bake into this instance's launch arguments.
    /// Callers decide how it was obtained (offline for dev-testing, or a
    /// real Microsoft/Xbox Live session) — `bootstrap` itself is agnostic.
    pub session: Session,
    /// Explicit Java executable; skips discovery when set.
    pub java: Option<PathBuf>,
    pub memory_max_mb: Option<u32>,
    pub workers: usize,
    pub endpoints: MetadataEndpoints,
}

fn write_file(path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn pick_java(request: &BootstrapRequest, doc: &VersionDoc) -> Result<PathBuf, String> {
    if let Some(java) = &request.java {
        return Ok(java.clone());
    }
    let installations = discover(&DiscoveryConfig::from_host());
    match &doc.java_version {
        Some(required) => select_compatible(&installations, required.major_version)
            .map(|i| i.executable.clone())
            .map_err(|e| e.to_string()),
        None => installations
            .first()
            .map(|i| i.executable.clone())
            .ok_or_else(|| "no Java installation was discovered; pass --java".to_string()),
    }
}

fn resolve_loader_profile(
    client: &MetadataClient<'_>,
    manifest: &packwand_minecraft::model::VersionManifest,
    game_version: &str,
    loader: &str,
    requested_version: Option<&str>,
) -> Result<(VersionDoc, Vec<packwand_minecraft::model::Library>), String> {
    match loader {
        "fabric" => {
            let loader = client
                .resolve_fabric_loader(game_version, requested_version)
                .map_err(|e| e.to_string())?;
            eprintln!("using fabric loader {loader}");
            let profile = client
                .fetch_fabric_profile(game_version, &loader)
                .map_err(|e| e.to_string())?;
            let doc = client
                .resolve_inheritance(manifest, profile)
                .map_err(|e| e.to_string())?;
            Ok((doc, Vec::new()))
        }
        "quilt" => {
            let loader = client
                .resolve_quilt_loader(game_version, requested_version)
                .map_err(|e| e.to_string())?;
            eprintln!("using quilt loader {loader}");
            let profile = client
                .fetch_quilt_profile(game_version, &loader)
                .map_err(|e| e.to_string())?;
            let doc = client
                .resolve_inheritance(manifest, profile)
                .map_err(|e| e.to_string())?;
            Ok((doc, Vec::new()))
        }
        "forge" => {
            let loader = client
                .resolve_forge_loader(game_version, requested_version)
                .map_err(|e| e.to_string())?;
            eprintln!("using forge loader {loader}");
            let InstallerProfile { version, libraries } = client
                .fetch_forge_profile(game_version, &loader)
                .map_err(|e| e.to_string())?;
            let doc = client
                .resolve_inheritance(manifest, version)
                .map_err(|e| e.to_string())?;
            Ok((doc, libraries))
        }
        "neoforge" => {
            let loader = client
                .resolve_neoforge_loader(game_version, requested_version)
                .map_err(|e| e.to_string())?;
            eprintln!("using neoforge loader {loader}");
            let InstallerProfile { version, libraries } = client
                .fetch_neoforge_profile(game_version, &loader)
                .map_err(|e| e.to_string())?;
            let doc = client
                .resolve_inheritance(manifest, version)
                .map_err(|e| e.to_string())?;
            Ok((doc, libraries))
        }
        other => Err(format!(
            "unsupported loader {other:?}; expected fabric, quilt, forge, or neoforge"
        )),
    }
}

fn print_progress(update: InstallProgress, throttle: &Mutex<(u64, usize)>) {
    let mut guard = throttle.lock().expect("progress throttle poisoned");
    let should_print = update.finished_downloads == update.total_downloads
        || update.downloaded_bytes.saturating_sub(guard.0) >= 4 * 1024 * 1024
        || update.finished_downloads > guard.1;
    if !should_print {
        return;
    }
    guard.0 = update.downloaded_bytes;
    guard.1 = update.finished_downloads;
    match update.total_bytes {
        Some(total_bytes) if total_bytes > 0 => {
            let percent = (update.downloaded_bytes as f64 / total_bytes as f64) * 100.0;
            eprint!(
                "\r{}/{} downloads | {:.1}% | {}/{} MiB",
                update.finished_downloads,
                update.total_downloads,
                percent,
                update.downloaded_bytes / (1024 * 1024),
                total_bytes / (1024 * 1024)
            );
        }
        _ => {
            eprint!(
                "\r{}/{} downloads | {} MiB transferred",
                update.finished_downloads,
                update.total_downloads,
                update.downloaded_bytes / (1024 * 1024)
            );
        }
    }
}

/// Runs `bootstrap` with a caller-supplied progress callback instead of the
/// CLI's own throttled stderr printer. Used by orchestration layers (e.g. the
/// Tauri adapter) that want to forward `InstallProgress` to a UI.
pub fn bootstrap_with_progress(
    request: &BootstrapRequest,
    on_progress: impl Fn(InstallProgress) + Sync,
) -> Result<InstanceRecord, String> {
    let repo = FsInstanceRepository::new(request.root.clone());
    let paths = repo.instance_paths(&request.id);
    let http = UreqClient::new();
    let client = MetadataClient::new(&http, request.endpoints.clone());

    // 1. Resolve metadata (vanilla, optionally overlaid by a loader profile).
    let manifest = client.fetch_manifest().map_err(|e| e.to_string())?;
    let entry = manifest.find(&request.minecraft).ok_or_else(|| {
        format!(
            "version {:?} was not found in the manifest",
            request.minecraft
        )
    })?;
    let vanilla = client.fetch_version(entry).map_err(|e| e.to_string())?;
    let (doc, installer_libraries) = match request.loader.as_deref() {
        None => (vanilla.value, Vec::new()),
        Some(loader) => resolve_loader_profile(
            &client,
            &manifest,
            &entry.id,
            loader,
            request.loader_version.as_deref(),
        )?,
    };

    // 2. Plan the installation.
    let host = Host::current();
    let layout = InstallLayout {
        versions_dir: request.root.join("versions"),
        libraries_dir: paths.libraries_dir.clone(),
        assets_dir: paths.assets_dir.clone(),
        natives_dir: paths.natives_dir.clone(),
        resources_dir: Some(paths.game_dir.join("resources")),
    };
    let mut plan = build_version_plan(&doc, &host, &layout).map_err(|e| e.to_string())?;
    for extra in
        build_library_downloads(&installer_libraries, &layout).map_err(|e| e.to_string())?
    {
        if !plan
            .downloads
            .iter()
            .any(|download| download.target == extra.target)
        {
            plan.downloads.push(extra);
        }
    }
    let index_ref = doc
        .asset_index
        .clone()
        .ok_or_else(|| format!("version {} has no asset index", doc.id))?;
    let index = client
        .fetch_asset_index(&index_ref)
        .map_err(|e| e.to_string())?;
    let asset_plan = build_asset_plan(
        &index_ref.id,
        &index.value,
        &layout,
        &request.endpoints.resources_url,
    )
    .map_err(|e| e.to_string())?;
    plan = merge_plans(plan, asset_plan);

    // 3. Persist the verified metadata alongside what it describes.
    let doc_bytes = serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?;
    write_file(
        &layout
            .versions_dir
            .join(&doc.id)
            .join(format!("{}.json", doc.id)),
        &doc_bytes,
    )?;
    write_file(
        &layout
            .assets_dir
            .join("indexes")
            .join(format!("{}.json", index_ref.id)),
        &index.bytes,
    )?;

    // 4. Execute the plan (verified, staged, resumable).
    eprintln!(
        "installing {}: {} downloads (~{} MiB known)",
        doc.id,
        plan.downloads.len(),
        plan.known_download_bytes() / (1024 * 1024)
    );
    let report = Installer::new(&http)
        .with_workers(request.workers)
        .execute(&plan, &on_progress)
        .map_err(|e| e.to_string())?;
    eprintln!(
        "\rdownloaded {}, verified existing {}, extracted {}, copied {}",
        report.downloaded, report.skipped, report.extracted, report.copied
    );

    // 5. Resolve arguments for the given session's identity.
    let context = LaunchContext {
        version_id: doc.id.clone(),
        version_type: doc.kind.clone().unwrap_or_else(|| "release".to_string()),
        assets_index_name: index_ref.id.clone(),
        player_name: request.session.username.clone(),
        player_uuid: request.session.uuid.clone(),
        user_type: request.session.user_type.clone(),
        launcher_name: "packwand".to_string(),
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        game_assets_dir: plan
            .game_assets_dir
            .as_ref()
            .map(|p| p.display().to_string()),
    };
    let resolved = resolve_launch_args(&doc, &host, &context).map_err(|e| e.to_string())?;
    for arg in resolved.jvm_args.iter().chain(&resolved.game_args) {
        if has_unresolved_placeholder(arg) {
            eprintln!("warning: argument {arg:?} still contains an unresolved placeholder");
        }
    }

    // 6. Select Java and store the record.
    let java_executable = pick_java(request, &doc)?;
    let spec = InstanceSpec {
        id: request.id.clone(),
        name: format!("{} ({})", request.id, doc.id),
        java_executable,
        jvm_args: resolved.jvm_args,
        main_class: resolved.main_class,
        classpath: plan.classpath.clone(),
        game_args: resolved.game_args,
        env: BTreeMap::new(),
        memory: MemoryLimits {
            initial_mb: None,
            max_mb: request.memory_max_mb,
        },
        session_placeholders: resolved.session_placeholders,
    };
    // Upsert: `create` fails with `AlreadyExists` when re-baking a different
    // session's identity onto an already-installed instance, or retrying a
    // bootstrap that got far enough to persist a record before an earlier
    // interruption. Overwrite in place instead of treating that as an error.
    match repo.create(&spec) {
        Ok(record) => Ok(record),
        Err(InstanceError::AlreadyExists(_)) => {
            let record = InstanceRecord::from_spec(&spec);
            repo.update(&spec.id, &record).map_err(|e| e.to_string())?;
            Ok(record)
        }
        Err(other) => Err(other.to_string()),
    }
}

/// Runs `bootstrap` with the CLI's own throttled stderr progress printer.
pub fn bootstrap(request: &BootstrapRequest) -> Result<InstanceRecord, String> {
    let throttle = Mutex::new((0u64, 0usize));
    bootstrap_with_progress(request, |update| print_progress(update, &throttle))
}
