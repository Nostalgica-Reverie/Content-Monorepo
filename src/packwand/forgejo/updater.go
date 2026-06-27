package forgejo

import (
	"errors"
	"fmt"
	"runtime"
	"sync"

	"github.com/dlclark/regexp2"
	"github.com/mitchellh/mapstructure"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
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
	Release  Release
}

func (u forgejoUpdater) CheckUpdate(mods []*core.Mod, pack core.Pack) ([]core.UpdateCheck, error) {
	results := make([]core.UpdateCheck, len(mods))
	sem := make(chan struct{}, max(1, min(runtime.NumCPU(), 8)))
	var wg sync.WaitGroup
	for i, mod := range mods {
		wg.Add(1)
		sem <- struct{}{}
		go func(i int, mod *core.Mod) {
			defer wg.Done()
			defer func() { <-sem }()

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

			if len(newRelease.Assets) == 0 {
				results[i] = core.UpdateCheck{Error: errors.New("new release doesn't have any assets")}
				return
			}

			var newFiles []Asset
			for _, v := range newRelease.Assets {
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
		}(i, mod)
	}
	wg.Wait()
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
		var file Asset
		for _, v := range release.Assets {
			if bl, _ := expr.MatchString(v.Name); bl {
				file = v
				break
			}
		}
		if file.Name == "" {
			return fmt.Errorf("no asset matching regex %q in release %s for %s", data.Regex, release.TagName, mod.Name)
		}

		hash, err := file.getHash()
		if err != nil {
			return err
		}

		mod.FileName = file.Name
		mod.Download = core.ModDownload{
			URL:        file.BrowserDownloadURL,
			HashFormat: core.DefaultHashFormat,
			Hash:       hash,
		}
		mod.Update["forgejo"]["tag"] = release.TagName
	}
	return nil
}
