package cmd

import (
	"os"
	"path/filepath"
	"testing"
)

func writePwToml(t *testing.T, dir, slug, filename string) {
	t.Helper()
	writePwTomlNamed(t, dir, slug, slug, filename)
}

func writePwTomlNamed(t *testing.T, dir, slug, name, filename string) {
	t.Helper()
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	content := "name = \"" + name + "\"\nfilename = \"" + filename + "\"\n"
	if err := os.WriteFile(filepath.Join(dir, slug+".pw.toml"), []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}
}

func TestPackParityReports(t *testing.T) {
	pack := filepath.Join(t.TempDir(), "testpack")
	if err := os.MkdirAll(pack, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(pack, "manifest.json"), []byte(`{"id":"testpack","type":"modpack"}`), 0o644); err != nil {
		t.Fatal(err)
	}

	// Variant "26.1" — drifted: one mod only on mr, one filename mismatch.
	writePwToml(t, filepath.Join(pack, "26.1-mr", "mods"), "sodium", "sodium-1.0.jar")
	writePwToml(t, filepath.Join(pack, "26.1-mr", "mods"), "mr-only", "mronly-1.0.jar")
	writePwToml(t, filepath.Join(pack, "26.1-mr", "mods"), "lithium", "lithium-2.0.jar")
	writePwToml(t, filepath.Join(pack, "26.1-cf", "mods"), "sodium", "sodium-1.0.jar")
	writePwToml(t, filepath.Join(pack, "26.1-cf", "mods"), "lithium", "lithium-1.9.jar")

	// Variant "26.2" — in sync, including a mod whose slug differs per
	// platform but whose display name matches ("FerriteCore").
	writePwToml(t, filepath.Join(pack, "26.2-mr", "mods"), "sodium", "sodium-1.1.jar")
	writePwToml(t, filepath.Join(pack, "26.2-cf", "mods"), "sodium", "sodium-1.1.jar")
	writePwTomlNamed(t, filepath.Join(pack, "26.2-mr", "mods"), "ferrite-core", "FerriteCore (Fabric)", "ferritecore-8.0.jar")
	writePwTomlNamed(t, filepath.Join(pack, "26.2-cf", "mods"), "ferritecore-fabric", "FerriteCore Fabric", "ferritecore-8.0.jar")

	// Variant "26.3" — single-platform.
	writePwToml(t, filepath.Join(pack, "26.3-mr", "mods"), "sodium", "sodium-1.2.jar")

	reports := packParityReports(pack)
	if len(reports) != 3 {
		t.Fatalf("got %d reports, want 3: %+v", len(reports), reports)
	}

	r1 := reports[0]
	if r1.Variant != "26.1" || !r1.Drifted() {
		t.Errorf("26.1 should be drifted: %+v", r1)
	}
	if len(r1.OnlyMr) != 1 || r1.OnlyMr[0] != "mr-only" {
		t.Errorf("26.1 only_mr = %v, want [mr-only]", r1.OnlyMr)
	}
	if len(r1.FileDrift) != 1 {
		t.Errorf("26.1 file_drift = %v, want one lithium entry", r1.FileDrift)
	}

	r2 := reports[1]
	if r2.Variant != "26.2" || r2.Drifted() || r2.MrCount != 2 {
		t.Errorf("26.2 should be in sync with 2 mods: %+v", r2)
	}

	r3 := reports[2]
	if r3.Variant != "26.3" || r3.MissingSide != "cf" || r3.Drifted() {
		t.Errorf("26.3 should be single-platform missing cf: %+v", r3)
	}
}

func TestPackParityReportsNonPack(t *testing.T) {
	dir := t.TempDir() // no manifest.json
	if got := packParityReports(dir); got != nil {
		t.Errorf("non-pack dir should yield nil, got %+v", got)
	}
}
