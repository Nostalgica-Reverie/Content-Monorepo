package registry

import (
	"os"
	"path/filepath"
	"testing"
)

// writeFile creates a file (and parents) under root from a slash path.
func writeFile(t *testing.T, root, rel, content string) {
	t.Helper()
	full := filepath.Join(root, filepath.FromSlash(rel))
	if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(full, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}
}

// modpackSubdir builds a representative modpack subdir fixture.
func modpackSubdir(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	writeFile(t, dir, "pack.toml", "name = \"Example\"\n")
	writeFile(t, dir, "mods/sodium.pw.toml", "name = \"Sodium\"\n")
	writeFile(t, dir, "mods/sodium-extra.pw.toml", "name = \"Sodium Extra\"\n")
	writeFile(t, dir, "mods/crash-assistant.pw.toml", "name = \"Crash Assistant\"\n")
	writeFile(t, dir, "config/crash_assistant/config.json", "{}")
	writeFile(t, dir, "config/sodium-extra-client.toml", "")
	writeFile(t, dir, "config/orphaned.toml", "")
	writeFile(t, dir, "global_packs/required_data/MyData/pack.mcmeta", `{"pack":{"pack_format":15}}`)
	writeFile(t, dir, "global_packs/required_data/MyData/data/mypack/functions/foo.mcfunction", "say hi")
	writeFile(t, dir, "global_packs/required_data/MyData/data/mypack/tags/functions/tick.json", `{"values":[]}`)
	writeFile(t, dir, "global_packs/required_resources/MyRes/assets/myres/textures/block/stone.png", "png")
	writeFile(t, dir, "global_packs/required_resources/MyRes/assets/myres/models/block/stone.json", `{"parent":"minecraft:block/cube_all"}`)
	writeFile(t, dir, "kubejs/data/kubejs/recipes/bar.json", "{}")
	writeFile(t, dir, "kubejs/assets/kubejs/blockstates/foo.json", "{}")
	writeFile(t, dir, "kubejs/server_scripts/recipes.js", "// recipes")
	writeFile(t, dir, "kubejs/startup_scripts/registry.js", "// registry")
	return dir
}

func findEntry(entries []Entry, id, kind string) *Entry {
	for i := range entries {
		if entries[i].ID == id && entries[i].Kind == kind {
			return &entries[i]
		}
	}
	return nil
}

func TestDatapackRegistryIndexesBundledAndKubeJSData(t *testing.T) {
	dir := modpackSubdir(t)
	reg, err := Build(dir, Datapack)
	if err != nil {
		t.Fatal(err)
	}
	fn := findEntry(reg.Entries, "mypack:foo", "function")
	if fn == nil || fn.Origin != "global_packs/required_data/MyData" {
		t.Fatalf("function entry missing or wrong origin: %#v", fn)
	}
	if findEntry(reg.Entries, "mypack:tick", "tag/function") == nil {
		t.Fatalf("function tag entry missing: %#v", reg.Entries)
	}
	recipe := findEntry(reg.Entries, "kubejs:bar", "recipe")
	if recipe == nil || recipe.Origin != "kubejs" {
		t.Fatalf("kubejs recipe entry missing or wrong origin: %#v", recipe)
	}
	if findEntry(reg.Entries, "pack.mcmeta", "pack_mcmeta") == nil {
		t.Fatal("pack.mcmeta entry missing")
	}
}

func TestStandaloneDatapackUsesVersionContentRoots(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, dir, "manifest.json", `{"id":"dp","type":"datapack"}`)
	writeFile(t, dir, "1.20/pack.mcmeta", `{"pack":{"pack_format":15}}`)
	writeFile(t, dir, "1.20/data/legacy/loot_tables/chests/spawn.json", "{}")
	reg, err := Build(dir, Datapack)
	if err != nil {
		t.Fatal(err)
	}
	loot := findEntry(reg.Entries, "legacy:chests/spawn", "loot_table")
	if loot == nil || loot.Origin != "1.20" {
		t.Fatalf("loot table entry missing or wrong origin: %#v", reg.Entries)
	}
}

func TestResourcePackRegistryIndexesBundledAndKubeJSAssets(t *testing.T) {
	dir := modpackSubdir(t)
	reg, err := Build(dir, ResourcePack)
	if err != nil {
		t.Fatal(err)
	}
	texture := findEntry(reg.Entries, "myres:block/stone", "texture")
	if texture == nil || texture.Origin != "global_packs/required_resources/MyRes" {
		t.Fatalf("texture entry missing or wrong origin: %#v", texture)
	}
	if findEntry(reg.Entries, "myres:block/stone", "model") == nil {
		t.Fatal("model entry missing")
	}
	blockstate := findEntry(reg.Entries, "kubejs:foo", "blockstate")
	if blockstate == nil || blockstate.Origin != "kubejs" {
		t.Fatalf("kubejs blockstate entry missing or wrong origin: %#v", blockstate)
	}
}

