FROM oven/bun:1-slim

WORKDIR /app

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY . .

RUN chmod +x docker-entrypoint.sh

ENV NODE_ENV=production

CMD ["./docker-entrypoint.sh"]
