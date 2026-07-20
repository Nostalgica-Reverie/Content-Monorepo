package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"

	"github.com/spf13/cobra"
)

// — workspace command group —

var workspaceCmd = &cobra.Command{
	Use:   "workspace",
	Short: "Multi-pack workspace operations across all packs",
}

func init() {
	workspaceCmd.GroupID = GroupWorkspace
	rootCmd.AddCommand(workspaceCmd)

	// status
	wsStatusCmd.Flags().Bool("json", false, "Output as JSON array instead of plain text")
	workspaceCmd.AddCommand(wsStatusCmd)

	// update
	wsUpdateCmd.Flags().Bool("all", false, "Run across all packs even when scoped")
	wsUpdateCmd.Flags().Bool("check", false, "Show what would update without applying (dry-run)")
	wsUpdateCmd.Flags().Bool("json", false, "With --check, output a JSON summary instead of plain text")
	wsUpdateCmd.Flags().Bool("ignored-only", false, "With --check, check packs opted out of auto-update instead of the normal set")
	wsUpdateCmd.Flags().String("report", "", "Write an aggregated machine-readable JSON update report to this file")
	workspaceCmd.AddCommand(wsUpdateCmd)

	// refresh
	wsRefreshCmd.Flags().Bool("all", false, "Run across all packs even when scoped")
	wsRefreshCmd.Flags().Bool("dry-run", false, "List pack subdirectories without refreshing them")
	workspaceCmd.AddCommand(wsRefreshCmd)

	// loader-update
	workspaceCmd.AddCommand(wsLoaderUpdateCmd)

	// migrate
	workspaceCmd.AddCommand(wsMigrateCmd)

	// sync
	wsSyncCmd.Flags().Bool("dry-run", false, "Show what would be synced without making changes")
	workspaceCmd.AddCommand(wsSyncCmd)

	// export
	wsExportCmd.Flags().Bool("all", false, "Run across all packs even when scoped")
	workspaceCmd.AddCommand(wsExportCmd)

	// mr / cf provider groups
	wsMrAddCmd.Flags().Bool("all", false, "Run across all packs even when scoped")
	wsMrCmd.AddCommand(wsMrAddCmd)
	workspaceCmd.AddCommand(wsMrCmd)
	wsCfAddCmd.Flags().Bool("all", false, "Run across all packs even when scoped")
	wsCfCmd.AddCommand(wsCfAddCmd)
	workspaceCmd.AddCommand(wsCfCmd)
}

// — status —

type WorkspaceStatus struct {
	ID         string                  `json:"id"`
	Name       string                  `json:"name"`
	Version    string                  `json:"version"`
	MCVersion  string                  `json:"mc_version,omitempty"`
	Loader     string                  `json:"loader,omitempty"`
	Lifecycle  string                  `json:"lifecycle"`
	AutoUpdate bool                    `json:"auto_update"`
	Subdirs    []WorkspaceSubdirStatus `json:"subdirs"`
	TotalMods  int                     `json:"total_mods"`
	FrozenMods int                     `json:"frozen_mods"`
}

type WorkspaceSubdirStatus struct {
	Key      string   `json:"key"`
	Platform string   `json:"platform"`
	ModCount int      `json:"mod_count"`
	Frozen   []string `json:"frozen,omitempty"`
}

