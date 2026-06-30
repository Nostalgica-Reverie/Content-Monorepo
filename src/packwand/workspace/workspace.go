// Package workspace implements the multi-pack orchestration engine.
// It provides tools discovery, parallel pack operations, progress display, linting,
// and the sync engine used by packwand workspace commands.
package workspace

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
	"sync/atomic"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/manifest"
)

// â€” Environment helpers â€”

// SelfBin returns the path to the running packwand binary for sub-invocations.
// Reads PACKWAND_BIN first; falls back to PACKWIZ_BIN for backward compatibility.
func SelfBin() string {
	if b := os.Getenv("PACKWAND_BIN"); b != "" {
		return b
	}
	if b := os.Getenv("PACKWIZ_BIN"); b != "" {
		fmt.Fprintln(os.Stderr, "warning: PACKWIZ_BIN is deprecated; use PACKWAND_BIN instead")
		return b
	}
	if exe, err := os.Executable(); err == nil {
		return exe
	}
	return "packwand"
}

// ModpacksDir returns the root directory containing all pack directories.
func ModpacksDir() string {
	if d := os.Getenv("MODPACKS_DIR"); d != "" {
		return d
	}
	return "modpacks"
}

// FindRepoRoot walks up from cwd looking for .git or modpacks/.
func FindRepoRoot() string {
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for {
		if _, err := os.Stat(filepath.Join(dir, ".git")); err == nil {
			return dir
		}
		if info, err := os.Stat(filepath.Join(dir, "modpacks")); err == nil && info.IsDir() {
			return dir
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return ""
		}
		dir = parent
	}
}

// MaxConcurrent returns the number of parallel workers to use.
func MaxConcurrent() int {
	return core.MaxConcurrent()
}

// CacheSlotCount returns the number of packwand export operations that may run
// concurrently against the on-disk pack cache. Controlled by PACKWAND_CACHE_SLOTS.
func CacheSlotCount() int {
	if v := os.Getenv("PACKWAND_CACHE_SLOTS"); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			return n
		}
	}
	return MaxConcurrent()
}

func envHasAny(names ...string) bool {
	for _, name := range names {
		if os.Getenv(name) != "" {
			return true
		}
	}
	return false
}

// ConfigureSubprocess keeps workspace-level fanout from multiplying by each
// child packwand process's internal fanout. Explicit user limits are preserved.
func ConfigureSubprocess(c *exec.Cmd) {
	env := os.Environ()
	if !envHasAny("PACKWAND_CONCURRENCY", "SOMNUS_CONCURRENCY") {
		env = append(env, "PACKWAND_CONCURRENCY=1")
	}
	if !envHasAny("PACKWAND_NETWORK_CONCURRENCY") {
		env = append(env, "PACKWAND_NETWORK_CONCURRENCY=1")
	}
	if !envHasAny("PACKWAND_HASH_CONCURRENCY") {
		env = append(env, "PACKWAND_HASH_CONCURRENCY=1")
	}
	c.Env = env
}

// WriteJSON writes v as indented JSON to path.
func WriteJSON(path string, v any) error {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal JSON: %w", err)
	}
	data = append(data, '\n')
	return os.WriteFile(path, data, 0o644)
}

// Indent prefixes every line of s with prefix.
func Indent(s, prefix string) string {
	lines := strings.Split(strings.TrimRight(s, "\n"), "\n")
	for i, l := range lines {
		lines[i] = prefix + l
	}
	return strings.Join(lines, "\n")
}

// â€” Progress display â€”

var sleepFrames = []string{"c(-.-)É” z  ", "c(-.-)É” zz ", "c(-.-)É” zzz", "c(-.-)É”  zz", "c(-.-)É”   z", "C(o.o)Æ† !  "}

func isTTY() bool {
	fi, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return fi.Mode()&os.ModeCharDevice != 0
}

type Progress struct {
	mu     sync.Mutex
	label  string
	total  int
	n      int
	last   string
	tty    bool
	ticker *time.Ticker
	stop   chan struct{}
	frame  int
}

