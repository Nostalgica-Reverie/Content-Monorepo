package cmd

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var runCmd = &cobra.Command{
	Use:   "run <script>",
	Short: "Execute a user-defined script from the [scripts] section of pack.toml",
	Long: `Run executes a named script defined under [scripts] in pack.toml.

Example pack.toml:
  [scripts]
  generate = "python ./tools/gen.py"
  pre-export = "sh ./tools/pre-export.sh"

  packwand run generate`,
	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		scriptName := args[0]

		pack, err := core.LoadPack()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		if len(pack.Scripts) == 0 {
			llFail("no [scripts] section defined in " + viper.GetString("pack-file"))
		}

		script, ok := pack.Scripts[scriptName]
		if !ok {
			var names []string
			for k := range pack.Scripts {
				names = append(names, k)
			}
			llFail(fmt.Sprintf("script %q not found; available: %s", scriptName, strings.Join(names, ", ")))
		}

		// Run in the directory containing pack.toml.
		packDir := filepath.Dir(viper.GetString("pack-file"))
		if packDir == "" {
			packDir = "."
		}

		fmt.Printf("Running script %q: %s\n", scriptName, script)

		var c *exec.Cmd
		if runtime.GOOS == "windows" {
			c = exec.Command("cmd", "/C", script)
		} else {
			c = exec.Command("sh", "-c", script)
		}
		c.Dir = packDir
		c.Stdout = os.Stdout
		c.Stderr = os.Stderr
		c.Stdin = os.Stdin

		if err := c.Run(); err != nil {
			llFail(fmt.Sprintf("script %q failed: %v", scriptName, err))
		}
	},
}

func init() {
	runCmd.GroupID = GroupOther
	rootCmd.AddCommand(runCmd)
}
