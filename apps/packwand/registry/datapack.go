package registry

import (
	"io/fs"
	"strings"
)

// datapackCategories maps a data/<ns>/<category> directory to the entry kind
// it holds. Both modern singular (1.21+) and legacy plural (1.20 and older)
// layouts are accepted.
var datapackCategories = map[string]string{
	"function": "function", "functions": "function",
	"predicate": "predicate", "predicates": "predicate",
	"loot_table": "loot_table", "loot_tables": "loot_table",
	"recipe": "recipe", "recipes": "recipe",
	"advancement": "advancement", "advancements": "advancement",
	"structure": "structure", "structures": "structure",
	"item_modifier": "item_modifier", "item_modifiers": "item_modifier",
	"damage_type":    "damage_type",
	"dimension":      "dimension",
	"dimension_type": "dimension_type",
	"enchantment":    "enchantment",
	"chat_type":      "chat_type",
}

// buildDatapack indexes every datapack resource location visible from dir:
// the pack's own data/ tree(s) for standalone datapacks, or the bundled
// packs (global_packs/required_data, optional_data) plus the kubejs/ virtual
// datapack for modpack subdirs (IDE.md §3.1).
func buildDatapack(dir string, b *builder) {
	for _, src := range datapackSources(dir) {
		src := src
		b.walkSource(src, func(rel string, _ fs.FileInfo) {
			if entry, ok := classifyData(rel); ok {
				entry.Origin = src.origin
				b.add(entry)
			}
		})
	}
}

func datapackSources(dir string) []sourceRoot {
	if !isModpackSubdir(dir) {
		return contentRoots(dir, "data")
	}
	var out []sourceRoot
	out = append(out, childPacks(dir, "global_packs/required_data", "data")...)
	out = append(out, childPacks(dir, "global_packs/optional_data", "data")...)
	if dirExists(joinSlash(dir, "kubejs/data")) {
		out = append(out, sourceRoot{dir: joinSlash(dir, "kubejs"), origin: "kubejs"})
	}
	return out
}

// classifyData turns a slash path relative to a datapack root into an entry.
func classifyData(rel string) (Entry, bool) {
	if rel == "pack.mcmeta" {
		return Entry{ID: "pack.mcmeta", Kind: "pack_mcmeta", Path: rel}, true
	}
	parts := strings.Split(rel, "/")
	if len(parts) < 4 || parts[0] != "data" {
		return Entry{}, false
	}
	ns, category := parts[1], parts[2]
	switch {
	case category == "tags" && len(parts) >= 5:
		tagType := strings.TrimSuffix(parts[3], "s")
		return Entry{
			ID:   ns + ":" + stripExt(strings.Join(parts[4:], "/")),
			Kind: "tag/" + tagType,
			Path: rel,
		}, true
	case category == "worldgen" && len(parts) >= 5:
		return Entry{
			ID:   ns + ":" + stripExt(strings.Join(parts[4:], "/")),
			Kind: "worldgen/" + parts[3],
			Path: rel,
		}, true
	}
	kind := datapackCategories[category]
	if kind == "" {
		kind = "data/" + category
	}
	return Entry{
		ID:   ns + ":" + stripExt(strings.Join(parts[3:], "/")),
		Kind: kind,
		Path: rel,
	}, true
}
