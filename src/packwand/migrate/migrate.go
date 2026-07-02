package migrate

import (
	"fmt"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
)

// migrateCmd represents the base command when called without any subcommands
var migrateCmd = &cobra.Command{
	Use:   "migrate [minecraft|loader|format]",
	Short: "Migrate Minecraft/loader versions or pack-format to a newer generation.",
}

var migrateFormatCmd = &cobra.Command{
	Use:   "format",
	Short: "Upgrade pack-format from packwiz:1.1.0 to packwand:" + fmt.Sprint(core.PackwandGeneration),
	Args:  cobra.NoArgs,
	Run: func(c *cobra.Command, args []string) {
		pack, err := core.LoadPack()
		if err != nil {
			cmdshared.Failf("loading pack: %v", err)
		}

		if pack.PackFormat == core.CurrentPackFormat {
			fmt.Println("Pack is already at", core.CurrentPackFormat)
			return
		}

		old := pack.PackFormat
		pack.PackFormat = core.CurrentPackFormat
		if err := pack.Write(); err != nil {
			cmdshared.Failf("writing pack.toml: %v", err)
		}
		fmt.Printf("pack-format upgraded: %s â†’ %s\n", old, core.CurrentPackFormat)
	},
}

func init() {
	cmd.AddToGroup(migrateCmd, cmd.GroupUpdates)
	migrateCmd.AddCommand(migrateFormatCmd)
}