var wsStatusCmd = &cobra.Command{
	Use:     "status",
	Short:   "Dashboard of all packs — version, mc, loader, mod counts, frozen mods",
	Aliases: []string{"info"},
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		asJSON, _ := cmd.Flags().GetBool("json")

		root := workspace.ModpacksDir()
		entries, err := os.ReadDir(root)
		if err != nil {
			llFail(fmt.Sprintf("failed to read %s: %v", root, err))
		}

		// Read-only scan: manifest reads + mods-dir listings fan out in
		// process (no subprocess spawns); results keep directory order.
		results := make([]*WorkspaceStatus, len(entries))
		core.ParallelFor(entries, core.MaxConcurrent(), func(idx int, e os.DirEntry) {
			if !e.IsDir() {
				return
			}
			packPath := filepath.Join(root, e.Name())
			m, err := manifest.Read(filepath.Join(packPath, "manifest.json"))
			if err != nil {
				return
			}

			auto := manifest.ReadAutomation(packPath)
			autoUpdate := auto.AutoUpdate == nil || *auto.AutoUpdate

			var subdirs []WorkspaceSubdirStatus
			totalMods, totalFrozen := 0, 0

			for _, sub := range manifest.SubDirsOf(packPath) {
				key := filepath.Base(sub)
				plat := "?"
				if strings.HasSuffix(key, "-mr") {
					plat = "mr"
				} else if strings.HasSuffix(key, "-cf") {
					plat = "cf"
				}

				modsDir := filepath.Join(sub, "mods")
				modEntries, _ := os.ReadDir(modsDir)
				modCount := 0
				for _, me := range modEntries {
					if !me.IsDir() && strings.HasSuffix(me.Name(), ".pw.toml") {
						modCount++
					}
				}

				frozen := auto.Freeze[key]
				totalMods += modCount
				totalFrozen += len(frozen)

				subdirs = append(subdirs, WorkspaceSubdirStatus{
					Key:      key,
					Platform: plat,
					ModCount: modCount,
					Frozen:   frozen,
				})
			}

			mcVersion := ""
			if m.MCVersion != nil {
				mcVersion = *m.MCVersion
			}

			results[idx] = &WorkspaceStatus{
				ID:         m.ID,
				Name:       m.Name,
				Version:    m.Version,
				MCVersion:  mcVersion,
				Loader:     m.Loader,
				Lifecycle:  m.Lifecycle,
				AutoUpdate: autoUpdate,
				Subdirs:    subdirs,
				TotalMods:  totalMods,
				FrozenMods: totalFrozen,
			}
		})
		var statuses []WorkspaceStatus
		for _, r := range results {
			if r != nil {
				statuses = append(statuses, *r)
			}
		}

		if asJSON {
			data, _ := json.MarshalIndent(statuses, "", "  ")
			fmt.Println(string(data))
			return
		}

		if len(statuses) == 0 {
			fmt.Println("no packs found")
			return
		}

		if Interactive() {
			rows := make([][]string, 0, len(statuses))
			for _, status := range statuses {
				lifecycle := status.Lifecycle
				if lifecycle == "" {
					lifecycle = "active"
				}
				auto := "yes"
				if !status.AutoUpdate {
					auto = "no"
				}
				rows = append(rows, []string{status.ID, status.Version, status.MCVersion, status.Loader, lifecycle, fmt.Sprint(status.TotalMods), fmt.Sprint(status.FrozenMods), auto})
			}
			fmt.Fprintln(os.Stderr, Table([]string{"PACK", "VERSION", "MC", "LOADER", "LIFECYCLE", "MODS", "FROZEN", "UPDATE"}, rows))
			return
		}
		for _, s := range statuses {
			autoStr := "auto-update"
			if !s.AutoUpdate {
				autoStr = "no-update"
			}
			frozenNote := ""
			if s.FrozenMods > 0 {
				frozenNote = fmt.Sprintf("  %d frozen", s.FrozenMods)
			}
			lcStr := s.Lifecycle
			if lcStr == "" {
				lcStr = "active"
			}
			fmt.Printf("%s  v%s  mc%s  %s  [%s]  [%s]  %d mods%s\n",
				s.ID, s.Version, s.MCVersion, s.Loader, autoStr, lcStr, s.TotalMods, frozenNote)
			for _, sub := range s.Subdirs {
				subFrozen := ""
				if len(sub.Frozen) > 0 {
					subFrozen = fmt.Sprintf("  (%d frozen)", len(sub.Frozen))
				}
				fmt.Printf("    %-32s [%s]  %d mods%s\n", sub.Key, sub.Platform, sub.ModCount, subFrozen)
			}
		}
		fmt.Printf("\n%d pack(s)\n", len(statuses))
	},
}

// — export —

