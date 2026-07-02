package utils

import (
	"fmt"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmdshared"
	"github.com/spf13/cobra"
	"github.com/spf13/cobra/doc"
	"github.com/spf13/viper"
)

// markdownCmd represents the markdown command
var markdownCmd = &cobra.Command{
	Use:     "markdown",
	Short:   "Generate markdown documentation (that you might be reading right now!!)",
	Aliases: []string{"md"},
	Args:    cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		outDir := viper.GetString("utils.markdown.dir")
		if err := os.MkdirAll(outDir, os.ModePerm); err != nil {
			cmdshared.Failf("creating output directory: %v", err)
		}
		disableTag(cmd.Root())
		if err := doc.GenMarkdownTree(cmd.Root(), outDir); err != nil {
			cmdshared.Failf("generating markdown: %v", err)
		}
		fmt.Println("Generated markdown successfully!")
	},
}

func disableTag(cmd *cobra.Command) {
	cmd.DisableAutoGenTag = true
	for _, v := range cmd.Commands() {
		disableTag(v)
	}
}

func init() {
	utilsCmd.AddCommand(markdownCmd)

	markdownCmd.Flags().String("dir", ".", "The destination directory to save docs in")
	_ = viper.BindPFlag("utils.markdown.dir", markdownCmd.Flags().Lookup("dir"))
}
