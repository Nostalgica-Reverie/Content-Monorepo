package gui

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestRoutesServeStaticFrontendAndVersionedAPI(t *testing.T) {
	handler := (&server{root: t.TempDir()}).routes()

	frontend := httptest.NewRecorder()
	handler.ServeHTTP(frontend, httptest.NewRequest(http.MethodGet, "/", nil))
	if frontend.Code != http.StatusOK || !strings.Contains(frontend.Body.String(), "Packwand") {
		t.Fatalf("frontend response: status=%d body=%q", frontend.Code, frontend.Body.String())
	}

	version := httptest.NewRecorder()
	handler.ServeHTTP(version, httptest.NewRequest(http.MethodGet, "/api/v1/version", nil))
	if version.Code != http.StatusOK {
		t.Fatalf("version endpoint status = %d, want %d", version.Code, http.StatusOK)
	}

	legacy := httptest.NewRecorder()
	handler.ServeHTTP(legacy, httptest.NewRequest(http.MethodGet, "/api/health", nil))
	if legacy.Code != http.StatusNotFound {
		t.Fatalf("removed legacy endpoint status = %d, want %d", legacy.Code, http.StatusNotFound)
	}
}
