package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

var validSides = map[string]bool{"client": true, "server": true, "both": true}

func cmdSide(args []string) {
	if len(args) < 2 {
		failUsage(verbUsage["side"])
	}
	packDir, slug := args[0], args[1]
	if _, err := os.Stat(filepath.Join(packDir, "manifest.json")); err != nil {
		failNotFound(fmt.Sprintf("no manifest.json in %s — side operates on a pack directory", packDir))
	}
	if len(args) < 3 {
		showSides(packDir, slug)
		return
	}
	newSide := args[2]
	if !validSides[newSide] {
		failUsage(fmt.Sprintf("invalid side %q (expected client, server, or both)", newSide))
	}
	setSides(packDir, slug, newSide)
}

func showSides(packDir, slug string) {
	found := 0
	for _, sub := range modSubdirsOf(packDir) {
		p := filepath.Join(sub, "mods", slug+".pw.toml")
		data, err := os.ReadFile(p)
		if err != nil {
			continue
		}
		found++
		fmt.Printf("  %-10s %s\n", currentSide(string(data)), sub)
	}
	if found == 0 {
		failNotFound(fmt.Sprintf("%s not found in any subdir of %s (checked mods/%s.pw.toml)", slug, packDir, slug))
	}
}

func setSides(packDir, slug, newSide string) {
	if _, err := exec.LookPath(packwizBin()); err != nil {
		failEnv("packwiz not found", "install with 'go install github.com/packwiz/packwiz@latest' or point PACKWIZ_BIN at a binary")
	}
	var touched []string
	for _, sub := range modSubdirsOf(packDir) {
		p := filepath.Join(sub, "mods", slug+".pw.toml")
		data, err := os.ReadFile(p)
		if err != nil {
			continue
		}
		updated, old, changed := rewriteSide(string(data), newSide)
		if !changed {
			fmt.Printf("  ok (already %s): %s\n", newSide, sub)
			continue
		}
		if err := os.WriteFile(p, []byte(updated), 0o644); err != nil {
			fail(fmt.Sprintf("failed to write %s: %v", p, err))
		}
		fmt.Printf("  %s -> %s: %s\n", old, newSide, sub)
		touched = append(touched, sub)
	}
	if len(touched) == 0 {
		fmt.Printf("nothing to change for %s in %s.\n", slug, packDir)
		return
	}
	for _, sub := range touched {
		cmd := exec.Command(packwizBin(), "refresh")
		cmd.Dir = sub
		if out, err := cmd.CombinedOutput(); err != nil {
			fail(fmt.Sprintf("packwiz refresh failed in %s: %v\n%s", sub, err, indent(string(out), "    ")))
		}
		fmt.Printf("  refreshed %s\n", sub)
	}
	fmt.Printf("%s is now %q in %d subdir(s).\n", slug, newSide, len(touched))
}

func currentSide(content string) string {
	for _, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[") {
			break // top-level section ended
		}
		if k, v, ok := splitKV(line); ok && k == "side" {
			return v
		}
	}
	return "both"
}

func rewriteSide(content, newSide string) (updated, old string, changed bool) {
	old = currentSide(content)
	if old == newSide {
		return content, old, false
	}
	lines := strings.Split(content, "\n")
	inTop := true
	filenameIdx := -1
	for i, raw := range lines {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[") {
			inTop = false
		}
		if !inTop {
			break
		}
		if k, _, ok := splitKV(line); ok {
			if k == "side" {
				lines[i] = fmt.Sprintf("side = %q", newSide)
				return strings.Join(lines, "\n"), old, true
			}
			if k == "filename" {
				filenameIdx = i
			}
		}
	}
	insert := fmt.Sprintf("side = %q", newSide)
	if filenameIdx >= 0 {
		lines = append(lines[:filenameIdx+1], append([]string{insert}, lines[filenameIdx+1:]...)...)
	} else {
		lines = append([]string{insert}, lines...)
	}
	return strings.Join(lines, "\n"), old, true
}
