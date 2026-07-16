package content

import (
	"encoding/json"
	"fmt"
	"path/filepath"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/registry"
	"github.com/spf13/cobra"
)

// LocalCIResult mirrors the locally reproducible validation stages. It keeps
// the CLI/API contract stable while build/export providers remain external.
type LocalCIResult struct {
	Dir    string         `json:"dir"`
	Stages []LocalCIStage `json:"stages"`
	OK     bool           `json:"ok"`
}

type LocalCIStage struct {
	Name    string `json:"name"`
	OK      bool   `json:"ok"`
	Message string `json:"message,omitempty"`
}

func init() {
	localCICmd.Flags().Bool("json", false, "Output localized CI report as JSON")
	cmd.AddToGroup(localCICmd, cmd.GroupInfo)
}

var localCICmd = &cobra.Command{
	Use:   "ci-local [dir]",
	Short: "Run the Packwand CI-equivalent validation stages for a subdir",
	Args:  cobra.MaximumNArgs(1),
	Run: func(c *cobra.Command, args []string) {
		dir := "."
		if len(args) == 1 {
			dir = args[0]
		}
		result := RunLocalizedCI(cmd.Abs(dir))
		asJSON, _ := c.Flags().GetBool("json")
		if asJSON {
			data, _ := json.MarshalIndent(result, "", "  ")
			fmt.Println(string(data))
		} else {
			for _, stage := range result.Stages {
				fmt.Printf("%s: %s %s\n", stage.Name, map[bool]string{true: "PASS", false: "FAIL"}[stage.OK], stage.Message)
			}
		}
		if !result.OK {
			cmd.Fail("localized CI failed")
		}
	},
}

// RunLocalizedCI is the one local definition consumed by the API job. The
// preflight stage contains manifest, syntax, content and registry checks; the
// registry stage verifies every incremental index can be rebuilt.
func RunLocalizedCI(dir string) LocalCIResult {
	result := LocalCIResult{Dir: filepath.ToSlash(dir), Stages: []LocalCIStage{}}
	preflight := RunPreflight(dir)
	result.Stages = append(result.Stages, LocalCIStage{Name: "preflight", OK: preflight.OK, Message: fmt.Sprintf("%d error(s), %d warning(s)", preflight.Errors, preflight.Warnings)})
	_, err := registry.BuildAll(dir)
	result.Stages = append(result.Stages, LocalCIStage{Name: "registry", OK: err == nil, Message: errorText(err)})
	result.OK = true
	for _, stage := range result.Stages {
		result.OK = result.OK && stage.OK
	}
	return result
}

func errorText(err error) string {
	if err == nil {
		return ""
	}
	return err.Error()
}
