package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
)

func maxConcurrent() int {
	if v := os.Getenv("SOMNUS_CONCURRENCY"); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			return n
		}
	}
	return 1
}

func cmdUpdate(args []string)  { run(opUpdate) }
func cmdRefresh(args []string) { run(opRefresh) }

func cmdLoaderUpdate(args []string) {
	target := "latest"
	if len(args) > 0 && (args[0] == "latest" || args[0] == "recommended") {
		target = args[0]
	}
	run(operation{
		name:        "loader-update",
		gerund:      "migrating loader (" + target + ") in",
		packwizArgs: []string{"migrate", "loader", target},
		honorIgnore: true,
	})
}

func cmdSync(args []string) {
	dryRun := false
	for _, a := range args {
		if a == "--dry-run" {
			dryRun = true
		}
	}
	runSync(dryRun)
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
	if _, err := exec.LookPath(packwizBin()); err != nil {
		fail("packwiz not found in PATH")
	}
	root := modpacksDir()
	if info, err := os.Stat(root); err != nil || !info.IsDir() {
		fail(fmt.Sprintf("modpacks directory not found: %s", root))
	}

	targets, skipped := collectTargets(root, op.honorIgnore)

	if len(skipped) > 0 {
		fmt.Printf("skipping %d opted-out pack(s):\n", len(skipped))
		for _, s := range skipped {
			fmt.Printf("  - %s\n", s)
		}
	}

	if len(targets) == 0 {
		fmt.Printf("no pack subdirs to %s.\n", op.name)
		return
	}

	fmt.Printf("queued %d subdir(s), running up to %d in parallel\n", len(targets), maxConcurrent())

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
			if skip, legacy := optedOutOfAutoUpdate(packPath); skip {
				if legacy {
					fmt.Fprintf(os.Stderr, "::warning::%s uses legacy auto-update-ignore.json; migrate to opt-out.json {\"auto_update\": false}\n", packPath)
				}
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
	results := make(chan string, len(targets))
	var wg sync.WaitGroup

	workers := maxConcurrent()
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

				cmd := exec.Command(packwizBin(), op.packwizArgs...)
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

func parseRole(raw json.RawMessage) (kind string, pb *performanceBase) {
	var s string
	if err := json.Unmarshal(raw, &s); err == nil {
		return s, nil
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

func runSync(dryRun bool) {
	if _, err := exec.LookPath(packwizBin()); err != nil {
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
	if dryRun {
		fmt.Println("[DRY RUN] no files will be copied, deleted, or refreshed")
	}

	syncedFolders := []string{"mods", "config", "resourcepacks", "global_packs"}

	for _, j := range jobs {
		fmt.Printf("syncing %s -> %s (base %s)\n", j.sourceDir, j.targetDir, j.baseID)

		excluded := readSyncExclude(filepath.Join(j.targetDir, "sync-exclude.json"))
		if len(excluded) > 0 {
			fmt.Fprintf(os.Stderr, "::warning::%s uses legacy sync-exclude.json; migrate to opt-out.json sync_exclude\n", j.targetDir)
		}
		for _, f := range readOptOut(filepath.Dir(j.targetDir)).SyncExclude {
			excluded[f] = true
		}
		if len(excluded) > 0 {
			fmt.Printf("  %d path(s) excluded from sync\n", len(excluded))
		}

		provided := map[string]bool{}
		for _, folder := range syncedFolders {
			srcFolder := filepath.Join(j.sourceDir, folder)
			if _, err := os.Stat(srcFolder); err != nil {
				continue
			}
			rels, err := relFilesUnder(srcFolder)
			if err != nil {
				fail(fmt.Sprintf("scanning %s for %s failed: %v", folder, j.consumerID, err))
			}
			for _, r := range rels {
				slash := filepath.ToSlash(filepath.Join(folder, r))
				if excluded[slash] {
					continue
				}
				provided[slash] = true
			}
		}

		statePath := filepath.Join(j.targetDir, "sync.json")
		prev := readSyncState(statePath)
		var toDelete []string
		for f := range prev {
			if excluded[f] {
				continue
			}
			if !provided[f] {
				toDelete = append(toDelete, f)
			}
		}
		sort.Strings(toDelete)

		placed := map[string]bool{}
		for _, folder := range syncedFolders {
			srcFolder := filepath.Join(j.sourceDir, folder)
			if _, err := os.Stat(srcFolder); err != nil {
				continue
			}
			if dryRun {
				rels, _ := relFilesUnder(srcFolder)
				kept := 0
				for _, r := range rels {
					slash := filepath.ToSlash(filepath.Join(folder, r))
					if excluded[slash] {
						continue
					}
					placed[slash] = true
					kept++
				}
				fmt.Printf("  [DRY RUN] would copy %d file(s) into %s/\n", kept, folder)
				continue
			}
			n, err := copyTreeRecording(srcFolder, filepath.Join(j.targetDir, folder), folder, placed, excluded)
			if err != nil {
				fail(fmt.Sprintf("copy %s for %s failed: %v", folder, j.consumerID, err))
			}
			fmt.Printf("  %s: %d file(s) copied\n", folder, n)
		}

		if len(toDelete) > 0 {
			if len(toDelete) > len(provided) {
				fail(fmt.Sprintf("ABORT: %s delete-set (%d) exceeds files the base provides (%d). Prior sync.json is likely stale/mismatched. Delete sync.json in this target and re-run to reset state. NO files were deleted.",
					j.targetDir, len(toDelete), len(provided)))
			}
			if dryRun {
				fmt.Printf("  [DRY RUN] would delete %d base-removed file(s):\n", len(toDelete))
				for _, f := range toDelete {
					fmt.Printf("      - %s\n", f)
				}
			} else {
				for _, f := range toDelete {
					p := filepath.FromSlash(filepath.Join(j.targetDir, f))
					if err := os.Remove(p); err != nil && !os.IsNotExist(err) {
						fmt.Fprintf(os.Stderr, "::warning::could not delete %s: %v\n", p, err)
					}
				}
				fmt.Printf("  pruned %d base-removed file(s)\n", len(toDelete))
			}
		}

		if dryRun {
			continue
		}

		if err := writeSyncState(statePath, placed); err != nil {
			fail(fmt.Sprintf("failed to write %s: %v", statePath, err))
		}

		cmd := exec.Command(packwizBin(), "refresh")
		cmd.Dir = j.targetDir
		if out, err := cmd.CombinedOutput(); err != nil {
			fail(fmt.Sprintf("packwiz refresh failed in %s: %v\n%s", j.targetDir, err, indent(string(out), "    ")))
		}
		fmt.Printf("  refreshed %s\n", j.targetDir)
	}

	if dryRun {
		fmt.Println("[DRY RUN] complete \u2014 nothing was changed.")
	} else {
		fmt.Println("all syncs completed.")
	}
}

func relFilesUnder(root string) ([]string, error) {
	var out []string
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		rel, err := filepath.Rel(root, path)
		if err != nil {
			return err
		}
		out = append(out, rel)
		return nil
	})
	return out, err
}

const syncStateVersion = 2

type syncState struct {
	Version int      `json:"version"`
	Files   []string `json:"files"`
}

func readSyncState(path string) map[string]bool {
	set := map[string]bool{}
	data, err := os.ReadFile(path)
	if err != nil {
		return set
	}
	var st syncState
	if err := json.Unmarshal(data, &st); err != nil {
		return set
	}
	if st.Version != syncStateVersion {
		fmt.Fprintf(os.Stderr, "::warning::ignoring %s (state version %d != %d); treating as fresh, no prune this run\n",
			path, st.Version, syncStateVersion)
		return set
	}
	for _, f := range st.Files {
		set[f] = true
	}
	return set
}

func writeSyncState(path string, placed map[string]bool) error {
	files := make([]string, 0, len(placed))
	for f := range placed {
		files = append(files, f)
	}
	sort.Strings(files)
	st := syncState{Version: syncStateVersion, Files: files}
	data, err := json.MarshalIndent(st, "", "  ")
	if err != nil {
		return err
	}
	data = append(data, '\n')
	return os.WriteFile(path, data, 0o644)
}

func copyTreeRecording(src, dst, folder string, placed, excluded map[string]bool) (int, error) {
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
		slash := filepath.ToSlash(filepath.Join(folder, rel))
		if !info.IsDir() && excluded[slash] {
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
		placed[slash] = true
		count++
		return nil
	})
	return count, err
}

func readSyncExclude(path string) map[string]bool {
	set := map[string]bool{}
	data, err := os.ReadFile(path)
	if err != nil {
		return set
	}
	var files []string
	if err := json.Unmarshal(data, &files); err != nil {
		return set
	}
	for _, f := range files {
		set[f] = true
	}
	return set
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
