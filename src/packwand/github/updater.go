package github

import (
	"errors"
	"fmt"
	"runtime"
	"strings"
	"sync"

	"github.com/dlclark/regexp2"
	"github.com/mitchellh/mapstructure"
	"packwand/core"
)

type ghUpdateData struct {
	Slug   string `mapstructure:"slug"`
	Tag    string `mapstructure:"tag"`
	Branch string `mapstructure:"branch"`
	Regex  string `mapstructure:"regex"`
}

type ghUpdater struct{}

func (u ghUpdater) ParseUpdate(updateUnparsed map[string]interface{}) (interface{}, error) {
	var updateData ghUpdateData
	err := mapstructure.Decode(updateUnparsed, &updateData)
	return updateData, err
}

type cachedStateStore struct {
	Slug    string
	Release Release
}

func (u ghUpdater) CheckUpdate(mods []*core.Mod, pack core.Pack) ([]core.UpdateCheck, error) {
	results := make([]core.UpdateCheck, len(mods))
	sem := make(chan struct{}, max(1, min(runtime.NumCPU(), 8)))
	var wg sync.WaitGroup
	for i, mod := range mods {
		wg.Add(1)
		sem <- struct{}{}
		go func(i int, mod *core.Mod) {
			defer wg.Done()
			defer func() { <-sem }()

			rawData, ok := mod.GetParsedUpdateData("github")
			if !ok {
				results[i] = core.UpdateCheck{Error: errors.New("failed to parse update metadata")}
				return
			}

			data := rawData.(ghUpdateData)

			newRelease, err := getLatestRelease(data.Slug, data.Branch)
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
				bl, _ := expr.MatchString(v.Name)
				if bl {
					newFiles = append(newFiles, v)
				}
			}

			if len(newFiles) == 0 {
				results[i] = core.UpdateCheck{Error: errors.New("release doesn't have any assets matching regex")}
				return
			}

			if len(newFiles) > 1 {
				results[i] = core.UpdateCheck{Error: errors.New("release has more than one asset matching regex")}
				return
			}

			newFile := newFiles[0]
			results[i] = core.UpdateCheck{
				UpdateAvailable: true,
				UpdateString:    mod.FileName + " -> " + newFile.Name,
				CachedState:     cachedStateStore{data.Slug, newRelease},
			}
		}(i, mod)
	}
	wg.Wait()
	return results, nil
}

func (u ghUpdater) DoUpdate(mods []*core.Mod, cachedState []interface{}) error {
	for i, mod := range mods {
		modState := cachedState[i].(cachedStateStore)
		var release = modState.Release

		// yes, this is duplicated - i guess we should just cache asset + tag instead of entire release...?
		var file = release.Assets[0]
		for _, v := range release.Assets {
			if strings.HasSuffix(v.Name, ".jar") {
				file = v
			}
		}

		hash, err := file.getSha256()
		if err != nil {
			return err
		}

		mod.FileName = file.Name
		mod.Download = core.ModDownload{
			URL:        file.BrowserDownloadURL,
			HashFormat: "sha256",
			Hash:       hash,
		}
		mod.Update["github"]["tag"] = release.TagName
	}

	return nil
}
