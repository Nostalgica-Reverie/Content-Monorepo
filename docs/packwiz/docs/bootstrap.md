# Bootstrap

The bootstrap is the small program a launcher actually invokes. It verifies that a suitable Java runtime is available, keeps `packwiz-installer.jar` up to date, launches it with your arguments, and passes the exit code through.

Two implementations are maintained:

## Go bootstrap (recommended)

Source: `src/packwand/cmd/packwiz-bootstrap`. A single native binary with no Java requirement of its own, following packwand's CLI conventions.

```sh
packwiz-bootstrap [options] <pack.toml URL> [-- installer options...]
```

| Option | Description |
| --- | --- |
| `--java <path>` | Path to the `java` executable (otherwise `$JAVA_HOME/bin/java`, then `PATH`) |
| `--min-java <version>` | Minimum Java major version to accept (defaults to 8) |
| `--jar <path>` | Location of `packwiz-installer.jar` (defaults to next to the bootstrap executable) |
| `--download-url <url>` | URL to download `packwiz-installer.jar` from when missing |
| `--sha256 <hash>` | Expected SHA-256 of a downloaded jar (verified before first use) |
| `-g`, `--no-gui` | Passed through to the installer: disable the GUI |
| `-s`, `--side <side>` | Passed through to the installer: `client` or `server` |

Behaviour:

1. Locates and verifies Java (`java -version` must report at least `--min-java`).
2. Ensures the installer jar exists; downloads it from `--download-url` if missing (with optional SHA-256 verification).
3. Runs `java -jar packwiz-installer.jar <passthrough args>` and exits with the installer's exit code.

Example (MultiMC/Prism pre-launch command):

```
"$INST_DIR/packwiz-bootstrap" -g -s client https://example.com/pack.toml
```

## Legacy Java bootstrap

Source: `lib/packwiz-installer/bootstrap` (built as a Gradle subproject of packwiz-installer). Kept for compatibility with existing instances that already ship `packwiz-installer-bootstrap.jar`.

```sh
java -jar packwiz-installer-bootstrap.jar [options] <pack.toml URL>
```

| Option | Description |
| --- | --- |
| `--bootstrap-update-url <url>` | GitHub API URL for checking for updates |
| `--bootstrap-update-token <token>` | GitHub API access token, for private repositories |
| `--bootstrap-no-update` | Don't update packwiz-installer |
| `--bootstrap-main-jar <path>` | Location of the packwiz-installer JAR file |
| `-g`, `--no-gui` | Don't display a GUI to show update progress |
| `-h`, `--help` | Display usage (includes the installer's options when the jar is present) |

All other arguments are passed through to packwiz-installer.
