# Installing and building Packwand 26.2.0

## Windows installer

1. Run `Packwand_26.2.0_x64-setup.exe`.
2. Start **Packwand** from the Start menu.
3. Choose the Lasting Legacy workspace when prompted.
4. Install a current Java runtime and ensure `java -version` succeeds if you
   want to use Diagnostics > Installer-driven pack test.

The installer contains the native Rust/Vue desktop application, the native
`packwand.exe` CLI, and the packwiz installer test resource. It does not install or execute the old Go
Packwand application and the desktop uses Tauri IPC rather than a loopback
API bridge.

## Portable install

Copy `packwand.exe` and `packwand-gui.exe` into the same directory. The CLI's
`packwand gui` command locates the desktop binary beside itself. Set
`PACKWAND_INSTALLER_JAR` when using installer tests from a portable layout
that does not contain the bundled resource.

## Build

Required tools:

- Rust stable, Cargo, and the `cargo-tauri` 2.x command.
- Bun 1.3 or newer.
- Windows MSVC C++ build tools and WebView2.
- Java for the installer validation smoke test.
- Packeater (nightly Rust build from `apps/packwandrs/packeater`) for folders containing
  `packeater.json`; set `PACKEATER_BIN` if it is not on `PATH`.
- The local `apps/packwand-installer/build/dist/packwiz-installer.jar`, which
  the NSIS build packages as a resource.

From `apps/packwandrs`:

```powershell
bun install --frozen-lockfile
bun run check
bun run test:frontend
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p packwand-cli
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
- `target/release/bundle/nsis/Packwand_26.2.0_x64-setup.exe`
