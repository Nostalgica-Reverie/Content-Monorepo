package curseforge

import (
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
)

func useTestAPIKey(t *testing.T, key string) {
	t.Helper()
	for _, name := range cfAPIKeyEnvironmentVariables {
		t.Setenv(name, "")
	}
	t.Setenv("CURSEFORGE_API_KEY", key)
}

func TestCurseForgeDownloadAuthenticatesCDNRequest(t *testing.T) {
	const apiKey = "test-curseforge-api-key"
	useTestAPIKey(t, apiKey)

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		if got := req.Header.Get("X-API-Key"); got != apiKey {
			t.Errorf("X-API-Key = %q, want %q", got, apiKey)
		}
		if got := req.Header.Get("User-Agent"); got != core.UserAgent {
			t.Errorf("User-Agent = %q, want %q", got, core.UserAgent)
		}
		if got := req.Header.Get("Accept"); got != "application/octet-stream" {
			t.Errorf("Accept = %q, want application/octet-stream", got)
		}
		_, _ = w.Write([]byte("mod contents"))
	}))
	defer server.Close()

	body, err := (&cfDownloadMetadata{url: server.URL}).DownloadFile()
	if err != nil {
		t.Fatal(err)
	}
	defer body.Close()

	contents, err := io.ReadAll(body)
	if err != nil {
		t.Fatal(err)
	}
	if got := string(contents); got != "mod contents" {
		t.Fatalf("downloaded %q, want %q", got, "mod contents")
	}
}

func TestCurseForgeDownloadExplainsRejectedKey(t *testing.T) {
	for _, status := range []int{http.StatusUnauthorized, http.StatusForbidden} {
		t.Run(http.StatusText(status), func(t *testing.T) {
			useTestAPIKey(t, "rejected-key")

			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
				http.Error(w, `{"error":"authentication failed"}`, status)
			}))
			defer server.Close()

			body, err := (&cfDownloadMetadata{url: server.URL}).DownloadFile()
			if body != nil {
				_ = body.Close()
				t.Fatal("expected no response body for a rejected download")
			}
			if err == nil || !strings.Contains(err.Error(), "CurseForge rejected the configured API key") ||
				!strings.Contains(err.Error(), "set CURSEFORGE_API_KEY") {
				t.Fatalf("unexpected error: %v", err)
			}
		})
	}
}

func TestCurseForgeDownloadUsesEmbeddedKeyByDefault(t *testing.T) {
	useTestAPIKey(t, "")

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		if got := req.Header.Get("X-API-Key"); got != cfAPIKeyDefault {
			t.Errorf("X-API-Key = %q, want embedded default", got)
		}
		_, _ = w.Write([]byte("mod contents"))
	}))
	defer server.Close()

	body, err := (&cfDownloadMetadata{url: server.URL}).DownloadFile()
	if err != nil {
		t.Fatal(err)
	}
	defer body.Close()
}
