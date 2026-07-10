package cmd

import "github.com/spf13/cobra"

// Cobra's normal help text remains unchanged. Interactive help gets the same
// branded context bar as command execution, while piped help stays byte-clean.
func init() {
	defaultHelp := rootCmd.HelpFunc()
	rootCmd.SetHelpFunc(func(command *cobra.Command, args []string) {
		if Interactive() {
			StatusBar(command.CommandPath() + " help")
		}
		defaultHelp(command, args)
	})
}
