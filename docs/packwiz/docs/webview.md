# curseforge_webview

A native webview (Rust, using [wry](https://github.com/tauri-apps/wry)) that displays real CurseForge project pages so users can download files that may not be distributed through the CurseForge API. Host applications drive it over a simple stdin/stdout line protocol and receive the resolved CDN download URLs.

Source: `lib/curseforge_webview`. Build output: `lib/curseforge_webview/target/release/curseforge_webview`.

## Platform requirements

- **Windows**: the [WebView2 runtime](https://go.microsoft.com/fwlink/p/?LinkId=2124703) (preinstalled on Windows 11)
- **Linux**: WebKitGTK (`webkit2gtk`)
- **macOS**: WKWebView (built in)

## Protocol

The host writes to the webview's **stdin**, one request per line, then `DONE`:

```
DATA /path/to/profile/dir        (optional: persistent browser profile)
3643025 https://www.curseforge.com/minecraft/mc-mods/jei
123456 https://www.curseforge.com/minecraft/mc-mods/sodium
DONE
```

Each request line is a numeric file ID, a space, and the project's page URL (which must match `https://(www.|beta.)curseforge.com/<game>/<category>/<slug>`).

The webview then opens the file page for each request in turn. Navigation is sandboxed: only pages for the requested file are allowed, `curseforge://` and other external links prompt the user, and unrelated links open in the system browser. A **Reload** and **Skip** menu are available; skipping a file advances to the next one without emitting output.

The host reads **stdout**:

```
curseforge_webview 0.1.0                          (version banner)
0 https://edge.forgecdn.net/files/.../mod.jar     (index + captured CDN URL)
1 https://media.forgecdn.net/files/.../other.jar
```

- Each `<index> <url>` line reports the download URL captured for the request at that (zero-based) index.
- On failure, a line reading `ERROR` is printed followed by error details.
- The process exits when every request has been downloaded or skipped, or when the window is closed.

## packwand GUI integration

The packwand GUI (`packwand gui`) bridges this protocol over HTTP + Server-Sent Events:

- `POST /api/webview/open` with `{"files": [{"file_id": 3643025, "slug": "jei"}]}` (or an explicit `"url"`) spawns the webview and returns a job ID.
- The job's event stream (`GET /api/jobs/{id}/events`) then carries a `DOWNLOAD <fileID> <url>` line for every captured file, live, followed by a summary line.
- The binary is located via `CURSEFORGE_WEBVIEW_BIN`, the in-repo cargo output (`lib/curseforge_webview/target/{release,debug}`), or `PATH`.

In the GUI's Mods view, CurseForge mods with a known file ID show a **CF Fetch** button that opens the webview for that mod and streams the captured URL into the Logs view.

## Licenses page

The About menu shows bundled third-party licenses from `src/licenses.html`. Regenerate it after dependency changes with `task gen-licenses` (or the commands in the README).
