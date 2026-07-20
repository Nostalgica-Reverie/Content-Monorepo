package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	llAutomationRunCmd.Flags().Bool("dry-run", false, "Run update/refresh/validate/tests/docs but skip the version bump")
	llAutomationRunCmd.Flags().String("report", "", "Write the JSON run report to this file")
	llAutomationRunCmd.Flags().Bool("json", false, "Print the run report as JSON instead of text")
	llAutomationCmd.AddCommand(llAutomationRunCmd)

	llAutomationCmd.AddCommand(llAutomationListFullAutoCmd)
}

// — automation run —

// AutomateStepResult is one pipeline step's outcome within an automation run.
type AutomateStepResult struct {
	Name   string `json:"name"`
	Status string `json:"status"` // ok | skipped | failed
	Detail string `json:"detail,omitempty"`
}

// AutomateReport is the machine-readable outcome of `automation run`.
type AutomateReport struct {
	PackDir     string               `json:"pack_dir"`
	PackID      string               `json:"pack_id"`
	Status      string               `json:"status"` // no_changes | ready_to_publish | failed
	DryRun      bool                 `json:"dry_run,omitempty"`
	OldVersion  string               `json:"old_version,omitempty"`
	NewVersion  string               `json:"new_version,omitempty"`
	Steps       []AutomateStepResult `json:"steps"`
	Error       string               `json:"error,omitempty"`
	GeneratedAt string               `json:"generated_at"`
}

var llAutomationRunCmd = &cobra.Command{
	Use:   "run <pack-dir>",
	Short: "Run the unattended release pipeline for a full_auto-enabled pack (update, refresh, nix-gen, validate, tests, docs, bump)",
	Long: "Runs update -> refresh -> nix-gen -> validate -> tests -> docs -> bump for a single pack that has " +
		"opted in via manifest.json \"automation\": { \"full_auto\": { \"enabled\": true } }. Stops after " +
		"bumping the manifest in the working tree — it never commits, builds, or publishes. Committing " +
		"and pushing the result is left to the caller (CI); pushing a version-bumped manifest to main " +
		"is what the existing 'packwand publish plan' / publish.yml pipeline already reacts to.",
	Args: cobra.ExactArgs(1),
	Run: func(c *cobra.Command, args []string) {
		packDir := llAbs(strings.TrimRight(strings.TrimRight(args[0], "/"), "\\"))
		dryRun, _ := c.Flags().GetBool("dry-run")
		reportPath, _ := c.Flags().GetString("report")
		asJSON, _ := c.Flags().GetBool("json")

		llChdir()
		rep := runAutomate(packDir, dryRun)
		emitAutomateReport(rep, reportPath, asJSON)
		writeAutomateGithubOutput(rep)
		if rep.Status == "failed" {
			llFail(rep.Error)
		}
	},
}

