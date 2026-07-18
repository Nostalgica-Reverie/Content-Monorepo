package content

import (
	"os"
	"path/filepath"
	"testing"
)

func preflightFixture(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	write := func(rel, content string) {
		t.Helper()
		full := filepath.Join(dir, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(full, []byte(content), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	write("manifest.json", `{"id":"example","name":"Example","type":"modpack","version":"1.0.0","role":"none"}`)
	write("sub-mr/pack.toml", "name = \"Example\"\n")
	write("sub-mr/global_packs/required_resources/Res/assets/ex/textures/block/stone.png", "png")
	write("sub-mr/global_packs/required_resources/Res/assets/ex/models/block/stone.json", `{"textures":{"all":"ex:block/stone"}}`)
	return filepath.Join(dir, "sub-mr")
}

func TestPreflightPassesOnCleanSubdir(t *testing.T) {
	dir := preflightFixture(t)
	result := RunPreflight(dir)
	if !result.OK || result.Errors != 0 {
		t.Fatalf("expected clean preflight, got %#v", result)
	}
	if len(result.Steps) != 3 {
		t.Fatalf("expected 3 steps, got %d", len(result.Steps))
	}
}

func TestPreflightFailsOnSyntaxAndReferenceErrors(t *testing.T) {
	dir := preflightFixture(t)
	broken := filepath.Join(dir, "config", "broken.json")
	if err := os.MkdirAll(filepath.Dir(broken), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(broken, []byte("{"), 0o644); err != nil {
		t.Fatal(err)
	}
	badModel := filepath.Join(dir, "global_packs", "required_resources", "Res", "assets", "ex", "models", "block", "bad.json")
	if err := os.WriteFile(badModel, []byte(`{"textures":{"all":"ex:block/missing"}}`), 0o644); err != nil {
		t.Fatal(err)
	}
	result := RunPreflight(dir)
	if result.OK || result.Errors < 2 {
		t.Fatalf("expected preflight failure with >=2 errors, got %#v", result)
	}
	names := map[string]int{}
	for _, step := range result.Steps {
		names[step.Name] = step.Errors
	}
	if names["syntax"] == 0 || names["references"] == 0 {
		t.Fatalf("expected both syntax and reference errors: %#v", names)
	}
}

func TestPreflightReportsMissingManifest(t *testing.T) {
	dir := t.TempDir()
	result := RunPreflight(dir)
	if result.OK {
		t.Fatalf("expected failure without a manifest, got %#v", result)
	}
}

func TestPreflightAcceptsCommentedConfigAndSkipsMacOSMetadata(t *testing.T) {
	dir := preflightFixture(t)
	commented := filepath.Join(dir, "config", "commented.json")
	if err := os.MkdirAll(filepath.Dir(commented), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(commented, []byte(`{
  // accepted mod config
  "url": "https://example.com/a//b",
  /* block comment */
  "enabled": true
}
`), 0o644); err != nil {
		t.Fatal(err)
	}
	metadata := filepath.Join(dir, "config", "paxi", "datapacks", "Example", "__MACOSX", "._pack.mcmeta")
	if err := os.MkdirAll(filepath.Dir(metadata), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(metadata, []byte{0, 1, 2}, 0o644); err != nil {
		t.Fatal(err)
	}

	result := RunPreflight(dir)
	if !result.OK || result.Errors != 0 {
		t.Fatalf("expected comments and OS metadata to be accepted, got %#v", result)
	}
	rootResult := RunPreflight(filepath.Dir(dir))
	if !rootResult.OK || rootResult.Errors != 0 {
		t.Fatalf("expected pack-root preflight to accept comments and OS metadata, got %#v", rootResult)
	}
}