// NewProgress creates and starts a TTY progress display.
func NewProgress(label string, total int) *Progress {
	p := &Progress{label: label, total: total, tty: isTTY()}
	if p.tty {
		p.stop = make(chan struct{})
		p.ticker = time.NewTicker(150 * time.Millisecond)
		go func() {
			for {
				select {
				case <-p.ticker.C:
					p.render()
				case <-p.stop:
					return
				}
			}
		}()
	}
	return p
}

// Step records one unit of progress.
func (p *Progress) Step(item string) {
	if !p.tty {
		return
	}
	p.mu.Lock()
	p.n++
	p.last = item
	p.mu.Unlock()
}

func (p *Progress) render() {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.frame = (p.frame + 1) % len(sleepFrames)
	line := fmt.Sprintf("\r%s %s [%d/%d] %s", sleepFrames[p.frame], p.label, p.n, p.total, p.last)
	if len(line) > 100 {
		line = line[:100]
	}
	fmt.Printf("%-100s\r", line[1:])
}

// Done stops the progress display and clears the line.
func (p *Progress) Done() {
	if !p.tty {
		return
	}
	close(p.stop)
	p.ticker.Stop()
	fmt.Printf("\r%-100s\r", "")
}

// â€” File copy â€”

const copyBufSize = 1 << 20

// CopyFileFast copies src to dst using a 1 MiB buffer.
func CopyFileFast(src, dst string) error {
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
	buf := make([]byte, copyBufSize)
	_, err = io.CopyBuffer(out, in, buf)
	return err
}

// â€” Pack target collection â€”

// Operation describes a packwand sub-command to run across pack subdirectories.
type Operation struct {
	Name        string
	Gerund      string
	PackwizArgs []string
	HonorIgnore bool
}

var (
	OpUpdate = Operation{
		Name:        "update",
		Gerund:      "updating",
		PackwizArgs: []string{"update", "--all", "-y"},
		HonorIgnore: true,
	}
	OpRefresh = Operation{
		Name:        "refresh",
		Gerund:      "refreshing",
		PackwizArgs: []string{"refresh"},
		HonorIgnore: false,
	}
)

// CollectTargets discovers pack subdirectories eligible for op under root.
func CollectTargets(root string, honorIgnore bool, packFilter string, explicit bool) (targets []string, skipped []string) {
	packs, err := os.ReadDir(root)
	if err != nil {
		return nil, nil
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
			lc := manifest.LifecycleState(packPath)
			switch lc {
			case "eol":
				fmt.Fprintf(os.Stderr, "warning: %s is end-of-life; running anyway (explicitly named)\n", packPath)
			case "archived":
				fmt.Printf("note: %s is archived; running anyway (explicitly named)\n", packPath)
			default:
				if skip, _ := manifest.OptedOutOfAutoUpdate(packPath); skip {
					fmt.Printf("note: %s is opted out of auto-update, running anyway (explicitly named)\n", packPath)
				}
			}
			targets = append(targets, manifest.SubDirsOf(packPath)...)
			continue
		}
		if honorIgnore {
			lc := manifest.LifecycleState(packPath)
			if lc == "archived" || lc == "eol" {
				label := lc
				skipped = append(skipped, packPath+" ("+label+")")
				continue
			}
			if skip, legacy := manifest.OptedOutOfAutoUpdate(packPath); skip {
				if legacy {
					fmt.Fprintf(os.Stderr, "warning: %s uses a legacy opt-out file; migrate to manifest.json automation\n", packPath)
				}
				skipped = append(skipped, packPath)
				continue
			}
		}
		targets = append(targets, manifest.SubDirsOf(packPath)...)
	}
	return targets, skipped
}

// WorkPool runs op across targets using the scheduler, returning paths that failed.
func WorkPool(targets []string, op Operation, prog *Progress) []string {
	sched := NewScheduler(MaxConcurrent())
	slots := CacheSlotCount()

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
				fmt.Printf("%s %s\n", op.Gerund, dir)
				cmd := exec.Command(SelfBin(), op.PackwizArgs...)
				cmd.Dir = dir
				ConfigureSubprocess(cmd)
				out, err := cmd.CombinedOutput()
				if err != nil {
					fmt.Fprintf(os.Stderr, "FAIL %s: %v\n", dir, err)
					if len(out) > 0 {
						fmt.Fprintf(os.Stderr, "%s\n", Indent(string(out), "    "))
					}
					return err
				}
				if prog != nil && prog.tty {
					prog.Step(dir)
				} else {
					fmt.Printf("ok: %s\n", dir)
				}
				return nil
			},
		})
	}
	sched.Close()

	var failures []string
	for i, c := range dones {
		if err := <-c; err != nil {
			failures = append(failures, targets[i])
		}
	}
	return failures
}

