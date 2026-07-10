# Building

All components build from the repository root with [Task](https://taskfile.dev) (`Taskfile.yml`), or directly with their native toolchains.

## Prerequisites

- **JDK 17+** (JDK 25 verified) for packwiz-installer â€” Gradle 9 is fetched by the wrapper
- **Rust** (cargo) for mod_browser_webview
- **Go 1.25+** for the Go bootstrap and packwand

## With Task

```sh
task build-installer   # packwiz-installer + legacy Java bootstrap (Gradle)
task build-webview     # mod_browser_webview, release profile (cargo)
task build-bootstrap   # Go packwiz-bootstrap
task gen-licenses      # regenerate the webview's third-party licenses page
task build             # everything
```

## Directly

```sh
# packwiz-installer (output: build/dist/packwiz-installer.jar)
cd lib/packwiz-installer && ./gradlew build

# legacy Java bootstrap (output: bootstrap/build/libs/bootstrap-*-all.jar)
cd lib/packwiz-installer && ./gradlew :bootstrap:shadowJar

# mod_browser_webview (output: target/release/mod_browser_webview)
cd apps/mod-browser-webview && cargo build --release

# Go bootstrap
cd apps/packwand && go build ./cmd/packwiz-bootstrap
```

::: info
The installer's R8-shrunk distribution jar is opt-in: `./gradlew build -PshrinkDist=true`. The default `build` ships the shadow jar, because R8 8.5 cannot read the class files of very new JDKs (e.g. Java 25) when they are passed as its library.
:::

