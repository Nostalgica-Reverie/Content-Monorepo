package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
)

func maxConcurrent() int {
	if v := os.Getenv("SOMNUS_CONCURRENCY"); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			return n
		}
	}
	n := runtime.NumCPU()
	if n > 8 {
		n = 8
	}
	if n < 1 {
		n = 1
	}
	return n
}

func cacheSlotCount() int {
	if v := os.Getenv("SOMNUS_CACHE_SLOTS"); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			return n
		}
	}
	return maxConcurrent()
}

func cmdUpdate(args []string)  { runScoped(opUpdate, args) }
func cmdRefresh(args []string) { runScoped(opRefresh, args) }

func runScoped(op operation, args []string) {
	packFilter, explicit := resolveScope(args)
	run(op, packFilter, explicit)
}

func cmdLoaderUpdate(args []string) {
	target := "latest"
	var rest []string
	for _, a := range args {
		if a == "latest" || a == "recommended" {
			target = a
		} else {
			rest = append(rest, a)
		}
	}
	runScoped(operation{
		name:        "loader-update",
		gerund:      "migrating loader (" + target + ") in",
		packwizArgs: []string{"migrate", "loader", target},
		honorIgnore: true,
	}, rest)
}

func resolveScope(args []string) (packFilter string, explicit bool) {
	for _, a := range args {
		if a == "--all" {
			return "", false
		}
	}
	for _, a := range args {
		if strings.HasPrefix(a, "-") {
			continue
		}
		dir := strings.TrimRight(a, "/")
		if _, err := os.Stat(filepath.Join(dir, "manifest.json")); err != nil {
			failNotFound(fmt.Sprintf("no manifest.json in %q — pass a pack directory, --all, or nothing", a))
		}
		return dir, true
	}
	if root, err := os.Getwd(); err == nil && startCwd != "" {
		if rel, err := filepath.Rel(root, startCwd); err == nil && !strings.HasPrefix(rel, "..") && rel != "." {
			parts := strings.Split(filepath.ToSlash(rel), "/")
			if len(parts) >= 2 && parts[0] == filepath.Base(modpacksDir()) {
				pack := filepath.Join(modpacksDir(), parts[1])
				fmt.Printf("scoped to %s (somnus was run inside it; pass --all for every pack)\n", pack)
				return pack, false
			}
		}
	}
	return "", false
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

func run(op operation, packFilter string, explicit bool) {
	if _, err := exec.LookPath(packwizBin()); err != nil {
		failEnv("packwiz not found", "install with 'go install github.com/packwiz/packwiz@latest' or point PACKWIZ_BIN at a binary")
	}
	root := modpacksDir()
	if info, err := os.Stat(root); err != nil || !info.IsDir() {
		failEnv(fmt.Sprintf("modpacks directory not found: %s", root), "run somnus from inside the monorepo")
	}

	targets, skipped := collectTargets(root, op.honorIgnore, packFilter, explicit)

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

	prog := newProgress(op.gerund, len(targets))
	failures := workPool(targets, op, prog)
	prog.done()

	if len(failures) > 0 {
		fmt.Fprintf(os.Stderr, "\n%d subdir(s) failed:\n", len(failures))
		for _, f := range failures {
			fmt.Fprintf(os.Stderr, "  - %s\n", f)
		}
		fail(fmt.Sprintf("one or more %ss failed", op.name))
	}

	fmt.Printf("all %ss finished successfully.\n", op.name)
}

func collectTargets(root string, honorIgnore bool, packFilter string, explicit bool) (targets []string, skipped []string) {
	packs, err := os.ReadDir(root)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", root, err))
	}

	for _, p := range packs {
		if !p.IsDir() {
			continue
		}
		packPath := filepath.Join(root, p.Name())

		if packFilter != "" && filepath.Clean(packPath) != filepath.Clean(packFilter) {
			continue
		}
		if packFilter != "" && explicit && honorIgnore {
			if skip, _ := optedOutOfAutoUpdate(packPath); skip {
				fmt.Printf("note: %s is opted out of auto-update, running anyway (explicitly named)\n", packPath)
			}
			targets = append(targets, modSubdirsOf(packPath)...)
			continue
		}
		if honorIgnore {
			if skip, legacy := optedOutOfAutoUpdate(packPath); skip {
				if legacy {
					fmt.Fprintf(os.Stderr, "::warning::%s uses a legacy opt-out file; migrate to manifest.json automation\n", packPath)
				}
				skipped = append(skipped, packPath)
				continue
			}
		}

		targets = append(targets, modSubdirsOf(packPath)...)
	}
	return targets, skipped
}

