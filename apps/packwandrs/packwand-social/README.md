# Packwand Social

`packwand-social` is Packwand's local ATProto OAuth client and XRPC bridge. It
signs in to an identity's existing PDS; Packwand does not host a PDS or hold an
account password.

The `login`, `whoami`, and `logout` commands are one-shot operations. `serve`
binds a bearer-token-protected API to loopback for the Rust identity client.
OAuth sessions are stored in the user's config directory with owner-only file
permissions. Set `PACKWAND_SOCIAL_STATE_DIR` to isolate that state in tests.

The bridge owns Packwand's custom Lexicons under `lexicons/` and exposes typed
operations for record creation, image blob upload, mutual-follow plus contact
discovery, addressed invite polling, and read-only Tangled repository lookup
through Bobbin. Public service roots can be replaced in integration tests with
`PACKWAND_SOCIAL_APPVIEW_URL` and `PACKWAND_SOCIAL_BOBBIN_URL`.

The production OAuth metadata is served from
`https://packwand.nostalgica.net/oauth/client-metadata.json`, and the callback
is pinned to `http://127.0.0.1:38427/callback` as required by ATProto OAuth.

```console
packwand-social login
packwand-social whoami
packwand-social serve --token-file token --generate-token --print-port-file port
packwand-social logout
```
