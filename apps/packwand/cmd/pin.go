package cmd

import (
	"fmt"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
)

func pinMod(args []string, pinned bool) {
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
	message := "pinned"
	if !pinned {
		message = "unpinned"
	}
	// All pin flips apply against the loaded index, then commit once.
	for _, name := range args {
		modPath, ok := index.FindMod(name)
		if !ok {
			fmt.Printf("Can't find file %q; please ensure you have run packwiz refresh and use the name of the .pw.toml file (defaults to the project slug)\n", name)
			os.Exit(1)
		}
		modData, err := core.LoadMod(modPath)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		modData.Pin = pinned
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
		fmt.Printf("%s %s successfully!\n", name, message)
	}
	if err = core.CommitChanges(&index, &pack); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

// pinCmd represents the pin command
var pinCmd = &cobra.Command{
	Use:     "pin [name]...",
	Short:   "Pin one or more files so they do not get updated automatically",
	Aliases: []string{"hold"},
	Args:    cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		pinMod(args, true)
	},
}

// unpinCmd represents the unpin command
var unpinCmd = &cobra.Command{
	Use:     "unpin [name]...",
	Short:   "Unpin one or more files so they receive updates",
	Aliases: []string{"unhold"},
	Args:    cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		pinMod(args, false)
	},
}

func init() {
	pinCmd.GroupID = GroupPackManagement
	rootCmd.AddCommand(pinCmd)
	unpinCmd.GroupID = GroupPackManagement
	rootCmd.AddCommand(unpinCmd)
}
