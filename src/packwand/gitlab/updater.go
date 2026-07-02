package gitlab

import (
	"errors"
	"fmt"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/dlclark/regexp2"
	"github.com/mitchellh/mapstructure"
)

type glUpdateData struct {
	Instance string `mapstructure:"instance"`
	Slug     string `mapstructure:"slug"`
	Tag      string `mapstructure:"tag"`
	Regex    string `mapstructure:"regex"`
}

type glUpdater struct{}

func (u glUpdater) ParseUpdate(updateUnparsed map[string]interface{}) (interface{}, error) {
	var updateData glUpdateData
	err := mapstructure.Decode(updateUnparsed, &updateData)
	return updateData, err
}

type cachedStateStore struct {
	Instance string
	Slug     string
	Release  glRelease
}

func (u glUpdater) CheckUpdate(mods []*core.Mod, pack core.Pack) ([]core.UpdateCheck, error) {
	results := make([]core.UpdateCheck, len(mods))
	core.ParallelFor(mods, core.NetworkConcurrent(), func(i int, mod *core.Mod) {
		rawData, ok := mod.GetParsedUpdateData("gitlab")
		if !ok {
			results[i] = core.UpdateCheck{Error: errors.New("failed to parse update metadata")}
			return
		}

		data := rawData.(glUpdateData)

		releases, err := newClient(data.Instance).listReleases(data.Slug)
		if err != nil {
			results[i] = core.UpdateCheck{Error: fmt.Errorf("failed to list releases: %v", err)}
			return
		}
		if len(releases) == 0 {
			results[i] = core.UpdateCheck{Error: errors.New("no releases found")}
			return
		}
		newRelease := releases[0]

		if newRelease.TagName == data.Tag {
			results[i] = core.UpdateCheck{UpdateAvailable: false}
			return
		}

		expr := regexp2.MustCompile(data.Regex, 0)

		if len(newRelease.Assets.Links) == 0 {
			results[i] = core.UpdateCheck{Error: errors.New("new release has no asset links")}
			return
		}

		var newFiles []*glLink
		for j := range newRelease.Assets.Links {
			if bl, _ := expr.MatchString(newRelease.Assets.Links[j].Name); bl {
				newFiles = append(newFiles, &newRelease.Assets.Links[j])
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

func (u glUpdater) DoUpdate(mods []*core.Mod, cachedState []interface{}) error {
	for i, mod := range mods {
		modState := cachedState[i].(cachedStateStore)
		release := modState.Release

		rawData, ok := mod.GetParsedUpdateData("gitlab")
		if !ok {
			return fmt.Errorf("missing gitlab update metadata for %s", mod.Name)
		}
		data := rawData.(glUpdateData)

		expr := regexp2.MustCompile(data.Regex, 0)
		var file *glLink
		for j := range release.Assets.Links {
			if bl, _ := expr.MatchString(release.Assets.Links[j].Name); bl {
				file = &release.Assets.Links[j]
				break
			}
		}
		if file == nil {
			return fmt.Errorf("no asset matching regex %q in release %s for %s", data.Regex, release.TagName, mod.Name)
		}

		hash, err := getLinkHash(file)
		if err != nil {
			return err
		}

		mod.FileName = file.Name
		mod.Download = core.ModDownload{
			URL:        file.URL,
			HashFormat: core.DefaultHashFormat,
			Hash:       hash,
		}
		mod.Update["gitlab"]["tag"] = release.TagName
	}
	return nil
}
