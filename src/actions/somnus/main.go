package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

func main() {
	if len(os.Args) < 2 {
		fail("usage: somnus <init|bump|export|build|sync|update|refresh|loader-update|modlist|pages|test|lint|port> [args]")
	}

	switch os.Args[1] {
	case "export", "build", "sync", "update", "refresh", "loader-update", "lint", "pages":
		if root := findRepoRoot(); root != "" {
			if err := os.Chdir(root); err != nil {
				fail(fmt.Sprintf("failed to enter repo root %s: %v", root, err))
			}
		} else {
			fail("could not locate repo root (no .git or modpacks/ found walking up from here)")
		}
	}

	switch os.Args[1] {
	case "init":
		cmdInit(os.Args[2:])
	case "bump":
		cmdBump(os.Args[2:])
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
	default:
		fail(fmt.Sprintf("unknown verb %q (expected init, bump, export, build, sync, update, refresh, loader-update, modlist, pages, test, lint, or port)", os.Args[1]))
	}
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

func fail(msg string) {
	fmt.Fprintf(os.Stderr, "::error::%s\n", msg)
	os.Exit(1)
}