func workPool(targets []string, op operation, prog *progress) []string {
	sched := NewScheduler(maxConcurrent())
	slots := cacheSlotCount()

	dones := make([]<-chan error, len(targets))
	for i, dir := range targets {
		dir := dir
		dones[i] = sched.Submit(Task{
			Name: dir,
			Needs: []Resource{
				Resource("subdir:" + dir),
				CacheSlot(dir, slots),
			},
			Run: func() error {
				fmt.Printf("%s %s\n", op.gerund, dir)
				cmd := exec.Command(packwizBin(), op.packwizArgs...)
				cmd.Dir = dir
				out, err := cmd.CombinedOutput()
				if err != nil {
					fmt.Fprintf(os.Stderr, "FAIL %s: %v\n", dir, err)
					if len(out) > 0 {
						fmt.Fprintf(os.Stderr, "%s\n", indent(string(out), "    "))
					}
					return err
				}
				if prog != nil && prog.tty {
					prog.step(dir)
				} else {
					fmt.Printf("ok: %s\n", dir)
				}
				return nil
			},
		})
	}

	var failures []string
	for i, c := range dones {
		if err := <-c; err != nil {
			failures = append(failures, targets[i])
		}
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
		failEnv("packwiz not found", "install with 'go install github.com/packwiz/packwiz@latest' or point PACKWIZ_BIN at a binary")
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

	sched := NewScheduler(maxConcurrent())
	slots := cacheSlotCount()
	dones := make([]<-chan error, len(jobs))
	for i, j := range jobs {
		j := j
		dones[i] = sched.Submit(Task{
			Name: j.consumerID,
			Needs: []Resource{
				Resource("sync-target:" + j.targetDir),
				CacheSlot(j.targetDir, slots),
			},
			Run: func() error { return runSyncJob(j, dryRun, syncedFolders) },
		})
	}

	failed := 0
	for _, c := range dones {
		if err := <-c; err != nil {
			fmt.Fprintf(os.Stderr, "sync error: %v\n", err)
			failed++
		}
	}
	if failed > 0 {
		fail(fmt.Sprintf("%d sync(s) failed", failed))
	}

	if dryRun {
		fmt.Println("[DRY RUN] complete \u2014 nothing was changed.")
	} else {
		fmt.Println("all syncs completed.")
		fmt.Println()
		cmdPages(nil)
	}
}

func runSyncJob(j syncJob, dryRun bool, syncedFolders []string) error {
	fmt.Printf("syncing %s -> %s (base %s)\n", j.sourceDir, j.targetDir, j.baseID)

	excluded := readSyncExclude(filepath.Join(j.targetDir, "sync-exclude.json"))
	if len(excluded) > 0 {
		fmt.Fprintf(os.Stderr, "::warning::%s uses legacy sync-exclude.json; migrate to manifest.json automation.sync_exclude\n", j.targetDir)
	}
	for _, f := range readAutomation(filepath.Dir(j.targetDir)).SyncExclude {
		excluded[f] = true
	}
	if len(excluded) > 0 {
		fmt.Printf("  %s: %d path(s) excluded from sync\n", j.consumerID, len(excluded))
	}

	provided := map[string]bool{}
	for _, folder := range syncedFolders {
		srcFolder := filepath.Join(j.sourceDir, folder)
		if _, err := os.Stat(srcFolder); err != nil {
			continue
		}
		rels, err := relFilesUnder(srcFolder)
		if err != nil {
			return fmt.Errorf("scanning %s for %s failed: %w", folder, j.consumerID, err)
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
			fmt.Printf("  %s: [DRY RUN] would copy %d file(s) into %s/\n", j.consumerID, kept, folder)
			continue
		}
		n, err := copyTreeRecording(srcFolder, filepath.Join(j.targetDir, folder), folder, placed, excluded)
		if err != nil {
			return fmt.Errorf("copy %s for %s failed: %w", folder, j.consumerID, err)
		}
		fmt.Printf("  %s: %s: %d file(s) copied\n", j.consumerID, folder, n)
	}

	if len(toDelete) > 0 {
		if len(toDelete) > len(provided) {
			return fmt.Errorf("ABORT: %s delete-set (%d) exceeds files the base provides (%d). Prior sync.json is likely stale/mismatched. Delete sync.json in this target and re-run to reset state. NO files were deleted",
				j.targetDir, len(toDelete), len(provided))
		}
		if dryRun {
			fmt.Printf("  %s: [DRY RUN] would delete %d base-removed file(s):\n", j.consumerID, len(toDelete))
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
			fmt.Printf("  %s: pruned %d base-removed file(s)\n", j.consumerID, len(toDelete))
		}
	}

	if dryRun {
		return nil
	}

	if err := writeSyncState(statePath, placed); err != nil {
		return fmt.Errorf("failed to write %s: %w", statePath, err)
	}

	cmd := exec.Command(packwizBin(), "refresh")
	cmd.Dir = j.targetDir
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("packwiz refresh failed in %s: %v\n%s", j.targetDir, err, indent(string(out), "    "))
	}
	fmt.Printf("  refreshed %s\n", j.targetDir)
	return nil
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
