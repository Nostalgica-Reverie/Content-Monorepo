package url

import (
	"fmt"
	"io"
	"net/url"
	"path"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var installCmd = &cobra.Command{
	Use:     "add [name] [url]",
	Short:   "Add an external file from a direct download link, for sites that are not directly supported by packwiz",
	Aliases: []string{"install", "get"},
	Args:    cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		pack, err := core.LoadPack()
		if err != nil {
			cmdshared.Failf("loading pack: %v", err)
		}

		dl, err := url.Parse(args[1])
		if err != nil {
			cmdshared.Failf("parsing URL: %v", err)
		}
		if dl.Scheme != "https" && dl.Scheme != "http" {
			cmdshared.Failf("unsupported URL scheme %q (expected http or https)", dl.Scheme)
		}

		force, err := cmd.Flags().GetBool("force")
		if !force && err == nil {
			var msg string
			if strings.HasSuffix(dl.Host, "modrinth.com") {
				msg = "modrinth add " + args[1]
			}
			if strings.HasSuffix(dl.Host, "curseforge.com") || strings.HasSuffix(dl.Host, "forgecdn.net") {
				msg = "curseforge add " + args[1]
			}
			if msg != "" {
				cmdshared.Failf("consider using 'packwand %s' instead; use --force to bypass this check", msg)
			}
		}

		hash, err := getHash(args[1])
		if err != nil {
			cmdshared.Failf("hashing file at %s: %v", args[1], err)
		}

		index, err := pack.LoadIndex()
		if err != nil {
			cmdshared.Failf("loading index: %v", err)
		}

		filename := path.Base(dl.Path)
		modMeta := core.Mod{
			Name:     args[0],
			FileName: filename,
			Side:     core.UniversalSide,
			Download: core.ModDownload{
				URL:        args[1],
				HashFormat: core.DefaultHashFormat,
				Hash:       hash,
			},
		}

		folder := viper.GetString("meta-folder")
		if folder == "" {
			folder = "mods"
		}
		destPathName, err := cmd.Flags().GetString("meta-name")
		if err != nil {
			cmdshared.Failf("reading --meta-name flag: %v", err)
		}
		if destPathName == "" {
			destPathName = core.SlugifyName(args[0])
		}
		destPath := modMeta.SetMetaPath(filepath.Join(viper.GetString("meta-folder-base"), folder,
			destPathName+core.MetaExtension))

		format, hash, err := modMeta.Write()
		if err != nil {
			cmdshared.Failf("writing mod metadata: %v", err)
		}
		err = index.RefreshFileWithHash(destPath, format, hash, true)
		if err != nil {
			cmdshared.Failf("refreshing index: %v", err)
		}
		if err = core.CommitChanges(&index, &pack); err != nil {
			cmdshared.Failf("committing changes: %v", err)
		}
		fmt.Printf("Successfully added %s (%s) from: %s\n", args[0], destPath, args[1])
	}}

func getHash(rawURL string) (string, error) {
	mainHasher, err := core.GetHashImpl(core.DefaultHashFormat)
	if err != nil {
		return "", err
	}
	resp, err := core.GetWithUA(rawURL, "application/octet-stream")
	if err != nil {
		return "", err
	}

	defer resp.Body.Close()
	if resp.StatusCode != 200 {
		return "", fmt.Errorf("failed to download: unexpected response status: %v", resp.Status)
	}

	_, err = io.Copy(mainHasher, resp.Body)
	if err != nil {
		return "", err
	}

	return mainHasher.HashToString(mainHasher.Sum(nil)), nil
}

func init() {
	urlCmd.AddCommand(installCmd)

	installCmd.Flags().Bool("force", false, "Add a file even if the download URL is supported by packwiz in an alternative command (which may support dependencies and updates)")
	installCmd.Flags().String("meta-name", "", "Filename to use for the created metadata file (defaults to a name generated from the name you supply)")
}
