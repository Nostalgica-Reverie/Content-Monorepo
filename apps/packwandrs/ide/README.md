# Packwand IDE fork

Packwand IDE is an in-monorepo fork of Code OSS 1.126.0 at commit
`7e7950df89d055b5a378379db9ee14290772148a`, with VSCodium's common privacy,
debranding, security, Copilot-removal, and onboarding-removal patches applied.
The complete patched source lives in `workbench/`; it is not a Monaco wrapper
and it does not depend on either supplied source drop at build or runtime.

The fork removes 62 built-in extensions and retains pack-authoring languages,
JSON/HTML/CSS/Markdown language services, media preview, merge conflict support,
and the Packwand extension. The Packwand extension provides the branded dark
theme, TOML and Minecraft function grammars, and `packeater.json` schema
validation. Product metadata contains no marketplace, account, Copilot,
onboarding, voice, or telemetry endpoints.

The web workbench is hosted inside the existing Vue route. A `packwand:` Code
OSS filesystem provider uses a narrow `postMessage` RPC bridge; Vue maps those
requests to binary-safe Tauri commands that remain confined to the selected
pack root. Code OSS owns the explorer, editor groups, tabs, search UI, settings,
undo/redo, and file operations. The standalone Monaco implementation has been
removed.

Build the generated (gitignored) `vscode-web/` distribution from
`apps/packwandrs`:

```sh
bun run ide:build
bun run build
```

The second command copies `ide/host/` and the generated distribution into the
Vite output. Development serving uses the same two directories through a
path-confined Vite middleware.

`scripts/apply-vscodium.ps1` verifies and patches a pristine external Code OSS
checkout when auditing the patch provenance. `scripts/prune-workbench.ps1`
reapplies the extension allowlist to a prepared fork, and
`scripts/prune-electron.ps1` removes the vendored Electron runtime and
packaging surface — the `electron-main`, `electron-browser`, and
`electron-utility` process layers, the desktop entry points, and the Electron
gulp tasks — which the web-only distribution never loads. Both run together via
`bun run ide:prune` and are re-runnable after an upstream resync. The original
`vscode-main/` and `vscodium-master/` source drops are not modified.
