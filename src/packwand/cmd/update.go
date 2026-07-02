package cmd

import (
	"fmt"
	"os"
	"sync/atomic"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// UpdateCmd represents the update command
var UpdateCmd = &cobra.Command{
	Use:     "update [name]",
	Short:   "Update an external file (or all external files) in the modpack",
	Aliases: []string{"upgrade"},
	Args:    cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		// TODO: specify multiple files to update at once?

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
		if viper.GetBool("update.all") {
			filesWithUpdater := make(map[string][]*core.Mod)
			fmt.Println("Reading metadata files...")
			mods, err := index.LoadAllMods()
			if err != nil {
				fmt.Printf("Failed to update all files: %v\n", err)
				os.Exit(1)
			}
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
			for r := range ch {
				if r.err != nil {
					// TODO: do we return err code 1?
					fmt.Printf("Failed to check updates for %s: %s\n", r.key, r.err.Error())
					continue
				}
				for i, check := range r.checks {
					if check.Error != nil {
						// TODO: do we return err code 1?
						fmt.Printf("Failed to check updates for %s: %s\n", r.mods[i].Name, check.Error.Error())
						continue
					}
					if check.UpdateAvailable {
						if r.mods[i].Pin {
							fmt.Printf("Update skipped for pinned mod %s\n", r.mods[i].Name)
							continue
						}

						if !updatesFound {
							fmt.Println("Updates found:")
							updatesFound = true
						}
						fmt.Printf("%s: %s\n", r.mods[i].Name, check.UpdateString)
						updatableFiles[r.key] = append(updatableFiles[r.key], r.mods[i])
						updaterCachedStateMap[r.key] = append(updaterCachedStateMap[r.key], check.CachedState)
					}
				}
			}

			if !updatesFound {
				fmt.Println("All files are up to date!")
				return
			}

			if viper.GetBool("update.dry-run") {
				count := 0
				for _, v := range updatableFiles {
					count += len(v)
				}
				fmt.Printf("dry-run: %d file(s) would be updated â€” rerun without --dry-run to apply\n", count)
				return
			}

			if !cmdshared.PromptYesNo("Do you want to update? [Y/n]: ") {
				fmt.Println("Cancelled!")
				return
			}

			for k, v := range updatableFiles {
				err := core.Updaters[k].DoUpdate(v, updaterCachedStateMap[k])
				if err != nil {
					// TODO: do we return err code 1?
					fmt.Println(err.Error())
					continue
				}
				for _, modData := range v {
					format, hash, err := modData.Write()
					if err != nil {
						fmt.Println(err.Error())
						continue
					}
					err = index.RefreshFileWithHash(modData.GetFilePath(), format, hash, true)
					if err != nil {
						fmt.Println(err.Error())
						continue
					}
				}
			}
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
				} else {
					fmt.Printf("\"%s\" is already up to date!\n", modData.Name)
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
}
