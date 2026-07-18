package curseforge

import (
	"io"
	"net/http"
	"strings"
	"testing"
)

type roundTripFunc func(*http.Request) (*http.Response, error)

func (f roundTripFunc) RoundTrip(req *http.Request) (*http.Response, error) {
	return f(req)
}

func TestCurseForgeAPIExplainsForbiddenKey(t *testing.T) {
	const apiKey = "test-api-key"
	useTestAPIKey(t, apiKey)

	client := cfApiClient{httpClient: &http.Client{Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
		if got := req.Header.Get("X-API-Key"); got != apiKey {
			t.Errorf("X-API-Key = %q, want %q", got, apiKey)
		}
		return &http.Response{
			StatusCode: http.StatusForbidden,
			Status:     "403 Forbidden",
			Header:     make(http.Header),
			Body:       io.NopCloser(strings.NewReader(`{"error":"Forbidden"}`)),
			Request:    req,
		}, nil
	})}}

	var target any
	err := client.getJSON("/v1/games", &target)
	if err == nil || !strings.Contains(err.Error(), "CurseForge rejected the configured API key (403 Forbidden)") ||
		!strings.Contains(err.Error(), "set CURSEFORGE_API_KEY") {
		t.Fatalf("unexpected error: %v", err)
	}
}
