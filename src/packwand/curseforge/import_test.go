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