// Run executes op across all eligible pack subdirs under ModpacksDir().
func Run(op Operation, packFilter string, explicit bool) error {
	if _, err := exec.LookPath(SelfBin()); err != nil {
		return fmt.Errorf("packwand binary not found: %w", err)
	}
	root := ModpacksDir()
	if info, err := os.Stat(root); err != nil || !info.IsDir() {
		return fmt.Errorf("modpacks directory not found: %s", root)
	}

	targets, skipped := CollectTargets(root, op.HonorIgnore, packFilter, explicit)

	if len(skipped) > 0 {
		fmt.Printf("skipping %d opted-out pack(s):\n", len(skipped))
		for _, s := range skipped {
			fmt.Printf("  - %s\n", s)
		}
	}
	if len(targets) == 0 {
		fmt.Printf("no pack subdirs to %s.\n", op.Name)
		return nil
	}

	fmt.Printf("queued %d subdir(s), running up to %d in parallel\n", len(targets), MaxConcurrent())
	prog := NewProgress(op.Gerund, len(targets))
	failures := WorkPool(targets, op, prog)
	prog.Done()

	if len(failures) > 0 {
		fmt.Fprintf(os.Stderr, "\n%d subdir(s) failed:\n", len(failures))
		for _, f := range failures {
			fmt.Fprintf(os.Stderr, "  - %s\n", f)
		}
		return fmt.Errorf("one or more %ss failed", op.Name)
	}

	fmt.Printf("all %ss finished successfully.\n", op.Name)
	AutoLintDirs(targets)
	return nil
}

// ResolveScope converts command args into a (packFilter, explicit) pair.
// Pass startCwd as the directory where the user invoked the command.
func ResolveScope(args []string, startCwd string) (packFilter string, explicit bool) {
	for _, a := range args {
		if a == "--all" {
			return "", false
		}
	}
	for _, a := range args {
		if strings.HasPrefix(a, "-") {
			continue
		}
		dir, _ := filepath.Abs(strings.TrimRight(a, "/"))
		if _, err := os.Stat(filepath.Join(dir, "manifest.json")); err != nil {
			continue
		}
		return dir, true
	}
	if startCwd != "" {
		root := ModpacksDir()
		if rel, err := filepath.Rel(root, startCwd); err == nil && !strings.HasPrefix(rel, "..") && rel != "." {
			parts := strings.Split(filepath.ToSlash(rel), "/")
			if len(parts) >= 1 {
				pack := filepath.Join(root, parts[0])
				if _, err := os.Stat(filepath.Join(pack, "manifest.json")); err == nil {
					fmt.Printf("scoped to %s (run inside it; pass --all for every pack)\n", pack)
					return pack, false
				}
			}
		}
	}
	return "", false
}

// CheckUpdatesInDir runs packwand update --all --dry-run and returns found update strings.
func CheckUpdatesInDir(dir string) ([]string, error) {
	cmd := exec.Command(SelfBin(), "update", "--all", "-y", "--dry-run")
	cmd.Dir = dir
	ConfigureSubprocess(cmd)
	out, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("packwand update --dry-run: %w", err)
	}
	var updates []string
	inUpdates := false
	for _, line := range strings.Split(string(out), "\n") {
		line = strings.TrimSpace(line)
		if line == "Updates found:" {
			inUpdates = true
			continue
		}
		if strings.HasPrefix(line, "dry-run:") {
			break
		}
		if inUpdates && line != "" {
			updates = append(updates, line)
		}
	}
	return updates, nil
}

// â€” Linting â€”

