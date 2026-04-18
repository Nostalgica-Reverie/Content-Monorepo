use crate::errors::SomnusError;
use crate::get_changed_files;
use std::fs;
use std::path::Path;

pub struct LintReport {
    pub checked: usize,
    pub failed: usize,
}

impl LintReport {
    pub fn is_ok(&self) -> bool {
        self.failed == 0
    }
}

pub fn lint_changed_files<M, P>(
    kind: &str,
    matches: M,
    parse: P,
) -> Result<LintReport, SomnusError>
where
    M: Fn(&str) -> bool,
    P: Fn(&str) -> Result<(), String>,
{
    let changed = get_changed_files()?;
    let mut checked = 0usize;
    let mut failed = 0usize;
    let kind_upper = kind.to_uppercase();

    for file_path in changed {
        if !matches(&file_path) {
            continue;
        }
        let path = Path::new(&file_path);
        if !path.exists() {
            continue;
        }

        println!("::group::Linting {kind}: {file_path}");
        checked += 1;

        match fs::read_to_string(path) {
            Ok(content) => {
                if let Err(msg) = parse(&content) {
                    eprintln!("::error file={file_path}::INVALID {kind_upper}: {msg}");
                    failed += 1;
                }
            }
            Err(e) => {
                eprintln!("::error file={file_path}::Failed to read: {e}");
                failed += 1;
            }
        }

        println!("::endgroup::");
    }

    Ok(LintReport { checked, failed })
}
