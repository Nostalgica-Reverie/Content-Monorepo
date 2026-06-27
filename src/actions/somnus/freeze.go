package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
)

func cmdFreeze(args []string) {
	asJSON := false
	var rest []string
	for _, a := range args {
		if a == "--json" {
			asJSON = true
		} else {
			rest = append(rest, a)
		}
	}
	if len(rest) < 1 {
		failUsage(verbUsage["freeze"])
	}
	subdir := absPath(strings.TrimRight(rest[0], "/"))
	slugs := rest[1:]
	packDir, subKey := splitPackSubdir(subdir)
	if len(slugs) == 0 {
		listFrozen(packDir, subKey, asJSON)
		return
	}
	applyFreeze(packDir, subKey, subdir, slugs, true)
}

func cmdUnfreeze(args []string) {
	if len(args) < 2 {
		failUsage(verbUsage["unfreeze"])
	}
	subdir := absPath(strings.TrimRight(args[0], "/"))
	packDir, subKey := splitPackSubdir(subdir)
	applyFreeze(packDir, subKey, subdir, args[1:], false)
}

func splitPackSubdir(subdir string) (packDir, subKey string) {
	subKey = filepath.Base(subdir)
	packDir = filepath.Dir(subdir)
	if !strings.HasSuffix(subKey, "-mr") && !strings.HasSuffix(subKey, "-cf") {
		failUsage(fmt.Sprintf("%q is not a pack subdir (expected a path ending in -mr or -cf, like modpacks/x/26.1.2-mr)", subdir))
	}
	if _, err := os.Stat(filepath.Join(packDir, "manifest.json")); err != nil {
		failNotFound(fmt.Sprintf("no manifest.json in %s — freeze records into the pack manifest", packDir))
	}
	return packDir, subKey
}

func listFrozen(packDir, subKey string, asJSON bool) {
	frozen := readAutomation(packDir).Freeze[subKey]
	if asJSON {
		if frozen == nil {
			frozen = []string{}
		}
		sort.Strings(frozen)
		data, _ := json.MarshalIndent(frozen, "", "  ")
		fmt.Println(string(data))
		return
	}
	if len(frozen) == 0 {
		fmt.Printf("no frozen mods declared for %s/%s.\n", packDir, subKey)
		return
	}
	sort.Strings(frozen)
	fmt.Printf("%d frozen mod(s) in %s/%s:\n", len(frozen), packDir, subKey)
	for _, s := range frozen {
		fmt.Printf("  - %s\n", s)
	}
}

func applyFreeze(packDir, subKey, subdir string, slugs []string, freeze bool) {
	if _, err := exec.LookPath(packwizBin()); err != nil {
		failEnv("packwiz not found", "install with 'go install github.com/packwiz/packwiz@latest' or point PACKWIZ_BIN at a binary")
	}

	verb, gerund := "pin", "freezing"
	if !freeze {
		verb, gerund = "unpin", "unfreezing"
	}

	failures := 0
	var applied []string
	for _, slug := range slugs {
		if _, err := os.Stat(filepath.Join(subdir, "mods", slug+".pw.toml")); err != nil {
			warnf("%s not found in %s (no mods/%s.pw.toml); skipped", slug, subdir, slug)
			continue
		}
		cmd := exec.Command(packwizBin(), verb, slug)
		cmd.Dir = subdir
		if out, err := cmd.CombinedOutput(); err != nil {
			fmt.Fprintf(os.Stderr, "  FAIL %s %s: %v\n%s", verb, slug, err, indent(string(out), "    "))
			failures++
			continue
		}
		fmt.Printf("  %s %s: %s\n", gerund, slug, subdir)
		applied = append(applied, slug)
	}

	if failures > 0 {
		fail(fmt.Sprintf("%d %s operation(s) failed; manifest NOT updated", failures, verb))
	}
	if len(applied) == 0 {
		fmt.Println("nothing changed.")
		return
	}

	current := map[string]bool{}
	for _, s := range readAutomation(packDir).Freeze[subKey] {
		current[s] = true
	}
	for _, s := range applied {
		if freeze {
			current[s] = true
		} else {
			delete(current, s)
		}
	}
	var list []string
	for s := range current {
		list = append(list, s)
	}
	sort.Strings(list)
	setAutomationFreeze(packDir, subKey, list)

	state := "frozen (updates will skip them)"
	if !freeze {
		state = "unfrozen (updates apply again)"
	}
	fmt.Printf("%d mod(s) %s; recorded in %s/manifest.json (automation.freeze.%s)\n", len(applied), state, packDir, subKey)
}

func modSubdirsOf(packDir string) []string {
	var out []string
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return nil
	}
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		if strings.HasSuffix(e.Name(), "-mr") || strings.HasSuffix(e.Name(), "-cf") {
			out = append(out, filepath.Join(packDir, e.Name()))
		}
	}
	return out
}

func pinDrift(packDir string, freezeMap map[string][]string) []string {
	var drift []string
	for subKey, slugs := range freezeMap {
		for _, slug := range slugs {
			p := filepath.Join(packDir, subKey, "mods", slug+".pw.toml")
			data, err := os.ReadFile(p)
			if err != nil {
				continue
			}
			if !strings.Contains(string(data), "pin = true") && !strings.Contains(string(data), "pin=true") {
				drift = append(drift, p)
			}
		}
	}
	return drift
}
