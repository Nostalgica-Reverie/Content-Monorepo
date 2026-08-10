# packwiz-installer

A Kotlin/JVM installer that downloads and updates packwiz/packwand-format packs on launch. It runs as a pre-launch task in MultiMC/Prism/ATLauncher, or in a server start script, and supports side-only mods as well as optional mods with a GUI (and a fully non-interactive mode for servers).

Source: `lib/packwiz-installer`. Build output: `lib/packwiz-installer/build/dist/packwiz-installer.jar`.

## Usage

packwiz-installer is normally launched through the [bootstrap](/bootstrap), which handles updates:

```sh
packwiz-bootstrap https://example.com/pack.toml
# or, with the legacy Java bootstrap:
java -jar packwiz-installer-bootstrap.jar https://example.com/pack.toml
```

Running the JAR directly also works (no auto-update):

```sh
java -jar packwiz-installer.jar [options] <pack.toml URL>
```

## Options

| Option                      | Description                                                                                     |
| --------------------------- | ----------------------------------------------------------------------------------------------- |
| `-s`, `--side <side>`       | Side to install mods from (`client`/`server`, defaults to `client`)                             |
| `--title <title>`           | Title of the installer window                                                                   |
| `--pack-folder <path>`      | Folder to install the pack to (defaults to the JAR directory)                                   |
| `--multimc-folder <path>`   | The MultiMC pack folder (defaults to the parent of the pack directory)                          |
| `--meta-file <path>`        | JSON file to store pack metadata, relative to the pack folder (defaults to `packwiz.json`)      |
| `-t`, `--timeout <seconds>` | Seconds to wait before automatically launching when asking about optional mods (defaults to 10) |
| `-g`, `--no-gui`            | Don't display a GUI to show update progress (for servers/CI)                                    |
| `-h`, `--help`              | Display usage                                                                                   |

The `--bootstrap-*` options are accepted (and ignored) so that the bootstrap can pass its own arguments through.

## Server usage

```sh
java -jar packwiz-installer-bootstrap.jar -g -s server https://example.com/pack.toml
```

- `-g` disables the GUI
- `-s server` downloads only server-side mods (side `server` or `both`)

## State

Installed-file state is tracked in `packwiz.json` (configurable with `--meta-file`) so that removed files are cleaned up, `preserve`d files are not overwritten, and unchanged files are not re-downloaded.
