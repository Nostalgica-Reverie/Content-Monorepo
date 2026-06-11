package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

const somnusVersion = "26.1-dev"

const usageText = `somnus CLI tool %s

usage: somnus <verb> [args]

content
  init <category> <name> [flags]      scaffold a pack (manifest, changelog, packwiz subdirs, .packwizignore)
  bump <pack-dir> <new-version>       set a pack's manifest version (--configs: also in-pack version files)
  packs list|get|set                  the pack registry: every manifest as an addressable object
  freeze <pack-dir> [mods...]         pin mods across a whole pack so updates skip them (no args: list)
  unfreeze <pack-dir> <mods...>       unpin previously frozen mods
  port <mr-subdir> <cf-subdir>        diff MR mods against the CF side (--add to port interactively)
  test <pack-subdir>                  packwiz serve + install into a local test instance

build & docs
  export [pack]                       build changed (or one named) pack locally
  build <sha> | --pack <name> <sha>   CI build of git-changed packs (or one named pack)
  modlist <pack-subdir>               write crash-assistant modlist.json for a subdir
  pages [pack]                        write modlist.md files; full runs also emit projects.json

maintenance
  update                              packwiz update --all in every pack subdir
  refresh                             packwiz refresh in every pack subdir
  loader-update [latest|recommended]  migrate loaders across all packs
  sync [--dry-run]                    propagate performance bases into consumers
  lint [files...]                     syntax-lint changed JSON / .pw.toml files

meta
  doctor                              check tools, repo root, and manifest health (alias: check)
  help [verb]                         show this help, or detailed usage for one verb
  version                             print the somnus version

aliases: docs -> pages, instance -> test, check -> doctor

exit codes: 0 ok | 1 runtime failure | 2 usage | 3 environment | 4 not found
`

func usage() string {
	return fmt.Sprintf(usageText, somnusVersion)
}

func main() {
	if len(os.Args) < 2 {
		fmt.Fprint(os.Stderr, usage())
		os.Exit(1)
	}
	verb := canonicalVerb(os.Args[1])

	switch verb {
	case "help", "-h", "--help":
		if len(os.Args) > 2 {
			printVerbHelp(canonicalVerb(os.Args[2]))
			return
		}
		fmt.Print(usage())
		return
	case "version", "-v", "--version":
		fmt.Println("somnus " + somnusVersion)
		return
	}

	switch verb {
	case "export", "build", "sync", "update", "refresh", "loader-update", "lint", "pages", "packs":
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
	case "packs":
		cmdPacks(os.Args[2:])
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
	case "port":
		cmdPort(os.Args[2:])
	case "doctor":
		cmdDoctor(os.Args[2:])
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

type manifest struct {
	ID       string          `json:"id"`
	Version  string          `json:"version"`
	Variants []variant       `json:"variants,omitempty"`
	Role     json.RawMessage `json:"role"`
}

type variant struct {
	MCVersion string `json:"mc_version"`
	ID        string `json:"id,omitempty"`
}

func readManifest(path string) (*manifest, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to open %s: %w", path, err)
	}
	var m manifest
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, fmt.Errorf("invalid JSON in %s: %w", path, err)
	}
	return &m, nil
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
