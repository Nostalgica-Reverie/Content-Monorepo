package workspace

import (
	"os"
	"path/filepath"
	"testing"
)

func testWorkspaceRepo(t *testing.T) (root, packDir, subdir string) {
	t.Helper()
	root = t.TempDir()
	if err := os.Mkdir(filepath.Join(root, ".git"), 0o755); err != nil {
		t.Fatal(err)
	}
	packDir = filepath.Join(root, "modpacks", "example")
	subdir = filepath.Join(packDir, "1.21.1-mr")
	if err := os.MkdirAll(subdir, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(packDir, "manifest.json"), []byte(`{"id":"example"}`), 0o644); err != nil {
		t.Fatal(err)
	}
	return root, packDir, subdir
}

func chdirForTest(t *testing.T, dir string) {
	t.Helper()
	previous, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	if err := os.Chdir(dir); err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := os.Chdir(previous); err != nil {
			t.Errorf("restore working directory: %v", err)
		}
	})
}

func TestResolvedModpacksDirCollectsPackSubdirs(t *testing.T) {
	root, _, subdir := testWorkspaceRepo(t)
	chdirForTest(t, root)
	t.Setenv("MODPACKS_DIR", "")

	modpacksRoot := resolvedModpacksDir()
	wantRoot := filepath.Join(root, "modpacks")
	if modpacksRoot != wantRoot {
		t.Fatalf("resolvedModpacksDir() = %q, want %q", modpacksRoot, wantRoot)
	}

	targets, _ := CollectTargets(modpacksRoot, true, "", false)
	if len(targets) != 1 || targets[0] != subdir {
		t.Fatalf("CollectTargets() = %#v, want [%q]", targets, subdir)
	}
}

func TestResolveScopeFromInsidePack(t *testing.T) {
	root, packDir, subdir := testWorkspaceRepo(t)
	chdirForTest(t, root)
	t.Setenv("MODPACKS_DIR", "")

	filter, explicit := ResolveScope(nil, subdir)
	if filter != packDir {
		t.Fatalf("ResolveScope() filter = %q, want %q", filter, packDir)
	}
	if explicit {
		t.Fatal("ResolveScope() explicit = true, want false for cwd-derived scope")
	}
}

func TestCollectTargetsMatchesAbsoluteFilterWithRelativeRoot(t *testing.T) {
	root, packDir, _ := testWorkspaceRepo(t)
	chdirForTest(t, root)
	t.Setenv("MODPACKS_DIR", "")

	targets, _ := CollectTargets("modpacks", true, packDir, false)
	if len(targets) != 1 {
		t.Fatalf("CollectTargets() = %#v, want one scoped target", targets)
	}
}
