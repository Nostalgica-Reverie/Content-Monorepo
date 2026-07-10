package manifest

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLoadAllDiscoversAndSortsManifestEntries(t *testing.T) {
	root := t.TempDir()
	writeManifest := func(category, dir, id string) {
		t.Helper()
		packDir := filepath.Join(root, category, dir)
		if err := os.MkdirAll(packDir, 0o755); err != nil {
			t.Fatal(err)
		}
		data := []byte(`{"id":"` + id + `","name":"` + id + `","type":"modpack","role":"none"}`)
		if err := os.WriteFile(filepath.Join(packDir, "manifest.json"), data, 0o644); err != nil {
			t.Fatal(err)
		}
	}
	writeManifest("resourcepacks", "zeta", "zeta")
	writeManifest("modpacks", "beta", "beta")
	writeManifest("modpacks", "alpha", "")
	if err := os.MkdirAll(filepath.Join(root, "datapacks", "invalid"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(root, "datapacks", "invalid", "manifest.json"), []byte("{"), 0o644); err != nil {
		t.Fatal(err)
	}

	entries, err := LoadAll(root)
	if err != nil {
		t.Fatal(err)
	}
	if len(entries) != 3 {
		t.Fatalf("got %d entries, want 3", len(entries))
	}
	got := []string{entries[0].Category + "/" + entries[0].ID, entries[1].Category + "/" + entries[1].ID, entries[2].Category + "/" + entries[2].ID}
	want := []string{"modpacks/alpha", "modpacks/beta", "resourcepacks/zeta"}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("entry %d = %q, want %q", i, got[i], want[i])
		}
	}
	if entries[0].Dir != filepath.Join(root, "modpacks", "alpha") {
		t.Fatalf("unexpected entry dir %q", entries[0].Dir)
	}
}
