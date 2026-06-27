package migrate

import (
	"fmt"
	"os"
	"slices"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
)

var loaderCommand = &cobra.Command{
	Use:   "loader [version|latest|recommended]",
	Short: "Migrate your modloader version to a newer version.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		modpack, err := core.LoadPack()
		if err != nil {
			if os.IsNotExist(err) {
				cmdshared.Fail("no pack.toml found â€” run 'packwand init' to create one")
			}
			cmdshared.Failf("loading pack: %v", err)
		}
		var currentLoaders = modpack.GetLoaders()
		if len(currentLoaders) == 0 {
			cmdshared.Fail("no loader set in pack.toml")
		} else if len(currentLoaders) > 1 {
			cmdshared.Fail("multiple loaders set in pack.toml â€” this is not supported")
		}
		mcVersion, err := modpack.GetMCVersion()
		if err != nil {
			cmdshared.Failf("getting Minecraft version: %v", err)
		}
		if args[0] == "latest" || args[0] == "recommended" {
			fmt.Printf("Updating to %s loader version\n", args[0])

			queryType := core.Latest
			if args[0] == "recommended" {
				queryType = core.Recommended
			}

			for _, loader := range currentLoaders {
				versionData, gottenLoader := getVersionsForLoader(loader, mcVersion, queryType)
				if !updatePackToVersion(versionData.Latest, modpack, gottenLoader) {
					continue
				}
				err = modpack.Write()
				if err != nil {
					cmdshared.Failf("writing pack.toml: %v", err)
				}
			}
		} else {
			fmt.Println("Updating to explicit loader version")
			versionData, loader := getVersionsForLoader(currentLoaders[0], mcVersion, core.Latest)
			if loader.Name == "forge" || loader.Name == "neoforge" {
				wantedVersion := cmdshared.GetRawForgeVersion(args[0])
				validateVersion(versionData.Versions, wantedVersion, loader)
				_ = updatePackToVersion(wantedVersion, modpack, loader)
			} else if loader.Name == "liteloader" {
				fmt.Println("LiteLoader only has 1 version per Minecraft version so we're unable to update!")
				os.Exit(0)
			} else {
				validateVersion(versionData.Versions, args[0], loader)
				if !updatePackToVersion(args[0], modpack, loader) {
					return
				}
			}
			err = modpack.Write()
			if err != nil {
				cmdshared.Failf("writing pack.toml: %v", err)
			}
		}
	},
}

func init() {
	migrateCmd.AddCommand(loaderCommand)
}

func getVersionsForLoader(loader, mcVersion string, queryType core.QueryType) (*core.ModLoaderVersions, core.ModLoaderComponent) {
	gottenLoader, ok := core.ModLoaders[loader]
	if !ok {
		cmdshared.Failf("unknown loader %q", loader)
	}
	versionData, err := core.DoQuery(core.MakeQuery(gottenLoader, mcVersion).WithQueryType(queryType))
	if err != nil {
		cmdshared.Failf("getting version list for %s: %v", gottenLoader.FriendlyName, err)
	}
	return versionData, gottenLoader
}

func validateVersion(versions []string, version string, gottenLoader core.ModLoaderComponent) {
	if !slices.Contains(versions, version) {
		cmdshared.Failf("version %q is not valid for %s", version, gottenLoader.FriendlyName)
	}
}

func updatePackToVersion(version string, modpack core.Pack, loader core.ModLoaderComponent) bool {
	if version == modpack.Versions[loader.Name] {
		fmt.Printf("%s is already on version %s!\n", loader.FriendlyName, version)
		return false
	}
	modpack.Versions[loader.Name] = version
	fmt.Printf("Updated %s to version %s\n", loader.FriendlyName, version)
	return true
}
