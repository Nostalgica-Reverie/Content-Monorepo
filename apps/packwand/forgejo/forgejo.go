package forgejo

import (
	"fmt"
	"io"
	"strings"

	gitea "code.gitea.io/sdk/gitea"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/mitchellh/mapstructure"
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

func (u forgejoUpdateData) ToMap() (map[string]interface{}, error) {
	newMap := make(map[string]interface{})
	err := mapstructure.Decode(u, &newMap)
	return newMap, err
}

func getAttachmentHash(a *gitea.Attachment) (string, error) {
	mainHasher, err := core.GetHashImpl(core.DefaultHashFormat)
	if err != nil {
		return "", err
	}

	resp, err := core.GetWithUA(a.DownloadURL, "application/octet-stream")
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if _, err := io.Copy(mainHasher, resp.Body); err != nil {
		return "", err
	}

	return mainHasher.HashToString(mainHasher.Sum(nil)), nil
}

func fetchRepo(instance, slug string) (*gitea.Repository, error) {
	parts := strings.SplitN(slug, "/", 2)
	if len(parts) != 2 {
		return nil, fmt.Errorf("invalid slug %q: expected owner/repo", slug)
	}
	repo, _, err := newClient(instance).GetRepo(parts[0], parts[1])
	return repo, err
}
