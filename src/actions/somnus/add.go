package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func cmdAdd(args []string) {
	if len(args) < 1 {
		failUsage(verbUsage["add"])
	}
	slug := args[0]

	noRefresh := false
	var targetArg string

	for _, a := range args[1:] {
		switch {
		case a == "--no-refresh":
			noRefresh = true
		case a == "--all":
			targetArg = ""
		case !strings.HasPrefix(a, "-"):
			targetArg = absPath(strings.TrimRight(a, "/"))
		}
	}

	if _, err := exec.LookPath(packwizBin()); err != nil {
		failEnv("packwiz not found", "install or set PACKWIZ_BIN")
	}

	targets := resolveAddTargets(targetArg)
	if len(targets) == 0 {
		fmt.Println("no pack subdirs found")
		return
	}

	fmt.Printf("adding %q to %d subdir(s)\n\n", slug, len(targets))
	added, failed, skipped := 0, 0, 0

	for _, dir := range targets {
		plat := platformSuffix(dir)
		var pwArgs []string
		switch plat {
		case "mr":
			pwArgs = []string{"modrinth", "add", "-y", slug}
		case "cf":
			pwArgs = []string{"curseforge", "add", "-y", slug}
		default:
			warnf("skipping %s — unrecognised suffix (need -mr or -cf)", dir)
			skipped++
			continue
		}
		if noRefresh {
			pwArgs = append(pwArgs, "--no-refresh")
		}

		fmt.Printf("[%s] %s\n", plat, dir)
		cmd := exec.Command(packwizBin(), pwArgs...)
		cmd.Dir = dir
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			warnf("%s: add failed — slug may not exist on %s under this name", dir, plat)
			failed++
			continue
		}
		added++
	}

	fmt.Printf("\nadd summary: %d added  %d not found/failed  %d skipped\n", added, failed, skipped)
	if failed > 0 && skipped < len(targets) {
		fmt.Printf("note: failures are expected when a mod has no release on that platform\n")
	}
	if added > 0 {
		autoLintDirs(targets)
	}
}

func resolveAddTargets(targetArg string) []string {
	if targetArg == "" {
		// all packs
		root := modpacksDir()
		entries, err := os.ReadDir(root)
		if err != nil {
			fail(fmt.Sprintf("failed to read %s: %v", root, err))
		}
		var out []string
		for _, e := range entries {
			if e.IsDir() {
				out = append(out, modSubdirsOf(filepath.Join(root, e.Name()))...)
			}
		}
		return out
	}

	base := filepath.Base(targetArg)
	if strings.HasSuffix(base, "-mr") || strings.HasSuffix(base, "-cf") {
		// specific subdir
		if _, err := os.Stat(targetArg); err != nil {
			failNotFound(fmt.Sprintf("subdir not found: %s", targetArg))
		}
		return []string{targetArg}
	}

	// pack directory — enumerate its subdirs
	if _, err := os.Stat(filepath.Join(targetArg, "manifest.json")); err != nil {
		failNotFound(fmt.Sprintf("no manifest.json in %s — pass a pack dir, a subdir, or nothing for all packs", targetArg))
	}
	return modSubdirsOf(targetArg)
}
