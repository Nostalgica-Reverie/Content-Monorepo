package gitlab

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"

	"github.com/dlclark/regexp2"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// GitLabRegex matches https://gitlab.com/owner/repo URLs.
var GitLabRegex = regexp.MustCompile(`^https?://gitlab\.com/([^/?#]+/[^/?#]+)`)

// GenericGitLabRegex matches https://{host}/owner/repo for self-hosted instances.
var GenericGitLabRegex = regexp.MustCompile(`^https?://([^/]+)/([^/?#]+/[^/?#]+)`)

var installCmd = &cobra.Command{
	Use:     "add [URL|slug]",
	Short:   "Add a project from a GitLab repository URL or owner/repo slug",
	Aliases: []string{"install", "get"},
	Args:    cobra.ArbitraryArgs,
	Run: func(cmd *cobra.Command, args []string) {
		pack, err := core.LoadPack()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		if len(args) == 0 || len(args[0]) == 0 {
			fmt.Println("You must specify a GitLab repository URL or owner/repo slug.")
			os.Exit(1)
		}

		instance := instanceFlag
		var slug string
		regex := defaultRegex

		if m := GitLabRegex.FindStringSubmatch(args[0]); len(m) == 2 {
			if instance == "" {
				instance = DefaultInstance
			}
			slug = m[1]
		} else if m := GenericGitLabRegex.FindStringSubmatch(args[0]); len(m) == 3 {
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

		if err = installMod(repo, instance, regex, pack); err != nil {
			fmt.Printf("Failed to add project: %s\n", err)
			os.Exit(1)
		}
	},
}

const defaultRegex = `^.+(?<!-api|-dev|-dev-preshadow|-sources)\.jar$`

func getLatestRelease(instance, slug string) (*glRelease, error) {
	releases, err := newClient(instance).listReleases(slug)
	if err != nil {
		return nil, err
	}
	if len(releases) == 0 {
		return nil, errors.New("no releases found")
	}
	return &releases[0], nil
}

func installMod(repo *glRepo, instance, regex string, pack core.Pack) error {
	release, err := getLatestRelease(instance, repo.PathWithNamespace)
	if err != nil {
		return fmt.Errorf("failed to get latest release: %v", err)
	}
	return installRelease(repo, instance, release, regex, pack)
}

func installRelease(repo *glRepo, instance string, release *glRelease, regex string, pack core.Pack) error {
	expr := regexp2.MustCompile(regex, 0)

	if len(release.Assets.Links) == 0 {
		return errors.New("release doesn't have any asset links")
	}

	var files []*glLink
	for i := range release.Assets.Links {
		if bl, _ := expr.MatchString(release.Assets.Links[i].Name); bl {
			files = append(files, &release.Assets.Links[i])
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
	updateMap["gitlab"], err = glUpdateData{
		Instance: instance,
		Slug:     repo.PathWithNamespace,
		Tag:      release.TagName,
		Regex:    regex,
	}.ToMap()
	if err != nil {
		return err
	}

	hash, err := getLinkHash(file)
	if err != nil {
		return err
	}

	modMeta := core.Mod{
		Name:     repo.Name,
		FileName: file.Name,
		Side:     core.UniversalSide,
		Download: core.ModDownload{
			URL:        file.URL,
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
var regexFlag string

func init() {
	gitlabCmd.AddCommand(installCmd)

	installCmd.Flags().StringVar(&instanceFlag, "instance", "", "GitLab instance hostname (default: gitlab.com)")
	installCmd.Flags().StringVar(&regexFlag, "regex", "", "Regular expression to match release assets against")
}
