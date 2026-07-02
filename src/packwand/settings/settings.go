package settings

import (
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"github.com/spf13/cobra"
)

// settingsCmd represents the base command when called without any subcommands
var settingsCmd = &cobra.Command{
	Use:   "settings",
	Short: "Manage pack settings",
}

func init() {
	cmd.AddToGroup(settingsCmd, cmd.GroupOther)
}
