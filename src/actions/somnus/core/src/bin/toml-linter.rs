use somnus_core::linter::lint_changed_files;

const WATCHED_PREFIXES: &[&str] = &["modpacks/", "datapacks/"];

fn main() {
    let report = match lint_changed_files(
        "toml",
        |path| {
            path.ends_with(".toml")
                && WATCHED_PREFIXES.iter().any(|p| path.starts_with(p))
        },
        |content| {
            content
                .parse::<toml::Value>()
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

    println!("linted {} toml file(s), {} failed.", report.checked, report.failed);

    if !report.is_ok() {
        eprintln!("fix yo toml chud...");
        std::process::exit(1);
    }
}
