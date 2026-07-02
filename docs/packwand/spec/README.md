# packwand-spec

A format for specifying Minecraft modpacks, designed to be easy to manipulate with tools. Derived from [packwiz-spec](https://github.com/packwiz/packwiz-spec); this copy documents the **packwand** dialect (`pack-format = "packwand:26"`), which remains compatible with legacy `packwiz:1.x` packs.

The human-readable specification lives in the packwand docs site at `docs/packwand/docs/reference/pack-format/` (pack.toml, index.toml, mod.pw.toml, .packwizignore). The JSON schemas in `schemas/` are the machine-readable form, suitable for editor integration (e.g. Taplo/Even Better TOML).

## packwand changes relative to packwiz-spec

- `pack-format` uses integer generations (`packwand:26`); `packwiz:1.x` accepted for compatibility
- Default hash format is `sha512` (was `sha256`); hashes may be omitted in `no-internal-hashes` mode
- `pack.toml`: `versions.neoforge` documented; new optional `[scripts]` table (`packwand run <name>`)
- `mod.pw.toml`: new `pin` field; `download.mode` documented (`url` / `metadata:curseforge`, so `download.url` is no longer unconditionally required); new update sources `[update.github]`, `[update.gitlab]`, `[update.forgejo]`

## Contributing

[Deno](https://deno.land/) is used to generate JSON schemas, with a custom DSL using TypeScript decorators. Run `deno task build` to re-generate schemas from the definitions in `src`.

> **Note:** the schemas in `schemas/` were updated by hand for the packwand dialect ahead of the generator sources. If you regenerate with `deno task build`, first port the changes listed above into `src/defs/`, or the hand-made updates will be overwritten.
