package migrate

import (
	"fmt"
	"os"

	packCmd "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var minecraftCommand = &cobra.Command{
	Use:     "minecraft [version]",
	Short:   "Migrate your Minecraft version to a newer version.",
	Aliases: []string{"mc"},
	Args:    cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		modpack, err := core.LoadPack()
		if err != nil {
			if os.IsNotExist(err) {
				cmdshared.Fail("no pack.toml found — run 'packwand init' to create one")
			}
			cmdshared.Failf("loading pack: %v", err)
		}
		currentVersion, err := modpack.GetMCVersion()
		if err != nil {
			cmdshared.Failf("getting Minecraft version from pack: %v", err)
		}
		wantedMCVersion := args[0]
		if wantedMCVersion == currentVersion {
			fmt.Printf("Minecraft version is already %s!\n", wantedMCVersion)
			os.Exit(0)
		}
		mcVersions, err := cmdshared.GetValidMCVersions()
		if err != nil {
			cmdshared.Failf("fetching Minecraft version list: %v", err)
		}
		mcVersions.CheckValid(wantedMCVersion)
		modpack.Versions["minecraft"] = wantedMCVersion
		err = modpack.Write()
		if err != nil {
			cmdshared.Failf("writing pack.toml: %v", err)
		}
		fmt.Printf("Successfully updated Minecraft version to %s\n", wantedMCVersion)
		if cmdshared.PromptYesNo("Would you like to update your loader version to the latest version for this Minecraft version? [Y/n] ") {
			loaderCommand.Run(loaderCommand, []string{"latest"})
		}
		if cmdshared.PromptYesNo("Would you like to update your mods to the latest versions for this Minecraft version? [Y/n] ") {
			viper.Set("update.all", true)
			packCmd.UpdateCmd.Run(packCmd.UpdateCmd, []string{})
		}
	},
}

func init() {
	migrateCmd.AddCommand(minecraftCommand)
}
