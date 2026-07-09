package cmd

import (
	"os"
	"path/filepath"
	"testing"
)

func writeTestFile(t *testing.T, path, content string) {
	t.Helper()
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}
}

func TestMinifyJSONPaths(t *testing.T) {
	dir := t.TempDir()
	pretty := "{\n  \"a\": 1,\n  \"b\": [1, 2, 3]\n}\n"
	writeTestFile(t, filepath.Join(dir, "pack", "data.json"), pretty)
	writeTestFile(t, filepath.Join(dir, "pack", "anim.mcmeta"), "{\n  \"animation\": {}\n}\n")
	writeTestFile(t, filepath.Join(dir, "pack", "notes.txt"), "not json, not scanned")
	writeTestFile(t, filepath.Join(dir, "pack", "node_modules", "dep.json"), pretty)
	writeTestFile(t, filepath.Join(dir, "pack", "commented.json"), "{\n  // json5 comment\n  \"a\": 1\n}\n")
	writeTestFile(t, filepath.Join(dir, "pack", "tiny.json"), `{"a":1}`)

	res, err := minifyJSONPaths([]string{dir}, false, false)
	if err != nil {
		t.Fatalf("minifyJSONPaths: %v", err)
	}
	if res.scanned != 4 {
		t.Errorf("scanned = %d, want 4 (node_modules and .txt excluded)", res.scanned)
	}
	if res.minified != 2 {
		t.Errorf("minified = %d, want 2", res.minified)
	}
	if res.skipped != 1 {
		t.Errorf("skipped = %d, want 1 (commented.json)", res.skipped)
	}

	got, err := os.ReadFile(filepath.Join(dir, "pack", "data.json"))
	if err != nil {
		t.Fatal(err)
	}
	if string(got) != `{"a":1,"b":[1,2,3]}` {
		t.Errorf("minified content = %q", got)
	}

	// Untouched files.
	if got, _ := os.ReadFile(filepath.Join(dir, "pack", "node_modules", "dep.json")); string(got) != pretty {
		t.Error("node_modules file was modified")
	}
	if got, _ := os.ReadFile(filepath.Join(dir, "pack", "commented.json")); len(got) == 0 || string(got) == `{"a":1}` {
		t.Error("invalid-JSON file was modified")
	}

	// Second run is a no-op.
	res, err = minifyJSONPaths([]string{dir}, false, false)
	if err != nil {
		t.Fatal(err)
	}
	if res.minified != 0 {
		t.Errorf("second run minified = %d, want 0", res.minified)
	}
}

func TestMinifyJSONPathsCheckMode(t *testing.T) {
	dir := t.TempDir()
	pretty := "{\n  \"a\": 1\n}\n"
	target := filepath.Join(dir, "data.json")
	writeTestFile(t, target, pretty)

	res, err := minifyJSONPaths([]string{target}, true, false)
	if err != nil {
		t.Fatal(err)
	}
	if res.minified != 1 {
		t.Errorf("check-mode minified = %d, want 1", res.minified)
	}
	if got, _ := os.ReadFile(target); string(got) != pretty {
		t.Error("check mode modified the file")
	}
}

func TestMinifyJSONPathsStrict(t *testing.T) {
	dir := t.TempDir()
	writeTestFile(t, filepath.Join(dir, "bad.json"), "{ // nope\n}")

	if _, err := minifyJSONPaths([]string{dir}, false, true); err == nil {
		t.Error("strict mode should fail on invalid JSON")
	}
}
