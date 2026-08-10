package xrpcapi

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"sort"
	"strings"
	"sync"
	"time"
)

const (
	contactCollection = "net.nostalgica.packwand.contact"
	inviteCollection  = "net.nostalgica.packwand.session.invite"
	maxPublicBody     = 4 << 20
	maxDiscoveryPages = 10
	maxInvitePeers    = 64
	inviteWorkers     = 8
)

// UploadBlob stores bytes in the signed-in repository's blob store.
func (service *Service) UploadBlob(ctx context.Context, mimeType string, data []byte) (BlobRef, error) {
	if !strings.HasPrefix(mimeType, "image/") {
		return BlobRef{}, fmt.Errorf("Packwand only uploads image blobs")
	}
	client, _, err := service.client(ctx)
	if err != nil {
		return BlobRef{}, err
	}
	var output struct {
		Blob BlobRef `json:"blob"`
	}
	if err := client.LexDo(ctx, http.MethodPost, mimeType, "com.atproto.repo.uploadBlob", nil, bytes.NewReader(data), &output); err != nil {
		return BlobRef{}, fmt.Errorf("upload blob: %w", err)
	}
	return output.Blob, nil
}

// ListFriends intersects Bluesky follows and followers, then unions explicit contacts.
func (service *Service) ListFriends(ctx context.Context) ([]Friend, error) {
	current, ok := service.CurrentIdentity()
	if !ok {
		return nil, fmt.Errorf("not signed in")
	}
	follows, err := service.listActors(ctx, "app.bsky.graph.getFollows", "follows", current.DID)
	followsErr := err
	followers, err := service.listActors(ctx, "app.bsky.graph.getFollowers", "followers", current.DID)
	followersErr := err

	byDID := make(map[string]Friend)
	for did, actor := range follows {
		if _, mutual := followers[did]; !mutual {
			continue
		}
		actor.Sources = []string{"mutual_follow"}
		byDID[did] = actor
	}

	cursor := ""
	for pageNumber := 0; pageNumber < maxDiscoveryPages; pageNumber++ {
		page, pageErr := service.ListRecords(ctx, current.DID, contactCollection, 100, cursor)
		if pageErr != nil {
			return nil, fmt.Errorf("list Packwand contacts: %w", pageErr)
		}
		for _, record := range page.Records {
			var value struct {
				DID string `json:"did"`
			}
			if json.Unmarshal(record.Value, &value) != nil || value.DID == "" || value.DID == current.DID {
				continue
			}
			friend := byDID[value.DID]
			friend.DID = value.DID
			friend.Sources = appendUnique(friend.Sources, "contact")
			if friend.Handle == "" {
				if identity, resolveErr := service.Resolve(ctx, value.DID); resolveErr == nil {
					friend.Handle = identity.Handle
					friend.PDS = identity.PDS
				}
			}
			byDID[value.DID] = friend
		}
		cursor = page.Cursor
		if cursor == "" {
			break
		}
	}

	friends := make([]Friend, 0, len(byDID))
	for _, friend := range byDID {
		friends = append(friends, friend)
	}
	if len(friends) == 0 && (followsErr != nil || followersErr != nil) {
		return nil, fmt.Errorf("discover Bluesky follows: follows=%v; followers=%v", followsErr, followersErr)
	}
	sort.Slice(friends, func(left, right int) bool {
		leftName := strings.ToLower(friends[left].Handle)
		rightName := strings.ToLower(friends[right].Handle)
		if leftName == rightName {
			return friends[left].DID < friends[right].DID
		}
		return leftName < rightName
	})
	return friends, nil
}

