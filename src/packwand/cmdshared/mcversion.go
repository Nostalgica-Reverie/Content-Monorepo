package cmdshared

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
)

const mcVersionManifestURL = "https://launchermeta.mojang.com/mc/game/version_manifest.json"
const mcVersionCacheTTL = time.Hour

type McVersionManifest struct {
	Latest struct {
		Release  string `json:"release"`
		Snapshot string `json:"snapshot"`
	} `json:"latest"`
	Versions []struct {
		ID          string    `json:"id"`
		Type        string    `json:"type"`
		URL         string    `json:"url"`
		Time        time.Time `json:"time"`
		ReleaseTime time.Time `json:"releaseTime"`
	} `json:"versions"`
}

func (m McVersionManifest) CheckValid(version string) {
	for _, v := range m.Versions {
		if v.ID == version {
			return
		}
	}
	Failf("%q is not a valid Minecraft version", version)
}

func GetValidMCVersions() (McVersionManifest, error) {
	cacheDir, err := core.GetPackwandCache()
	if err == nil {
		cacheFile := filepath.Join(cacheDir, "mc-version-manifest.json")
		if fi, err := os.Stat(cacheFile); err == nil && time.Since(fi.ModTime()) < mcVersionCacheTTL {
			if data, err := os.ReadFile(cacheFile); err == nil {
				var cached McVersionManifest
				if json.Unmarshal(data, &cached) == nil {
					sortManifest(&cached)
					return cached, nil
				}
			}
		}
		manifest, err := fetchVersionManifest()
		if err != nil {
			return McVersionManifest{}, err
		}
		_ = os.MkdirAll(cacheDir, 0o755)
		if data, err := json.Marshal(manifest); err == nil {
			_ = os.WriteFile(cacheFile, data, 0o644)
		}
		return manifest, nil
	}
	return fetchVersionManifest()
}

func fetchVersionManifest() (McVersionManifest, error) {
	res, err := core.GetWithUA(mcVersionManifestURL, "application/json")
	if err != nil {
		return McVersionManifest{}, err
	}
	defer res.Body.Close()
	var out McVersionManifest
	if err := json.NewDecoder(res.Body).Decode(&out); err != nil {
		return McVersionManifest{}, err
	}
	sortManifest(&out)
	return out, nil
}

func sortManifest(m *McVersionManifest) {
	sort.Slice(m.Versions, func(i, j int) bool {
		return m.Versions[i].ReleaseTime.Before(m.Versions[j].ReleaseTime)
	})
}
