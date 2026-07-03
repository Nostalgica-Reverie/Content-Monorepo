package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
)

func testServer(t *testing.T, token string) *Server {
	t.Helper()
	root := t.TempDir()
	if err := os.MkdirAll(filepath.Join(root, "modpacks", "example", "1.21-mr", "mods"), 0o755); err != nil {
		t.Fatal(err)
	}
	manifest := `{"id":"example","name":"Example","type":"modpack","mc_version":"1.21","version":"1.0.0","role":"none"}`
	if err := os.WriteFile(filepath.Join(root, "modpacks", "example", "manifest.json"), []byte(manifest), 0o644); err != nil {
		t.Fatal(err)
	}
	server, err := New(root, Options{Token: token})
	if err != nil {
		t.Fatal(err)
	}
	return server
}

func TestVersionIsPublicAndOtherRoutesRequireToken(t *testing.T) {
	server := testServer(t, "secret")
	for _, test := range []struct {
		path   string
		status int
	}{{Prefix + "/version", 200}, {Prefix + "/packs", 401}} {
		response := httptest.NewRecorder()
		server.Handler(nil).ServeHTTP(response, httptest.NewRequest(http.MethodGet, test.path, nil))
		if response.Code != test.status {
			t.Fatalf("%s: got %d, want %d", test.path, response.Code, test.status)
		}
	}
	request := httptest.NewRequest(http.MethodGet, Prefix+"/packs", nil)
	request.Header.Set("Authorization", "Bearer secret")
	response := httptest.NewRecorder()
	server.Handler(nil).ServeHTTP(response, request)
	if response.Code != 200 {
		t.Fatalf("authorized request: %d: %s", response.Code, response.Body.String())
	}
}

func TestPacksAreDiscoveredFromManifests(t *testing.T) {
	server := testServer(t, "")
	response := httptest.NewRecorder()
	server.Handler(nil).ServeHTTP(response, httptest.NewRequest(http.MethodGet, Prefix+"/packs", nil))
	var body struct {
		Packs []Pack `json:"packs"`
	}
	if err := json.Unmarshal(response.Body.Bytes(), &body); err != nil {
		t.Fatal(err)
	}
	if len(body.Packs) != 1 || body.Packs[0].ID != "example" || len(body.Packs[0].Subdirs) != 1 {
		t.Fatalf("unexpected packs: %#v", body.Packs)
	}
}

func TestOpenAPIComesFromActionRegistry(t *testing.T) {
	server := testServer(t, "")
	response := httptest.NewRecorder()
	server.Handler(nil).ServeHTTP(response, httptest.NewRequest(http.MethodGet, Prefix+"/openapi.json", nil))
	var document struct {
		OpenAPI string         `json:"openapi"`
		Paths   map[string]any `json:"paths"`
	}
	if err := json.Unmarshal(response.Body.Bytes(), &document); err != nil {
		t.Fatal(err)
	}
	if document.OpenAPI != "3.1.0" || document.Paths["/workspace/status"] == nil {
		t.Fatalf("unexpected OpenAPI document: %#v", document)
	}
}
