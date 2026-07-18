package registry

import (
	"io/fs"
	"strings"
)

// assetCategories maps an assets/<ns>/<category> directory to the entry kind
// it holds.
var assetCategories = map[string]string{
	"textures":    "texture",
	"models":      "model",
	"blockstates": "blockstate",
	"lang":        "lang",
	"sounds":      "sound",
	"font":        "font",
	"atlases":     "atlas",
	"particles":   "particle",
	"shaders":     "shader",
	"texts":       "text",
}

// buildResourcePack indexes every resource pack asset visible from dir: the
// pack's own assets/ tree(s) for standalone resource packs, or the bundled
// packs (resourcepacks/, global_packs/required_resources) plus the kubejs/
// virtual resource pack for modpack subdirs (IDE.md §3.3).
func buildResourcePack(dir string, b *builder) {
	for _, src := range resourcePackSources(dir) {
		src := src
		b.walkSource(src, func(rel string, _ fs.FileInfo) {
			if entry, ok := classifyAsset(rel); ok {
				entry.Origin = src.origin
				b.add(entry)
			}
		})
	}
}

func resourcePackSources(dir string) []sourceRoot {
	if !isModpackSubdir(dir) {
		return contentRoots(dir, "assets")
	}
	var out []sourceRoot
	out = append(out, childPacks(dir, "resourcepacks", "assets")...)
	out = append(out, childPacks(dir, "global_packs/required_resources", "assets")...)
	if dirExists(joinSlash(dir, "kubejs/assets")) {
		out = append(out, sourceRoot{dir: joinSlash(dir, "kubejs"), origin: "kubejs"})
	}
	return out
}

// classifyAsset turns a slash path relative to a resource pack root into an
// entry. *.mcmeta companions inside category directories (e.g. animated
// texture metadata) are not referenceable IDs and are skipped; they still
// contribute to the version hash via walkSource.
func classifyAsset(rel string) (Entry, bool) {
	if rel == "pack.mcmeta" {
		return Entry{ID: "pack.mcmeta", Kind: "pack_mcmeta", Path: rel}, true
	}
	parts := strings.Split(rel, "/")
	if len(parts) < 3 || parts[0] != "assets" {
		return Entry{}, false
	}
	ns := parts[1]
	if len(parts) == 3 {
		// Files directly under a namespace, e.g. sounds.json.
		kind := "asset"
		if parts[2] == "sounds.json" {
			kind = "sound_definitions"
		}
		return Entry{ID: ns + ":" + stripExt(parts[2]), Kind: kind, Path: rel}, true
	}
	if strings.HasSuffix(rel, ".mcmeta") {
		return Entry{}, false
	}
	category := parts[2]
	kind := assetCategories[category]
	if kind == "" {
		kind = "asset/" + category
	}
	return Entry{
		ID:   ns + ":" + stripExt(strings.Join(parts[3:], "/")),
		Kind: kind,
		Path: rel,
	}, true
}
