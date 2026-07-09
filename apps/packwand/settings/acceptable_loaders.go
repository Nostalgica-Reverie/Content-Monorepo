package settings

import (
	"fmt"
	"os"
	"slices"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
)

var acceptableLoadersCommand = &cobra.Command{
	Use:     "acceptable-loaders",
	Short:   "Manage your pack's acceptable mod loaders. Takes a comma-separated list, e.g. fabric,forge",
	Aliases: []string{"al"},
	Args:    cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		modpack, err := core.LoadPack()
		if err != nil {
			if os.IsNotExist(err) {
				cmdshared.Fail("no pack.toml found — run 'packwand init' to create one")
			}
			cmdshared.Failf("loading pack: %v", err)
		}
		var currentLoaders []string
		if modpack.Options == nil {
			modpack.Options = make(map[string]interface{})
		}
		if modpack.Options["acceptable-game-loaders"] != nil {
			for _, v := range modpack.Options["acceptable-game-loaders"].([]interface{}) {
				currentLoaders = append(currentLoaders, v.(string))
			}
		}
		if flagAlAdd {
			loader := args[0]
			if slices.Contains(currentLoaders, loader) {
				cmdshared.Failf("loader %q is already in the acceptable loaders list", loader)
			}
			currentLoaders = append(currentLoaders, loader)
			modpack.Options["acceptable-game-loaders"] = currentLoaders
			err = modpack.Write()
			if err != nil {
				cmdshared.Failf("writing pack: %v", err)
			}
			fmt.Printf("Added %s to acceptable loaders list, now %s\n", loader, strings.Join(currentLoaders, ", "))
		} else if flagAlRemove {
			loader := args[0]
			if !slices.Contains(currentLoaders, loader) {
				cmdshared.Failf("loader %q is not in the acceptable loaders list", loader)
			}
			i := slices.Index(currentLoaders, loader)
			currentLoaders = slices.Delete(currentLoaders, i, i+1)
			modpack.Options["acceptable-game-loaders"] = currentLoaders
			err = modpack.Write()
			if err != nil {
				cmdshared.Failf("writing pack: %v", err)
			}
			fmt.Printf("Removed %s from acceptable loaders list, now %s\n", loader, strings.Join(currentLoaders, ", "))
		} else {
			loadersList := strings.Split(args[0], ",")
			loadersDeduped := []string(nil)
			for i, v := range loadersList {
				if !slices.Contains(loadersList[i+1:], v) {
					loadersDeduped = append(loadersDeduped, v)
				}
			}
			modpack.Options["acceptable-game-loaders"] = loadersDeduped
			err = modpack.Write()
			if err != nil {
				cmdshared.Failf("writing pack: %v", err)
			}
			fmt.Printf("Set acceptable loaders to %s\n", strings.Join(loadersDeduped, ", "))
		}
	},
}

var flagAlAdd bool
var flagAlRemove bool

func init() {
	settingsCmd.AddCommand(acceptableLoadersCommand)
	acceptableLoadersCommand.Flags().BoolVarP(&flagAlAdd, "add", "a", false, "Add a loader to the list")
	acceptableLoadersCommand.Flags().BoolVarP(&flagAlRemove, "remove", "r", false, "Remove a loader from the list")
}
