# apps/

Deliverables — components that ship to users as a binary, jar, or bundled app.

| App | Language | What it is |
| --- | -------- | ---------- |
| [`packwand`](packwand/) | Go (+ Gleam UI, Tauri shell) | The Packwand CLI and GUI — our packwiz fork merged with the repo tooling |
| [`packwand-installer`](packwand-installer/) | Kotlin + legacy Java bootstrap | packwiz-installer fork used by shipped packs |
| [`mod-browser-webview`](mod-browser-webview/) | Rust (wry) | Embedded provider-browser window used by the GUI |
| [`bot`](bot/) | TypeScript (discord.js, Bun) | Pineapple, the Reverie Discord bot (forked from Modrinth's discord-bot) |
| [`api`](api/) | Go | `cursorapi`, a standalone host for Packwand's versioned manifest API |

Shared libraries live in [`packages/`](../packages/). See
[ARCHITECTURE.md](../ARCHITECTURE.md) for the full repository layout and
conventions.
