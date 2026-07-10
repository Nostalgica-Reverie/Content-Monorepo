package content

import (
	"crypto/sha256"
	"encoding/hex"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
)

func TestDownloadInstallerJarVerifiesAndCaches(t *testing.T) {
	payload := []byte("fake installer jar")
	digest := sha256.Sum256(payload)
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(payload)
	}))
	defer server.Close()

	destination := filepath.Join(t.TempDir(), "installer", "packwiz-installer-bootstrap.jar")
	if err := downloadInstallerJar(destination, server.URL, hex.EncodeToString(digest[:])); err != nil {
		t.Fatal(err)
	}
	data, err := os.ReadFile(destination)
	if err != nil {
		t.Fatal(err)
	}
	if string(data) != string(payload) {
		t.Fatalf("cached %q", data)
	}
}

func TestDownloadInstallerJarRejectsBadHash(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte("not expected"))
	}))
	defer server.Close()
	destination := filepath.Join(t.TempDir(), "installer.jar")
	if err := downloadInstallerJar(destination, server.URL, "deadbeef"); err == nil {
		t.Fatal("expected hash mismatch")
	}
	if _, err := os.Stat(destination); !os.IsNotExist(err) {
		t.Fatalf("invalid jar should not be cached: %v", err)
	}
}