// AutoLintDirs lints all .pw.toml files under mods/ in each given dir.
// Warnings are printed but the function never exits â€” it is a best-effort post-op check.
func AutoLintDirs(dirs []string) {
	seen := map[string]bool{}
	var files []string
	for _, dir := range dirs {
		modsDir := filepath.Join(dir, "mods")
		entries, _ := os.ReadDir(modsDir)
		for _, e := range entries {
			if !e.IsDir() && strings.HasSuffix(e.Name(), ".pw.toml") {
				p := filepath.Join(modsDir, e.Name())
				if !seen[p] {
					files = append(files, p)
					seen[p] = true
				}
			}
		}
	}
	if len(files) == 0 {
		return
	}
	fmt.Printf("linting %d mod file(s)...\n", len(files))
	failed, checked := RunLintFiles(files)
	if failed > 0 {
		fmt.Fprintf(os.Stderr, "warning: %d of %d mod file(s) failed lint â€” run 'packwand lint' for details\n", failed, checked)
	} else {
		fmt.Printf("âœ“ %d mod file(s) lint clean\n", checked)
	}
}

// RunLintFiles lints files concurrently and returns (failed, checked) counts.
func RunLintFiles(files []string) (failed, checked int64) {
	sched := NewScheduler(MaxConcurrent())
	dones := make([]<-chan error, 0, len(files))
	for _, f := range files {
		f := f
		if _, err := os.Stat(f); err != nil {
			continue
		}
		atomic.AddInt64(&checked, 1)
		dones = append(dones, sched.Submit(Task{
			Name: f,
			Run: func() error {
				if err := LintOne(f); err != nil {
					if isTTY() {
						fmt.Fprintf(os.Stderr, "  error: %s: %v\n", f, err)
					} else {
						fmt.Fprintf(os.Stderr, "::error file=%s::%v\n", f, err)
					}
					atomic.AddInt64(&failed, 1)
				}
				return nil
			},
		}))
	}
	sched.Close()
	for _, c := range dones {
		<-c
	}
	return failed, checked
}

// LintOne validates a single .json or .pw.toml file for syntax.
func LintOne(path string) error {
	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("could not read: %w", err)
	}
	if strings.HasSuffix(path, ".json") {
		var v any
		if err := json.Unmarshal(data, &v); err != nil {
			return fmt.Errorf("INVALID JSON: %w", err)
		}
		return nil
	}
	return lintTomlStructure(string(data))
}

func lintTomlStructure(content string) error {
	for i, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		if strings.HasPrefix(line, "[") {
			if !strings.HasSuffix(line, "]") {
				return fmt.Errorf("line %d: malformed section header: %q", i+1, line)
			}
			continue
		}
		if !strings.Contains(line, "=") {
			return fmt.Errorf("line %d: not a section or key=value: %q", i+1, line)
		}
	}
	return nil
}

// GitChangedFiles returns filenames changed in the last commit.
func GitChangedFiles() []string {
	out, err := exec.Command("git", "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").Output()
	if err != nil {
		return nil
	}
	var files []string
	for _, l := range strings.Split(string(out), "\n") {
		if l = strings.TrimSpace(l); l != "" {
			files = append(files, l)
		}
	}
	return files
}

// â€” Sync engine â€”

type syncJob struct {
	consumerID string
	baseID     string
	sourceDir  string
	targetDir  string
}

