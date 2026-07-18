# Modpack Dev Handbook

Single-app handbook for modpack-development guidance plus pack-management documentation.

## Local development

```bash
bun install
bun run check
bun run build
```

The packwand command reference is generated into the handbook route tree during `check` and `build`.

## Architecture

The handbook has three boundaries:

- `src/routes` is the content source of truth. Markdown and MDsveX pages are compiled into the static site.
- `scripts/prepare-docs.mts` is the content compiler. It owns navigation metadata and the compact, lazy-loaded search artifacts under `src/lib/generated`.
- Svelte components under `src/lib` are the runtime shell. They consume generated metadata but do not scan or rewrite content files.

Generated metadata should not be edited by hand. Run `bun run docs:index` after content changes, or `bun run docs:index:check` to verify that committed artifacts are current without modifying them. Packwand references are generated transactionally through `docs/packwand/build-commands.mts`, so a failed generation preserves the previous reference tree.