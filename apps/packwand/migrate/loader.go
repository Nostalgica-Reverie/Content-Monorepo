package migrate

import (
	"fmt"
	"os"
	"slices"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
)

var loaderCommand = &cobra.Command{
	Use:   "loader [version|latest|recommended]",
	Short: "Migrate every configured modloader to a newer version.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		modpack, err := core.LoadPack()
		if err != nil {
			if os.IsNotExist(err) {
				cmdshared.Fail("no pack.toml found — run 'packwand init' to create one")
			}
			cmdshared.Failf("loading pack: %v", err)
		}
		currentLoaders := modpack.GetLoaders()
		if len(currentLoaders) == 0 {
			cmdshared.Fail("no loader set in pack.toml")
		}
		mcVersion, err := modpack.GetMCVersion()
		if err != nil {
			cmdshared.Failf("getting Minecraft version: %v", err)
		}

		target := args[0]
		if target == "latest" || target == "recommended" {
			fmt.Printf("Updating all loaders to %s versions\n", target)
			queryType := core.Latest
			if target == "recommended" {
				queryType = core.Recommended
			}
			for _, currentLoader := range currentLoaders {
				versionData, loader := getVersionsForLoader(currentLoader, mcVersion, queryType)
				_ = updatePackToVersion(versionData.Latest, modpack, loader)
			}
		} else {
			fmt.Println("Updating all loaders to the explicit version where supported")
			for _, currentLoader := range currentLoaders {
				versionData, loader := getVersionsForLoader(currentLoader, mcVersion, core.Latest)
				wantedVersion := target
				switch loader.Name {
				case "forge", "neoforge":
					wantedVersion = cmdshared.GetRawForgeVersion(wantedVersion)
				case "liteloader":
					fmt.Printf("%s only has one version per Minecraft version; skipping explicit migration\n", loader.FriendlyName)
					continue
				}
				validateVersion(versionData.Versions, wantedVersion, loader)
				_ = updatePackToVersion(wantedVersion, modpack, loader)
			}
		}
		if err = modpack.Write(); err != nil {
			cmdshared.Failf("writing pack.toml: %v", err)
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
