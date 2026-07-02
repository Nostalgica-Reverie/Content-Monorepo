package forgejo

import (
	"errors"
	"fmt"

	gitea "code.gitea.io/sdk/gitea"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/dlclark/regexp2"
	"github.com/mitchellh/mapstructure"
)

type forgejoUpdateData struct {
	Instance string `mapstructure:"instance"`
	Slug     string `mapstructure:"slug"`
	Tag      string `mapstructure:"tag"`
	Branch   string `mapstructure:"branch"`
	Regex    string `mapstructure:"regex"`
}

type forgejoUpdater struct{}

func (u forgejoUpdater) ParseUpdate(updateUnparsed map[string]interface{}) (interface{}, error) {
	var updateData forgejoUpdateData
	err := mapstructure.Decode(updateUnparsed, &updateData)
	return updateData, err
}

type cachedStateStore struct {
	Instance string
	Slug     string
	Release  *gitea.Release
}

func (u forgejoUpdater) CheckUpdate(mods []*core.Mod, pack core.Pack) ([]core.UpdateCheck, error) {
	results := make([]core.UpdateCheck, len(mods))
	core.ParallelFor(mods, core.NetworkConcurrent(), func(i int, mod *core.Mod) {
		rawData, ok := mod.GetParsedUpdateData("forgejo")
		if !ok {
			results[i] = core.UpdateCheck{Error: errors.New("failed to parse update metadata")}
			return
		}

		data := rawData.(forgejoUpdateData)

		newRelease, err := getLatestRelease(data.Instance, data.Slug, data.Branch)
		if err != nil {
			results[i] = core.UpdateCheck{Error: fmt.Errorf("failed to get latest release: %v", err)}
			return
		}

		if newRelease.TagName == data.Tag {
			results[i] = core.UpdateCheck{UpdateAvailable: false}
			return
		}

		expr := regexp2.MustCompile(data.Regex, 0)

		if len(newRelease.Attachments) == 0 {
			results[i] = core.UpdateCheck{Error: errors.New("new release doesn't have any assets")}
			return
		}

		var newFiles []*gitea.Attachment
		for _, v := range newRelease.Attachments {
			if bl, _ := expr.MatchString(v.Name); bl {
				newFiles = append(newFiles, v)
			}
		}

		switch len(newFiles) {
		case 0:
			results[i] = core.UpdateCheck{Error: errors.New("release has no assets matching regex")}
		case 1:
			results[i] = core.UpdateCheck{
				UpdateAvailable: true,
				UpdateString:    mod.FileName + " -> " + newFiles[0].Name,
				CachedState:     cachedStateStore{data.Instance, data.Slug, newRelease},
			}
		default:
			results[i] = core.UpdateCheck{Error: errors.New("release has more than one asset matching regex")}
		}
	})
	return results, nil
}

func (u forgejoUpdater) DoUpdate(mods []*core.Mod, cachedState []interface{}) error {
	for i, mod := range mods {
		modState := cachedState[i].(cachedStateStore)
		release := modState.Release

		rawData, ok := mod.GetParsedUpdateData("forgejo")
		if !ok {
			return fmt.Errorf("missing forgejo update metadata for %s", mod.Name)
		}
		data := rawData.(forgejoUpdateData)

		expr := regexp2.MustCompile(data.Regex, 0)
		var file *gitea.Attachment
		for _, v := range release.Attachments {
			if bl, _ := expr.MatchString(v.Name); bl {
				file = v
				break
			}
		}
		if file == nil {
			return fmt.Errorf("no asset matching regex %q in release %s for %s", data.Regex, release.TagName, mod.Name)
		}

		hash, err := getAttachmentHash(file)
		if err != nil {
			return err
		}

		mod.FileName = file.Name
		mod.Download = core.ModDownload{
			URL:        file.DownloadURL,
			HashFormat: core.DefaultHashFormat,
			Hash:       hash,
		}
		mod.Update["forgejo"]["tag"] = release.TagName
	}
	return nil
}
