package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func requestJSON(t *testing.T, server *Server, method, url, body string) *httptest.ResponseRecorder {
	t.Helper()
	response := httptest.NewRecorder()
	request := httptest.NewRequest(method, url, strings.NewReader(body))
	if body != "" {
		request.Header.Set("Content-Type", "application/json")
	}
	server.Handler(nil).ServeHTTP(response, request)
	return response
}

func TestTreeGroupsPackMetadataAndConfigs(t *testing.T) {
	server := registryTestServer(t)
	var body struct {
		Groups []treeGroup `json:"groups"`
	}
	response := requestJSON(t, server, http.MethodGet, Prefix+"/packs/example/subdirs/1.21-mr/tree", "")
	if response.Code != 200 {
		t.Fatalf("status = %d: %s", response.Code, response.Body.String())
	}
	if err := json.Unmarshal(response.Body.Bytes(), &body); err != nil {
		t.Fatal(err)
	}
	groups := map[string]int{}
	for _, group := range body.Groups {
		groups[group.Name] = len(group.Files)
	}
	if groups["Pack"] != 1 || groups["Configs"] != 2 {
		t.Fatalf("unexpected groups: %#v", groups)
	}
}

func TestFileReadWriteRoundtrip(t *testing.T) {
	server := registryTestServer(t)
	save := `{"path":"config/new.json","content":"{\"a\": 1}"}`
	if response := requestJSON(t, server, http.MethodPut, Prefix+"/packs/example/subdirs/1.21-mr/file", save); response.Code != 200 {
		t.Fatalf("save status = %d: %s", response.Code, response.Body.String())
	}
	response := requestJSON(t, server, http.MethodGet, Prefix+"/packs/example/subdirs/1.21-mr/file?path=config%2Fnew.json", "")
	if response.Code != 200 {
		t.Fatalf("read status = %d: %s", response.Code, response.Body.String())
	}
	var body struct {
		Content string `json:"content"`
	}
	if err := json.Unmarshal(response.Body.Bytes(), &body); err != nil {
		t.Fatal(err)
	}
	if body.Content != `{"a": 1}` {
		t.Fatalf("content = %q", body.Content)
	}
}

func TestFileEndpointsRejectTraversal(t *testing.T) {
	server := registryTestServer(t)
	if response := requestJSON(t, server, http.MethodGet, Prefix+"/packs/example/subdirs/1.21-mr/file?path=..%2F..%2Fmanifest.json", ""); response.Code != 400 {
		t.Fatalf("read traversal: status = %d", response.Code)
	}
	save := `{"path":"../../escape.json","content":"{}"}`
	if response := requestJSON(t, server, http.MethodPut, Prefix+"/packs/example/subdirs/1.21-mr/file", save); response.Code != 400 {
		t.Fatalf("write traversal: status = %d", response.Code)
	}
}

func TestCreateFileNormalizesContentPathsAndConflicts(t *testing.T) {
	server := registryTestServer(t)
	body := `{"path":"global_packs/required_data/Pack/data/NS/Functions/Foo.json","content":"{}"}`
	response := requestJSON(t, server, http.MethodPost, Prefix+"/packs/example/subdirs/1.21-mr/files", body)
	if response.Code != 201 {
		t.Fatalf("create status = %d: %s", response.Code, response.Body.String())
	}
	var created struct {
		Path string `json:"path"`
	}
	if err := json.Unmarshal(response.Body.Bytes(), &created); err != nil {
		t.Fatal(err)
	}
	if created.Path != "global_packs/required_data/Pack/data/ns/functions/foo.json" {
		t.Fatalf("path was not normalized: %q", created.Path)
	}
	if response := requestJSON(t, server, http.MethodPost, Prefix+"/packs/example/subdirs/1.21-mr/files", body); response.Code != 409 {
		t.Fatalf("expected conflict, got %d", response.Code)
	}
}

func TestCreateFileCopiesFromSiblingSubdir(t *testing.T) {
	server := registryTestServer(t)
	sibling := filepath.Join(server.root, "modpacks", "example", "1.21-cf", "config")
	if err := os.MkdirAll(sibling, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(sibling, "shared.toml"), []byte("a = 1\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	body := `{"path":"","from_sub":"1.21-cf","from_path":"config/shared.toml"}`
	response := requestJSON(t, server, http.MethodPost, Prefix+"/packs/example/subdirs/1.21-mr/files", body)
	if response.Code != 201 {
		t.Fatalf("copy status = %d: %s", response.Code, response.Body.String())
	}
	copied, err := os.ReadFile(filepath.Join(server.root, "modpacks", "example", "1.21-mr", "config", "shared.toml"))
	if err != nil || string(copied) != "a = 1\n" {
		t.Fatalf("copied content = %q, err = %v", copied, err)
	}
}

func TestCheckEndpointFlagsInvalidBuffers(t *testing.T) {
	server := registryTestServer(t)
	body := `{"file":"config/x.json","content":"{\"a\": }"}`
	response := requestJSON(t, server, http.MethodPost, Prefix+"/packs/example/subdirs/1.21-mr/check", body)
	if response.Code != 200 {
		t.Fatalf("status = %d: %s", response.Code, response.Body.String())
	}
	var result struct {
		Valid       bool `json:"valid"`
		Diagnostics []struct {
			Severity string `json:"severity"`
			Code     string `json:"code"`
		} `json:"diagnostics"`
	}
	if err := json.Unmarshal(response.Body.Bytes(), &result); err != nil {
		t.Fatal(err)
	}
	if result.Valid || len(result.Diagnostics) != 1 || result.Diagnostics[0].Code != "syntax" {
		t.Fatalf("unexpected check result: %#v", result)
	}
}

func TestKubeJSRuntimeLogEndpointReturnsLocation(t *testing.T) {
	server := registryTestServer(t)
	body := `{"content":"[ERROR] kubejs/server_scripts/recipes.js:17:4 boom"}`
	response := requestJSON(t, server, http.MethodPost, Prefix+"/packs/example/subdirs/1.21-mr/kubejs/runtime-log", body)
	if response.Code != 200 || !strings.Contains(response.Body.String(), `"line": 17`) {
		t.Fatalf("status=%d body=%s", response.Code, response.Body.String())
	}
}

func TestPreflightActionRunsPreflightJSON(t *testing.T) {
	server := registryTestServer(t)
	action, ok := actionByName("packs.preflight")
	if !ok {
		t.Fatal("packs.preflight is not registered")
	}
	request := newActionRequest(http.MethodPost, "/x", "example", "", "")
	request.SetPathValue("sub", "1.21-mr")
	dir, args, err := action.Build(server, request)
	if err != nil {
		t.Fatal(err)
	}
	subdir := filepath.Join(server.root, "modpacks", "example", "1.21-mr")
	if dir != subdir || len(args) != 3 || args[0] != "preflight" || args[2] != "--json" {
		t.Fatalf("dir=%q args=%#v", dir, args)
	}
}
