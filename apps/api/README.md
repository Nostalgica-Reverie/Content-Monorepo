# cursorapi

`cursorapi` is a standalone host for Packwand's versioned manifest API. It
uses the same `GET /api/v1/packs`, OpenAPI, and bearer-token behavior as
`packwand api serve`.

From the repository root:

```sh
go run -C apps/api ./cursorapi --root .
```

It binds to `127.0.0.1:8097` by default. A non-loopback `--bind` requires
`--token-file`; add `--generate-token` to create that file securely when it
does not already exist.