func runAutomate(packDir string, dryRun bool) *AutomateReport {
	rep := &AutomateReport{
		PackDir:     packDir,
		Steps:       []AutomateStepResult{},
		DryRun:      dryRun,
		GeneratedAt: time.Now().UTC().Format(time.RFC3339),
	}

	mfPath := filepath.Join(packDir, "manifest.json")
	m, err := manifest.Read(mfPath)
	if err != nil {
		return failReport(rep, "read-manifest", fmt.Sprintf("failed to read %s: %v", mfPath, err))
	}
	rep.PackID = m.ID
	rep.OldVersion = m.Version

	if !manifest.FullAutoEnabled(packDir) {
		return failReport(rep, "opt-in-check", fmt.Sprintf("%s is not opted into full automation (set automation.full_auto.enabled in manifest.json)", packDir))
	}

	dirty, err := gitDirty(packDir)
	if err != nil {
		return failReport(rep, "clean-check", fmt.Sprintf("failed to check git status for %s: %v", packDir, err))
	}
	if dirty {
		return failReport(rep, "clean-check", fmt.Sprintf("%s has uncommitted changes; commit or stash before running automation", packDir))
	}
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: "clean-check", Status: "ok"})

	targets := manifest.SubDirsOf(packDir)
	if len(targets) == 0 {
		return failReport(rep, "update", fmt.Sprintf("no pack subdirs (ending -mr/-cf) found under %s", packDir))
	}

	// Pre-validate before mutating anything: a manifest complaint costs
	// seconds here versus minutes after update/refresh — which a later
	// validate failure would then roll back wholesale.
	if out, err := exec.Command(workspace.SelfBin(), "validate", mfPath).CombinedOutput(); err != nil {
		return failReport(rep, "pre-validate", lastLines(string(out), 20))
	}
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: "pre-validate", Status: "ok"})

	if failures := workspace.WorkPool(targets, workspace.OpUpdate, nil); len(failures) > 0 {
		return rollbackFail(rep, packDir, "update", fmt.Sprintf("update failed in: %s", strings.Join(failures, ", ")))
	}
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: "update", Status: "ok"})

	if failures := workspace.WorkPool(targets, workspace.OpRefresh, nil); len(failures) > 0 {
		return rollbackFail(rep, packDir, "refresh", fmt.Sprintf("refresh failed in: %s", strings.Join(failures, ", ")))
	}
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: "refresh", Status: "ok"})
	workspace.AutoLintDirs(targets)

	dirty, err = gitDirty(packDir)
	if err != nil {
		return rollbackFail(rep, packDir, "no-op-guard", fmt.Sprintf("failed to check git status for %s: %v", packDir, err))
	}
	if !dirty {
		rep.Steps = append(rep.Steps, AutomateStepResult{Name: "no-op-guard", Status: "ok", Detail: "no changes after update/refresh"})
		rep.Status = "no_changes"
		return rep
	}

	// Regenerate Nix checksum inventories for subdirs that maintain one, so
	// the flake's view of the pack (checks.modpack-inventory, legacyPackages)
	// lands in the same commit as the version bump instead of lagging it.
	// Subdirs without a checksums.json haven't opted into Nix consumption
	// (e.g. CurseForge metadata-mode packs) and are left alone.
	regenerated := 0
	for _, target := range targets {
		if _, statErr := os.Stat(filepath.Join(target, "checksums.json")); statErr != nil {
			continue
		}
		c := exec.Command(workspace.SelfBin(), "nix", "gen")
		c.Dir = target
		workspace.ConfigureSubprocess(c)
		if out, err := c.CombinedOutput(); err != nil {
			return rollbackFail(rep, packDir, "nix-gen", lastLines(string(out), 5))
		}
		regenerated++
	}
	if regenerated > 0 {
		rep.Steps = append(rep.Steps, AutomateStepResult{Name: "nix-gen", Status: "ok", Detail: fmt.Sprintf("%d checksum inventory(ies) regenerated", regenerated)})
	} else {
		rep.Steps = append(rep.Steps, AutomateStepResult{Name: "nix-gen", Status: "skipped", Detail: "no checksums.json maintained in this pack"})
	}

	if out, err := exec.Command(workspace.SelfBin(), "validate", mfPath).CombinedOutput(); err != nil {
		return rollbackFail(rep, packDir, "validate", lastLines(string(out), 20))
	}
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: "validate", Status: "ok"})

	auto := manifest.ReadAutomation(packDir)
	var tests []string
	if auto.FullAuto != nil {
		tests = auto.FullAuto.Tests
	}
	for i, testCmd := range tests {
		cmd := exec.Command("sh", "-c", testCmd)
		cmd.Dir = packDir
		out, err := cmd.CombinedOutput()
		if err != nil {
			return rollbackFail(rep, packDir, fmt.Sprintf("test[%d]", i), fmt.Sprintf("%q failed: %v\n%s", testCmd, err, lastLines(string(out), 10)))
		}
	}
	if len(tests) > 0 {
		rep.Steps = append(rep.Steps, AutomateStepResult{Name: "tests", Status: "ok", Detail: fmt.Sprintf("%d command(s) passed", len(tests))})
	} else {
		rep.Steps = append(rep.Steps, AutomateStepResult{Name: "tests", Status: "skipped", Detail: "no automation.full_auto.tests configured"})
	}

	runPages(packDir, false)
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: "docs", Status: "ok"})

	newVersion := nextCalVer(m.Version)
	rep.NewVersion = newVersion

	if dryRun {
		rep.Steps = append(rep.Steps, AutomateStepResult{Name: "bump", Status: "skipped", Detail: fmt.Sprintf("--dry-run: would bump %s -> %s", m.Version, newVersion)})
		rep.Status = "ready_to_publish"
		return rep
	}

	if out, err := exec.Command(workspace.SelfBin(), "bump", packDir, newVersion, "--configs").CombinedOutput(); err != nil {
		return rollbackFail(rep, packDir, "bump", lastLines(string(out), 10))
	}
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: "bump", Status: "ok", Detail: fmt.Sprintf("%s -> %s", m.Version, newVersion)})

	rep.Status = "ready_to_publish"
	return rep
}

