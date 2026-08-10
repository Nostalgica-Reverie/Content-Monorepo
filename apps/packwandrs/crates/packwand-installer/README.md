# Packwand installer

This is the native successor to the legacy Java `packwiz-installer` fork. It
installs pack content transactionally, verifies every declared hash, and keeps
the Gradle/JVM implementation available for compatibility with external tools.

Packwand's built-in launcher is the primary integration. It invokes this binary
before Minecraft bootstrap and treats a non-zero exit status as a hard launch
failure, so partially installed content is never launched. The standalone
command remains available for servers and external integrations:

```text
packwand-installer --side client https://example.invalid/pack.toml
```

Prism, MultiMC, and other third-party launchers may continue using the legacy
Java bootstrap where their existing instance contract requires a JVM task.
That compatibility path does not constrain Packwand's built-in launcher.
