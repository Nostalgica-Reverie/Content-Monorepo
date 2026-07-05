package utils

import (
	"encoding/json"
	"fmt"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmdshared"
	"github.com/spf13/cobra"
)

// commandsCmd lists the registered CLI command tree, for tooling integrations
// (CI, docs-coverage checks, the GUI) rather than everyday interactive use.
var commandsCmd = &cobra.Command{
	Use:    "commands",
	Short:  "List the registered CLI command tree",
	Hidden: true,
	Args:   cobra.NoArgs,
	Run: func(c *cobra.Command, args []string) {
		asJSON, _ := c.Flags().GetBool("json")
		catalog := cmd.CommandCatalog()
		if asJSON {
			data, err := json.MarshalIndent(catalog, "", "  ")
			if err != nil {
				cmdshared.Fail(err.Error())
			}
			fmt.Println(string(data))
			return
		}
		for _, entry := range catalog {
			fmt.Println(entry.Path)
		}
	},
}

func init() {
	commandsCmd.Flags().Bool("json", false, "Output the full command catalog as JSON instead of one path per line")
	utilsCmd.AddCommand(commandsCmd)
}