func failReport(rep *AutomateReport, step, msg string) *AutomateReport {
	rep.Steps = append(rep.Steps, AutomateStepResult{Name: step, Status: "failed", Detail: msg})
	rep.Status = "failed"
	rep.Error = fmt.Sprintf("%s: %s", step, msg)
	return rep
}

// rollbackFail discards any working-tree changes automation made to packDir
// (update/refresh/docs/bump are all local, uncommitted mutations up to this
// point) before recording the failure, so a failed run never leaves a pack
// dir half-changed.
func rollbackFail(rep *AutomateReport, packDir, step, msg string) *AutomateReport {
	rollbackPackDir(packDir)
	return failReport(rep, step, msg)
}

func rollbackPackDir(packDir string) {
	exec.Command("git", "checkout", "--", packDir).Run()     //nolint:errcheck
	exec.Command("git", "clean", "-fd", "--", packDir).Run() //nolint:errcheck
}

func gitDirty(packDir string) (bool, error) {
	out, err := exec.Command("git", "status", "--porcelain", "--", packDir).Output()
	if err != nil {
		return false, err
	}
	return strings.TrimSpace(string(out)) != "", nil
}

func lastLines(s string, n int) string {
	s = strings.TrimSpace(s)
	if s == "" {
		return ""
	}
	lines := strings.Split(s, "\n")
	if len(lines) > n {
		lines = lines[len(lines)-n:]
	}
	return strings.Join(lines, "\n")
}

// calVerRe already validates the shape in validate.go; this regex additionally
// captures the cycle/patch groups needed to compute the next version.
var calVerCyclePatchRe = regexp.MustCompile(`^(\d{2}\.\d{2})(?:\.(\d+))?`)

// nextCalVer computes the next CalVer version from current, following the
// convention already used across the workspace (e.g. "26.06" -> "26.06.1"
// within the same month; a new month starts a bare "YY.MM" with no patch
// suffix, matching how vital/qualt/lce-common are versioned today).
func nextCalVer(current string) string {
	return nextCalVerAt(current, time.Now().UTC())
}

func nextCalVerAt(current string, now time.Time) string {
	cycle := fmt.Sprintf("%02d.%02d", now.Year()%100, int(now.Month()))
	m := calVerCyclePatchRe.FindStringSubmatch(current)
	if m == nil || m[1] != cycle {
		return cycle
	}
	patch := 0
	if m[2] != "" {
		patch, _ = strconv.Atoi(m[2])
	}
	return fmt.Sprintf("%s.%d", cycle, patch+1)
}

func lastStepName(rep *AutomateReport) string {
	if len(rep.Steps) == 0 {
		return "start"
	}
	return rep.Steps[len(rep.Steps)-1].Name
}

