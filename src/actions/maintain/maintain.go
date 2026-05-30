package main

import (
	"encoding/json"
	"fmt"
	"io"
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
		fail("usage: maintain <update|refresh|sync>")
	}

	switch os.Args[1] {
	case "update":
		run(opUpdate)
	case "refresh":
		run(opRefresh)
	case "sync":
		runSync()
	default:
		fail(fmt.Sprintf("unknown subcommand %q (expected update, refresh, or sync)", os.Args[1]))
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

type mapping struct {
	Source string `json:"source"`
	Target string `json:"target"`
}
type performanceBase struct {
	Pack     string    `json:"pack"`
	Mappings []mapping `json:"mappings"`
}
type manifest struct {
	ID   string          `json:"id"`
	Role json.RawMessage `json:"role"`
}

func parseRole(raw json.RawMessage) (kind string, pb *performanceBase) {
	var s string
	if err := json.Unmarshal(raw, &s); err == nil {
		return s, nil // "none" or "base"
	}
	var obj struct {
		PerformanceBase *performanceBase `json:"performance_base"`
	}
	if err := json.Unmarshal(raw, &obj); err == nil && obj.PerformanceBase != nil {
		return "consumer", obj.PerformanceBase
	}
	return "", nil
}

func platformSuffix(s string) string {
	if strings.HasSuffix(s, "-mr") {
		return "mr"
	}
	if strings.HasSuffix(s, "-cf") {
		return "cf"
	}
	return ""
}

type syncJob struct {
	consumerID string
	baseID     string
	sourceDir  string
	targetDir  string
}

func readManifest(path string) (*manifest, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var m manifest
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, err
	}
	return &m, nil
}

func runSync() {
	if _, err := exec.LookPath("packwiz"); err != nil {
		fail("packwiz not found in PATH")
	}
	root := modpacksDir()
	packs, err := os.ReadDir(root)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", root, err))
	}

	roleOf := make(map[string]string)
	for _, p := range packs {
		if !p.IsDir() {
			continue
		}
		m, err := readManifest(filepath.Join(root, p.Name(), "manifest.json"))
		if err != nil {
			continue
		}
		kind, _ := parseRole(m.Role)
		roleOf[m.ID] = kind
	}

	var jobs []syncJob
	for _, p := range packs {
		if !p.IsDir() {
			continue
		}
		packPath := filepath.Join(root, p.Name())
		m, err := readManifest(filepath.Join(packPath, "manifest.json"))
		if err != nil {
			continue
		}
		kind, pb := parseRole(m.Role)
		if kind != "consumer" || pb == nil {
			continue
		}

		if roleOf[pb.Pack] != "base" {
			fail(fmt.Sprintf("consumer '%s' references base '%s', which is not role 'base'", m.ID, pb.Pack))
		}
		basePackDir := filepath.Join(root, pb.Pack)

		for _, mp := range pb.Mappings {
			sp, tp := platformSuffix(mp.Source), platformSuffix(mp.Target)
			if sp == "" || tp == "" {
				fail(fmt.Sprintf("consumer '%s': mapping %s->%s has a non -mr/-cf suffix", m.ID, mp.Source, mp.Target))
			}
			if sp != tp {
				fail(fmt.Sprintf("consumer '%s': FORBIDDEN cross-platform mapping %s (%s) -> %s (%s). MR/CF must never cross (license risk).",
					m.ID, mp.Source, sp, mp.Target, tp))
			}
			src := filepath.Join(basePackDir, mp.Source)
			dst := filepath.Join(packPath, mp.Target)
			if _, err := os.Stat(src); err != nil {
				fail(fmt.Sprintf("consumer '%s': mapping source %s missing in base '%s'", m.ID, mp.Source, pb.Pack))
			}
			if _, err := os.Stat(dst); err != nil {
				fail(fmt.Sprintf("consumer '%s': mapping target %s missing in this pack", m.ID, mp.Target))
			}
			jobs = append(jobs, syncJob{consumerID: m.ID, baseID: pb.Pack, sourceDir: src, targetDir: dst})
		}
	}

	if len(jobs) == 0 {
		fmt.Println("no consumers to sync.")
		return
	}
	fmt.Printf("resolved %d sync job(s) from manifests\n", len(jobs))

	for _, j := range jobs {
		fmt.Printf("syncing %s -> %s (base %s)\n", j.sourceDir, j.targetDir, j.baseID)

		for _, folder := range []string{"mods", "config"} {
			srcFolder := filepath.Join(j.sourceDir, folder)
			if _, err := os.Stat(srcFolder); err != nil {
				continue
			}
			n, err := copyTree(srcFolder, filepath.Join(j.targetDir, folder))
			if err != nil {
				fail(fmt.Sprintf("copy %s for %s failed: %v", folder, j.consumerID, err))
			}
			fmt.Printf("  %s: %d file(s) copied -> i did this!\n", folder, n)
		}

		cmd := exec.Command("packwiz", "refresh")
		cmd.Dir = j.targetDir
		if out, err := cmd.CombinedOutput(); err != nil {
			fail(fmt.Sprintf("packwiz refresh failed in %s: %v\n%s", j.targetDir, err, indent(string(out), "    ")))
		}
		fmt.Printf("  refreshed %s\n", j.targetDir)
	}

	fmt.Println("all syncs completed.")
}

func copyTree(src, dst string) (int, error) {
	count := 0
	err := filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		if rel == "." {
			return nil
		}
		target := filepath.Join(dst, rel)
		if info.IsDir() {
			return os.MkdirAll(target, 0o755)
		}
		if err := os.MkdirAll(filepath.Dir(target), 0o755); err != nil {
			return err
		}
		if err := copyFile(path, target); err != nil {
			return err
		}
		count++
		return nil
	})
	return count, err
}

func copyFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()
	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()
	_, err = io.Copy(out, in)
	return err
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
