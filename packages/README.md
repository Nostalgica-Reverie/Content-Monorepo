# packages/

Shared libraries — code that exists to be depended on by apps or downstream
consumers, not run directly.

| Package | Language | What it is |
| ------- | -------- | ---------- |
| [`packwand-core`](packwand-core/) | Rust | Launcher-core crates (packwand-runtime, -minecraft, -auth, -instance, -launch, -devboot, -msa); consumed by the Tauri GUI via path dependencies |
| [`packwand2nix`](packwand2nix/) | Nix | Vendored packwiz2nix fork; re-exported by the root `flake.nix` |