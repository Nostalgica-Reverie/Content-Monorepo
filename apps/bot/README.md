# Pineapple

The second Discord bot for the Reverie Projects Discord server. Forked from
[Modrinth's discord-bot](https://github.com/modrinth/discord-bot) (AGPL-3.0 —
see LICENSE); all credit to its original authors.

TypeScript on the [Bun](https://bun.sh) runtime (no build step — Bun runs the
sources directly), discord.js v14, Drizzle ORM on Postgres.

## Environment variables

See `.env.example` — the application ID and public key are pre-filled;
`.env.test` carries non-secret defaults for a local test guild. Server IDs
(guild, channels, roles, tags) must be filled in for the new Discord before
launch.

## Development

1. Copy `.env.example` to `.env` and fill it in.
2. Install deps and run in dev (watch) mode:

```
bun install
bun run dev
```

Register slash commands with Discord:

```
bun run deploy
```

Format and lint:

```
bun run fix
```

From the repository root you can also use the justfile entrypoints CI runs:
`just lint-bot`, `just build-bot` (typecheck + bundle smoke-test).

## Deployment

`Dockerfile` (oven/bun) + `docker-entrypoint.sh` build a self-contained image;
Drizzle migrations run on start. Build it with the monorepo root as context:
`docker build -f apps/bot/Dockerfile .`.
