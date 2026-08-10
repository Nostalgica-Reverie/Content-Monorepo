# Installing and building Packwand 26.2.0

## Windows installer

1. Run `Packwand_26.2.0_x64-setup.exe`.
2. Start **Packwand** from the Start menu.
3. Choose the Lasting Legacy workspace when prompted.
4. Create or link an instance, then choose **Install** or **Play**. Packwand
   installs pack content with its bundled native installer before bootstrapping
   Minecraft.

The installer contains the native Rust/Vue desktop application, the native
`packwand.exe` CLI, and `packwand-installer.exe`. It does not install or execute
the old Go Packwand application, and the desktop uses Tauri IPC rather than a
loopback API bridge.

## Portable install

Copy `packwand.exe`, `packwand-gui.exe`, and `packwand-installer.exe` into the
same directory. The CLI's `packwand gui` command locates the desktop binary
beside itself. Set `PACKWAND_INSTALLER_BIN` when the native installer lives
elsewhere.

## Build

Required tools:

- Rust stable, Cargo, and the `cargo-tauri` 2.x command.
- Bun 1.3 or newer.
- Windows MSVC C++ build tools and WebView2.
- Java only when deliberately building or testing the legacy external-launcher
  compatibility installer.
- Packeater for folders containing `packeater.json`. It is a member of the
  packwandrs workspace, so `cargo build -p packeater_cli` puts the binary in the
  shared `target/`; set `PACKEATER_BIN` if it is not on `PATH`.
- A release build of `packwand-installer`, produced automatically by the
  repository's `build-gui` recipe before the Tauri bundle is assembled.

From `apps/packwandrs`:

```powershell
bun install --frozen-lockfile
bun run check
bun run test:frontend
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p packwand-cli
cargo build --release -p packwand-installer
cargo tauri build
```

Build Packeater separately from its history-preserving fork workspace:

```powershell
cd packeater
cargo build --release -p packeater_cli
$env:PACKEATER_BIN = (Resolve-Path target\release\packeater.exe)
```

Outputs:

- `target/release/packwand.exe`
- `target/release/packwand-gui.exe`
- `target/release/packwand-installer.exe`
- `target/release/bundle/nsis/Packwand_26.2.0_x64-setup.exe`
