package curseforge

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
)

const cfApiServer = "api.curseforge.com"

// This is a client identifier distributed with PackWand, matching packwiz's
// CurseForge integration model. Environment variables can override it so the
// key can be rotated without rebuilding PackWand.
const cfAPIKeyDefault = "$2a$10$xOGBgtaSrq1idVZ3lOWfueL5n16U5fyNMZqTExBL3vq1v7zyjvJty"

const cfAPIKeyInstructions = "set CURSEFORGE_API_KEY to override PackWand's embedded client key"

var cfAPIKeyEnvironmentVariables = [...]string{
	"PACKWAND_CURSEFORGE_API_KEY",
	"CURSEFORGE_API_KEY",
	"CF_API_KEY",
}

func getAPIKey() string {
	for _, name := range cfAPIKeyEnvironmentVariables {
		if key := strings.TrimSpace(os.Getenv(name)); key != "" {
			return key
		}
	}
	return cfAPIKeyDefault
}

func rejectedAPIKeyError(status string) error {
	return fmt.Errorf("CurseForge rejected the configured API key (%s); %s", status, cfAPIKeyInstructions)
}

type cfApiClient struct {
	httpClient *http.Client
}

var cfDefaultClient = cfApiClient{core.NewClient()}

// doJSON performs an API request and decodes the JSON response into target,
// always closing the response body.
func (c *cfApiClient) doJSON(method, endpoint string, body io.Reader, target any) error {
	apiKey := getAPIKey()
	req, err := http.NewRequest(method, "https://"+cfApiServer+endpoint, body)
	if err != nil {
		return err
	}

	req.Header.Set("User-Agent", core.UserAgent)
	req.Header.Set("Accept", "application/json")
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}
	req.Header.Set("X-API-Key", apiKey)

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return err
	}
	defer func() { _ = resp.Body.Close() }()

	if resp.StatusCode == http.StatusUnauthorized || resp.StatusCode == http.StatusForbidden {
		return rejectedAPIKeyError(resp.Status)
	}
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("invalid response status: %v", resp.Status)
	}
	if err := json.NewDecoder(resp.Body).Decode(target); err != nil && err != io.EOF {
		return err
	}
	return nil
}

func (c *cfApiClient) getJSON(endpoint string, target any) error {
	return c.doJSON(http.MethodGet, endpoint, nil, target)
}

func (c *cfApiClient) postJSON(endpoint string, requestBody any, target any) error {
	data, err := json.Marshal(requestBody)
	if err != nil {
		return err
	}
	return c.doJSON(http.MethodPost, endpoint, bytes.NewReader(data), target)
}

type fileType uint8

// noinspection GoUnusedConst
const (
	fileTypeRelease fileType = iota + 1
	fileTypeBeta
	fileTypeAlpha
)

type dependencyType uint8

// noinspection GoUnusedConst
const (
	dependencyTypeEmbedded dependencyType = iota + 1
	dependencyTypeOptional
	dependencyTypeRequired
	dependencyTypeTool
	dependencyTypeIncompatible
	dependencyTypeInclude
)

type modloaderType uint8

// noinspection GoUnusedConst
const (
	// modloaderTypeAny should not be passed to the API - it does not work
	modloaderTypeAny modloaderType = iota
	modloaderTypeForge
	modloaderTypeCauldron
	modloaderTypeLiteloader
	modloaderTypeFabric
	modloaderTypeQuilt
	modloaderTypeNeoForge
)

var modloaderNames = [...]string{
	"",
	"Forge",
	"Cauldron",
	"Liteloader",
	"Fabric",
	"Quilt",
	"NeoForge",
}

var modloaderIds = [...]string{
	"",
	"forge",
	"cauldron",
	"liteloader",
	"fabric",
	"quilt",
	"neoforge",
}

type hashAlgo uint8

// noinspection GoUnusedConst
const (
	hashAlgoSHA1 hashAlgo = iota + 1
	hashAlgoMD5
)

// modInfo is a subset of the deserialised JSON response from the Curse API for mods (addons)
type modInfo struct {
	Name                   string        `json:"name"`
	Summary                string        `json:"summary"`
	Slug                   string        `json:"slug"`
	ID                     uint32        `json:"id"`
	GameID                 uint32        `json:"gameId"`
	PrimaryCategoryID      uint32        `json:"primaryCategoryId"`
	ClassID                uint32        `json:"classId"`
	LatestFiles            []modFileInfo `json:"latestFiles"`
	GameVersionLatestFiles []struct {
		// TODO: check how twitch launcher chooses which one to use, when you are on beta/alpha channel?!
		// or does it not have the concept of release channels?!
		GameVersion string        `json:"gameVersion"`
		ID          uint32        `json:"fileId"`
		Name        string        `json:"filename"`
		FileType    fileType      `json:"releaseType"`
		Modloader   modloaderType `json:"modLoader"`
	} `json:"latestFilesIndexes"`
	ModLoaders []string `json:"modLoaders"`
	Links      struct {
		WebsiteURL string `json:"websiteUrl"`
	} `json:"links"`
}

