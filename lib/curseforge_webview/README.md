# curseforge_webview
A webview using wry, used by other applications to display the CurseForge website.

### Updating dependencies
When dependencies are updated/added/removed, the third-party licenses file should be regenerated. This can be done with the following commands:

```
cargo install --locked cargo-about
cargo about generate about.hbs -o src/licenses.html
```