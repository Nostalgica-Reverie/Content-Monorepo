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
	tool("packwiz", "every pack operation", true)
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
