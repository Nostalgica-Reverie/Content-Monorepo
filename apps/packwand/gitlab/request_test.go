package gitlab

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

func stubClient(t *testing.T, status int, body string, wantToken string) *glApiClient {
	t.Helper()
	return &glApiClient{
		instance: "gitlab.example.com",
		token:    wantToken,
		httpClient: &http.Client{Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
			if wantToken != "" {
				if got := req.Header.Get("PRIVATE-TOKEN"); got != wantToken {
					t.Errorf("PRIVATE-TOKEN = %q, want %q", got, wantToken)
				}
			}
			return &http.Response{
				StatusCode: status,
				Status:     http.StatusText(status),
				Header:     make(http.Header),
				Body:       io.NopCloser(strings.NewReader(body)),
				Request:    req,
			}, nil
		})},
	}
}

func TestGitLabRegexes(t *testing.T) {
	if m := GitLabRegex.FindStringSubmatch("https://gitlab.com/owner/repo"); len(m) != 2 || m[1] != "owner/repo" {
		t.Errorf("GitLabRegex = %v", m)
	}
	if m := GenericGitLabRegex.FindStringSubmatch("https://git.example.net/owner/repo"); len(m) != 3 || m[1] != "git.example.net" || m[2] != "owner/repo" {
		t.Errorf("GenericGitLabRegex = %v", m)
	}
}

func TestGetProjectDecodesAndSendsToken(t *testing.T) {
	client := stubClient(t, http.StatusOK, `{"name":"Repo","path":"repo","path_with_namespace":"owner/repo"}`, "secret-token")
	repo, err := client.getProject("owner/repo")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if repo.Name != "Repo" || repo.PathWithNamespace != "owner/repo" {
		t.Errorf("unexpected repo: %+v", repo)
	}
}

func TestMakeGetErrorStatuses(t *testing.T) {
	for status, wantSubstr := range map[int]string{
		http.StatusUnauthorized: "unauthorized",
		http.StatusNotFound:     "not found",
		http.StatusBadGateway:   "GitLab API error",
	} {
		client := stubClient(t, status, "", "")
		_, err := client.makeGet("projects/owner%2Frepo")
		if err == nil || !strings.Contains(err.Error(), wantSubstr) {
			t.Errorf("status %d: expected error containing %q, got %v", status, wantSubstr, err)
		}
	}
}

func TestListReleasesDecode(t *testing.T) {
	body := `[{"tag_name":"v1.2.3","assets":{"links":[{"name":"mod-1.2.3.jar","url":"https://gitlab.example.com/x/mod-1.2.3.jar","link_type":"package"}]}}]`
	client := stubClient(t, http.StatusOK, body, "")
	releases, err := client.listReleases("owner/repo")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(releases) != 1 || releases[0].TagName != "v1.2.3" || len(releases[0].Assets.Links) != 1 {
		t.Errorf("unexpected releases: %+v", releases)
	}
}
