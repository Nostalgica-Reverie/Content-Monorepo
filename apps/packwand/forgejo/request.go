package forgejo

import (
	gitea "code.gitea.io/sdk/gitea"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/viper"
)

const DefaultInstance = "codeberg.org"

func newClient(instance string) *gitea.Client {
	if instance == "" {
		instance = DefaultInstance
	}
	token := viper.GetString("forgejo." + instance + ".token")
	if token == "" {
		token = viper.GetString("forgejo.token")
	}
	opts := []gitea.ClientOption{
		gitea.SetUserAgent(core.UserAgent),
		gitea.SetHTTPClient(core.NewClient()),
	}
	if token != "" {
		opts = append(opts, gitea.SetToken(token))
	}
	c, _ := gitea.NewClient("https://"+instance, opts...)
	return c
}
