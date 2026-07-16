package registry

import (
	"io/fs"
	"strings"
)

// buildConfig indexes the mod configuration trees of a modpack subdir
// (IDE.md §3.2). Each file becomes one entry whose Owner is the mod slug it
// belongs to when that can be determined from mods/*.pw.toml; entries with
// no owner are orphan candidates (config present, mod removed).
func buildConfig(dir string, b *builder) {
	slugs := modSlugs(dir)
	for _, origin := range []string{"config", "defaultconfigs"} {
		root := joinSlash(dir, origin)
		if !dirExists(root) {
			continue
		}
		origin := origin
		b.walkSource(sourceRoot{dir: root, origin: origin}, func(rel string, _ fs.FileInfo) {
			b.add(Entry{
				ID:     origin + "/" + rel,
				Kind:   "config_file",
				Origin: origin,
				Path:   rel,
				Owner:  configOwner(rel, slugs),
			})
		})
	}
}

// configOwner matches a config path against mod slugs. Both the top-level
// directory name and the file stem are tried, with common per-side suffixes
// stripped and '-'/'_' treated as equivalent, so "crash_assistant/config.json"
// and "sodium-extra-client.toml" resolve to their mods.
func configOwner(rel string, slugs []string) string {
	bySlug := make(map[string]string, len(slugs))
	for _, slug := range slugs {
		bySlug[normalizeConfigName(slug)] = slug
	}
	candidates := []string{stripExt(baseName(rel))}
	if i := strings.IndexByte(rel, '/'); i > 0 {
		candidates = append(candidates, rel[:i])
	}
	for _, candidate := range candidates {
		name := normalizeConfigName(candidate)
		for _, suffix := range []string{"", "-common", "-client", "-server", "-general"} {
			if owner, ok := bySlug[strings.TrimSuffix(name, suffix)]; ok {
				return owner
			}
		}
	}
	return ""
}

func normalizeConfigName(name string) string {
	return strings.ReplaceAll(strings.ToLower(name), "_", "-")
}

func baseName(rel string) string {
	if i := strings.LastIndexByte(rel, '/'); i >= 0 {
		return rel[i+1:]
	}
	return rel
}
