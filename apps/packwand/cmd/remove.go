package cmd

import (
	"fmt"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
)

// removeCmd represents the remove command
var removeCmd = &cobra.Command{
	Use:     "remove [name]...",
	Short:   "Remove one or more external files from the modpack; equivalent to manually removing the file and running packwiz refresh",
	Aliases: []string{"delete", "uninstall", "rm"},
	Args:    cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
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
		// All removals apply against the loaded index, then commit once.
		for _, name := range args {
			resolvedMod, ok := index.FindMod(name)
			if !ok {
				fmt.Printf("Can't find file %q; please ensure you have run packwiz refresh and use the name of the .pw.toml file (defaults to the project slug)\n", name)
				os.Exit(1)
			}
			err = os.Remove(resolvedMod)
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
			fmt.Printf("Removing %s from index...\n", name)
			err = index.RemoveFile(resolvedMod)
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		}
		if err = core.CommitChanges(&index, &pack); err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		if len(args) == 1 {
			fmt.Printf("%s removed successfully!\n", args[0])
		} else {
			fmt.Printf("%d files removed successfully!\n", len(args))
		}
	},
}

func init() {
	removeCmd.GroupID = GroupPackManagement
	rootCmd.AddCommand(removeCmd)
}
