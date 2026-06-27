package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/workspace"

	"github.com/spf13/cobra"
)

// â€” workspace command group â€”

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
	workspaceCmd.AddCommand(wsUpdateCmd)

	// refresh
	wsRefreshCmd.Flags().Bool("all", false, "Run across all packs even when scoped")
	workspaceCmd.AddCommand(wsRefreshCmd)

	// loader-update
	workspaceCmd.AddCommand(wsLoaderUpdateCmd)

	// migrate
	workspaceCmd.AddCommand(wsMigrateCmd)

	// sync
	wsSyncCmd.Flags().Bool("dry-run", false, "Show what would be synced without making changes")
	workspaceCmd.AddCommand(wsSyncCmd)
}

// â€” status â€”

type packStatus struct {
	ID         string       `json:"id"`
	Name       string       `json:"name"`
	Version    string       `json:"version"`
	MCVersion  string       `json:"mc_version,omitempty"`
	Loader     string       `json:"loader,omitempty"`
	AutoUpdate bool         `json:"auto_update"`
	Subdirs    []subdirStat `json:"subdirs"`
	TotalMods  int          `json:"total_mods"`
	FrozenMods int          `json:"frozen_mods"`
}

type subdirStat struct {
	Key      string   `json:"key"`
	Platform string   `json:"platform"`
	ModCount int      `json:"mod_count"`
	Frozen   []string `json:"frozen,omitempty"`
}

var wsStatusCmd = &cobra.Command{
	Use:     "status",
	Short:   "Dashboard of all packs â€” version, mc, loader, mod counts, frozen mods",
	Aliases: []string{"info"},
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		asJSON, _ := cmd.Flags().GetBool("json")

		root := workspace.ModpacksDir()
		entries, err := os.ReadDir(root)
		if err != nil {
			llFail(fmt.Sprintf("failed to read %s: %v", root, err))
		}

		var statuses []packStatus
		for _, e := range entries {
			if !e.IsDir() {
				continue
			}
			packPath := filepath.Join(root, e.Name())
			m, err := manifest.Read(filepath.Join(packPath, "manifest.json"))
			if err != nil {
				continue
			}

			auto := manifest.ReadAutomation(packPath)
			autoUpdate := auto.AutoUpdate == nil || *auto.AutoUpdate

			var subdirs []subdirStat
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

				subdirs = append(subdirs, subdirStat{
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

			statuses = append(statuses, packStatus{
				ID:         m.ID,
				Name:       m.Name,
				Version:    m.Version,
				MCVersion:  mcVersion,
				Loader:     m.Loader,
				AutoUpdate: autoUpdate,
				Subdirs:    subdirs,
				TotalMods:  totalMods,
				FrozenMods: totalFrozen,
			})
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

		for _, s := range statuses {
			autoStr := "auto-update"
			if !s.AutoUpdate {
				autoStr = "no-update"
			}
			frozenNote := ""
			if s.FrozenMods > 0 {
				frozenNote = fmt.Sprintf("  %d frozen", s.FrozenMods)
			}
			fmt.Printf("%s  v%s  mc%s  %s  [%s]  %d mods%s\n",
				s.ID, s.Version, s.MCVersion, s.Loader, autoStr, s.TotalMods, frozenNote)
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

// â€” update â€”

var wsUpdateCmd = &cobra.Command{
	Use:   "update [pack-dir]",
	Short: "Run packwand update --all in every pack subdir (honors auto_update)",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		check, _ := cmd.Flags().GetBool("check")
		if check {
			wsRunUpdateCheck(args)
			return
		}
		packFilter, explicit := workspace.ResolveScope(args, llStartCwd)
		if err := workspace.Run(workspace.OpUpdate, packFilter, explicit); err != nil {
			llFail(err.Error())
		}
	},
}

func wsRunUpdateCheck(args []string) {
	packFilter, _ := workspace.ResolveScope(args, llStartCwd)
	root := workspace.ModpacksDir()
	targets, _ := workspace.CollectTargets(root, true, packFilter, false)
	if len(targets) == 0 {
		fmt.Println("no pack subdirs to check")
		return
	}

	totalUpdates := 0
	for _, dir := range targets {
		updates, err := workspace.CheckUpdatesInDir(dir)
		if err != nil {
			llWarn("%s: check failed: %v", dir, err)
			continue
		}
		if len(updates) > 0 {
			fmt.Printf("%s: %d update(s) available\n", dir, len(updates))
			for _, u := range updates {
				fmt.Printf("  ~ %s\n", u)
			}
			totalUpdates += len(updates)
		} else {
			fmt.Printf("%s: up to date\n", dir)
		}
	}
	if totalUpdates == 0 {
		fmt.Println("\neverything is up to date")
	} else {
		fmt.Printf("\n%d update(s) available â€” run 'packwand workspace update' to apply\n", totalUpdates)
	}
}

// â€” refresh â€”

var wsRefreshCmd = &cobra.Command{
	Use:   "refresh [pack-dir]",
	Short: "Run packwand refresh in every pack subdir",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		packFilter, explicit := workspace.ResolveScope(args, llStartCwd)
		if err := workspace.Run(workspace.OpRefresh, packFilter, explicit); err != nil {
			llFail(err.Error())
		}
	},
}

// â€” loader-update â€”

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

// â€” migrate â€”

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

// â€” sync â€”

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
			runPages("")
		}
	},
}
