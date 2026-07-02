package github

import (
	"fmt"
	"net/http"
	"strconv"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
	"github.com/spf13/viper"
)

const ghApiServer = "api.github.com"

type ghApiClient struct {
	httpClient *http.Client
}

var ghDefaultClient = ghApiClient{&http.Client{}}

func (c *ghApiClient) makeGet(url string) (*http.Response, error) {
	ghApiToken := viper.GetString("github.token")

	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("User-Agent", core.UserAgent)
	req.Header.Set("Accept", "application/vnd.github+json")
	if ghApiToken != "" {
		req.Header.Set("Authorization", "Bearer "+ghApiToken)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}

	ratelimit := 999
	if header := resp.Header.Get("x-ratelimit-remaining"); header != "" {
		ratelimit, err = strconv.Atoi(header)
		if err != nil {
			_ = resp.Body.Close()
			return nil, err
		}
	}

	if resp.StatusCode == http.StatusForbidden && ratelimit == 0 {
		_ = resp.Body.Close()
		return nil, fmt.Errorf("GitHub API ratelimit exceeded; time of reset: %v", resp.Header.Get("x-ratelimit-reset"))
	}
	if resp.StatusCode != http.StatusOK {
		_ = resp.Body.Close()
		return nil, fmt.Errorf("invalid response status: %v", resp.Status)
	}

	if ratelimit < 10 {
		fmt.Printf("Warning: GitHub API allows %v more requests before ratelimiting\n", ratelimit)
		fmt.Println("Specifying a token is recommended; see documentation")
	}

	return resp, nil
}

func (c *ghApiClient) getRepo(slug string) (*http.Response, error) {
	resp, err := c.makeGet("https://" + ghApiServer + "/repos/" + slug)
	if err != nil {
		return resp, err
	}

	return resp, nil
}

func (c *ghApiClient) getReleases(slug string) (*http.Response, error) {
	resp, err := c.getRepo(slug + "/releases")
	if err != nil {
		return resp, err
	}

	return resp, nil
}