// RunSync performs the baseâ†’consumer pack sync. dryRun prints what would change.
func RunSync(dryRun bool) error {
	if _, err := exec.LookPath(SelfBin()); err != nil {
		return fmt.Errorf("packwand binary not found: %w", err)
	}
	root := ModpacksDir()
	packs, err := os.ReadDir(root)
	if err != nil {
		return fmt.Errorf("failed to read %s: %w", root, err)
	}

	isBase := make(map[string]bool)
	for _, p := range packs {
		if !p.IsDir() {
			continue
		}
		m, err := manifest.Read(filepath.Join(root, p.Name(), "manifest.json"))
		if err != nil {
			continue
		}
		if m.Role.IsBase() {
			isBase[m.ID] = true
		}
	}

	var jobs []syncJob
	for _, p := range packs {
		if !p.IsDir() {
			continue
		}
		packPath := filepath.Join(root, p.Name())
		m, err := manifest.Read(filepath.Join(packPath, "manifest.json"))
		if err != nil {
			continue
		}
		pb := m.Role.ConsumerBase()
		if pb == nil {
			continue
		}
		if !isBase[pb.Pack] {
			return fmt.Errorf("consumer '%s' references base '%s', which is not role 'base'", m.ID, pb.Pack)
		}
		basePackDir := filepath.Join(root, pb.Pack)
		for _, mp := range pb.Mappings {
			sp, tp := platformSuffix(mp.Source), platformSuffix(mp.Target)
			if sp == "" || tp == "" {
				return fmt.Errorf("consumer '%s': mapping %s->%s has a non -mr/-cf suffix", m.ID, mp.Source, mp.Target)
			}
			if sp != tp {
				return fmt.Errorf("consumer '%s': FORBIDDEN cross-platform mapping %s (%s) -> %s (%s)", m.ID, mp.Source, sp, mp.Target, tp)
			}
			src := filepath.Join(basePackDir, mp.Source)
			dst := filepath.Join(packPath, mp.Target)
			if _, err := os.Stat(src); err != nil {
				return fmt.Errorf("consumer '%s': mapping source %s missing in base '%s'", m.ID, mp.Source, pb.Pack)
			}
			if _, err := os.Stat(dst); err != nil {
				return fmt.Errorf("consumer '%s': mapping target %s missing in this pack", m.ID, mp.Target)
			}
			jobs = append(jobs, syncJob{m.ID, pb.Pack, src, dst})
		}
	}

	if len(jobs) == 0 {
		fmt.Println("no consumers to sync.")
		return nil
	}
	fmt.Printf("resolved %d sync job(s) from manifests\n", len(jobs))
	if dryRun {
		fmt.Println("[DRY RUN] no files will be copied, deleted, or refreshed")
	}

	syncedFolders := []string{"mods", "config", "resourcepacks", "global_packs"}
	sched := NewScheduler(MaxConcurrent())
	slots := CacheSlotCount()
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
	sched.Close()

	failed := 0
	for _, c := range dones {
		if err := <-c; err != nil {
			fmt.Fprintf(os.Stderr, "sync error: %v\n", err)
			failed++
		}
	}
	if failed > 0 {
		return fmt.Errorf("%d sync(s) failed", failed)
	}
	if dryRun {
		fmt.Println("[DRY RUN] complete â€” nothing was changed.")
	} else {
		fmt.Println("all syncs completed.")
	}
	return nil
}

func runSyncJob(j syncJob, dryRun bool, syncedFolders []string) error {
	fmt.Printf("syncing %s -> %s (base %s)\n", j.sourceDir, j.targetDir, j.baseID)

	excluded := readSyncExclude(filepath.Join(j.targetDir, "sync-exclude.json"))
	for _, f := range manifest.ReadAutomation(filepath.Dir(j.targetDir)).SyncExclude {
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
			if !excluded[slash] {
				provided[slash] = true
			}
		}
	}

	statePath := filepath.Join(j.targetDir, "sync.json")
	prev := readSyncState(statePath)
	var toDelete []string
	for f := range prev {
		if !excluded[f] && !provided[f] {
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
				if !excluded[slash] {
					placed[slash] = true
					kept++
				}
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
			return fmt.Errorf("ABORT: %s delete-set (%d) exceeds files the base provides (%d). Prior sync.json is likely stale. Delete it and re-run to reset.", j.targetDir, len(toDelete), len(provided))
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
					fmt.Fprintf(os.Stderr, "warning: could not delete %s: %v\n", p, err)
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
	cmd := exec.Command(SelfBin(), "refresh")
	cmd.Dir = j.targetDir
	ConfigureSubprocess(cmd)
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("packwand refresh failed in %s: %v\n%s", j.targetDir, err, Indent(string(out), "    "))
	}
	fmt.Printf("  refreshed %s\n", j.targetDir)
	return nil
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
		fmt.Fprintf(os.Stderr, "warning: ignoring %s (state version %d != %d); treating as fresh\n", path, st.Version, syncStateVersion)
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
		if err := CopyFileFast(path, target); err != nil {
			return err
		}
		placed[slash] = true
		count++
		return nil
	})
	return count, err
}