// filterPlatformSubdirs keeps targets whose base name carries a platform
// suffix. An empty suffix keeps every platform subdir (-mr and -cf).
func filterPlatformSubdirs(targets []string, suffix string) []string {
	var out []string
	for _, t := range targets {
		base := filepath.Base(t)
		if suffix != "" {
			if strings.HasSuffix(base, suffix) {
				out = append(out, t)
			}
		} else if strings.HasSuffix(base, "-mr") || strings.HasSuffix(base, "-cf") {
			out = append(out, t)
		}
	}
	return out
}

var wsExportCmd = &cobra.Command{
	Use:   "export [pack-dir]",
	Short: "Run mr/cf export in every platform pack subdir (files land in each subdir)",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		all, _ := cmd.Flags().GetBool("all")
		packFilter, explicit := resolveWorkspaceScope(args, llStartCwd, all)
		targets, _ := workspace.CollectTargets(workspace.ModpacksDir(), false, packFilter, explicit)
		targets = filterPlatformSubdirs(targets, "")
		if len(targets) == 0 {
			fmt.Println("no platform pack subdirs (-mr/-cf) found")
			return
		}
		op := workspace.Operation{
			Name:   "export",
			Gerund: "exporting",
			// The platform subcommand depends on the subdir's suffix, so the
			// whole argv is per-target.
			ExtraArgsFor: func(dir string) []string {
				if strings.HasSuffix(filepath.Base(dir), "-cf") {
					return []string{"cf", "export"}
				}
				return []string{"mr", "export"}
			},
		}
		fmt.Printf("exporting %d subdir(s), running up to %d in parallel\n", len(targets), workspace.MaxConcurrent())
		if failures := workspace.WorkPool(targets, op, nil); len(failures) > 0 {
			llFail(fmt.Sprintf("export failed in %d subdir(s): %s", len(failures), strings.Join(failures, ", ")))
		}
		fmt.Printf("exported %d subdir(s)\n", len(targets))
	},
}

// — mr/cf add —

var wsMrCmd = &cobra.Command{
	Use:     "mr",
	Short:   "Modrinth operations across all packs",
	Aliases: []string{"modrinth"},
}

var wsCfCmd = &cobra.Command{
	Use:     "cf",
	Short:   "CurseForge operations across all packs",
	Aliases: []string{"curseforge"},
}

var wsMrAddCmd = &cobra.Command{
	Use:     "add <slug-or-url>...",
	Short:   "Add Modrinth mod(s) to every -mr pack subdir",
	Aliases: []string{"install"},
	Args:    cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		all, _ := cmd.Flags().GetBool("all")
		wsPlatformAdd("mr", "-mr", args, all)
	},
}

var wsCfAddCmd = &cobra.Command{
	Use:     "add <slug-or-url>...",
	Short:   "Add CurseForge mod(s) to every -cf pack subdir",
	Aliases: []string{"install"},
	Args:    cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		all, _ := cmd.Flags().GetBool("all")
		wsPlatformAdd("cf", "-cf", args, all)
	},
}

// wsPlatformAdd installs each slug into every pack subdir of the matching
// platform. Children run non-interactively (ConfigureSubprocess), so
// ambiguous search terms resolve to the default choice — prefer exact slugs
// or URLs here.
func wsPlatformAdd(cli, suffix string, slugs []string, all bool) {
	llChdir()
	packFilter, explicit := resolveWorkspaceScope(nil, llStartCwd, all)
	targets, _ := workspace.CollectTargets(workspace.ModpacksDir(), false, packFilter, explicit)
	targets = filterPlatformSubdirs(targets, suffix)
	if len(targets) == 0 {
		fmt.Printf("no %s pack subdirs found\n", suffix)
		return
	}
	failedTotal := 0
	for _, slug := range slugs {
		op := workspace.Operation{
			Name:        cli + "-add",
			Gerund:      "adding " + slug + " in",
			PackwizArgs: []string{cli, "add", slug},
		}
		fmt.Printf("adding %s to %d subdir(s), running up to %d in parallel\n", slug, len(targets), workspace.MaxConcurrent())
		failedTotal += len(workspace.WorkPool(targets, op, nil))
	}
	if failedTotal > 0 {
		llFail(fmt.Sprintf("%d add operation(s) failed", failedTotal))
	}
}