func emitAutomateReport(rep *AutomateReport, reportPath string, asJSON bool) {
	if reportPath != "" {
		llWriteJSON(reportPath, rep)
	}
	if asJSON {
		printJSON(rep)
		return
	}
	fmt.Printf("automation run: %s (%s)\n", rep.PackDir, rep.PackID)
	for _, s := range rep.Steps {
		fmt.Printf("  %-14s %-8s %s\n", s.Name, s.Status, s.Detail)
	}
	switch rep.Status {
	case "no_changes":
		fmt.Println("result: up to date, nothing to publish")
	case "ready_to_publish":
		if rep.DryRun {
			fmt.Printf("result: [DRY RUN] would bump %s -> %s\n", rep.OldVersion, rep.NewVersion)
		} else {
			fmt.Printf("result: bumped %s -> %s; commit + push to trigger publish.yml\n", rep.OldVersion, rep.NewVersion)
		}
	case "failed":
		fmt.Printf("result: FAILED at %s - working tree rolled back\n", lastStepName(rep))
	}
}

func writeAutomateGithubOutput(rep *AutomateReport) {
	outPath := os.Getenv("GITHUB_OUTPUT")
	if outPath == "" {
		return
	}
	f, err := os.OpenFile(outPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		llWarn("failed to open GITHUB_OUTPUT: %v", err)
		return
	}
	defer f.Close()
	changed := rep.Status == "ready_to_publish" && !rep.DryRun
	fmt.Fprintf(f, "changed=%t\n", changed)
	fmt.Fprintf(f, "new_version=%s\n", rep.NewVersion)
	fmt.Fprintf(f, "pack_id=%s\n", rep.PackID)
}

// — automation list-full-auto —

// FullAutoListEntry is one pack in `automation list-full-auto`'s output.
type FullAutoListEntry struct {
	Dir     string `json:"dir"`
	ID      string `json:"id"`
	Version string `json:"version"`
}

var llAutomationListFullAutoCmd = &cobra.Command{
	Use:   "list-full-auto",
	Short: "List pack directories opted into automation.full_auto (JSON on stdout)",
	Args:  cobra.NoArgs,
	Run: func(c *cobra.Command, args []string) {
		llChdir()
		listFullAuto()
	},
}

func listFullAuto() {
	root := workspace.ModpacksDir()
	entries, err := os.ReadDir(root)
	if err != nil {
		llFail(fmt.Sprintf("failed to read %s: %v", root, err))
	}

	found := []FullAutoListEntry{}
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		dir := filepath.Join(root, e.Name())
		if !manifest.FullAutoEnabled(dir) {
			continue
		}
		m, err := manifest.Read(filepath.Join(dir, "manifest.json"))
		if err != nil {
			llWarn("list-full-auto: %v", err)
			continue
		}
		found = append(found, FullAutoListEntry{Dir: filepath.ToSlash(dir), ID: m.ID, Version: m.Version})
	}
	sort.Slice(found, func(i, j int) bool { return found[i].ID < found[j].ID })

	for _, f := range found {
		fmt.Fprintf(os.Stderr, "full-auto: include %s (%s)\n", f.Dir, f.ID)
	}

	data, err := json.Marshal(found)
	if err != nil {
		llFail(fmt.Sprintf("failed to render list: %v", err))
	}
	fmt.Println(string(data))

	if outPath := os.Getenv("GITHUB_OUTPUT"); outPath != "" {
		f, err := os.OpenFile(outPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
		if err != nil {
			llFail(fmt.Sprintf("failed to open GITHUB_OUTPUT: %v", err))
		}
		defer f.Close()
		dirs := make([]string, len(found))
		for i, e := range found {
			dirs[i] = e.Dir
		}
		dirsJSON, _ := json.Marshal(dirs)
		fmt.Fprintf(f, "entries=%s\n", dirsJSON)
		fmt.Fprintf(f, "has_entries=%t\n", len(found) > 0)
	}
}
