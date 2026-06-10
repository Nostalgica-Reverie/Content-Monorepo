package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

func cmdDoctor(args []string) {
	fmt.Printf("somnus %s doctor\n\n", somnusVersion)
	problems := 0

	tool := func(name, why string, required bool) {
		if p, err := exec.LookPath(name); err == nil {
			fmt.Printf("  ok    %-9s %s\n", name, p)
		} else if required {
			fmt.Printf("  MISS  %-9s required: %s\n", name, why)
			problems++
		} else {
			fmt.Printf("  warn  %-9s optional: %s\n", name, why)
		}
	}
	tool("git", "change detection, changelogs, sync anchoring", true)
	tool(packwizBin(), "every pack operation", true)
	tool("java", "only needed for 'somnus test'", false)
	tool("zip", "datapack/resourcepack builds via the publisher", false)

	root := findRepoRoot()
	if root == "" {
		fmt.Println("  MISS  repo      no .git or modpacks/ found walking up from here")
		fail("doctor found problems — run somnus from inside the monorepo")
	}
	fmt.Printf("  ok    repo      %s\n", root)

	total, broken := 0, 0
	for _, cat := range []string{"modpacks", "datapacks", "resourcepacks"} {
		dir := filepath.Join(root, cat)
		entries, err := os.ReadDir(dir)
		if err != nil {
			continue
		}
		n := 0
		for _, e := range entries {
			if !e.IsDir() {
				continue
			}
			mf := filepath.Join(dir, e.Name(), "manifest.json")
			if _, err := os.Stat(mf); err != nil {
				continue
			}
			n++
			if _, err := readManifest(mf); err != nil {
				fmt.Printf("  BAD   manifest  %s: %v\n", mf, err)
				broken++
			}
			packPath := filepath.Join(dir, e.Name())
			if _, err := os.Stat(filepath.Join(packPath, "auto-update-ignore.json")); err == nil {
				fmt.Printf("  warn  legacy    %s: auto-update-ignore.json -> migrate to opt-out.json\n", packPath)
			}
			if subs, err := os.ReadDir(packPath); err == nil {
				for _, s := range subs {
					if s.IsDir() {
						if _, err := os.Stat(filepath.Join(packPath, s.Name(), "sync-exclude.json")); err == nil {
							fmt.Printf("  warn  legacy    %s: sync-exclude.json -> migrate to opt-out.json\n", filepath.Join(packPath, s.Name()))
						}
					}
				}
			}
		}
		fmt.Printf("  ok    %-9s %d manifest(s)\n", cat, n)
		total += n
	}

	fmt.Printf("\n%d project manifest(s) found, %d unparsable\n", total, broken)
	if problems > 0 || broken > 0 {
		fail(fmt.Sprintf("doctor found %d problem(s)", problems+broken))
	}
	fmt.Println("environment looks healthy.")
}
