package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"sync/atomic"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// — update report —

type updateReportEntry struct {
	Name   string `json:"name"`
	Change string `json:"change,omitempty"`
	Error  string `json:"error,omitempty"`
}

// UpdateReport is the machine-readable outcome of one `update --all` run.
// The workspace command aggregates these across pack subdirs.
type UpdateReport struct {
	Dir          string              `json:"dir"`
	DryRun       bool                `json:"dry_run,omitempty"`
	Updated      []updateReportEntry `json:"updated"`
	Pinned       []updateReportEntry `json:"pinned"`
	Incompatible []updateReportEntry `json:"incompatible"`
	Failed       []updateReportEntry `json:"failed"`
	Skipped      []updateReportEntry `json:"skipped"`
	UpToDate     int                 `json:"up_to_date"`
	Checked      int                 `json:"checked"`
}

func newUpdateReport() *UpdateReport {
	return &UpdateReport{
		Updated:      []updateReportEntry{},
		Pinned:       []updateReportEntry{},
		Incompatible: []updateReportEntry{},
		Failed:       []updateReportEntry{},
		Skipped:      []updateReportEntry{},
	}
}

// isIncompatibleError matches updater errors that mean "no release exists for
// this Minecraft version / loader" rather than a transient failure.
func isIncompatibleError(msg string) bool {
	return strings.Contains(msg, "no valid versions") ||
		strings.Contains(msg, "not available for the configured Minecraft version")
}

func writeUpdateReport(path string, rep *UpdateReport) {
	if path == "" || rep == nil {
		return
	}
	if wd, err := os.Getwd(); err == nil {
		rep.Dir = wd
	}
	data, err := json.MarshalIndent(rep, "", "  ")
	if err != nil {
		fmt.Printf("failed to render update report: %v\n", err)
		return
	}
	if err := os.WriteFile(path, append(data, '\n'), 0o644); err != nil {
		fmt.Printf("failed to write update report %s: %v\n", path, err)
	}
}

func updateFailureError(rep *UpdateReport) error {
	if rep == nil || len(rep.Failed) == 0 {
		return nil
	}
	return fmt.Errorf("%d mod update(s) failed", len(rep.Failed))
}

func finishUpdateReport(path string, rep *UpdateReport) {
	writeUpdateReport(path, rep)
	if viper.GetBool("update.json") && rep != nil {
		if wd, err := os.Getwd(); err == nil {
			rep.Dir = wd
		}
		printJSON(rep)
	}
	if err := updateFailureError(rep); err != nil {
		cmdshared.Fail(err.Error())
	}
}