// — update —

var wsUpdateCmd = &cobra.Command{
	Use:   "update [pack-dir]",
	Short: "Run packwand update --all in every pack subdir (honors auto_update)",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		all, _ := cmd.Flags().GetBool("all")
		check, _ := cmd.Flags().GetBool("check")
		if check {
			asJSON, _ := cmd.Flags().GetBool("json")
			ignoredOnly, _ := cmd.Flags().GetBool("ignored-only")
			wsRunUpdateCheck(args, asJSON, ignoredOnly, all)
			return
		}
		reportPath, _ := cmd.Flags().GetString("report")
		packFilter, explicit := resolveWorkspaceScope(args, llStartCwd, all)

		op := workspace.OpUpdate
		var reportDir string
		if reportPath != "" {
			var err error
			reportDir, err = os.MkdirTemp("", "packwand-update-report-")
			if err != nil {
				llFail(fmt.Sprintf("failed to create report temp dir: %v", err))
			}
			defer os.RemoveAll(reportDir)
			op.ExtraArgsFor = func(dir string) []string {
				return []string{"--report", filepath.Join(reportDir, wsReportFileName(dir))}
			}
		}

		runErr := workspace.Run(op, packFilter, explicit)
		if reportPath != "" {
			wsWriteAggregateReport(reportPath, reportDir, runErr)
		}
		if runErr != nil {
			llFail(runErr.Error())
		}
	},
}

// wsReportFileName maps a pack subdir path to a unique, filesystem-safe name.
func wsReportFileName(dir string) string {
	r := strings.NewReplacer("/", "_", "\\", "_", ":", "_", " ", "_")
	return r.Replace(dir) + ".json"
}

// workspaceUpdateReport is the aggregate of per-subdir UpdateReports; the GUI
// and CI both render this shape.
type workspaceUpdateReport struct {
	GeneratedAt string         `json:"generated_at"`
	RunError    string         `json:"run_error,omitempty"`
	Packs       []UpdateReport `json:"packs"`
	Totals      struct {
		Updated      int `json:"updated"`
		Pinned       int `json:"pinned"`
		Incompatible int `json:"incompatible"`
		Failed       int `json:"failed"`
		Skipped      int `json:"skipped"`
		UpToDate     int `json:"up_to_date"`
		Checked      int `json:"checked"`
	} `json:"totals"`
}

func wsWriteAggregateReport(reportPath, reportDir string, runErr error) {
	agg := workspaceUpdateReport{
		GeneratedAt: time.Now().UTC().Format(time.RFC3339),
		Packs:       []UpdateReport{},
	}
	if runErr != nil {
		agg.RunError = runErr.Error()
	}

	entries, err := os.ReadDir(reportDir)
	if err != nil {
		llWarn("update report: failed to read %s: %v", reportDir, err)
	}
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".json") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(reportDir, e.Name()))
		if err != nil {
			llWarn("update report: failed to read %s: %v", e.Name(), err)
			continue
		}
		var rep UpdateReport
		if err := json.Unmarshal(data, &rep); err != nil {
			llWarn("update report: invalid JSON in %s: %v", e.Name(), err)
			continue
		}
		agg.Packs = append(agg.Packs, rep)
		agg.Totals.Updated += len(rep.Updated)
		agg.Totals.Pinned += len(rep.Pinned)
		agg.Totals.Incompatible += len(rep.Incompatible)
		agg.Totals.Failed += len(rep.Failed)
		agg.Totals.Skipped += len(rep.Skipped)
		agg.Totals.UpToDate += rep.UpToDate
		agg.Totals.Checked += rep.Checked
	}
	sort.Slice(agg.Packs, func(i, j int) bool { return agg.Packs[i].Dir < agg.Packs[j].Dir })

	data, err := json.MarshalIndent(agg, "", "  ")
	if err != nil {
		llWarn("update report: failed to render: %v", err)
		return
	}
	if err := os.WriteFile(reportPath, append(data, '\n'), 0o644); err != nil {
		llWarn("update report: failed to write %s: %v", reportPath, err)
		return
	}
	fmt.Printf("update report written to %s (%d pack subdir(s), %d updated, %d failed)\n",
		reportPath, len(agg.Packs), agg.Totals.Updated, agg.Totals.Failed)
}

