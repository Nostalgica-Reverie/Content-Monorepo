package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// listCmd represents the list command
var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List all the mods in the modpack",
	Args:  cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {

		// Load pack
		pack, err := core.LoadPack()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		// Load index
		index, err := pack.LoadIndex()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		// Load mods
		mods, err := index.LoadAllMods()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		// Filter mods by side
		if viper.IsSet("list.side") {
			side := viper.GetString("list.side")
			if side != core.UniversalSide && side != core.ServerSide && side != core.ClientSide {
				fmt.Printf("Invalid side %q, must be one of client, server, or both (default)\n", side)
				os.Exit(1)
			}

			i := 0
			for _, mod := range mods {
				if mod.Side == side || mod.Side == core.EmptySide || mod.Side == core.UniversalSide || side == core.UniversalSide {
					mods[i] = mod
					i++
				}
			}
			mods = mods[:i]
		}

		sort.Slice(mods, func(i, j int) bool {
			return strings.ToLower(mods[i].Name) < strings.ToLower(mods[j].Name)
		})

		if viper.GetBool("list.json") {
			type entry struct {
				Name      string   `json:"name"`
				FileName  string   `json:"filename"`
				Side      string   `json:"side,omitempty"`
				Pin       bool     `json:"pin,omitempty"`
				Platforms []string `json:"platforms,omitempty"`
			}
			out := make([]entry, len(mods))
			for i, mod := range mods {
				var plats []string
				for k := range mod.Update {
					plats = append(plats, k)
				}
				sort.Strings(plats)
				out[i] = entry{mod.Name, mod.FileName, mod.Side, mod.Pin, plats}
			}
			data, _ := json.MarshalIndent(out, "", "  ")
			fmt.Println(string(data))
			return
		}

		if Interactive() {
			rows := make([][]string, 0, len(mods))
			for _, mod := range mods {
				platforms := make([]string, 0, len(mod.Update))
				for platform := range mod.Update {
					platforms = append(platforms, platform)
				}
				sort.Strings(platforms)
				pinned := ""
				if mod.Pin {
					pinned = "yes"
				}
				rows = append(rows, []string{mod.Name, mod.FileName, string(mod.Side), pinned, strings.Join(platforms, ", ")})
			}
			fmt.Fprintln(os.Stderr, Table([]string{"MOD", "FILE", "SIDE", "PINNED", "UPDATES"}, rows))
			return
		}
		// Print mods
		if viper.GetBool("list.version") {
			for _, mod := range mods {
				fmt.Printf("%s (%s)\n", mod.Name, mod.FileName)
			}
		} else {
			for _, mod := range mods {
				fmt.Println(mod.Name)
			}
		}
	},
}

func init() {
	listCmd.GroupID = GroupInfo
	rootCmd.AddCommand(listCmd)

	listCmd.Flags().BoolP("version", "v", false, "Print name and version")
	_ = viper.BindPFlag("list.version", listCmd.Flags().Lookup("version"))
	listCmd.Flags().StringP("side", "s", "", "Filter mods by side (e.g., client or server)")
	_ = viper.BindPFlag("list.side", listCmd.Flags().Lookup("side"))
	listCmd.Flags().Bool("json", false, "Output as JSON array instead of plain text")
	_ = viper.BindPFlag("list.json", listCmd.Flags().Lookup("json"))
}