func (c *cfApiClient) getModInfo(modID uint32) (modInfo, error) {
	var infoRes struct {
		Data modInfo `json:"data"`
	}

	idStr := strconv.FormatUint(uint64(modID), 10)
	if err := c.getJSON("/v1/mods/"+idStr, &infoRes); err != nil {
		return modInfo{}, fmt.Errorf("failed to request project data for ID %d: %w", modID, err)
	}

	if infoRes.Data.ID != modID {
		return modInfo{}, fmt.Errorf("unexpected project ID in CurseForge response: %d (expected %d)", infoRes.Data.ID, modID)
	}

	return infoRes.Data, nil
}

func (c *cfApiClient) getModInfoMultiple(modIDs []uint32) ([]modInfo, error) {
	var infoRes struct {
		Data []modInfo `json:"data"`
	}

	body := struct {
		ModIDs []uint32 `json:"modIds"`
	}{ModIDs: modIDs}
	if err := c.postJSON("/v1/mods", body, &infoRes); err != nil {
		return []modInfo{}, fmt.Errorf("failed to request project data: %w", err)
	}

	return infoRes.Data, nil
}

// modFileInfo is a subset of the deserialised JSON response from the Curse API for mod files
type modFileInfo struct {
	ID           uint32    `json:"id"`
	ModID        uint32    `json:"modId"`
	FileName     string    `json:"fileName"`
	FriendlyName string    `json:"displayName"`
	Date         time.Time `json:"fileDate"`
	Length       uint64    `json:"fileLength"`
	FileType     fileType  `json:"releaseType"`
	// According to the CurseForge API T&Cs, this must not be saved or cached
	DownloadURL  string   `json:"downloadUrl"`
	GameVersions []string `json:"gameVersions"`
	Fingerprint  uint32   `json:"fileFingerprint"`
	Dependencies []struct {
		ModID uint32         `json:"modId"`
		Type  dependencyType `json:"relationType"`
	} `json:"dependencies"`

	Hashes []struct {
		Value     string   `json:"value"`
		Algorithm hashAlgo `json:"algo"`
	} `json:"hashes"`
}

func (i modFileInfo) getBestHash() (hash string, hashFormat string, err error) {
	// A zero fingerprint means CurseForge hasn't finished processing the file;
	// it must not be accepted as a valid murmur2 hash.
	hashPreferred := -1
	if i.Fingerprint != 0 {
		hash = strconv.FormatUint(uint64(i.Fingerprint), 10)
		hashFormat = "murmur2"
		hashPreferred = 0
	}

	// Prefer SHA1, then MD5 if found:
	for _, v := range i.Hashes {
		if v.Algorithm == hashAlgoMD5 && v.Value != "" && hashPreferred < 1 {
			hashPreferred = 1

			hash = v.Value
			hashFormat = "md5"
		} else if v.Algorithm == hashAlgoSHA1 && v.Value != "" && hashPreferred < 2 {
			hashPreferred = 2

			hash = v.Value
			hashFormat = "sha1"
		}
	}

	if hashPreferred < 0 {
		return "", "", fmt.Errorf("file %s (ID %d) has no usable hash — CurseForge may still be processing it; try again shortly", i.FileName, i.ID)
	}
	return hash, hashFormat, nil
}

func (c *cfApiClient) getFileInfo(modID uint32, fileID uint32) (modFileInfo, error) {
	var infoRes struct {
		Data modFileInfo `json:"data"`
	}

	modIDStr := strconv.FormatUint(uint64(modID), 10)
	fileIDStr := strconv.FormatUint(uint64(fileID), 10)

	if err := c.getJSON("/v1/mods/"+modIDStr+"/files/"+fileIDStr, &infoRes); err != nil {
		return modFileInfo{}, fmt.Errorf("failed to request file data for project ID %d, file ID %d: %w", modID, fileID, err)
	}

	if infoRes.Data.ID != fileID {
		return modFileInfo{}, fmt.Errorf("unexpected file ID for project %d in CurseForge response: %d (expected %d)", modID, infoRes.Data.ID, fileID)
	}

	return infoRes.Data, nil
}

