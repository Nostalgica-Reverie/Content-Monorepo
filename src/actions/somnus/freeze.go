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
	if len(args) < 1 {
		failUsage(verbUsage["freeze"])
	}
	packDir := args[0]
	slugs := args[1:]
	if len(slugs) == 0 {
		listFrozen(packDir)
		return
	}
	applyFreeze(packDir, slugs, true)
}

func cmdUnfreeze(args []string) {
	if len(args) < 2 {
		failUsage(verbUsage["unfreeze"])
	}
	applyFreeze(args[0], args[1:], false)
}

func listFrozen(packDir string) {
	o := readOptOut(packDir)
	if len(o.Freeze) == 0 {
		fmt.Printf("no frozen mods declared for %s.\n", packDir)
		return
	}
	sort.Strings(o.Freeze)
	fmt.Printf("%d frozen mod(s) in %s:\n", len(o.Freeze), packDir)
	for _, s := range o.Freeze {
		fmt.Printf("  - %s\n", s)
	}
}

func applyFreeze(packDir string, slugs []string, freeze bool) {
	if _, err := os.Stat(filepath.Join(packDir, "manifest.json")); err != nil {
		failNotFound(fmt.Sprintf("no manifest.json in %s — freeze operates on a pack directory", packDir))
	}
	if _, err := exec.LookPath(packwizBin()); err != nil {
		failEnv("packwiz not found", "install with 'go install github.com/packwiz/packwiz@latest' or point PACKWIZ_BIN at a binary")
	}

	subdirs := modSubdirsOf(packDir)
	if len(subdirs) == 0 {
		failNotFound(fmt.Sprintf("no -mr/-cf subdirs in %s", packDir))
	}

	verb := "pin"
	gerund := "freezing"
	if !freeze {
		verb = "unpin"
		gerund = "unfreezing"
	}

	failures := 0
	for _, slug := range slugs {
		fmt.Printf("%s %s ...\n", gerund, slug)
		applied := 0
		for _, sub := range subdirs {
			if _, err := os.Stat(filepath.Join(sub, "mods", slug+".pw.toml")); err != nil {
				continue
			}
			cmd := exec.Command(packwizBin(), verb, slug)
			cmd.Dir = sub
			if out, err := cmd.CombinedOutput(); err != nil {
				fmt.Fprintf(os.Stderr, "  FAIL %s in %s: %v\n%s", verb, sub, err, indent(string(out), "    "))
				failures++
				continue
			}
			fmt.Printf("  %s: %s\n", verb, sub)
			applied++
		}
		if applied == 0 {
			fmt.Fprintf(os.Stderr, "::warning::%s not found in any subdir of %s (checked mods/%s.pw.toml)\n", slug, packDir, slug)
		}
	}

	if failures > 0 {
		fail(fmt.Sprintf("%d %s operation(s) failed; opt-out.json NOT updated", failures, verb))
	}

	updateFreezeRecord(packDir, slugs, freeze)
	state := "frozen (updates will skip them)"
	if !freeze {
		state = "unfrozen (updates apply again)"
	}
	fmt.Printf("%d mod(s) %s; recorded in %s/opt-out.json\n", len(slugs), state, packDir)
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

func updateFreezeRecord(packDir string, slugs []string, freeze bool) {
	p := filepath.Join(packDir, "opt-out.json")
	raw := map[string]any{}
	if data, err := os.ReadFile(p); err == nil {
		if err := json.Unmarshal(data, &raw); err != nil {
			fail(fmt.Sprintf("opt-out.json in %s exists but is invalid JSON; fix it before freezing: %v", packDir, err))
		}
	}
	set := map[string]bool{}
	if existing, ok := raw["freeze"].([]any); ok {
		for _, v := range existing {
			if s, ok := v.(string); ok {
				set[s] = true
			}
		}
	}
	for _, s := range slugs {
		if freeze {
			set[s] = true
		} else {
			delete(set, s)
		}
	}
	if len(set) == 0 {
		delete(raw, "freeze")
		if len(raw) == 0 {
			_ = os.Remove(p)
			return
		}
	} else {
		var list []string
		for s := range set {
			list = append(list, s)
		}
		sort.Strings(list)
		raw["freeze"] = list
	}
	writeJSON(p, raw)
}

func pinDrift(packDir string, frozen []string) []string {
	var drift []string
	for _, slug := range frozen {
		for _, sub := range modSubdirsOf(packDir) {
			p := filepath.Join(sub, "mods", slug+".pw.toml")
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
