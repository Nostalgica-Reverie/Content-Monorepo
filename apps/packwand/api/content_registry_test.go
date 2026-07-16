package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/registry"
)

// registryTestServer extends the shared fixture with content the registry
// routes can index.
func registryTestServer(t *testing.T) *Server {
	t.Helper()
	server := testServer(t, "")
	subdir := filepath.Join(server.root, "modpacks", "example", "1.21-mr")
	for rel, content := range map[string]string{
		"pack.toml":                  "name = \"Example\"\n",
		"mods/sodium.pw.toml":        "name = \"Sodium\"\n",
		"config/sodium-options.json": "{}",
		"config/orphan.toml":         "",
	} {
		full := filepath.Join(subdir, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(full, []byte(content), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	return server
}

func getJSON(t *testing.T, server *Server, url string, target any) *httptest.ResponseRecorder {
	t.Helper()
	response := httptest.NewRecorder()
	server.Handler(nil).ServeHTTP(response, httptest.NewRequest(http.MethodGet, url, nil))
	if target != nil && response.Code == http.StatusOK {
		if err := json.Unmarshal(response.Body.Bytes(), target); err != nil {
			t.Fatal(err)
		}
	}
	return response
}

func TestRegistryRouteServesConfigRegistry(t *testing.T) {
	server := registryTestServer(t)
	var reg registry.Registry
	response := getJSON(t, server, Prefix+"/packs/example/subdirs/1.21-mr/registry/config", &reg)
	if response.Code != http.StatusOK {
		t.Fatalf("status = %d: %s", response.Code, response.Body.String())
	}
	if reg.Kind != registry.Config || reg.Scope != "modpacks/example/1.21-mr" || len(reg.Entries) != 2 {
		t.Fatalf("unexpected registry: %#v", reg)
	}
}

func TestRegistryRouteRejectsUnknownSubdirAndKind(t *testing.T) {
	server := registryTestServer(t)
	if response := getJSON(t, server, Prefix+"/packs/example/subdirs/nope-mr/registry/config", nil); response.Code != http.StatusNotFound {
		t.Fatalf("unknown subdir: status = %d", response.Code)
	}
	if response := getJSON(t, server, Prefix+"/packs/example/subdirs/1.21-mr/registry/bogus", nil); response.Code != http.StatusBadRequest {
		t.Fatalf("unknown kind: status = %d", response.Code)
	}
}

func TestRegistryRouteAcceptsRootForPacksWithoutSubdirs(t *testing.T) {
	server := registryTestServer(t)
	var reg registry.Registry
	response := getJSON(t, server, Prefix+"/packs/example/subdirs/root/registry/datapack", &reg)
	if response.Code != http.StatusOK || reg.Scope != "modpacks/example" {
		t.Fatalf("status = %d, registry = %#v", response.Code, reg)
	}
}

func TestRegistryCompleteFiltersByQuery(t *testing.T) {
	server := registryTestServer(t)
	var body struct {
		Query string           `json:"query"`
		Items []registry.Entry `json:"items"`
	}
	response := getJSON(t, server, Prefix+"/packs/example/subdirs/1.21-mr/registry/config/complete?q=sodium", &body)
	if response.Code != http.StatusOK {
		t.Fatalf("status = %d: %s", response.Code, response.Body.String())
	}
	if len(body.Items) != 1 || body.Items[0].ID != "config/sodium-options.json" {
		t.Fatalf("unexpected completion: %#v", body.Items)
	}
}

func TestRegistryCompleteRejectsEscapingFilePaths(t *testing.T) {
	server := registryTestServer(t)
	response := getJSON(t, server, Prefix+"/packs/example/subdirs/1.21-mr/registry/config/complete?file=..%2F..%2Fsecret&offset=0", nil)
	if response.Code != http.StatusBadRequest {
		t.Fatalf("status = %d: %s", response.Code, response.Body.String())
	}
}

func TestRegistryRebuildActionRunsRegistryAll(t *testing.T) {
	server := registryTestServer(t)
	action, ok := actionByName("packs.registry.rebuild")
	if !ok {
		t.Fatal("packs.registry.rebuild is not registered")
	}
	request := newActionRequest(http.MethodPost, "/x", "example", "", "")
	request.SetPathValue("sub", "1.21-mr")
	dir, args, err := action.Build(server, request)
	if err != nil {
		t.Fatal(err)
	}
	subdir := filepath.Join(server.root, "modpacks", "example", "1.21-mr")
	if dir != subdir || len(args) != 3 || args[0] != "registry" || args[1] != "all" || args[2] != "--json" {
		t.Fatalf("dir=%q args=%#v", dir, args)
	}
}
