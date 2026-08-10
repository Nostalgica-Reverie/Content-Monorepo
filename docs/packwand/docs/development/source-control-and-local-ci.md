# Stacked changes and local CI

Packwand can layer Jujutsu's change model over an existing Git checkout and can
run the repository's real Tangled workflows before those changes are pushed.
Both features are optional: Git remains the interchange format, and local CI
works without an ATProto account.

## Stacked changes

Open **Source Control** in the desktop app. If the selected workspace has a
`.git` directory but no `.jj` directory, choose **Enable for this repository**.
This performs a colocated initialization, leaving Git and Jujutsu metadata side
by side. Packwand opens a repository only for the duration of each operation,
so the desktop app does not retain a lock while idle.

The same operations are available from the CLI:

```sh
packwand change enable
packwand change new
packwand change describe <change-id> 'Describe the change'
packwand change log
packwand change squash <change-id> --into-parent
```

Change IDs survive rewrites; commit IDs do not. A divergent change is shown as
an error that requires an explicit resolution rather than Packwand choosing a
candidate. Squashing removes the squashed change ID, so the desktop app asks
for confirmation.

Packwand pins both its embedded `jj-lib` API and managed standalone JJ tool to
version 0.41.0. Tool upgrades are reviewed independently because neither API is
treated as stable.

## Somnus workflows

Somnus reads `.tangled/workflows/*.yml`, evaluates the declared branch and path
triggers, checks dependencies on `PATH`, and runs each step's command in order.
The default run selects workflows using paths changed in the current JJ
working-copy change, falling back to `git diff --name-only HEAD` when JJ is not
enabled.

```sh
packwand somnus list
packwand somnus run
packwand somnus run .tangled/workflows/rust.yml
packwand somnus status --json
```

Somnus deliberately does not emulate Spindle's `microvm` or Nixery isolation.
Commands execute directly on the developer's machine with the user's own
permissions and inherited terminal streams. Review workflow files before
running them. The last result is stored in `.somnus/status.json`.

When Packwand has an active ATProto session, a successful CLI invocation also
attempts to publish `sh.tangled.pipeline.status` records to that identity's own
repository. Reporting failures are warnings and never turn an otherwise valid
offline run into a failure.

## Built-in launcher installation

The Rust `packwand-installer` is the native content installer used by Packwand's
installer validation and built-in launcher. **Install** and **Play** both run it
before Minecraft bootstrap. A failed installer exit marks the instance failed
and prevents the Java process from starting. It verifies declared hashes, uses
staged file replacement, observes side/optional/preserve rules, removes content
dropped by a later pack version, and keeps a content-addressed local cache.

The legacy Gradle installer remains available only for third-party launchers or
server setups whose established integration requires the Java bootstrap. Its
Prism/MultiMC contract does not apply to Packwand's own launcher.
