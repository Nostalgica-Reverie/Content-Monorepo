# Somnus

Somnus runs the repository's real `.tangled/workflows/*.yml` steps locally so
developers can see approximately what Tangled's Spindle will run after a push.
It evaluates branches and changed-path globs, verifies declared dependencies,
stops on the first failing step, and stores the last result in
`.somnus/status.json`.

This MVP deliberately does not reproduce Spindle isolation. A workflow that
declares `engine: microvm` and `image: nixos` still runs as ordinary child
processes on the developer's host, with their stdin/stdout/stderr inherited.
Commands therefore have the same authority as the user running Somnus. Full
QEMU/Nixery parity is a separate follow-up.

When invoked through `packwand somnus run`, changed paths come from Jujutsu's
working-copy change with a Git fallback. If an ATProto identity is signed in,
the CLI also attempts to publish `sh.tangled.pipeline.status` records; an
offline run remains fully supported.