func (c *cfApiClient) getFileInfoMultiple(fileIDs []uint32) ([]modFileInfo, error) {
	var infoRes struct {
		Data []modFileInfo `json:"data"`
	}

	body := struct {
		FileIDs []uint32 `json:"fileIds"`
	}{FileIDs: fileIDs}
	if err := c.postJSON("/v1/mods/files", body, &infoRes); err != nil {
		return []modFileInfo{}, fmt.Errorf("failed to request file data: %w", err)
	}

	return infoRes.Data, nil
}

func (c *cfApiClient) getSearch(searchTerm string, slug string, gameID uint32, classID uint32, categoryID uint32, gameVersion string, modloaderType modloaderType) ([]modInfo, error) {
	var infoRes struct {
		Data []modInfo `json:"data"`
	}

	q := url.Values{}
	q.Set("gameId", strconv.FormatUint(uint64(gameID), 10))
	q.Set("pageSize", "10")
	if classID != 0 {
		q.Set("classId", strconv.FormatUint(uint64(classID), 10))
	}
	if slug != "" {
		q.Set("slug", slug)
	}
	// If classID and slug are provided, don't bother filtering by anything else (should be unique)
	if classID == 0 && slug == "" {
		if categoryID != 0 {
			q.Set("categoryId", strconv.FormatUint(uint64(categoryID), 10))
		}
		if searchTerm != "" {
			q.Set("searchFilter", searchTerm)
		}
		if gameVersion != "" {
			q.Set("gameVersion", gameVersion)
		}
		if modloaderType != modloaderTypeAny {
			q.Set("modLoaderType", strconv.FormatUint(uint64(modloaderType), 10))
		}
	}

	if err := c.getJSON("/v1/mods/search?"+q.Encode(), &infoRes); err != nil {
		return []modInfo{}, fmt.Errorf("failed to retrieve search results: %w", err)
	}

	return infoRes.Data, nil
}

type gameStatus uint8

// noinspection GoUnusedConst
const (
	gameStatusDraft gameStatus = iota + 1
	gameStatusTest
	gameStatusPendingReview
	gameStatusRejected
	gameStatusApproved
	gameStatusLive
)

type gameApiStatus uint8

// noinspection GoUnusedConst
const (
	gameApiStatusPrivate gameApiStatus = iota + 1
	gameApiStatusPublic
)

type cfGame struct {
	ID        uint32        `json:"id"`
	Name      string        `json:"name"`
	Slug      string        `json:"slug"`
	Status    gameStatus    `json:"status"`
	APIStatus gameApiStatus `json:"apiStatus"`
}

func (c *cfApiClient) getGames() ([]cfGame, error) {
	var infoRes struct {
		Data []cfGame `json:"data"`
	}

	if err := c.getJSON("/v1/games", &infoRes); err != nil {
		return []cfGame{}, fmt.Errorf("failed to retrieve game list: %w", err)
	}

	return infoRes.Data, nil
}

type cfCategory struct {
	ID      uint32 `json:"id"`
	Slug    string `json:"slug"`
	IsClass bool   `json:"isClass"`
	ClassID uint32 `json:"classId"`
}

func (c *cfApiClient) getCategories(gameID uint32) ([]cfCategory, error) {
	var infoRes struct {
		Data []cfCategory `json:"data"`
	}

	if err := c.getJSON("/v1/categories?gameId="+strconv.FormatUint(uint64(gameID), 10), &infoRes); err != nil {
		return []cfCategory{}, fmt.Errorf("failed to retrieve category list for game %v: %w", gameID, err)
	}

	return infoRes.Data, nil
}

type addonFingerprintResponse struct {
	IsCacheBuilt bool `json:"isCacheBuilt"`
	ExactMatches []struct {
		ID          uint32        `json:"id"`
		File        modFileInfo   `json:"file"`
		LatestFiles []modFileInfo `json:"latestFiles"`
	} `json:"exactMatches"`
	ExactFingerprints        []uint32 `json:"exactFingerprints"`
	PartialMatches           []uint32 `json:"partialMatches"`
	PartialMatchFingerprints struct{} `json:"partialMatchFingerprints"`
	InstalledFingerprints    []uint32 `json:"installedFingerprints"`
	UnmatchedFingerprints    []uint32 `json:"unmatchedFingerprints"`
}

func (c *cfApiClient) getFingerprintInfo(hashes []uint32) (addonFingerprintResponse, error) {
	var infoRes struct {
		Data addonFingerprintResponse `json:"data"`
	}

	body := struct {
		Fingerprints []uint32 `json:"fingerprints"`
	}{Fingerprints: hashes}
	if err := c.postJSON("/v1/fingerprints", body, &infoRes); err != nil {
		return addonFingerprintResponse{}, fmt.Errorf("failed to retrieve fingerprint results: %w", err)
	}

	return infoRes.Data, nil
}
