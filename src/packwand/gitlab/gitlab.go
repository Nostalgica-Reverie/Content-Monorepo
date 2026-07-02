package gitlab

import (
	"io"

	"github.com/mitchellh/mapstructure"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/cobra"
)

var gitlabCmd = &cobra.Command{
	Use:   "gitlab",
	Short: "Manage projects released on GitLab or self-hosted GitLab instances",
}

func init() {
	cmd.AddToGroup(gitlabCmd, cmd.GroupPackManagement)
	core.Updaters["gitlab"] = glUpdater{}
}

func (u glUpdateData) ToMap() (map[string]interface{}, error) {
	newMap := make(map[string]interface{})
	err := mapstructure.Decode(u, &newMap)
	return newMap, err
}

func getLinkHash(link *glLink) (string, error) {
	mainHasher, err := core.GetHashImpl(core.DefaultHashFormat)
	if err != nil {
		return "", err
	}

	resp, err := core.GetWithUA(link.URL, "application/octet-stream")
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if _, err := io.Copy(mainHasher, resp.Body); err != nil {
		return "", err
	}

	return mainHasher.HashToString(mainHasher.Sum(nil)), nil
}

func fetchRepo(instance, slug string) (*glRepo, error) {
	return newClient(instance).getProject(slug)
}