// UpdateCmd represents the update command
var UpdateCmd = &cobra.Command{
	Use:     "update [name]",
	Short:   "Update an external file (or all external files) in the modpack",
	Aliases: []string{"upgrade"},
	Args:    cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		// TODO: specify multiple files to update at once?

		reportPath := viper.GetString("update.report")

		fmt.Println("Loading modpack...")
		pack, err := core.LoadPack()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		index, err := pack.LoadIndex()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		var singleUpdatedName string
		var allReport *UpdateReport
		if viper.GetBool("update.all") {
			rep := newUpdateReport()
			rep.DryRun = viper.GetBool("update.dry-run")

			filesWithUpdater := make(map[string][]*core.Mod)
			fmt.Println("Reading metadata files...")
			mods, err := index.LoadAllMods()
			if err != nil {
				fmt.Printf("Failed to update all files: %v\n", err)
				os.Exit(1)
			}
			rep.Checked = len(mods)
			for _, modData := range mods {
				updaterFound := false
				for k := range modData.Update {
					slice, ok := filesWithUpdater[k]
					if !ok {
						_, ok = core.Updaters[k]
						if !ok {
							continue
						}
						slice = []*core.Mod{}
					}
					updaterFound = true
					filesWithUpdater[k] = append(slice, modData)
				}
				if !updaterFound {
					fmt.Printf("A supported update system for \"%s\" cannot be found.\n", modData.Name)
					rep.Skipped = append(rep.Skipped, updateReportEntry{Name: modData.Name, Error: "no supported update system"})
				}
			}

			updatesFound := false
			updatableFiles := make(map[string][]*core.Mod)
			updaterCachedStateMap := make(map[string][]interface{})
			type checkResult struct {
				key    string
				mods   []*core.Mod
				checks []core.UpdateCheck
				err    error
			}
			type providerCheck struct {
				key  string
				mods []*core.Mod
			}
			var providerChecks []providerCheck
			for k, v := range filesWithUpdater {
				providerChecks = append(providerChecks, providerCheck{key: k, mods: v})
			}
			ch := make(chan checkResult, len(filesWithUpdater))
			_ = cmdshared.WithSpinner("Checking for updates", func(update func(string)) error {
				var checked atomic.Int32
				core.ParallelFor(providerChecks, core.NetworkConcurrent(), func(_ int, check providerCheck) {
					update(fmt.Sprintf("%d/%d providers  (%s: %d mod(s))", checked.Load(), len(providerChecks), check.key, len(check.mods)))
					checks, err := core.Updaters[check.key].CheckUpdate(check.mods, pack)
					checked.Add(1)
					ch <- checkResult{check.key, check.mods, checks, err}
				})
				return nil
			})
			close(ch)
			updateStrings := make(map[*core.Mod]string)
			for r := range ch {
				if r.err != nil {
					fmt.Printf("Failed to check updates for %s: %s\n", r.key, r.err.Error())
					for _, m := range r.mods {
						rep.Failed = append(rep.Failed, updateReportEntry{Name: m.Name, Error: r.err.Error()})
					}
					continue
				}
				for i, check := range r.checks {
					if check.Error != nil {
						fmt.Printf("Failed to check updates for %s: %s\n", r.mods[i].Name, check.Error.Error())
						entry := updateReportEntry{Name: r.mods[i].Name, Error: check.Error.Error()}
						if isIncompatibleError(check.Error.Error()) {
							rep.Incompatible = append(rep.Incompatible, entry)
						} else {
							rep.Failed = append(rep.Failed, entry)
						}
						continue
					}
					if check.UpdateAvailable {
						if r.mods[i].Pin {
							fmt.Printf("Update skipped for pinned mod %s\n", r.mods[i].Name)
							rep.Pinned = append(rep.Pinned, updateReportEntry{Name: r.mods[i].Name, Change: check.UpdateString})
							continue
						}

						if !updatesFound {
							fmt.Println("Updates found:")
							updatesFound = true
						}
						fmt.Printf("%s: %s\n", r.mods[i].Name, check.UpdateString)
						updatableFiles[r.key] = append(updatableFiles[r.key], r.mods[i])
						updaterCachedStateMap[r.key] = append(updaterCachedStateMap[r.key], check.CachedState)
						updateStrings[r.mods[i]] = check.UpdateString
					} else {
						rep.UpToDate++
					}
				}
			}

			if !updatesFound {
				fmt.Println("All files are up to date!")
				finishUpdateReport(reportPath, rep)
				return
			}

			if viper.GetBool("update.dry-run") {
				count := 0
				for _, v := range updatableFiles {
					count += len(v)
					for _, m := range v {
						rep.Updated = append(rep.Updated, updateReportEntry{Name: m.Name, Change: updateStrings[m]})
					}
				}
				fmt.Printf("dry-run: %d file(s) would be updated — rerun without --dry-run to apply\n", count)
				finishUpdateReport(reportPath, rep)
				return
			}

			if !cmdshared.PromptYesNo("Do you want to update? [Y/n]: ") {
				fmt.Println("Cancelled!")
				finishUpdateReport(reportPath, rep)
				return
			}

			for k, v := range updatableFiles {
				err := core.Updaters[k].DoUpdate(v, updaterCachedStateMap[k])
				if err != nil {
					fmt.Println(err.Error())
					for _, m := range v {
						rep.Failed = append(rep.Failed, updateReportEntry{Name: m.Name, Change: updateStrings[m], Error: err.Error()})
					}
					continue
				}
				for _, modData := range v {
					format, hash, err := modData.Write()
					if err != nil {
						fmt.Println(err.Error())
						rep.Failed = append(rep.Failed, updateReportEntry{Name: modData.Name, Change: updateStrings[modData], Error: err.Error()})
						continue
					}
					err = index.RefreshFileWithHash(modData.GetFilePath(), format, hash, true)
					if err != nil {
						fmt.Println(err.Error())
						rep.Failed = append(rep.Failed, updateReportEntry{Name: modData.Name, Change: updateStrings[modData], Error: err.Error()})
						continue
					}
					rep.Updated = append(rep.Updated, updateReportEntry{Name: modData.Name, Change: updateStrings[modData]})
				}
			}
			allReport = rep
		} else {
			if len(args) < 1 || len(args[0]) == 0 {
				fmt.Println("Must specify a valid file, or use the --all flag!")
				os.Exit(1)
			}
			modPath, ok := index.FindMod(args[0])
			if !ok {
				fmt.Println("Can't find this file; please ensure you have run packwiz refresh and use the name of the .pw.toml file (defaults to the project slug)")
				os.Exit(1)
			}
			modData, err := core.LoadMod(modPath)
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
			if modData.Pin {
				fmt.Println("Version is pinned; run the unpin command to allow updating")
				os.Exit(1)
			}
			singleUpdatedName = modData.Name
			updaterFound := false
			for k := range modData.Update {
				updater, ok := core.Updaters[k]
				if !ok {
					continue
				}
				updaterFound = true

				check, err := updater.CheckUpdate([]*core.Mod{&modData}, pack)
				if err != nil {
					fmt.Println(err)
					os.Exit(1)
				}
				if len(check) != 1 {
					fmt.Println("Invalid update check response")
					os.Exit(1)
				}

				if check[0].UpdateAvailable {
					fmt.Printf("Update available: %s\n", check[0].UpdateString)

					err = updater.DoUpdate([]*core.Mod{&modData}, []interface{}{check[0].CachedState})
					if err != nil {
						fmt.Println(err)
						os.Exit(1)
					}

					format, hash, err := modData.Write()
					if err != nil {
						fmt.Println(err)
						os.Exit(1)
					}
					err = index.RefreshFileWithHash(modPath, format, hash, true)
					if err != nil {
						fmt.Println(err)
						os.Exit(1)
					}
					allReport = newUpdateReport()
					allReport.Checked = 1
					allReport.Updated = append(allReport.Updated, updateReportEntry{Name: modData.Name, Change: check[0].UpdateString})
				} else {
					fmt.Printf("\"%s\" is already up to date!\n", modData.Name)
					rep := newUpdateReport()
					rep.Checked = 1
					rep.UpToDate = 1
					finishUpdateReport(reportPath, rep)
					return
				}

				break
			}
			if !updaterFound {
				// TODO: use file name instead of Name when len(Name) == 0 in all places?
				fmt.Println("A supported update system for \"" + modData.Name + "\" cannot be found.")
				os.Exit(1)
			}
		}

		if err = core.CommitChanges(&index, &pack); err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		finishUpdateReport(reportPath, allReport)
		if viper.GetBool("update.all") {
			fmt.Println("Files updated!")
		} else {
			fmt.Printf("\"%s\" updated!\n", singleUpdatedName)
		}
	},
}

func init() {
	UpdateCmd.GroupID = GroupUpdates
	rootCmd.AddCommand(UpdateCmd)

	UpdateCmd.Flags().BoolP("all", "a", false, "Update all external files")
	_ = viper.BindPFlag("update.all", UpdateCmd.Flags().Lookup("all"))
	UpdateCmd.Flags().Bool("dry-run", false, "Show what would be updated without making any changes")
	_ = viper.BindPFlag("update.dry-run", UpdateCmd.Flags().Lookup("dry-run"))
	UpdateCmd.Flags().String("report", "", "Write a machine-readable JSON update report to this file (requires --all)")
	_ = viper.BindPFlag("update.report", UpdateCmd.Flags().Lookup("report"))
	UpdateCmd.Flags().Bool("json", false, "Print the update report as JSON on stdout")
	_ = viper.BindPFlag("update.json", UpdateCmd.Flags().Lookup("json"))
}
