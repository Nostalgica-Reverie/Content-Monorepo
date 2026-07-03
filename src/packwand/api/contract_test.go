package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
)

func TestUnversionedRoutesAreRemoved(t *testing.T) {
	server := testServer(t, "")
	response := httptest.NewRecorder()
	server.Handler(http.NotFoundHandler()).ServeHTTP(response, httptest.NewRequest(http.MethodGet, "/api/health", nil))
	if response.Code != http.StatusNotFound {
		t.Fatalf("status = %d", response.Code)
	}
	var body errorEnvelope
	if err := json.Unmarshal(response.Body.Bytes(), &body); err != nil {
		t.Fatal(err)
	}
	if body.Error.Code != "not_found" {
		t.Fatalf("unexpected error: %#v", body)
	}
}

func TestPackLookupUsesManifestID(t *testing.T) {
	server := testServer(t, "")
	manifest := `{"id":"manifest-id","name":"Example","type":"modpack","version":"1.0.0","role":"none"}`
	path := filepath.Join(server.root, "modpacks", "example", "manifest.json")
	if err := os.WriteFile(path, []byte(manifest), 0o644); err != nil {
		t.Fatal(err)
	}
	dir, err := server.packDir("manifest-id")
	if err != nil || filepath.Base(dir) != "example" {
		t.Fatalf("dir = %q, err = %v", dir, err)
	}
}
