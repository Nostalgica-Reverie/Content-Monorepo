package cmd

import (
	"os"
	"path/filepath"
	"testing"
)

func writeTestManifest(t *testing.T, packDir, id string, autoUpdate *string) {
	t.Helper()
	if err := os.MkdirAll(filepath.Join(packDir, "1.21-mr", "mods"), 0o755); err != nil {
		t.Fatal(err)
	}
	automation := ""
	if autoUpdate != nil {
		automation = `,"automation":{"auto_update":` + *autoUpdate + `}`
	}
	manifest := `{"id":"` + id + `","name":"` + id + `","type":"modpack","role":"none"` + automation + `}`
	if err := os.WriteFile(filepath.Join(packDir, "manifest.json"), []byte(manifest), 0o644); err != nil {
		t.Fatal(err)
	}
}

func TestIgnoredPackSubdirsFindsOptedOutPacks(t *testing.T) {
	root := t.TempDir()
	falseVal, trueVal := "false", "true"

	writeTestManifest(t, filepath.Join(root, "opted-out"), "opted-out", &falseVal)
	writeTestManifest(t, filepath.Join(root, "opted-in"), "opted-in", &trueVal)
	writeTestManifest(t, filepath.Join(root, "default"), "default", nil)

	subdirs := ignoredPackSubdirs(root)
	if len(subdirs) != 1 {
		t.Fatalf("subdirs = %#v, want exactly the opted-out pack's subdir", subdirs)
	}
	want := filepath.Join(root, "opted-out", "1.21-mr")
	if subdirs[0] != want {
		t.Fatalf("subdirs[0] = %q, want %q", subdirs[0], want)
	}
}

func TestIgnoredPackSubdirsExcludesArchivedPacks(t *testing.T) {
	root := t.TempDir()
	packDir := filepath.Join(root, "archived-pack")
	if err := os.MkdirAll(filepath.Join(packDir, "1.21-mr", "mods"), 0o755); err != nil {
		t.Fatal(err)
	}
	manifest := `{"id":"archived-pack","name":"archived-pack","type":"modpack","role":"none","lifecycle":"archived"}`
	if err := os.WriteFile(filepath.Join(packDir, "manifest.json"), []byte(manifest), 0o644); err != nil {
		t.Fatal(err)
	}

	if subdirs := ignoredPackSubdirs(root); len(subdirs) != 0 {
		t.Fatalf("expected archived packs to be excluded (they're a different skip reason), got %#v", subdirs)
	}
}

func TestResolveWorkspaceScopeAllOverridesPackCwd(t *testing.T) {
	packDir := t.TempDir()
	if err := os.WriteFile(filepath.Join(packDir, "manifest.json"), []byte(`{"id":"example"}`), 0o644); err != nil {
		t.Fatal(err)
	}

	filter, explicit := resolveWorkspaceScope(nil, packDir, true)
	if filter != "" || explicit {
		t.Fatalf("resolveWorkspaceScope(..., all=true) = (%q, %v), want (empty, false)", filter, explicit)
	}
}
