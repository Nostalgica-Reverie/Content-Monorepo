package forgejo

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"regexp"

	"github.com/dlclark/regexp2"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// CodebergRegex matches https://codeberg.org/owner/repo URLs.
var CodebergRegex = regexp.MustCompile(`^https?://codeberg\.org/([^/]+/[^/]+)`)

// GenericForgejoRegex matches https://{host}/owner/repo for known forge hosts.
// Extend this list or use --instance for custom deployments.
var GenericForgejoRegex = regexp.MustCompile(`^https?://([^/]+)/([^/]+/[^/]+)`)

var installCmd = &cobra.Command{
	Use:     "add [URL|slug]",
	Short:   "Add a project from a Forgejo/Gitea/Codeberg repository URL or slug",
	Aliases: []string{"install", "get"},
	Args:    cobra.ArbitraryArgs,
	Run: func(cmd *cobra.Command, args []string) {
		pack, err := core.LoadPack()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		if len(args) == 0 || len(args[0]) == 0 {
			fmt.Println("You must specify a Forgejo/Gitea/Codeberg repository URL or slug.")
			os.Exit(1)
		}

		instance := instanceFlag
		var slug string
		regex := defaultRegex

		// Try Codeberg URL first.
		if m := CodebergRegex.FindStringSubmatch(args[0]); len(m) == 2 {
			if instance == "" {
				instance = "codeberg.org"
			}
			slug = m[1]
		} else if m := GenericForgejoRegex.FindStringSubmatch(args[0]); len(m) == 3 {
			// Generic URL: extract instance and slug.
			if instance == "" {
				instance = m[1]
			}
			slug = m[2]
		} else {
			slug = args[0]
		}

		if instance == "" {
			instance = DefaultInstance
		}
		if regexFlag != "" {
			regex = regexFlag
		}

		repo, err := fetchRepo(instance, slug)
		if err != nil {
			fmt.Printf("Failed to add project: %s\n", err)
			os.Exit(1)
		}

		if err = installMod(repo, instance, branchFlag, regex, pack); err != nil {
			fmt.Printf("Failed to add project: %s\n", err)
			os.Exit(1)
		}
	},
}

const defaultRegex = `^.+(?<!-api|-dev|-dev-preshadow|-sources)\.jar$`

func getLatestRelease(instance, slug, branch string) (Release, error) {
	var releases []Release

	resp, err := newClient(instance).getReleases(slug)
	if err != nil {
		return Release{}, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return Release{}, err
	}

	if err = json.Unmarshal(body, &releases); err != nil {
		return Release{}, err
	}

	if len(releases) == 0 {
		return Release{}, errors.New("no releases found")
	}

	if branch != "" {
		for _, r := range releases {
			if r.TargetCommitish == branch {
				return r, nil
			}
		}
		return Release{}, fmt.Errorf("no release found for branch %q", branch)
	}

	return releases[0], nil
}

func installMod(repo Repo, instance, branch, regex string, pack core.Pack) error {
	latestRelease, err := getLatestRelease(instance, repo.FullName, branch)
	if err != nil {
		return fmt.Errorf("failed to get latest release: %v", err)
	}
	return installRelease(repo, instance, branch, latestRelease, regex, pack)
}

func installRelease(repo Repo, instance, branch string, release Release, regex string, pack core.Pack) error {
	expr := regexp2.MustCompile(regex, 0)

	if len(release.Assets) == 0 {
		return errors.New("release doesn't have any assets attached")
	}

	var files []Asset
	for _, v := range release.Assets {
		if bl, _ := expr.MatchString(v.Name); bl {
			files = append(files, v)
		}
	}

	switch len(files) {
	case 0:
		return errors.New("release has no assets matching regex")
	case 1:
		// ok
	default:
		return errors.New("release has more than one asset matching regex")
	}

	file := files[0]
	fmt.Printf("Installing %s from %s release %s\n", file.Name, instance, release.TagName)

	index, err := pack.LoadIndex()
	if err != nil {
		return err
	}

	updateMap := make(map[string]map[string]interface{})
	updateMap["forgejo"], err = forgejoUpdateData{
		Instance: instance,
		Slug:     repo.FullName,
		Tag:      release.TagName,
		Branch:   branch, // only stored when user specified --branch
		Regex:    regex,
	}.ToMap()
	if err != nil {
		return err
	}

	hash, err := file.getHash()
	if err != nil {
		return err
	}

	modMeta := core.Mod{
		Name:     repo.Name,
		FileName: file.Name,
		Side:     core.UniversalSide,
		Download: core.ModDownload{
			URL:        file.BrowserDownloadURL,
			HashFormat: core.DefaultHashFormat,
			Hash:       hash,
		},
		Update: updateMap,
	}

	folder := viper.GetString("meta-folder")
	if folder == "" {
		folder = "mods"
	}
	path := modMeta.SetMetaPath(filepath.Join(viper.GetString("meta-folder-base"), folder, core.SlugifyName(repo.Name)+core.MetaExtension))

	format, hash, err := modMeta.Write()
	if err != nil {
		return err
	}

	if err = index.RefreshFileWithHash(path, format, hash, true); err != nil {
		return err
	}
	if err = core.CommitChanges(&index, &pack); err != nil {
		return err
	}

	fmt.Printf("Project %q successfully added! (%s)\n", repo.Name, file.Name)
	return nil
}

var instanceFlag string
var branchFlag string
var regexFlag string

func init() {
	forgejoCmd.AddCommand(installCmd)

	installCmd.Flags().StringVar(&instanceFlag, "instance", "", "Forgejo/Gitea instance hostname (default: codeberg.org)")
	installCmd.Flags().StringVar(&branchFlag, "branch", "", "Repository branch to retrieve releases for")
	installCmd.Flags().StringVar(&regexFlag, "regex", "", "Regular expression to match release assets against")
}
