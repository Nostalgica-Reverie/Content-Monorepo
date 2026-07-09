package curseforge

import (
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
)

func TestDownloadCurseForgeImport(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("PK-test-archive"))
	}))
	defer server.Close()

	path, err := downloadCurseForgeImport(server.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(path)
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if string(data) != "PK-test-archive" {
		t.Fatalf("downloaded %q", data)
	}
}

func TestDownloadCurseForgeImportRejectsHTTPError(t *testing.T) {
	server := httptest.NewServer(http.NotFoundHandler())
	defer server.Close()
	if _, err := downloadCurseForgeImport(server.URL); err == nil {
		t.Fatal("expected a non-200 response to fail")
	}
}

func TestNormalizeImportPathMatchesDespiteCaseAndSeparators(t *testing.T) {
	// The CurseForge API's FileName and the zip's override entry often
	// disagree on case; both must normalize to the same key so the
	// override jar is skipped instead of copied next to its metafile.
	cases := [][2]string{
		{"mods/Sodium-1.2.jar", "mods/sodium-1.2.JAR"},
		{"mods/foo.jar", "mods/foo.jar.disabled"},
	}
	for _, c := range cases {
		if normalizeImportPath(c[0]) != normalizeImportPath(c[1]) {
			t.Errorf("expected %q and %q to normalize equally (%q vs %q)",
				c[0], c[1], normalizeImportPath(c[0]), normalizeImportPath(c[1]))
		}
	}
	if normalizeImportPath("mods/a.jar") == normalizeImportPath("mods/b.jar") {
		t.Error("distinct files must not normalize equally")
	}
}
