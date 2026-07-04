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

func newActionRequest(method, url, id, slug, body string) *http.Request {
	r := httptest.NewRequest(method, url, strings.NewReader(body))
	if id != "" {
		r.SetPathValue("id", id)
	}
	if slug != "" {
		r.SetPathValue("slug", slug)
	}
	return r
}

// exampleServer returns a server with a single-subdir pack "example" at
// modpacks/example/1.21-mr, matching testServer's fixture.
func exampleServer(t *testing.T) (*Server, string) {
	t.Helper()
	server := testServer(t, "")
	return server, filepath.Join(server.root, "modpacks", "example", "1.21-mr")
}

func TestGroup1ActionsAreRegistered(t *testing.T) {
	for _, name := range []string{
		"packs.run",
		"packs.migrate.format", "packs.migrate.loader", "packs.migrate.minecraft",
		"packs.docs.modlist", "packs.docs.pages", "docs.diff",
		"packs.bump", "packs.freeze.list", "packs.freeze.apply", "packs.unfreeze",
		"packs.mods.side.get", "packs.mods.side.set",
	} {
		if _, ok := actionByName(name); !ok {
			t.Errorf("action %q is not registered", name)
		}
	}
}

func TestPacksRunBuildsScriptArgs(t *testing.T) {
	server, subdir := exampleServer(t)
	action, _ := actionByName("packs.run")
	request := newActionRequest(http.MethodPost, "/x", "example", "", "")
	request.SetPathValue("script", "generate")
	dir, args, err := action.Build(server, request)
	if err != nil {
		t.Fatal(err)
	}
	if dir != subdir || len(args) != 2 || args[0] != "run" || args[1] != "generate" {
		t.Fatalf("dir=%q args=%#v", dir, args)
	}
}

func TestPacksMigrateLoaderRequiresTarget(t *testing.T) {
	server, subdir := exampleServer(t)
	action, _ := actionByName("packs.migrate.loader")
	if _, _, err := action.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{}`)); err == nil {
		t.Fatal("expected an error when target is missing")
	}
	dir, args, err := action.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{"target":"latest"}`))
	if err != nil {
		t.Fatal(err)
	}
	if dir != subdir || args[0] != "migrate" || args[1] != "loader" || args[2] != "latest" {
		t.Fatalf("dir=%q args=%#v", dir, args)
	}
}

func TestPacksMigrateMinecraftRequiresVersion(t *testing.T) {
	server, _ := exampleServer(t)
	action, _ := actionByName("packs.migrate.minecraft")
	if _, _, err := action.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{}`)); err == nil {
		t.Fatal("expected an error when version is missing")
	}
	_, args, err := action.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{"version":"1.21.2"}`))
	if err != nil {
		t.Fatal(err)
	}
	if args[0] != "migrate" || args[1] != "minecraft" || args[2] != "1.21.2" {
		t.Fatalf("args=%#v", args)
	}
}

func TestPacksDocsModlistAndPagesUseJSON(t *testing.T) {
	server, subdir := exampleServer(t)
	packDir := filepath.Join(server.root, "modpacks", "example")

	modlist, _ := actionByName("packs.docs.modlist")
	dir, args, err := modlist.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", ""))
	if err != nil || dir != server.root || len(args) != 3 || args[0] != "modlist" || args[1] != subdir || args[2] != "--json" {
		t.Fatalf("dir=%q args=%#v err=%v", dir, args, err)
	}

	pages, _ := actionByName("packs.docs.pages")
	dir, args, err = pages.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", ""))
	if err != nil || dir != server.root || len(args) != 4 || args[0] != "pages" || args[2] != packDir || args[3] != "--json" {
		t.Fatalf("dir=%q args=%#v err=%v", dir, args, err)
	}
}

func TestDocsDiffRequiresOldAndNewRefs(t *testing.T) {
	server, _ := exampleServer(t)
	action, _ := actionByName("docs.diff")
	if _, _, err := action.Build(server, newActionRequest(http.MethodGet, "/x", "", "", "")); err == nil {
		t.Fatal("expected an error when old/new are missing")
	}
	dir, args, err := action.Build(server, newActionRequest(http.MethodGet, "/x?old=HEAD~1&new=HEAD&prefix=modpacks", "", "", ""))
	if err != nil {
		t.Fatal(err)
	}
	if dir != server.root || len(args) != 5 || args[1] != "HEAD~1" || args[2] != "HEAD" || args[4] != "modpacks" {
		t.Fatalf("dir=%q args=%#v", dir, args)
	}
}

func TestPacksBumpRequiresVersion(t *testing.T) {
	server, _ := exampleServer(t)
	packDir := filepath.Join(server.root, "modpacks", "example")
	action, _ := actionByName("packs.bump")
	if _, _, err := action.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{}`)); err == nil {
		t.Fatal("expected an error when version is missing")
	}
	dir, args, err := action.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{"version":"2.0.0","configs":true}`))
	if err != nil {
		t.Fatal(err)
	}
	if dir != server.root || args[0] != "bump" || args[1] != packDir || args[2] != "2.0.0" || args[3] != "--configs" {
		t.Fatalf("dir=%q args=%#v", dir, args)
	}
}

func TestPacksFreezeAndUnfreezeRequireSlugs(t *testing.T) {
	server, subdir := exampleServer(t)

	list, _ := actionByName("packs.freeze.list")
	dir, args, err := list.Build(server, newActionRequest(http.MethodGet, "/x", "example", "", ""))
	if err != nil || dir != server.root || len(args) != 3 || args[1] != subdir || args[2] != "--json" {
		t.Fatalf("dir=%q args=%#v err=%v", dir, args, err)
	}

	apply, _ := actionByName("packs.freeze.apply")
	if _, _, err := apply.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{"slugs":[]}`)); err == nil {
		t.Fatal("expected an error when slugs is empty")
	}
	_, args, err = apply.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{"slugs":["sodium"]}`))
	if err != nil {
		t.Fatal(err)
	}
	if args[0] != "freeze" || args[1] != subdir || args[2] != "sodium" {
		t.Fatalf("args=%#v", args)
	}

	unfreeze, _ := actionByName("packs.unfreeze")
	if _, _, err := unfreeze.Build(server, newActionRequest(http.MethodPost, "/x", "example", "", `{}`)); err == nil {
		t.Fatal("expected an error when slugs is missing")
	}
}

func TestPacksModsSideGetAndSet(t *testing.T) {
	server, _ := exampleServer(t)
	packDir := filepath.Join(server.root, "modpacks", "example")

	get, _ := actionByName("packs.mods.side.get")
	dir, args, err := get.Build(server, newActionRequest(http.MethodGet, "/x", "example", "sodium", ""))
	if err != nil || dir != server.root || args[0] != "side" || args[1] != packDir || args[2] != "sodium" {
		t.Fatalf("dir=%q args=%#v err=%v", dir, args, err)
	}

	set, _ := actionByName("packs.mods.side.set")
	if _, _, err := set.Build(server, newActionRequest(http.MethodPut, "/x", "example", "sodium", `{"side":"bogus"}`)); err == nil {
		t.Fatal("expected an error for an invalid side value")
	}
	_, args, err = set.Build(server, newActionRequest(http.MethodPut, "/x", "example", "sodium", `{"side":"client"}`))
	if err != nil {
		t.Fatal(err)
	}
	if args[0] != "side" || args[2] != "sodium" || args[3] != "client" {
		t.Fatalf("args=%#v", args)
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
