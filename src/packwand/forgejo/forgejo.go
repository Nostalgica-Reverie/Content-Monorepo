package forgejo

import (
	"encoding/json"
	"errors"
	"io"

	"github.com/mitchellh/mapstructure"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
)

var forgejoCmd = &cobra.Command{
	Use:     "forgejo",
	Aliases: []string{"gitea", "codeberg"},
	Short:   "Manage projects released on Forgejo, Gitea, or Codeberg",
}

func init() {
	cmd.AddToGroup(forgejoCmd, cmd.GroupPackManagement)
	core.Updaters["forgejo"] = forgejoUpdater{}
}

type Repo struct {
	ID       int    `json:"id"`
	Name     string `json:"name"`
	FullName string `json:"full_name"`
}

type Release struct {
	URL             string  `json:"url"`
	TagName         string  `json:"tag_name"`
	TargetCommitish string  `json:"target_commitish"`
	Name            string  `json:"name"`
	CreatedAt       string  `json:"created_at"`
	Assets          []Asset `json:"assets"`
}

type Asset struct {
	ID                 int    `json:"id"`
	Name               string `json:"name"`
	BrowserDownloadURL string `json:"browser_download_url"`
}

func (u forgejoUpdateData) ToMap() (map[string]interface{}, error) {
	newMap := make(map[string]interface{})
	err := mapstructure.Decode(u, &newMap)
	return newMap, err
}

func (u Asset) getHash() (string, error) {
	mainHasher, err := core.GetHashImpl(core.DefaultHashFormat)
	if err != nil {
		return "", err
	}

	resp, err := core.GetWithUA(u.BrowserDownloadURL, "application/octet-stream")
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if _, err := io.Copy(mainHasher, resp.Body); err != nil {
		return "", err
	}

	return mainHasher.HashToString(mainHasher.Sum(nil)), nil
}

func fetchRepo(instance, slug string) (Repo, error) {
	var repo Repo

	res, err := newClient(instance).getRepo(slug)
	if err != nil {
		return repo, err
	}
	defer res.Body.Close()

	repoBody, err := io.ReadAll(res.Body)
	if err != nil {
		return repo, err
	}

	if err = json.Unmarshal(repoBody, &repo); err != nil {
		return repo, err
	}

	if repo.FullName == "" {
		return repo, errors.New("invalid response while fetching repo: " + slug)
	}

	return repo, nil
}
