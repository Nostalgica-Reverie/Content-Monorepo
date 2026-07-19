package cmd

import (
	"fmt"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// refreshCmd represents the refresh command
var refreshCmd = &cobra.Command{
	Use:   "refresh",
	Short: "Refresh the index file",
	Args:  cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Loading modpack...")
		pack, err := core.LoadPack()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		build, err := cmd.Flags().GetBool("build")
		if err == nil && build {
			viper.Set("no-internal-hashes", false)
		} else if viper.GetBool("no-internal-hashes") {
			fmt.Println("Note: no-internal-hashes mode is set, no hashes will be saved. Use --build to override this for distribution.")
		}
		index, err := pack.LoadIndexForRefresh()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		stats, err := index.Refresh()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		err = index.Write()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		if build {
			err = pack.UpdateIndexHash()
		} else {
			pack.ClearIndexHash()
		}
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		err = pack.Write()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		if stats.HashUpgraded {
			fmt.Printf("Index hash-format upgraded to %s\n", core.DefaultHashFormat)
		}
		fmt.Printf("Index refreshed: +%d added  ~%d updated  -%d removed\n", stats.Added, stats.Updated, stats.Removed)
	},
}

func init() {
	rootCmd.AddCommand(refreshCmd)
	refreshCmd.GroupID = GroupUpdates

	refreshCmd.Flags().Bool("build", false, "Generate the index and matching pack hash for distribution with packwiz-installer")
}
