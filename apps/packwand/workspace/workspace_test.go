package workspace

import (
	"os"
	"os/exec"
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

func TestConfigureSubprocessNonInteractive(t *testing.T) {
	t.Setenv("PACKWAND_NON_INTERACTIVE", "")
	os.Unsetenv("PACKWAND_NON_INTERACTIVE")

	c := exec.Command("packwand", "update", "--all")
	ConfigureSubprocess(c)

	if c.Stdin != nil {
		t.Error("ConfigureSubprocess must leave stdin detached (nil → null device)")
	}
	found := false
	for _, kv := range c.Env {
		if kv == "PACKWAND_NON_INTERACTIVE=true" {
			found = true
		}
	}
	if !found {
		t.Error("ConfigureSubprocess did not set PACKWAND_NON_INTERACTIVE=true")
	}

	// An explicit user value must win.
	t.Setenv("PACKWAND_NON_INTERACTIVE", "false")
	c2 := exec.Command("packwand", "refresh")
	ConfigureSubprocess(c2)
	for _, kv := range c2.Env {
		if kv == "PACKWAND_NON_INTERACTIVE=true" {
			t.Error("ConfigureSubprocess overrode an explicit PACKWAND_NON_INTERACTIVE")
		}
	}
}

func TestStreamCommandStreamsAndReportsFailure(t *testing.T) {
	ok := exec.Command("go", "version")
	if err := StreamCommand(ok, "test-ok"); err != nil {
		t.Fatalf("StreamCommand(go version) = %v, want nil", err)
	}

	bad := exec.Command("go", "definitely-not-a-subcommand")
	if err := StreamCommand(bad, "test-bad"); err == nil {
		t.Error("StreamCommand(failing child) = nil, want error")
	}
}
