# Packwiz Components

This site documents the packwiz ecosystem components vendored into the Lasting Legacy monorepo. They install and update packs in the packwiz/packwand format on end-user machines — the counterpart to [packwand](https://git.nostalgica.net/Lasting-Legacy/Lasting-Legacy-Monorepo), which creators use to author packs.

| Component | Language | Location | Purpose |
| --- | --- | --- | --- |
| [packwiz-installer](/installer) | Kotlin | `lib/packwiz-installer` | Downloads and updates pack contents on launch, with optional-mod UI and side-only filtering |
| [bootstrap](/bootstrap) | Go (new) / Java (legacy) | `src/packwand/cmd/packwiz-bootstrap`, `lib/packwiz-installer/bootstrap` | Verifies a JDK, keeps packwiz-installer up to date, and launches it |
| [mod_browser_webview](/webview) | Rust (wry) | `lib/mod-browser-webview` | Native webview for downloading CurseForge files that disallow API distribution; bridged into the packwand GUI |

All three are built from this repository — see [Building](/building).

## How they fit together

1. A launcher instance (MultiMC/Prism/ATLauncher) or server start script runs the **bootstrap** as a pre-launch command.
2. The bootstrap verifies Java, updates **packwiz-installer** if needed, and hands over your pack URL.
3. packwiz-installer reads `pack.toml`, downloads changed files, prompts for optional mods, and writes its state to `packwiz.json`.
4. For CurseForge files that cannot be downloaded through the API, tooling can open **mod_browser_webview** so the user downloads them from the real CurseForge site; the resulting CDN URLs are captured programmatically.