func TestConfigRegistryMatchesOwnersAndFlagsOrphans(t *testing.T) {
	dir := modpackSubdir(t)
	reg, err := Build(dir, Config)
	if err != nil {
		t.Fatal(err)
	}
	byDir := findEntry(reg.Entries, "config/crash_assistant/config.json", "config_file")
	if byDir == nil || byDir.Owner != "crash-assistant" {
		t.Fatalf("directory-owned config wrong: %#v", byDir)
	}
	bySuffix := findEntry(reg.Entries, "config/sodium-extra-client.toml", "config_file")
	if bySuffix == nil || bySuffix.Owner != "sodium-extra" {
		t.Fatalf("suffix-stripped config wrong: %#v", bySuffix)
	}
	orphan := findEntry(reg.Entries, "config/orphaned.toml", "config_file")
	if orphan == nil || orphan.Owner != "" {
		t.Fatalf("orphaned config should have no owner: %#v", orphan)
	}
}

func TestKubeJSRegistryIndexesScriptsEventsAndMods(t *testing.T) {
	dir := modpackSubdir(t)
	reg, err := Build(dir, KubeJS)
	if err != nil {
		t.Fatal(err)
	}
	if findEntry(reg.Entries, "recipes.js", "script/server") == nil {
		t.Fatal("server script entry missing")
	}
	if findEntry(reg.Entries, "registry.js", "script/startup") == nil {
		t.Fatal("startup script entry missing")
	}
	if findEntry(reg.Entries, "ServerEvents.recipes", "event/server") == nil {
		t.Fatal("builtin event entry missing")
	}
	if findEntry(reg.Entries, "sodium", "mod") == nil {
		t.Fatal("mod entry missing")
	}
}

func TestKubeJSRegistryIsEmptyWithoutKubeJSDir(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, dir, "pack.toml", "")
	reg, err := Build(dir, KubeJS)
	if err != nil {
		t.Fatal(err)
	}
	if len(reg.Entries) != 0 {
		t.Fatalf("expected no entries, got %d", len(reg.Entries))
	}
}

func TestCompleteRanksPrefixBeforeSubstringAndFiltersKinds(t *testing.T) {
	reg := &Registry{Entries: []Entry{
		{ID: "mypack:stone", Kind: "function"},
		{ID: "mypack:cobblestone", Kind: "function"},
		{ID: "other:stone_path", Kind: "tag/function"},
	}}
	items := reg.Complete("stone", nil, 0)
	if len(items) != 3 || items[0].ID != "mypack:stone" {
		t.Fatalf("unexpected ranking: %#v", items)
	}
	tags := reg.Complete("", []string{"tag"}, 0)
	if len(tags) != 1 || tags[0].Kind != "tag/function" {
		t.Fatalf("kind group filter failed: %#v", tags)
	}
}

func TestInferFromFileReadsTokenAndKeyContext(t *testing.T) {
	dir := t.TempDir()
	model := `{"parent": "myres:block/sto`
	writeFile(t, dir, "model.json", model)
	query, kinds, err := InferFromFile(filepath.Join(dir, "model.json"), len(model))
	if err != nil {
		t.Fatal(err)
	}
	if query != "myres:block/sto" || len(kinds) != 1 || kinds[0] != "model" {
		t.Fatalf("query=%q kinds=%#v", query, kinds)
	}

	tag := `{"values": ["#mypack:ti`
	writeFile(t, dir, "tag.json", tag)
	query, kinds, err = InferFromFile(filepath.Join(dir, "tag.json"), len(tag))
	if err != nil {
		t.Fatal(err)
	}
	if query != "mypack:ti" || len(kinds) != 1 || kinds[0] != "tag" {
		t.Fatalf("query=%q kinds=%#v", query, kinds)
	}

	if _, _, err := InferFromFile(filepath.Join(dir, "model.json"), 9999); err == nil {
		t.Fatal("expected an out-of-range offset error")
	}
}

func TestVersionChangesWhenContentChanges(t *testing.T) {
	dir := modpackSubdir(t)
	before, err := Build(dir, Datapack)
	if err != nil {
		t.Fatal(err)
	}
	again, err := Build(dir, Datapack)
	if err != nil {
		t.Fatal(err)
	}
	if before.Version != again.Version {
		t.Fatal("version should be deterministic for unchanged content")
	}
	writeFile(t, dir, "global_packs/required_data/MyData/data/mypack/functions/foo.mcfunction", "say hi there")
	after, err := Build(dir, Datapack)
	if err != nil {
		t.Fatal(err)
	}
	if before.Version == after.Version {
		t.Fatal("version should change when file content changes")
	}
}

func TestBuildAllReturnsEveryKind(t *testing.T) {
	dir := modpackSubdir(t)
	registries, err := BuildAll(dir)
	if err != nil {
		t.Fatal(err)
	}
	if len(registries) != 4 {
		t.Fatalf("expected 4 registries, got %d", len(registries))
	}
}
