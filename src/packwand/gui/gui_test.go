package gui

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"reflect"
	"testing"
)

func TestResolveActionScopesCommands(t *testing.T) {
	root := t.TempDir()
	subdir := filepath.Join(root, "modpacks", "example", "example-mr")
	if err := os.MkdirAll(subdir, 0o755); err != nil {
		t.Fatal(err)
	}
	s := &server{root: root, jobs: &jobStore{jobs: map[string]*job{}}}

	tests := []struct {
		name string
		req  actionRequest
		dir  string
		args []string
	}{
		{"doctor", actionRequest{Action: "doctor"}, root, []string{"doctor"}},
		{"update check", actionRequest{Action: "workspace-update-check"}, root, []string{"workspace", "update", "--all", "--check"}},
		{"validate project", actionRequest{Action: "validate-project", Path: "modpacks/example"}, filepath.Dir(subdir), []string{"validate", "manifest.json"}},
		{"build", actionRequest{Action: "build", Subdir: "modpacks/example/example-mr"}, subdir, []string{"build"}},
		{"rehash", actionRequest{Action: "rehash", Subdir: "modpacks/example/example-mr"}, subdir, []string{"rehash"}},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			dir, args, err := s.resolveAction(test.req)
			if err != nil {
				t.Fatal(err)
			}
			if dir != test.dir {
				t.Fatalf("dir = %q, want %q", dir, test.dir)
			}
			if !reflect.DeepEqual(args, test.args) {
				t.Fatalf("args = %#v, want %#v", args, test.args)
			}
		})
	}
}

func TestResolveActionRejectsEscapingPath(t *testing.T) {
	s := &server{root: t.TempDir(), jobs: &jobStore{jobs: map[string]*job{}}}
	if _, _, err := s.resolveAction(actionRequest{Action: "build", Path: "../outside"}); err == nil {
		t.Fatal("expected an escaping path to be rejected")
	}
}

func TestFeaturesEndpointCrossReferencesCommandCatalog(t *testing.T) {
	s := &server{root: t.TempDir(), jobs: &jobStore{jobs: map[string]*job{}}}
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/api/features", nil)
	s.handleFeatures(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", recorder.Code, http.StatusOK)
	}
	var response struct {
		PackwandVersion string              `json:"packwand_version"`
		Features        []featureCapability `json:"features"`
	}
	if err := json.Unmarshal(recorder.Body.Bytes(), &response); err != nil {
		t.Fatal(err)
	}
	if response.PackwandVersion == "" || len(response.Features) == 0 {
		t.Fatalf("incomplete feature response: %#v", response)
	}
	for _, feature := range response.Features {
		if feature.Command == "doctor" {
			if feature.GUIStatus != "integrated" || feature.GUIAction != "doctor" {
				t.Fatalf("doctor integration = %#v", feature)
			}
			return
		}
	}
	t.Fatal("doctor command missing from the feature catalog")
}
