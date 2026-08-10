# Live collaboration

The Packwand desktop app can share one pack as a host-authoritative live session. The guest does not clone or copy the workspace: file operations, accepted text edits, and approved git operations run against the host's selected pack.

## Start or join a session

Open **Live Collaboration** from the activity rail.

- To host, select a pack target and choose **Start session**. Copy the generated `pw://` invite and send it to the guest through a trusted channel.
- To invite an ATProto friend, sign in from the **Friends** section after starting the session, then choose **Send invite** beside a mutual follow or Packwand contact. The invite is written to your own ATProto repository and addressed to that friend's DID.
- To join, paste the invite into **Invite code** and choose **Join session**.
- Addressed, unexpired ATProto invites appear under **Pending invites** and join through the same existing collaboration flow.
- Choose **Follow** beside the other participant to open the file and reveal the selection they are currently using.
- The host can turn guest stage and commit access on or off while the session is running.

An invite has the form `pw://<host>:<port>#<key>`. The fragment contains a randomly generated 32-byte pre-shared key. Anyone who has the complete invite can authenticate to the session, so treat it like a temporary password and stop the session when it is no longer needed.

## ATProto identity and discovery

Packwand is an OAuth client for any existing ATProto account; it does not operate a PDS or create a separate Packwand account. OAuth credentials remain encrypted in the local social helper and never enter the webview.

The Friends section combines mutual `app.bsky.graph.follow` relationships with explicit `net.nostalgica.packwand.contact` records. Invite discovery reads `net.nostalgica.packwand.session.invite` records from those friends' PDSes, then filters locally for the signed-in DID and expiration time. This is bounded polling, not a firehose subscription.

The CLI exposes the same path:

```sh
packwand account login
packwand friends list
packwand friends invite did:plc:example 'pw://host:port#key'
```

Pack summaries, snippets, and images can also be published to the signed-in repository with `packwand share pack`, `packwand share snippet`, and `packwand share image`. Pack records can embed a strong reference to a linked Tangled repository; Packwand reads those links through Tangled's Bobbin AppView.

For colocated Jujutsu changes and local Tangled workflow execution, see [Stacked changes and local CI](./source-control-and-local-ci.md).

## Shared behavior

The guest sees the host's selected pack and works against the host's checkout. The session provides:

- directory browsing, file reads, writes, creates, deletes, renames, and search within the selected pack;
- live text edits with host-authoritative revision ordering and operational-transform reconciliation;
- remote cursors, selections, and follow mode;
- host-executed status, diff, log, branches, stage, unstage, and commit operations;
- `Co-authored-by:` trailers for guests whose accepted document edits contributed to a commit;
- mirrored host output, problems, and job progress.

Repository creation, checkout, identity/config changes, remote changes, fetch, pull, and push remain host-only. Guests also cannot start or cancel host jobs.

## Security model

The collaboration socket lives entirely in Rust. The webview receives no network capability, remote Tauri grant, or additional Content Security Policy permission.

The wire uses `Noise_NNpsk0_25519_ChaChaPoly_BLAKE2s`. Authentication completes before the first application message is decoded, and encrypted frames have a hard size limit.

Remote operations are not generic Tauri invocations. The protocol exposes closed filesystem and git allowlists. File requests are pinned to the pack selected when hosting and pass through all three local confinement checks:

1. pack-root resolution;
2. relative-path validation;
3. safe joining that rejects traversal and symlinks.

Account/keychain, shell, settings, instance, theme, workspace, and Packeater commands have no remotely callable protocol representation.

## Connection behavior

Version 1 accepts one guest at a time. If the guest disconnects, the host listener remains available for a replacement connection. A guest whose host disappears moves to a disconnected state, and pending filesystem or git requests fail rather than waiting indefinitely.

Changing or corrupting the invite key causes the Noise handshake to fail before any collaboration message is read.

## Development checks

From `apps/packwandrs`, run the collaboration crate and desktop tests:

```sh
cargo test -p packwand-collab
cargo test -p packwand-gui --lib
```

Then verify the frontend and the unchanged capability boundary:

```sh
cd frontend
bunx vue-tsc --noEmit
bunx vite build
bun test tests
bun scripts/audit-capabilities.mjs
cd core
gleam test --target javascript
```

Use two desktop instances for the final behavior check. Test simultaneous edits, follow mode, guest stage/commit, host output mirroring, abrupt host termination, a corrupted invite key, and a raw non-Noise TCP client.
