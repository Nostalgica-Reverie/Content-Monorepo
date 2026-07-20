package cmd

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TestValidateReportsAllIssuesAtOnce guards the fix-one-rerun-fix-next loop:
// a manifest with several problems must surface every one of them in a single
// validation run.
func TestValidateReportsAllIssuesAtOnce(t *testing.T) {
	dir := t.TempDir()
	packDir := filepath.Join(dir, "brokenpack")
	if err := os.MkdirAll(packDir, 0o755); err != nil {
		t.Fatal(err)
	}
	// Broken in at least four independent ways: missing name, missing
	// release_type, missing role, no platform id, no mc_version/variants,
	// no version, no changelog.md.
	manifestPath := filepath.Join(packDir, "manifest.json")
	if err := os.WriteFile(manifestPath, []byte(`{"id":"brokenpack","type":"modpack","loader":"fabric"}`), 0o644); err != nil {
		t.Fatal(err)
	}

	err := validateManifestFileErr(manifestPath)
	if err == nil {
		t.Fatal("broken manifest validated clean")
	}
	msg := err.Error()
	for _, want := range []string{
		"missing required field: name",
		"missing required field: release_type",
		"missing required field: role",
		"either 'mc_version' or 'variants'",
		"missing required field: version",
		"at least one platform id",
		"changelog.md is missing",
	} {
		if !strings.Contains(msg, want) {
			t.Errorf("single run did not report %q\nfull error:\n%s", want, msg)
		}
	}
}

// TestValidateSingleIssueStaysCompact verifies the one-issue case keeps the
// plain single-line error format.
func TestValidateSingleIssueStaysCompact(t *testing.T) {
	vr := &validateRun{}
	vr.fail("only problem")
	if got := vr.err().Error(); got != "only problem" {
		t.Errorf("single-issue error = %q, want %q", got, "only problem")
	}
	if (&validateRun{}).err() != nil {
		t.Error("no-issue run returned a non-nil error")
	}
}
