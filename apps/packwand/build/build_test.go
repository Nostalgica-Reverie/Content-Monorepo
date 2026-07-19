package build

import (
	"os"
	"path/filepath"
	"testing"
)

func TestFindSingleVersionDirResourcepackRoot(t *testing.T) {
	packDir := t.TempDir()
	writeBuildTestFile(t, filepath.Join(packDir, "pack.mcmeta"), `{}`)
	writeBuildTestFile(t, filepath.Join(packDir, "manifest.json"), `{
  "id": "example",
  "name": "Example",
  "type": "resourcepack",
  "version": "1.2.3",
  "mc_version": "1.21.1",
  "release_type": "release",
  "role": "none"
}`)
	if err := os.Mkdir(filepath.Join(packDir, "assets"), 0o755); err != nil {
		t.Fatal(err)
	}

	dir, version, err := findSingleVersionDir(packDir)
	if err != nil {
		t.Fatal(err)
	}
	if dir != packDir {
		t.Fatalf("dir = %q, want pack root %q", dir, packDir)
	}
	if version != "1.2.3" {
		t.Fatalf("version = %q, want 1.2.3", version)
	}
}

func TestFindSingleVersionDirNestedDatapack(t *testing.T) {
	packDir := t.TempDir()
	versionDir := filepath.Join(packDir, "1.21.1")
	if err := os.Mkdir(versionDir, 0o755); err != nil {
		t.Fatal(err)
	}

	dir, version, err := findSingleVersionDir(packDir)
	if err != nil {
		t.Fatal(err)
	}
	if dir != versionDir {
		t.Fatalf("dir = %q, want %q", dir, versionDir)
	}
	if version != "1.21.1" {
		t.Fatalf("version = %q, want 1.21.1", version)
	}
}

func TestFindBuiltModJarSelectsDistributableJar(t *testing.T) {
	libsDir := t.TempDir()
	writeBuildTestFile(t, filepath.Join(libsDir, "claritymod-fabric-26.1.2-1.0.0.jar"), "release")
	writeBuildTestFile(t, filepath.Join(libsDir, "claritymod-fabric-26.1.2-1.0.0-sources.jar"), "sources")
	writeBuildTestFile(t, filepath.Join(libsDir, "claritymod-fabric-26.1.2-1.0.0-dev.jar"), "dev")

	got, err := findBuiltModJar(libsDir)
	if err != nil {
		t.Fatal(err)
	}
	want := filepath.Join(libsDir, "claritymod-fabric-26.1.2-1.0.0.jar")
	if got != want {
		t.Fatalf("jar = %q, want %q", got, want)
	}
}

func TestFindBuiltModJarRejectsAmbiguousOutputs(t *testing.T) {
	libsDir := t.TempDir()
	writeBuildTestFile(t, filepath.Join(libsDir, "one.jar"), "one")
	writeBuildTestFile(t, filepath.Join(libsDir, "two.jar"), "two")
	if _, err := findBuiltModJar(libsDir); err == nil {
		t.Fatal("expected ambiguous Gradle outputs to fail")
	}
}

func TestPubArtifactNameMod(t *testing.T) {
	r := pubResolved{pType: "mod", pName: "Clarity-Mod", pVer: "1.0.0-26.1.2-fabric"}
	want := "Clarity-Mod-1.0.0-26.1.2-fabric.jar"
	if got := pubArtifactName(r, platModrinth); got != want {
		t.Fatalf("artifact name = %q, want %q", got, want)
	}
	if got := pubArtifactName(r, platCurseforge); got != want {
		t.Fatalf("CurseForge artifact name = %q, want %q", got, want)
	}
}

func TestPlanPatternsIncludeMods(t *testing.T) {
	if !planManifestRe.MatchString("mods/claritymod/manifest.json") {
		t.Fatal("mod manifest was not recognized by publish planning")
	}
	if got := planPackDirRe.FindString("mods/claritymod/src/main/java/Example.java"); got != "mods/claritymod/" {
		t.Fatalf("mod directory match = %q", got)
	}
}

func writeBuildTestFile(t *testing.T, path, contents string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(contents), 0o644); err != nil {
		t.Fatal(err)
	}
}
