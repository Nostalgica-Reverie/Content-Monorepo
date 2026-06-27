package main

import (
	"fmt"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
)

func cmdDiff(args []string) {
	if len(args) < 2 {
		failUsage(verbUsage["diff"])
	}
	oldRef, newRef := args[0], args[1]
	var pathPrefix string
	if len(args) > 2 {
		pathPrefix = args[2]
	}

	// Get all .pw.toml paths that changed between refs.
	out, err := exec.Command("git", "diff", "--name-only", oldRef+".."+newRef).Output()
	if err != nil {
		fail(fmt.Sprintf("git diff failed: %v", err))
	}

	var changed []string
	for l := range strings.SplitSeq(string(out), "\n") {
		l = strings.TrimSpace(l)
		if l == "" || !strings.HasSuffix(l, ".pw.toml") {
			continue
		}
		if pathPrefix != "" && !strings.HasPrefix(l, pathPrefix) {
			continue
		}
		changed = append(changed, l)
	}

	if len(changed) == 0 {
		fmt.Printf("no .pw.toml changes between %s and %s\n", oldRef, newRef)
		return
	}

	// Group by subdir for readable output.
	bySubdir := map[string][]string{}
	for _, p := range changed {
		sub := filepath.Dir(filepath.Dir(p)) // strip /mods/<file>
		bySubdir[sub] = append(bySubdir[sub], p)
	}
	subdirs := make([]string, 0, len(bySubdir))
	for s := range bySubdir {
		subdirs = append(subdirs, s)
	}
	sort.Strings(subdirs)

	totalAdded, totalRemoved, totalUpdated := 0, 0, 0

	for _, sub := range subdirs {
		files := bySubdir[sub]
		sort.Strings(files)

		added, removed, updated := 0, 0, 0
		var lines []string

		for _, path := range files {
			oldContent := gitShowFile(oldRef, path)
			newContent := gitShowFile(newRef, path)
			slug := strings.TrimSuffix(filepath.Base(path), ".pw.toml")

			switch {
			case oldContent == "" && newContent != "":
				ver := pwFilename(newContent)
				lines = append(lines, fmt.Sprintf("  + %-38s %s", slug, ver))
				added++
			case oldContent != "" && newContent == "":
				ver := pwFilename(oldContent)
				lines = append(lines, fmt.Sprintf("  - %-38s %s", slug, ver))
				removed++
			default:
				oldFn := pwFilename(oldContent)
				newFn := pwFilename(newContent)
				if oldFn != newFn {
					lines = append(lines, fmt.Sprintf("  ~ %-38s %s -> %s", slug, oldFn, newFn))
				} else {
					lines = append(lines, fmt.Sprintf("  ~ %s", slug))
				}
				updated++
			}
		}

		totalAdded += added
		totalRemoved += removed
		totalUpdated += updated

		fmt.Printf("%s:\n", sub)
		for _, l := range lines {
			fmt.Println(l)
		}
		fmt.Printf("  +%d -%d ~%d\n\n", added, removed, updated)
	}

	fmt.Printf("%s..%s: +%d added  -%d removed  ~%d updated\n",
		oldRef, newRef, totalAdded, totalRemoved, totalUpdated)
}

func gitShowFile(ref, path string) string {
	out, err := exec.Command("git", "show", ref+":"+path).Output()
	if err != nil {
		return ""
	}
	return string(out)
}

// pwFilename extracts the filename field from pw.toml content (top-level only).
func pwFilename(content string) string {
	inSection := false
	for _, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[") {
			inSection = true
			continue
		}
		if inSection {
			continue
		}
		k, v, ok := splitKV(line)
		if ok && k == "filename" {
			return strings.Trim(v, `"`)
		}
	}
	return ""
}

// pwVersion extracts the version string from the [update.modrinth] or [update.curseforge] section.
func pwVersion(content string) string {
	inUpdate := false
	for _, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[update") {
			inUpdate = true
			continue
		}
		if strings.HasPrefix(line, "[") {
			inUpdate = false
			continue
		}
		if !inUpdate {
			continue
		}
		k, v, ok := splitKV(line)
		if ok && k == "version" {
			return strings.Trim(v, `"`)
		}
	}
	return ""
}
