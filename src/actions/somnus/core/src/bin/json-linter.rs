use somnus_core::lint_changed_files;

const WATCHED_PREFIXES: &[&str] = &["modpacks/", "datapacks/"];
const WATCHED_EXTS: &[&str] = &[".json", ".mcmeta"];

fn main() {
    let report = match lint_changed_files(
        "json",
        |path| {
            WATCHED_PREFIXES.iter().any(|p| path.starts_with(p))
                && WATCHED_EXTS.iter().any(|e| path.ends_with(e))
        },
        |content| {
            serde_json::from_str::<serde_json::Value>(content)
                .map(|_| ())
                .map_err(|e| e.to_string())
        },
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("::error::lint harness failed: {e}");
            std::process::exit(1);
        }
    };

    println!("linted {} json file(s), {} failed.", report.checked, report.failed);

    if !report.is_ok() {
        eprintln!("fix yo json chud...");
        std::process::exit(1);
    }
}
