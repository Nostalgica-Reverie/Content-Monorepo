package registry

import (
	"strings"
	"testing"
)

func TestCheckDocumentReportsSyntaxErrorsWithPosition(t *testing.T) {
	dir := modpackSubdir(t)
	diags := CheckDocument(dir, "config/broken.json", []byte("{\n  \"a\": ,\n}"))
	if len(diags) != 1 || diags[0].Code != "syntax" || diags[0].Severity != "error" {
		t.Fatalf("unexpected diagnostics: %#v", diags)
	}
	if diags[0].Line != 2 {
		t.Fatalf("syntax error should point at line 2, got line %d", diags[0].Line)
	}
}

func TestCheckDocumentValidatesModelReferences(t *testing.T) {
	dir := modpackSubdir(t)
	rel := "global_packs/required_resources/MyRes/assets/myres/models/block/other.json"

	good := `{"parent": "myres:block/stone", "textures": {"all": "myres:block/stone"}}`
	if diags := CheckDocument(dir, rel, []byte(good)); len(diags) != 0 {
		t.Fatalf("expected no diagnostics, got %#v", diags)
	}

	bad := `{"parent": "myres:block/missing", "textures": {"all": "myres:block/gone"}}`
	diags := CheckDocument(dir, rel, []byte(bad))
	if len(diags) != 2 {
		t.Fatalf("expected 2 reference errors, got %#v", diags)
	}
	for _, diag := range diags {
		if diag.Code != "reference" || diag.Severity != "error" {
			t.Fatalf("unexpected diagnostic: %#v", diag)
		}
	}
	// Vanilla and unknown namespaces are skipped.
	external := `{"parent": "minecraft:block/cube_all", "textures": {"all": "somemod:block/thing"}}`
	if diags := CheckDocument(dir, rel, []byte(external)); len(diags) != 0 {
		t.Fatalf("external namespaces should be skipped, got %#v", diags)
	}
}

func TestCheckDocumentValidatesFunctionTags(t *testing.T) {
	dir := modpackSubdir(t)
	rel := "global_packs/required_data/MyData/data/mypack/tags/functions/load.json"

	good := `{"values": ["mypack:foo", "#mypack:tick"]}`
	if diags := CheckDocument(dir, rel, []byte(good)); len(diags) != 0 {
		t.Fatalf("expected no diagnostics, got %#v", diags)
	}
	bad := `{"values": ["mypack:missing"]}`
	diags := CheckDocument(dir, rel, []byte(bad))
	if len(diags) != 1 || diags[0].Code != "reference" || !strings.Contains(diags[0].Message, "mypack:missing") {
		t.Fatalf("unexpected diagnostics: %#v", diags)
	}
	optional := `{"values": [{"id": "mypack:missing", "required": false}]}`
	if diags := CheckDocument(dir, rel, []byte(optional)); len(diags) != 0 {
		t.Fatalf("optional entries should not error, got %#v", diags)
	}
}

func TestCheckDocumentValidatesPackMcmeta(t *testing.T) {
	dir := modpackSubdir(t)
	diags := CheckDocument(dir, "global_packs/required_data/MyData/pack.mcmeta", []byte(`{"pack": {}}`))
	if len(diags) != 2 {
		t.Fatalf("expected pack_format error and description warning, got %#v", diags)
	}
}

func TestCheckDocumentValidatesTOMLAndLang(t *testing.T) {
	dir := modpackSubdir(t)
	if diags := CheckDocument(dir, "config/x.toml", []byte("a = \"unterminated\nb = 1\n")); len(diags) != 1 || diags[0].Code != "syntax" {
		t.Fatalf("unexpected TOML diagnostics: %#v", diags)
	}
	lang := `{"key.one": "ok", "key.two": 5}`
	diags := CheckDocument(dir, "global_packs/required_resources/MyRes/assets/myres/lang/en_us.json", []byte(lang))
	if len(diags) != 1 || diags[0].Code != "structure" || !strings.Contains(diags[0].Message, "key.two") {
		t.Fatalf("unexpected lang diagnostics: %#v", diags)
	}
}

func TestCheckDocumentValidatesKubeJSFolderAndFunctionReferences(t *testing.T) {
	dir := modpackSubdir(t)
	writeFile(t, dir, "kubejs/server_scripts/events.js", "ClientEvents.loggedIn(event => {})")
	writeFile(t, dir, "global_packs/required_data/MyData/data/mypack/functions/entry.mcfunction", "function mypack:missing")

	js := CheckDocument(dir, "kubejs/server_scripts/events.js", []byte("ClientEvents.loggedIn(event => {})"))
	if len(js) == 0 || js[0].Severity != "error" {
		t.Fatalf("expected KubeJS folder diagnostic, got %#v", js)
	}
	functions := CheckDocument(dir, "global_packs/required_data/MyData/data/mypack/functions/entry.mcfunction", []byte("function mypack:missing"))
	if len(functions) == 0 || functions[0].Severity != "error" {
		t.Fatalf("expected missing function diagnostic, got %#v", functions)
	}
}
