# SparkUniverse tooling — what we can use

*Research note, July 2026. SparkUniverse (formerly EssentialGG) maintains the
infrastructure behind the Essential mod. This maps their repos onto our
platform: the packs + packwand toolchain in this monorepo, and the mod family
(Legacy4J, Legacy Skins, …) developed in separate repos but documented here
(`docs/docs/mods/`).*

## Principles adopted into this repo (July 2026)

Beyond library reuse, SparkUniverse's stack encodes engineering principles
that translate directly to this platform. Two are now implemented:

### 1. Variants are build targets, not forks → `packwand parity`
Their preprocessor treats each Minecraft version as a *compilation target* of
one source tree, with `api-validation` catching accidental divergence between
targets. Our analog of "targets" is a pack's `-mr`/`-cf` variant pair — and
nothing was checking that both platforms actually ship the same content.
`packwand parity` now diffs every pair (matching mods across platform slug
and naming differences, by slug → display name → jar filename), and the
weekly drift-report workflow includes a parity table. First run found real
drift in all 8 pairs, including a silent platform substitution
(`no-chat-reports` on CurseForge vs `no-chat-restrictions` on Modrinth).

### 2. Self-updated code is a trust boundary → bootstrap verification
EssentialLoader ships a tiny stable stub that verifies what it self-updates.
Our `packwiz-bootstrap` executed the downloaded installer jar unverified
unless someone remembered `--sha256`. Verification is now the default path:
`--sha256` wins, then `--checksums-url`, then a `<download-url>.sha256`
sibling is probed automatically (CI now publishes that sibling next to the
installer jar); running unverified prints a prominent warning unless
`--no-verify` acknowledges it.

Two more principles we already follow, worth keeping deliberate:

- **Convention-over-configuration build defaults** (`gg.essential.defaults`)
  — our equivalents are the root Taskfile + composite `setup-*` actions +
  repo-wide rustfmt/clippy/editorconfig. New components must plug into
  those, not roll their own.
- **Fast paths that skip the heavyweight runtime** (UniversalCraft's
  standalone edition) — our equivalents: the GUI's browser-served frontend,
  launcher-core tests that don't boot Minecraft (`#[ignore]`d real-boot
  tests), and the Go characterization suite. Preserve them; they're what
  keeps iteration fast.

## Directly valuable — for the mod repos

Legacy4J's core problem — "adapt Legacy Console Edition features to modern
Minecraft versions" — is exactly the multi-version problem SparkUniverse's
stack was built to solve. Today that usually means per-version branches and
manual backports; their stack collapses it into one codebase:

### essential-gradle-toolkit (adopt first)
Gradle plugins that make one source tree compile for many Minecraft versions
and loaders (Forge/Fabric, 1.8.9 → modern) via architectury-loom + the
ReplayMod preprocessor:
- `gg.essential.multi-version` / `.root` — the version-tree build structure;
  one `src/`, N artifacts.
- `.api-validation` — catches accidental ABI breaks between versions (great
  for Legacy Skins depending on Legacy4J's API).
- `gg.essential.defaults` + Prebundle/RelocationTransform — sane
  Java/Loom/publishing defaults and dependency bundling with relocation.

**Action:** trial it in the Legacy4J repo on the next MC-version bump instead
of cutting another version branch. Requires their Maven
(`repo.essential.gg`) + Architectury Maven, and their architectury-loom fork.

### UniversalCraft (adopt with the toolkit)
Java interop library wrapping Minecraft classes across 1.8.9–1.21.x and
Forge/Fabric/NeoForge, so version-specific code shrinks to preprocessor
islands. Two features matter specifically for us:
- **Standalone edition** (no Minecraft dependency): run and test GUI code
  without booting the game — Legacy4J's console-style UI could get fast
  iteration and CI-testable rendering logic.
- NeoForge coverage — where modern packs are heading.
Note for Forge targets: classes must be shadow-relocated to avoid conflicts.

### Elementa + Vigilance (evaluate)
Declarative GUI library and a config-screen library built on it (both on
UniversalCraft). Legacy4J hand-builds a lot of custom UI; Elementa's
constraint-based declarative model would reduce that code substantially, and
Vigilance gives settings screens for free. Evaluate against Legacy4J's
existing UI layer before committing — a UI-toolkit migration is only worth it
for new screens or a planned rewrite. Licensing (GPL/LGPL family) is
compatible with the mods' licenses.

## Pattern-valuable — for this monorepo

### EssentialLoader → packwand-installer / packwiz-bootstrap
Their loader is a three-stage design: a tiny, essentially-never-changing
stage0 stub ships inside the mod jar; it downloads/updates stage1, which
manages stage2 (the real logic). Plus: when two mods ship the same dependency,
the newer wins (jar-in-jar dedup).

Our `packwiz-installer-bootstrap` + `packwiz-installer` split is already a
two-stage version of this. Worth stealing:
- **Keep the shipped stage dumb and stable.** Anything shipped inside packs
  (the bootstrap jar / Go bootstrap) should change as rarely as possible;
  all evolving logic belongs in the self-updated stage (installer jar).
- **Verify what you self-update.** *Implemented — see "Principles adopted"
  above: packwiz-bootstrap now verifies downloads by default via explicit
  hash, checksums file, or auto-probed `.sha256` sibling.*

### EssentialInstaller / PartnerModIntegration (no action)
Their standalone GUI installer is the analog of the Packwand GUI + installer
we already ship. PartnerModIntegration is a documented third-party
integration surface — the role our versioned packwand HTTP API spec already
plays. Nothing to adopt beyond validation that our approach matches theirs.

### architectury-loom fork (transitively)
Only needed as the backend of essential-gradle-toolkit; pin whatever version
the toolkit requires, don't adopt independently.

## Suggested order

1. Legacy4J repo: trial `essential-gradle-toolkit` + UniversalCraft on one
   additional MC version target (biggest leverage, isolated risk).
2. ~~This repo: installer-update verification~~ — done (see "Principles
   adopted"), along with `packwand parity` for variant-target validation.
3. Legacy4J UI: prototype one new screen in Elementa (+ Vigilance for config)
   using UniversalCraft's standalone edition for fast iteration; decide on
   wider adoption from that.
4. Triage the parity findings: each mr-only/cf-only entry is either a
   platform-availability gap (fine, consider documenting) or an accidental
   omission (fix); the weekly report keeps them visible either way.
