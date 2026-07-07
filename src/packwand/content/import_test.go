package content

import (
	"archive/zip"
	"bytes"
	"os"
	"path/filepath"
	"testing"
)

func buildOverridesZip(t *testing.T, entries map[string]string) *zip.Reader {
	t.Helper()
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)
	for name, body := range entries {
		w, err := zw.Create(name)
		if err != nil {
			t.Fatal(err)
		}
		if _, err := w.Write([]byte(body)); err != nil {
			t.Fatal(err)
		}
	}
	if err := zw.Close(); err != nil {
		t.Fatal(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(buf.Bytes()), int64(buf.Len()))
	if err != nil {
		t.Fatal(err)
	}
	return zr
}

func TestExtractOverridesSkipsIndexedFiles(t *testing.T) {
	zr := buildOverridesZip(t, map[string]string{
		"overrides/mods/indexed-mod.jar": "jar bytes",
		"overrides/config/settings.toml": "config",
	})
	dir := t.TempDir()

	// indexed-mod.jar is referenced by modrinth.index.json, so a .pw.toml is
	// written for it; the override copy must be skipped or refresh would
	// index the same mod twice.
	indexed := map[string]bool{"mods/indexed-mod.jar": true}
	count, jars := extractOverrides(zr, dir, indexed)

	if count != 1 {
		t.Fatalf("expected 1 extracted file, got %d", count)
	}
	if len(jars) != 0 {
		t.Errorf("expected no jar overrides (indexed-mod.jar was skipped, not copied), got %v", jars)
	}
	if _, err := os.Stat(filepath.Join(dir, "mods", "indexed-mod.jar")); !os.IsNotExist(err) {
		t.Error("indexed override jar should not have been extracted")
	}
	if _, err := os.Stat(filepath.Join(dir, "config", "settings.toml")); err != nil {
		t.Errorf("non-indexed override should have been extracted: %v", err)
	}
}

func TestExtractOverridesSkipsIndexedFilesCaseInsensitively(t *testing.T) {
	zr := buildOverridesZip(t, map[string]string{
		"overrides/mods/Some-Mod.jar": "jar bytes",
	})
	dir := t.TempDir()
	indexed := map[string]bool{"mods/some-mod.jar": true}
	if count, _ := extractOverrides(zr, dir, indexed); count != 0 {
		t.Fatalf("expected 0 extracted files, got %d", count)
	}
}

func TestExtractOverridesFlagsUnindexedJars(t *testing.T) {
	zr := buildOverridesZip(t, map[string]string{
		"overrides/mods/unresolved-mod.jar": "jar bytes",
		"overrides/config/settings.toml":    "config",
	})
	dir := t.TempDir()

	// No files indexed at all here (simulates a mod the source pack's export
	// couldn't resolve to a hosted file): the jar still gets copied (it's a
	// legitimate override), but must be flagged distinctly from ordinary
	// config overrides.
	count, jars := extractOverrides(zr, dir, nil)

	if count != 2 {
		t.Fatalf("expected 2 extracted files, got %d", count)
	}
	if len(jars) != 1 || jars[0] != "mods/unresolved-mod.jar" {
		t.Fatalf("expected exactly [mods/unresolved-mod.jar] flagged as a jar override, got %v", jars)
	}
}
