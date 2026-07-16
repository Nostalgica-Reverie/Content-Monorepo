package content

import (
	"encoding/json"
	"fmt"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/registry"
	"github.com/spf13/cobra"
)

func init() {
	registryCmd.Flags().Bool("json", false, "Output the registries as JSON")
	cmd.AddToGroup(registryCmd, cmd.GroupInfo)
}

var registryCmd = &cobra.Command{
	Use:   "registry <datapack|config|resourcepack|kubejs|all> [dir]",
	Short: "Build content registries — indexes of referenceable IDs backing IDE completion and validation",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(c *cobra.Command, args []string) {
		asJSON, _ := c.Flags().GetBool("json")
		dir := "."
		if len(args) == 2 {
			dir = args[1]
		}
		dir = cmd.Abs(dir)

		var kinds []registry.Kind
		if strings.EqualFold(args[0], "all") {
			kinds = registry.Kinds()
		} else {
			kind, err := registry.ParseKind(args[0])
			if err != nil {
				cmd.Fail(err.Error())
			}
			kinds = []registry.Kind{kind}
		}

		// JSON output is always an array so consumers decode one shape for
		// both single-kind and all-kind runs (the API rebuild job relies on it).
		registries := make([]registry.Registry, 0, len(kinds))
		for _, kind := range kinds {
			reg, err := registry.Build(dir, kind)
			if err != nil {
				cmd.Fail(err.Error())
			}
			registries = append(registries, *reg)
		}

		if asJSON {
			data, _ := json.MarshalIndent(registries, "", "  ")
			fmt.Println(string(data))
			return
		}
		for _, reg := range registries {
			printRegistrySummary(reg)
		}
	},
}

func printRegistrySummary(reg registry.Registry) {
	counts := map[string]int{}
	for _, entry := range reg.Entries {
		counts[entry.Kind]++
	}
	kinds := make([]string, 0, len(counts))
	for kind := range counts {
		kinds = append(kinds, kind)
	}
	sort.Strings(kinds)

	lines := []string{
		fmt.Sprintf("%d source(s) · %d entries · version %.12s", len(reg.Sources), len(reg.Entries), reg.Version),
	}
	for _, kind := range kinds {
		lines = append(lines, fmt.Sprintf("%s: %d", kind, counts[kind]))
	}
	if cmd.Interactive() {
		cmd.Boxed(string(reg.Kind)+" registry", lines)
		return
	}
	fmt.Printf("%s registry: %d source(s), %d entries, version %.12s\n", reg.Kind, len(reg.Sources), len(reg.Entries), reg.Version)
	for _, kind := range kinds {
		fmt.Printf("  %s: %d\n", kind, counts[kind])
	}
}
