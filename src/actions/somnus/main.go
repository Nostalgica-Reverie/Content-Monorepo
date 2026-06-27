package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

const somnusVersion = "26.1-dev"

const usageText = `somnus CLI tool %s
packwiz: custom patched build — run 'somnus packwiz build' to compile from source

usage: somnus <verb> [args]

content
  init <category> <name> [flags]      sets up a new pack with a manifest, changelog, and packwiz subdirs
  bump <pack-dir> <new-version>       bumps the manifest version (--configs: also bumps in-pack version files)
  add <slug> [subdir] [--no-refresh]  adds a mod by slug, routes to modrinth or curseforge by subdir suffix
  automation get <pack-dir>           spits out the effective automation settings for a pack as json
  packs list|get|set                  look up or tweak any pack's manifest fields by id
  freeze <pack-dir> [mods...]         pins mods so updates skip them — no args lists what's frozen
  unfreeze <pack-dir> <mods...>       unpins mods so they're fair game for updates again
  port <mr-subdir> <cf-subdir>        shows which mr mods are missing on the cf side (--add to fill them in)
  import <url-or-file> [--id <id>]    pulls a modrinth mrpack or curseforge zip in as a new pack
  side <pack-dir> <mod> [side]        checks or fixes a mod's side across all subdirs in a pack
  test <pack-subdir>                  spins up packwiz serve and installs into a local test instance

build & docs
  export [pack]                       builds changed (or one named) pack locally into artifacts/
  build <sha> | --pack <name> <sha>   ci-style build of git-changed packs tagged with the sha
  modlist <pack-subdir>               writes the crash-assistant modlist.json for a subdir
  pages [pack]                        writes modlist.md files and projects.json for the docs site
  publish <mode> <manifest> [variant] runs the release pipeline — please don't poke this locally

maintenance
  update [--check]                    runs packwiz update --all across every pack subdir (--check: dry run)
  refresh                             runs packwiz refresh across every pack subdir
  loader-update [latest|recommended]  migrates loaders across all packs
  sync [--dry-run]                    copies performance base content into consumer packs
  lint [files...]                     checks changed json and pw.toml files for syntax errors
  validate <manifest> [--all]         validates manifests — fields, subdirs, changelog, role, automation
  status [--json]                     shows a dashboard of all packs with version, mods, and frozen counts
  diff <old-ref> <new-ref> [path]     shows mod changes between two git refs

meta
  packwiz build [--output <path>]     clones packwiz at the pinned sha, applies patches, and builds it
  doctor                              checks that tools, repo root, and manifests are all looking good
  completion bash|fish|zsh            prints shell completion script — eval it in your rc file
  help [verb]                         shows this text, or detailed usage for a specific verb
  version                             prints the somnus version

aliases: docs -> pages, instance -> test, check -> doctor

exit codes: 0 ok | 1 runtime failure | 2 usage | 3 environment | 4 not found
`

func usage() string {
	return fmt.Sprintf(usageText, somnusVersion)
}

var startCwd string

func main() {
	startCwd, _ = os.Getwd()
	if len(os.Args) < 2 {
		printMascot()
		fmt.Print(usage())
		return
	}
	verb := canonicalVerb(os.Args[1])

	switch verb {
	case "help", "-h", "--help":
		if len(os.Args) > 2 {
			printVerbHelp(canonicalVerb(os.Args[2]))
			return
		}
		printMascot()
		fmt.Print(usage())
		return
	case "version", "-v", "--version":
		printMascot()
		fmt.Println("somnus " + somnusVersion)
		return
	}

	switch verb {
	case "init", "bump", "add", "side", "freeze", "unfreeze", "test", "modlist", "port",
		"export", "build", "sync", "update", "refresh", "loader-update", "lint",
		"pages", "packs", "import", "publish", "validate", "automation", "packwiz",
		"status", "diff":
		if root := findRepoRoot(); root != "" {
			if err := os.Chdir(root); err != nil {
				fail(fmt.Sprintf("failed to enter repo root %s: %v", root, err))
			}
		} else {
			fail("could not locate repo root (no .git or modpacks/ found walking up from here)")
		}
	}

	switch verb {
	case "init":
		cmdInit(os.Args[2:])
	case "bump":
		cmdBump(os.Args[2:])
	case "add":
		cmdAdd(os.Args[2:])
	case "packs":
		cmdPacks(os.Args[2:])
	case "side":
		cmdSide(os.Args[2:])
	case "import":
		cmdImport(os.Args[2:])
	case "publish":
		cmdPublish(os.Args[2:])
	case "freeze":
		cmdFreeze(os.Args[2:])
	case "unfreeze":
		cmdUnfreeze(os.Args[2:])
	case "export":
		cmdExport(os.Args[2:])
	case "build":
		cmdBuild(os.Args[2:])
	case "sync":
		cmdSync(os.Args[2:])
	case "update":
		cmdUpdate(os.Args[2:])
	case "refresh":
		cmdRefresh(os.Args[2:])
	case "loader-update":
		cmdLoaderUpdate(os.Args[2:])
	case "modlist":
		cmdModlist(os.Args[2:])
	case "pages":
		cmdPages(os.Args[2:])
	case "test":
		cmdTest(os.Args[2:])
	case "lint":
		cmdLint(os.Args[2:])
	case "validate":
		cmdValidate(os.Args[2:])
	case "automation":
		cmdAutomation(os.Args[2:])
	case "port":
		cmdPort(os.Args[2:])
	case "packwiz":
		cmdPackwiz(os.Args[2:])
	case "doctor":
		cmdDoctor(os.Args[2:])
	case "status":
		cmdStatus(os.Args[2:])
	case "diff":
		cmdDiff(os.Args[2:])
	case "completion":
		cmdCompletion(os.Args[2:])
	default:
		failUsage(fmt.Sprintf("unknown verb %q", os.Args[1]))
	}
}

var verbAliases = map[string]string{
	"docs":     "pages",
	"instance": "test",
	"check":    "doctor",
}

func canonicalVerb(v string) string {
	if c, ok := verbAliases[v]; ok {
		return c
	}
	return v
}

func printVerbHelp(verb string) {
	if u, ok := verbUsage[verb]; ok {
		fmt.Println(u)
		return
	}
	fmt.Printf("no detailed help for %q — run 'somnus help' for the full list\n", verb)
}

func findRepoRoot() string {
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for {
		if _, err := os.Stat(filepath.Join(dir, ".git")); err == nil {
			return dir
		}
		if info, err := os.Stat(filepath.Join(dir, "modpacks")); err == nil && info.IsDir() {
			return dir
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return ""
		}
		dir = parent
	}
}

func modpacksDir() string {
	if d := os.Getenv("MODPACKS_DIR"); d != "" {
		return d
	}
	return "modpacks"
}

func packwizBin() string {
	if b := os.Getenv("PACKWIZ_BIN"); b != "" {
		return b
	}
	return "packwiz"
}


func absPath(p string) string {
	if p == "" || filepath.IsAbs(p) {
		return p
	}
	return filepath.Join(startCwd, p)
}

func writeJSON(path string, v any) {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		fail(fmt.Sprintf("failed to marshal JSON: %v", err))
	}
	data = append(data, '\n')
	if err := os.WriteFile(path, data, 0o644); err != nil {
		fail(fmt.Sprintf("failed to write %s: %v", path, err))
	}
}
