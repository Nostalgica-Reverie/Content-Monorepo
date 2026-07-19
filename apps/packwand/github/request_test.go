package github

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

func stubClient(status int, body string, header http.Header) *ghApiClient {
	if header == nil {
		header = make(http.Header)
	}
	return &ghApiClient{httpClient: &http.Client{Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
		return &http.Response{
			StatusCode: status,
			Status:     http.StatusText(status),
			Header:     header,
			Body:       io.NopCloser(strings.NewReader(body)),
			Request:    req,
		}, nil
	})}}
}

func TestGithubRegex(t *testing.T) {
	for input, want := range map[string]string{
		"https://github.com/CaffeineMC/sodium":          "CaffeineMC/sodium",
		"https://www.github.com/CaffeineMC/sodium":      "CaffeineMC/sodium",
		"http://github.com/owner/repo/releases/tag/1.0": "owner/repo",
		"https://gitlab.com/owner/repo":                 "",
	} {
		matches := GithubRegex.FindStringSubmatch(input)
		got := ""
		if len(matches) == 2 {
			got = matches[1]
		}
		if got != want {
			t.Errorf("GithubRegex(%q) = %q, want %q", input, got, want)
		}
	}
}

func TestMakeGetRatelimitExceeded(t *testing.T) {
	header := make(http.Header)
	header.Set("x-ratelimit-remaining", "0")
	header.Set("x-ratelimit-reset", "1234567890")
	client := stubClient(http.StatusForbidden, "", header)

	_, err := client.makeGet("https://api.github.com/repos/owner/repo")
	if err == nil || !strings.Contains(err.Error(), "ratelimit exceeded") {
		t.Fatalf("expected ratelimit error, got %v", err)
	}
}

func TestMakeGetNonOKStatus(t *testing.T) {
	client := stubClient(http.StatusNotFound, "", nil)
	_, err := client.makeGet("https://api.github.com/repos/owner/missing")
	if err == nil || !strings.Contains(err.Error(), "invalid response status") {
		t.Fatalf("expected status error, got %v", err)
	}
}

func TestMakeGetSuccessClosesNothing(t *testing.T) {
	client := stubClient(http.StatusOK, `{"full_name":"owner/repo"}`, nil)
	resp, err := client.makeGet("https://api.github.com/repos/owner/repo")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	if !strings.Contains(string(body), "owner/repo") {
		t.Errorf("unexpected body: %s", body)
	}
}
