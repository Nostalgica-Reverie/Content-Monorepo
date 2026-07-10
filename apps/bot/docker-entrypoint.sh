#!/bin/sh
bun node_modules/drizzle-kit/bin.cjs migrate
exec bun src/index.ts
