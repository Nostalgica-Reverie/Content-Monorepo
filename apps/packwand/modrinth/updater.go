package modrinth

import (
	"errors"
	"fmt"
	"os"
	"strings"

	modrinthApi "codeberg.org/jmansfield/go-modrinth/modrinth"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/mitchellh/mapstructure"
)

type mrUpdateData struct {
	// TODO(format): change to "project-id"
	ProjectID string `mapstructure:"mod-id"`
	// TODO(format): change to "version-id"
	InstalledVersion string `mapstructure:"version"`
}

func (u mrUpdateData) ToMap() (map[string]interface{}, error) {
	newMap := make(map[string]interface{})
	err := mapstructure.Decode(u, &newMap)
	return newMap, err
}

type mrUpdater struct{}

func (u mrUpdater) ParseUpdate(updateUnparsed map[string]interface{}) (interface{}, error) {
	var updateData mrUpdateData
	err := mapstructure.Decode(updateUnparsed, &updateData)
	return updateData, err
}

type cachedStateStore struct {
	ProjectID string
	Version   *modrinthApi.Version
}

// batchUpdateChecksEnabled gates the bulk hash-based update check.
// PACKWAND_MR_BATCH=0 forces the pre-batch one-request-per-mod path, kept as
// an escape hatch while validating that the server-side "latest" selection
// matches the local findLatestVersion semantics on real packs.
func batchUpdateChecksEnabled() bool {
	switch strings.TrimSpace(os.Getenv("PACKWAND_MR_BATCH")) {
	case "0", "false", "off":
		return false
	}
	return true
}

// batchHashAlgos are the algorithms POST /v2/version_files/update accepts.
var batchHashAlgos = map[string]bool{"sha1": true, "sha512": true}

func (u mrUpdater) CheckUpdate(mods []*core.Mod, pack core.Pack) ([]core.UpdateCheck, error) {
	results := make([]core.UpdateCheck, len(mods))
	gameVersions, loaders, err := versionSearchParams(pack)
	if err != nil {
		return nil, err
	}

	// Bulk path: one POST /version_files/update per hash algorithm resolves
	// the whole provider group — the Modrinth analogue of the CurseForge
	// batch design. Mods the batch can't answer (no usable stored hash, hash
	// unknown to Modrinth, batch request failed) drop to the per-item path.
	fallback := make([]int, 0, len(mods))
	if batchUpdateChecksEnabled() {
		byAlgo := make(map[string][]int)
		for i, mod := range mods {
			if _, ok := mod.GetParsedUpdateData("modrinth"); !ok {
				results[i] = core.UpdateCheck{Error: errors.New("failed to parse update metadata")}
				continue
			}
			if batchHashAlgos[mod.Download.HashFormat] && mod.Download.Hash != "" {
				byAlgo[mod.Download.HashFormat] = append(byAlgo[mod.Download.HashFormat], i)
			} else {
				fallback = append(fallback, i)
			}
		}
		for algo, idxs := range byAlgo {
			hashes := make([]string, len(idxs))
			for j, i := range idxs {
				hashes[j] = mods[i].Download.Hash
			}
			latest, err := latestVersionsByHash(hashes, algo, loaders, gameVersions)
			if err != nil {
				fmt.Printf("Warning: bulk Modrinth update check failed (%v); falling back to per-mod checks\n", err)
				fallback = append(fallback, idxs...)
				continue
			}
			for _, i := range idxs {
				if v, ok := latest[mods[i].Download.Hash]; ok && v != nil {
					results[i] = buildUpdateCheck(mods[i], v)
				} else {
					fallback = append(fallback, i)
				}
			}
		}
	} else {
		for i, mod := range mods {
			if _, ok := mod.GetParsedUpdateData("modrinth"); !ok {
				results[i] = core.UpdateCheck{Error: errors.New("failed to parse update metadata")}
				continue
			}
			fallback = append(fallback, i)
		}
	}

	core.ParallelFor(fallback, core.NetworkConcurrent(), func(_ int, i int) {
		mod := mods[i]
		rawData, _ := mod.GetParsedUpdateData("modrinth")
		data := rawData.(mrUpdateData)

		newVersion, err := getLatestVersionFiltered(data.ProjectID, mod.Name, gameVersions, loaders)
		if err != nil {
			results[i] = core.UpdateCheck{Error: fmt.Errorf("failed to get latest version: %v", err)}
			return
		}
		results[i] = buildUpdateCheck(mod, newVersion)
	})
	return results, nil
}

// buildUpdateCheck compares a resolved latest version against the mod's
// installed version and produces the UpdateCheck for it.
func buildUpdateCheck(mod *core.Mod, newVersion *modrinthApi.Version) core.UpdateCheck {
	rawData, ok := mod.GetParsedUpdateData("modrinth")
	if !ok {
		return core.UpdateCheck{Error: errors.New("failed to parse update metadata")}
	}
	data := rawData.(mrUpdateData)

	if *newVersion.ID == data.InstalledVersion {
		return core.UpdateCheck{UpdateAvailable: false}
	}

	if len(newVersion.Files) == 0 {
		return core.UpdateCheck{Error: errors.New("new version doesn't have any files")}
	}

	newFilename := newVersion.Files[0].Filename
	for _, v := range newVersion.Files {
		if *v.Primary {
			newFilename = v.Filename
		}
	}

	return core.UpdateCheck{
		UpdateAvailable: true,
		UpdateString:    mod.FileName + " -> " + *newFilename,
		CachedState:     cachedStateStore{data.ProjectID, newVersion},
	}
}

func (u mrUpdater) DoUpdate(mods []*core.Mod, cachedState []interface{}) error {
	for i, mod := range mods {
		modState := cachedState[i].(cachedStateStore)
		var version = modState.Version

		var file = version.Files[0]
		// Prefer the primary file
		for _, v := range version.Files {
			if *v.Primary {
				file = v
			}
		}

		download, err := downloadFromFile(file)
		if err != nil {
			return errors.New("file for project " + mod.Name + " doesn't have a valid hash")
		}

		mod.FileName = *file.Filename
		mod.Download = download
		mod.Update["modrinth"]["version"] = version.ID
	}

	return nil
}