// WorkspaceUpdateCheckSubdir is one pack subdir's outcome within
// `workspace update --check --json`.
type WorkspaceUpdateCheckSubdir struct {
	Dir     string   `json:"dir"`
	Ignored bool     `json:"ignored,omitempty"`
	Updates []string `json:"updates,omitempty"`
	Error   string   `json:"error,omitempty"`
}

// WorkspaceUpdateCheckResult is the machine-readable outcome of
// `workspace update --check --json`.
type WorkspaceUpdateCheckResult struct {
	Subdirs      []WorkspaceUpdateCheckSubdir `json:"subdirs"`
	TotalUpdates int                          `json:"total_updates"`
	FailedChecks int                          `json:"failed_checks"`
}

// ignoredPackSubdirs returns subdirs of packs opted out of auto-update
// (via manifest.json automation.auto_update or the legacy
// auto-update-ignore.json file), excluding archived/EOL packs — those are
// a separate, unrelated reason to skip a pack.
func ignoredPackSubdirs(root string) []string {
	entries, err := os.ReadDir(root)
	if err != nil {
		return nil
	}
	var subdirs []string
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		packPath := filepath.Join(root, e.Name())
		if lc := manifest.LifecycleState(packPath); lc == "archived" || lc == "eol" {
			continue
		}
		if skip, _ := manifest.OptedOutOfAutoUpdate(packPath); skip {
			subdirs = append(subdirs, manifest.SubDirsOf(packPath)...)
		}
	}
	return subdirs
}

func resolveWorkspaceScope(args []string, startCwd string, all bool) (packFilter string, explicit bool) {
	if all {
		return "", false
	}
	return workspace.ResolveScope(args, startCwd)
}

func wsRunUpdateCheck(args []string, asJSON, ignoredOnly, all bool) {
	root := workspace.ModpacksDir()
	var targets []string
	if ignoredOnly {
		targets = ignoredPackSubdirs(root)
	} else {
		packFilter, _ := resolveWorkspaceScope(args, llStartCwd, all)
		targets, _ = workspace.CollectTargets(root, true, packFilter, false)
	}
	if len(targets) == 0 {
		if asJSON {
			printJSON(WorkspaceUpdateCheckResult{Subdirs: []WorkspaceUpdateCheckSubdir{}})
			return
		}
		fmt.Println("no pack subdirs to check")
		return
	}

	type checkOutput struct {
		dir     string
		updates []string
		err     error
	}
	if !asJSON {
		fmt.Printf("checking %d subdir(s), running up to %d in parallel\n", len(targets), workspace.MaxConcurrent())
	}
	results := make([]checkOutput, len(targets))
	sched := workspace.NewScheduler(workspace.MaxConcurrent())
	dones := make([]<-chan error, len(targets))
	for i, dir := range targets {
		i, dir := i, dir
		dones[i] = sched.Submit(workspace.Task{
			Name:  dir,
			Needs: []workspace.Resource{workspace.Resource("check:" + dir)},
			Run: func() error {
				updates, err := workspace.CheckUpdatesInDir(dir)
				results[i] = checkOutput{dir: dir, updates: updates, err: err}
				return nil
			},
		})
	}
	sched.Close()
	for _, done := range dones {
		<-done
	}

	totalUpdates := 0
	failedChecks := 0
	jsonResult := WorkspaceUpdateCheckResult{Subdirs: make([]WorkspaceUpdateCheckSubdir, 0, len(results))}
	for _, result := range results {
		if result.err != nil {
			failedChecks++
			if asJSON {
				jsonResult.Subdirs = append(jsonResult.Subdirs, WorkspaceUpdateCheckSubdir{Dir: result.dir, Ignored: ignoredOnly, Error: result.err.Error()})
			} else {
				llWarn("%s: check failed: %v", result.dir, result.err)
			}
			continue
		}
		if len(result.updates) > 0 {
			totalUpdates += len(result.updates)
			if asJSON {
				jsonResult.Subdirs = append(jsonResult.Subdirs, WorkspaceUpdateCheckSubdir{Dir: result.dir, Ignored: ignoredOnly, Updates: result.updates})
				continue
			}
			fmt.Printf("%s: %d update(s) available\n", result.dir, len(result.updates))
			for _, u := range result.updates {
				fmt.Printf("  ~ %s\n", u)
			}
		} else if !asJSON {
			fmt.Printf("%s: up to date\n", result.dir)
		}
	}

	if asJSON {
		jsonResult.TotalUpdates = totalUpdates
		jsonResult.FailedChecks = failedChecks
		printJSON(jsonResult)
		if failedChecks > 0 {
			os.Exit(1)
		}
		return
	}

	if totalUpdates == 0 && failedChecks == 0 {
		fmt.Println("\neverything is up to date")
	} else if totalUpdates > 0 {
		fmt.Printf("\n%d update(s) available — run 'packwand workspace update' to apply\n", totalUpdates)
	}
	if failedChecks > 0 {
		llFail(fmt.Sprintf("%d update check(s) failed", failedChecks))
	}
}

