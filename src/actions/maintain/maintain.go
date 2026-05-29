package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
)

const maxConcurrent = 8

func modpacksDir() string {
	if d := os.Getenv("MODPACKS_DIR"); d != "" {
		return d
	}
	return "modpacks"
}

func main() {
	if len(os.Args) < 2 {
		fail("usage: maintain <update|refresh>")
	}

	switch os.Args[1] {
	case "update":
		run(opUpdate)
	case "refresh":
		run(opRefresh)
	default:
		fail(fmt.Sprintf("unknown subcommand %q (expected 'update' or 'refresh')", os.Args[1]))
	}
}

type operation struct {
	name        string
	gerund      string
	packwizArgs []string
	honorIgnore bool
}

var (
	opUpdate = operation{
		name:        "update",
		gerund:      "updating",
		packwizArgs: []string{"update", "--all", "-y"},
		honorIgnore: true,
	}
	opRefresh = operation{
		name:        "refresh",
		gerund:      "refreshing",
		packwizArgs: []string{"refresh"},
		honorIgnore: false,
	}
)

func run(op operation) {
	if _, err := exec.LookPath("packwiz"); err != nil {
		fail("packwiz not found in PATH")
	}
	root := modpacksDir()
	if info, err := os.Stat(root); err != nil || !info.IsDir() {
		fail(fmt.Sprintf("modpacks directory not found: %s", root))
	}

	targets, skipped := collectTargets(root, op.honorIgnore)

	if len(skipped) > 0 {
		fmt.Printf("skipping %d pack(s) with auto-update-ignore.json:\n", len(skipped))
		for _, s := range skipped {
			fmt.Printf("  - %s\n", s)
		}
	}

	if len(targets) == 0 {
		fmt.Printf("no pack subdirs to %s.\n", op.name)
		return
	}

	fmt.Printf("queued %d subdir(s), running up to %d in parallel\n", len(targets), maxConcurrent)

	failures := workPool(targets, op)

	if len(failures) > 0 {
		fmt.Fprintf(os.Stderr, "\n%d subdir(s) failed:\n", len(failures))
		for _, f := range failures {
			fmt.Fprintf(os.Stderr, "  - %s\n", f)
		}
		fail(fmt.Sprintf("one or more %ss failed", op.name))
	}

	fmt.Printf("all %ss finished successfully.\n", op.name)
}

func collectTargets(root string, honorIgnore bool) (targets []string, skipped []string) {
	packs, err := os.ReadDir(root)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", root, err))
	}

	for _, p := range packs {
		if !p.IsDir() {
			continue
		}
		packPath := filepath.Join(root, p.Name())

		if honorIgnore {
			if _, err := os.Stat(filepath.Join(packPath, "auto-update-ignore.json")); err == nil {
				skipped = append(skipped, packPath)
				continue
			}
		}

		subs, err := os.ReadDir(packPath)
		if err != nil {
			continue
		}
		for _, s := range subs {
			if !s.IsDir() {
				continue
			}
			name := s.Name()
			if strings.HasSuffix(name, "-mr") || strings.HasSuffix(name, "-cf") {
				targets = append(targets, filepath.Join(packPath, name))
			}
		}
	}
	return targets, skipped
}

func workPool(targets []string, op operation) []string {
	jobs := make(chan string)
	results := make(chan string, len(targets)) // failed dirs
	var wg sync.WaitGroup

	workers := maxConcurrent
	if len(targets) < workers {
		workers = len(targets)
	}

	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			for dir := range jobs {
				label := dir
				fmt.Printf("[W%d] %s %s\n", id, op.gerund, label)

				cmd := exec.Command("packwiz", op.packwizArgs...)
				cmd.Dir = dir
				out, err := cmd.CombinedOutput()
				if err != nil {
					fmt.Fprintf(os.Stderr, "[W%d] FAIL %s: %v\n", id, label, err)
					if len(out) > 0 {
						fmt.Fprintf(os.Stderr, "%s\n", indent(string(out), "    "))
					}
					results <- dir
				} else {
					fmt.Printf("[W%d] ok: %s\n", id, label)
				}
			}
		}(w)
	}

	for _, t := range targets {
		jobs <- t
	}
	close(jobs)

	wg.Wait()
	close(results)

	var failures []string
	for f := range results {
		failures = append(failures, f)
	}
	return failures
}

func indent(s, prefix string) string {
	lines := strings.Split(strings.TrimRight(s, "\n"), "\n")
	for i, l := range lines {
		lines[i] = prefix + l
	}
	return strings.Join(lines, "\n")
}

func fail(msg string) {
	fmt.Fprintf(os.Stderr, "::error::%s\n", msg)
	os.Exit(1)
}
