package utils

import (
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"github.com/spf13/cobra"
)

// utilsCmd represents the base command when called without any subcommands
var utilsCmd = &cobra.Command{
	Use:   "utils",
	Short: "Utilities for managing packwiz itself",
}

func init() {
	cmd.AddToGroup(utilsCmd, cmd.GroupOther)
}