// ListPendingInvites discovers addressed, unexpired invitations in friends' repositories.
func (service *Service) ListPendingInvites(ctx context.Context) ([]PendingInvite, error) {
	current, ok := service.CurrentIdentity()
	if !ok {
		return nil, fmt.Errorf("not signed in")
	}
	var friends []Friend
	var err error
	if service.friends != nil {
		friends, err = service.friends(ctx)
	} else {
		friends, err = service.ListFriends(ctx)
	}
	if err != nil {
		return nil, err
	}
	if len(friends) > maxInvitePeers {
		friends = friends[:maxInvitePeers]
	}
	now := time.Now()
	invites := make([]PendingInvite, 0)
	semaphore := make(chan struct{}, inviteWorkers)
	results := make(chan []PendingInvite, len(friends))
	var workers sync.WaitGroup
	for _, friend := range friends {
		workers.Add(1)
		go func(friend Friend) {
			defer workers.Done()
			select {
			case semaphore <- struct{}{}:
				defer func() { <-semaphore }()
			case <-ctx.Done():
				return
			}
			peerContext, cancel := context.WithTimeout(ctx, 5*time.Second)
			defer cancel()
			if found := service.pendingInvitesFromFriend(peerContext, current.DID, friend, now); len(found) > 0 {
				results <- found
			}
		}(friend)
	}
	workers.Wait()
	close(results)
	for found := range results {
		invites = append(invites, found...)
	}
	sort.Slice(invites, func(left, right int) bool { return invites[left].CreatedAt > invites[right].CreatedAt })
	return invites, nil
}

func (service *Service) pendingInvitesFromFriend(ctx context.Context, currentDID string, friend Friend, now time.Time) []PendingInvite {
	if friend.PDS == "" {
		if identity, err := service.Resolve(ctx, friend.DID); err == nil {
			friend.PDS = identity.PDS
			if friend.Handle == "" {
				friend.Handle = identity.Handle
			}
		}
	}
	if friend.PDS == "" {
		return nil
	}
	page, err := service.listPublicRecords(ctx, friend.PDS, friend.DID, inviteCollection)
	if err != nil {
		return nil
	}
	invites := make([]PendingInvite, 0)
	for _, record := range page {
		var value struct {
			To        string `json:"to"`
			Invite    string `json:"invite"`
			CreatedAt string `json:"createdAt"`
			ExpiresAt string `json:"expiresAt"`
		}
		if json.Unmarshal(record.Value, &value) != nil || value.To != currentDID || !strings.HasPrefix(value.Invite, "pw://") {
			continue
		}
		expiresAt, parseErr := time.Parse(time.RFC3339, value.ExpiresAt)
		if parseErr != nil || !expiresAt.After(now) {
			continue
		}
		invites = append(invites, PendingInvite{
			From: friend.DID, FromHandle: friend.Handle, Invite: value.Invite,
			CreatedAt: value.CreatedAt, ExpiresAt: value.ExpiresAt, URI: record.URI, CID: record.CID,
		})
	}
	return invites
}

// LinkedTangledRepos returns the repositories Bobbin associates with the signed-in DID.
func (service *Service) LinkedTangledRepos(ctx context.Context) ([]TangledRepo, error) {
	current, ok := service.CurrentIdentity()
	if !ok {
		return nil, fmt.Errorf("not signed in")
	}
	endpoint, err := url.Parse(service.bobbin + "/xrpc/sh.tangled.repo.listRepos")
	if err != nil {
		return nil, fmt.Errorf("build Bobbin URL: %w", err)
	}
	query := endpoint.Query()
	query.Set("subject", current.DID)
	query.Set("limit", "100")
	endpoint.RawQuery = query.Encode()
	var output struct {
		Items []TangledRepo `json:"items"`
	}
	if err := service.getPublicJSON(ctx, endpoint.String(), &output); err != nil {
		return nil, fmt.Errorf("list Tangled repositories: %w", err)
	}
	return output.Items, nil
}

