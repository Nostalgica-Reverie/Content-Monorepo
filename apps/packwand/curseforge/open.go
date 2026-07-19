package curseforge

import (
	"errors"
	"fmt"
	"strconv"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/skratchdot/open-golang/open"
	"github.com/spf13/cobra"
)

// openCmd represents the open command
var openCmd = &cobra.Command{
	Use:     "open [name]",
	Short:   "Open the project page for a CurseForge file in your browser",
	Aliases: []string{"doc"},
	Args:    cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("Loading modpack...")
		pack, err := core.LoadPack()
		if err != nil {
			return err
		}
		index, err := pack.LoadIndex()
		if err != nil {
			return err
		}
		resolvedMod, ok := index.FindMod(args[0])
		if !ok {
			// TODO: should this auto-refresh?
			return errors.New("can't find this file; please ensure you have run packwiz refresh and use the name of the .pw.toml file (defaults to the project slug)")
		}
		modData, err := core.LoadMod(resolvedMod)
		if err != nil {
			return err
		}
		updateData, ok := modData.GetParsedUpdateData("curseforge")
		if !ok {
			return errors.New("can't find CurseForge update metadata for this file")
		}
		cfUpdateData := updateData.(cfUpdateData)
		fmt.Println("Opening browser...")
		url := "https://www.curseforge.com/projects/" + strconv.FormatUint(uint64(cfUpdateData.ProjectID), 10)
		err = open.Start(url)
		if err != nil {
			fmt.Println("Opening page failed, direct link:")
			fmt.Println(url)
		}
		return nil
	},
}

func init() {
	curseforgeCmd.AddCommand(openCmd)
}
