# Packwand VCS

`packwand-vcs` is Packwand's typed boundary around Jujutsu stacked changes. It
pins `jj-lib` exactly and deliberately keeps no repository handle in application
state. Every operation opens the workspace, runs one bounded command, and drops
all state before returning so the standalone `jj` CLI can coexist safely.
