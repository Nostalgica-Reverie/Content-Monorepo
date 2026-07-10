# Building the native GUI app

The Packwand GUI ships in two forms: the browser-based `packwand gui` command, and a native desktop app built with [Tauri v2](https://v2.tauri.app/) that wraps the same frontend and server. The Tauri shell lives in `apps/packwand/gui/tauri/`.

## Architecture

The app follows the pattern used by the Modrinth App: a small Rust backend acts as the privileged bridge, and the webview renders the existing Gleam frontend.

- On launch, a bundled boot page calls the single exposed IPC command, `backend_url`.
- The Rust backend locates the `packwand` binary (`PACKWAND_BIN`, next to the app executable, then `PATH`), spawns `packwand gui --no-open --port 0` as a managed child process, and reads the bound `http://127.0.0.1:<port>/` address from its startup banner.
- The window then navigates to the local server. From that point everything works exactly like the browser GUI — same Gleam frontend, same HTTP API, same SSE job streams.
- The server pages are deliberately given **no** Tauri IPC access (the capability only covers the boot page), so the webview cannot reach system APIs beyond what the packwand HTTP API already exposes. The backend process is terminated when the app exits.

## Prerequisites

Follow the [Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/) for your platform. In short:

- **Rust** (stable, via [rustup](https://rustup.rs/))
- **Go 1.25+** (builds the `packwand` backend the app spawns)
- **Node.js 22.18+** (only needed when rebuilding the Gleam frontend via `gui/ui/build.mts`; the build script is TypeScript run via Node’s native type stripping)
- The Tauri CLI: `cargo install tauri-cli --version "^2" --locked`

Platform-specific webview dependencies:

| Platform | Requirement |
| --- | --- |
| Windows | [WebView2 runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (preinstalled on Windows 11) and the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) |
| Linux | `webkit2gtk-4.1`, `libgtk-3-dev`, `build-essential`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` (names vary by distro — see the Tauri guide) |
| macOS | Xcode Command Line Tools (`xcode-select --install`) |

## Building

From the repository root:

```sh
task build-gui
```

This builds the `packwand` CLI first, then runs `cargo tauri build` in `apps/packwand/gui/tauri`, producing a platform installer/bundle under `apps/packwand/gui/tauri/src-tauri/target/release/bundle/`.

::: warning
The packaged app expects a `packwand` executable next to it or on `PATH` (or `PACKWAND_BIN` set). When distributing, ship the `packwand` binary alongside the app bundle.
:::

## Development

```sh
cd apps/packwand/gui/tauri/src-tauri
cargo tauri dev
```

`tauri dev` starts `packwand gui --no-open --port 8654` (via `beforeDevCommand`) and points the window at it, so frontend/API changes are picked up by restarting the server. To iterate on the Gleam frontend, rebuild it with `task gui-frontend` (the server serves the embedded static files, so rebuild the Go binary — or just restart `cargo tauri dev` — after changing them).

## Security boundaries

- `tauri.conf.json` sets a strict CSP for bundled assets and enables no Tauri plugins.
- `capabilities/default.json` grants only `core:default` to the boot page; no filesystem, shell, or HTTP scopes are exposed to the webview.
- All pack management operations flow through the `packwand gui` HTTP API on `127.0.0.1`, which binds to the loopback interface only.