// — refresh —

var wsRefreshCmd = &cobra.Command{
	Use:   "refresh [pack-dir]",
	Short: "Run packwand refresh in every pack subdir",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		all, _ := cmd.Flags().GetBool("all")
		packFilter, explicit := resolveWorkspaceScope(args, llStartCwd, all)
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		if dryRun {
			targets, _ := workspace.CollectTargets(workspace.ModpacksDir(), false, packFilter, explicit)
			fmt.Printf("dry-run: %d pack subdir(s) would be refreshed\n", len(targets))
			for _, target := range targets {
				fmt.Printf("  - %s\n", target)
			}
			return
		}
		if err := workspace.Run(workspace.OpRefresh, packFilter, explicit); err != nil {
			llFail(err.Error())
		}
	},
}

// — loader-update —

var wsLoaderUpdateCmd = &cobra.Command{
	Use:   "loader-update [latest|recommended] [pack-dir]",
	Short: "Migrate loaders across packs (honors auto_update)",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		target := "latest"
		var rest []string
		for _, a := range args {
			if a == "latest" || a == "recommended" {
				target = a
			} else {
				rest = append(rest, a)
			}
		}
		op := workspace.Operation{
			Name:        "loader-update",
			Gerund:      "migrating loader (" + target + ") in",
			PackwizArgs: []string{"migrate", "loader", target},
			HonorIgnore: true,
		}
		packFilter, explicit := workspace.ResolveScope(rest, llStartCwd)
		if err := workspace.Run(op, packFilter, explicit); err != nil {
			llFail(err.Error())
		}
	},
}

// — migrate —

var wsMigrateCmd = &cobra.Command{
	Use:   "migrate [format|loader [version]|minecraft [version]]",
	Short: "Run packwand migrate across all pack subdirs",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		packFilter, explicit := workspace.ResolveScope(nil, llStartCwd)
		op := workspace.Operation{
			Name:        "migrate-" + args[0],
			Gerund:      "migrating (" + strings.Join(args, " ") + ") in",
			PackwizArgs: append([]string{"migrate"}, args...),
			HonorIgnore: false,
		}
		if err := workspace.Run(op, packFilter, explicit); err != nil {
			llFail(err.Error())
		}
	},
}

// — sync —

var wsSyncCmd = &cobra.Command{
	Use:   "sync",
	Short: "Copy performance base content into consumer packs per manifest mappings",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		if err := workspace.RunSync(dryRun); err != nil {
			llFail(err.Error())
		}
		if !dryRun {
			// Regenerate docs after sync
			fmt.Println()
			runPages("", false)
		}
	},
}
