package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func cmdPort(args []string) {
	if len(args) < 2 {
		fail("usage: somnus port <mr-subdir> <cf-subdir> [--add]\n" +
			"  e.g. somnus port modpacks/rc-plus/26.1.2-mr modpacks/rc-plus/26.1.2-cf")
	}
	mrDir, cfDir := args[0], args[1]
	doAdd := false
	for _, a := range args[2:] {
		if a == "--add" {
			doAdd = true
		}
	}

	mrMods := filepath.Join(mrDir, "mods")
	if info, err := os.Stat(mrMods); err != nil || !info.IsDir() {
		fail(fmt.Sprintf("no mods/ in MR subdir %s", mrDir))
	}
	if _, err := os.Stat(filepath.Join(cfDir, "pack.toml")); err != nil {
		fail(fmt.Sprintf("CF subdir %s has no pack.toml (run packwiz/somnus init there first)", cfDir))
	}
	if doAdd {
		if _, err := exec.LookPath("packwiz"); err != nil {
			fail("packwiz not found in PATH")
		}
	}

	entries, err := os.ReadDir(mrMods)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", mrMods, err))
	}
	var mrNames []string
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".pw.toml") {
			mrNames = append(mrNames, strings.TrimSuffix(e.Name(), ".pw.toml"))
		}
	}

	cfMods := filepath.Join(cfDir, "mods")
	existing := make(map[string]bool)
	if cfEntries, err := os.ReadDir(cfMods); err == nil {
		for _, e := range cfEntries {
			if !e.IsDir() && strings.HasSuffix(e.Name(), ".pw.toml") {
				existing[strings.TrimSuffix(e.Name(), ".pw.toml")] = true
			}
		}
	}

	var missing []string
	for _, n := range mrNames {
		if !existing[n] {
			missing = append(missing, n)
		}
	}

	fmt.Printf("MR mods: %d | already on CF: %d | missing on CF: %d\n",
		len(mrNames), len(mrNames)-len(missing), len(missing))
	if len(missing) == 0 {
		fmt.Println("nothing to port \u2014 CF side already has matching slugs for every MR mod.")
		return
	}

	if !doAdd {
		fmt.Println("\nmods needing a CF entry (re-run with --add to add them interactively):")
		for _, n := range missing {
			fmt.Printf("  - %s\n", n)
		}
		fmt.Println("\nNote: 'missing' is matched by .pw.toml slug. Some may be Modrinth-only")
		fmt.Println("(no CF release) \u2014 those will simply not be found when you --add, which is fine.")
		return
	}

	fmt.Printf("\nadding %d mod(s) to %s via packwiz (you confirm each match; no -y)\n", len(missing), cfDir)
	var added, skipped, notFound []string
	for _, n := range missing {
		fmt.Printf("\n=== %s ===\n", n)
		cmd := exec.Command("packwiz", "curseforge", "add", n)
		cmd.Dir = cfDir
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "  (packwiz add did not complete for %s: %v)\n", n, err)
			notFound = append(notFound, n)
			continue
		}
		if _, err := os.Stat(filepath.Join(cfMods, n+".pw.toml")); err == nil {
			added = append(added, n)
		} else {
			skipped = append(skipped, n)
		}
	}

	fmt.Printf("\nport summary for %s:\n", cfDir)
	fmt.Printf("  added: %d\n", len(added))
	fmt.Printf("  skipped (declined/no file written): %d\n", len(skipped))
	fmt.Printf("  not found / errored: %d\n", len(notFound))
	if len(notFound) > 0 {
		fmt.Println("  these likely have no CurseForge release (Modrinth-only) \u2014 handle manually:")
		for _, n := range notFound {
			fmt.Printf("    - %s\n", n)
		}
	}
	fmt.Println("\nremember to run packwiz refresh in the CF subdir and verify the matches are correct.")
}
