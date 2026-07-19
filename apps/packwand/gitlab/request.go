package gitlab

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"github.com/spf13/viper"
)

const DefaultInstance = "gitlab.com"

type glRepo struct {
	Name              string `json:"name"`
	Path              string `json:"path"`
	PathWithNamespace string `json:"path_with_namespace"`
}

type glRelease struct {
	TagName string   `json:"tag_name"`
	Assets  glAssets `json:"assets"`
}

type glAssets struct {
	Links []glLink `json:"links"`
}

type glLink struct {
	Name     string `json:"name"`
	URL      string `json:"url"`
	LinkType string `json:"link_type"`
}

type glApiClient struct {
	httpClient *http.Client
	instance   string
	token      string
}

func newClient(instance string) *glApiClient {
	if instance == "" {
		instance = DefaultInstance
	}
	token := viper.GetString("gitlab." + instance + ".token")
	if token == "" {
		token = viper.GetString("gitlab.token")
	}
	return &glApiClient{
		httpClient: core.NewClient(),
		instance:   instance,
		token:      token,
	}
}

func (c *glApiClient) makeGet(endpoint string) (*http.Response, error) {
	req, err := http.NewRequest("GET", "https://"+c.instance+"/api/v4/"+endpoint, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("User-Agent", core.UserAgent)
	if c.token != "" {
		req.Header.Set("PRIVATE-TOKEN", c.token)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}

	switch resp.StatusCode {
	case 200:
		return resp, nil
	case 401:
		resp.Body.Close()
		return nil, fmt.Errorf("GitLab API unauthorized; set a token via gitlab.token or gitlab.%s.token", c.instance)
	case 404:
		resp.Body.Close()
		return nil, fmt.Errorf("project not found on %s", c.instance)
	default:
		resp.Body.Close()
		return nil, fmt.Errorf("GitLab API error: %s", resp.Status)
	}
}

func (c *glApiClient) getProject(slug string) (*glRepo, error) {
	resp, err := c.makeGet("projects/" + url.PathEscape(slug))
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	var repo glRepo
	if err := json.NewDecoder(resp.Body).Decode(&repo); err != nil {
		return nil, err
	}
	return &repo, nil
}

func (c *glApiClient) listReleases(slug string) ([]glRelease, error) {
	resp, err := c.makeGet("projects/" + url.PathEscape(slug) + "/releases?order_by=released_at&sort=desc&per_page=20")
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	var releases []glRelease
	if err := json.NewDecoder(resp.Body).Decode(&releases); err != nil {
		return nil, err
	}
	return releases, nil
}
