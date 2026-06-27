package forgejo

import (
	"fmt"
	"net/http"
	"strconv"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/viper"
)

const DefaultInstance = "codeberg.org"

type apiClient struct {
	httpClient *http.Client
	instance   string
}

func newClient(instance string) *apiClient {
	if instance == "" {
		instance = DefaultInstance
	}
	return &apiClient{httpClient: &http.Client{}, instance: instance}
}

func (c *apiClient) baseURL() string {
	return "https://" + c.instance + "/api/v1"
}

func (c *apiClient) tokenKey() string {
	return "forgejo." + c.instance + ".token"
}

func (c *apiClient) makeGet(url string) (*http.Response, error) {
	token := viper.GetString(c.tokenKey())
	if token == "" {
		token = viper.GetString("forgejo.token")
	}

	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("User-Agent", core.UserAgent)
	req.Header.Set("Accept", "application/json")
	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}

	ratelimit := 999
	if rl := resp.Header.Get("x-ratelimit-remaining"); rl != "" {
		if n, err := strconv.Atoi(rl); err == nil {
			ratelimit = n
		}
	}

	if resp.StatusCode == 403 && ratelimit == 0 {
		return nil, fmt.Errorf("Forgejo API rate limit exceeded on %s", c.instance)
	}
	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("invalid response from %s: %v", c.instance, resp.Status)
	}

	if ratelimit < 10 && ratelimit != 999 {
		fmt.Printf("Warning: Forgejo API on %s allows %d more requests before rate-limiting\n", c.instance, ratelimit)
	}

	return resp, nil
}

func (c *apiClient) getRepo(slug string) (*http.Response, error) {
	return c.makeGet(c.baseURL() + "/repos/" + slug)
}

func (c *apiClient) getReleases(slug string) (*http.Response, error) {
	return c.makeGet(c.baseURL() + "/repos/" + slug + "/releases")
}
