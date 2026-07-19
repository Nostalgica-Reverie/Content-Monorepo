FROM oven/bun:1.3.14-slim

WORKDIR /app

COPY package.json bun.lock ./
COPY apps/bot/package.json apps/bot/package.json
COPY docs/package.json docs/package.json
COPY docs/modpack-dev-handbook/package.json docs/modpack-dev-handbook/package.json
COPY docs/packwand/package.json docs/packwand/package.json
COPY docs/packwiz/package.json docs/packwiz/package.json
COPY tools/package.json tools/package.json
RUN bun install --frozen-lockfile --production --filter @reverie/pineapple

WORKDIR /app/apps/bot
COPY apps/bot/ .

RUN chmod +x docker-entrypoint.sh

ENV NODE_ENV=production

CMD ["./docker-entrypoint.sh"]