func (service *Service) listActors(ctx context.Context, method, field, actor string) (map[string]Friend, error) {
	actors := make(map[string]Friend)
	cursor := ""
	for page := 0; page < maxDiscoveryPages; page++ {
		endpoint, err := url.Parse(service.appView + "/xrpc/" + method)
		if err != nil {
			return nil, fmt.Errorf("build AppView URL: %w", err)
		}
		query := endpoint.Query()
		query.Set("actor", actor)
		query.Set("limit", "100")
		if cursor != "" {
			query.Set("cursor", cursor)
		}
		endpoint.RawQuery = query.Encode()
		var output struct {
			Follows   []actorView `json:"follows"`
			Followers []actorView `json:"followers"`
			Cursor    string      `json:"cursor"`
		}
		if err := service.getPublicJSON(ctx, endpoint.String(), &output); err != nil {
			return nil, fmt.Errorf("%s: %w", method, err)
		}
		pageActors := output.Follows
		if field == "followers" {
			pageActors = output.Followers
		}
		for _, actor := range pageActors {
			actors[actor.DID] = Friend{DID: actor.DID, Handle: actor.Handle, DisplayName: actor.DisplayName, Avatar: actor.Avatar}
		}
		cursor = output.Cursor
		if cursor == "" {
			break
		}
	}
	return actors, nil
}

type actorView struct {
	DID         string `json:"did"`
	Handle      string `json:"handle"`
	DisplayName string `json:"displayName"`
	Avatar      string `json:"avatar"`
}

func (service *Service) listPublicRecords(ctx context.Context, pds, repo, collection string) ([]Record, error) {
	if err := safeServiceURL(pds); err != nil {
		return nil, err
	}
	all := make([]Record, 0)
	cursor := ""
	for page := 0; page < maxDiscoveryPages; page++ {
		endpoint, err := url.Parse(strings.TrimRight(pds, "/") + "/xrpc/com.atproto.repo.listRecords")
		if err != nil {
			return nil, err
		}
		query := endpoint.Query()
		query.Set("repo", repo)
		query.Set("collection", collection)
		query.Set("limit", "100")
		if cursor != "" {
			query.Set("cursor", cursor)
		}
		endpoint.RawQuery = query.Encode()
		var output RecordPage
		if err := service.getPublicJSON(ctx, endpoint.String(), &output); err != nil {
			return nil, err
		}
		all = append(all, output.Records...)
		cursor = output.Cursor
		if cursor == "" {
			break
		}
	}
	return all, nil
}

func (service *Service) getPublicJSON(ctx context.Context, endpoint string, output any) error {
	request, err := http.NewRequestWithContext(ctx, http.MethodGet, endpoint, nil)
	if err != nil {
		return err
	}
	request.Header.Set("Accept", "application/json")
	request.Header.Set("User-Agent", "Packwand/26.2.0")
	response, err := service.http.Do(request)
	if err != nil {
		return err
	}
	defer response.Body.Close()
	if response.StatusCode < 200 || response.StatusCode >= 300 {
		_, _ = io.Copy(io.Discard, io.LimitReader(response.Body, 4096))
		return fmt.Errorf("HTTP %d from %s", response.StatusCode, request.URL.Host)
	}
	decoder := json.NewDecoder(io.LimitReader(response.Body, maxPublicBody))
	if err := decoder.Decode(output); err != nil {
		return fmt.Errorf("decode response from %s: %w", request.URL.Host, err)
	}
	return nil
}

func safeServiceURL(raw string) error {
	parsed, err := url.Parse(raw)
	if err != nil || parsed.Host == "" {
		return fmt.Errorf("invalid PDS URL")
	}
	if parsed.Scheme == "https" {
		return nil
	}
	host := parsed.Hostname()
	if parsed.Scheme == "http" && (host == "localhost" || net.ParseIP(host).IsLoopback()) {
		return nil
	}
	return fmt.Errorf("PDS URL must use HTTPS")
}

func appendUnique(values []string, value string) []string {
	for _, existing := range values {
		if existing == value {
			return values
		}
	}
	return append(values, value)
}
