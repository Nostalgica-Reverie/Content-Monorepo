package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"os"
	"path/filepath"
	"strings"
)

const (
	modrinthAPI   = "https://api.modrinth.com/v2"
	curseforgeAPI = "https://minecraft.curseforge.com/api"
)

func pubUpload(manifestPath, variant string, live bool) {
	pDir := filepath.Dir(manifestPath)
	r := pubResolve(manifestPath, variant)

	if r.pType != "modpack" {
		fail(fmt.Sprintf("upload currently supports modpacks only (got '%s')", r.pType))
	}

	changelog := fmt.Sprintf("Update for %s", r.rawName)
	if data, err := os.ReadFile(filepath.Join(pDir, "changelog.md")); err == nil {
		changelog = string(data)
	}

	workspace := os.Getenv("GITHUB_WORKSPACE")
	if workspace == "" {
		workspace = "."
	}
	artifactsDir := filepath.Join(workspace, pDir, "artifacts")
	filenameBase := fmt.Sprintf("%s-%s-%s-%s", r.pName, r.mcVer, r.loader, r.pVer)

	if !live {
		fmt.Println("[DRY RUN] publish upload — nothing will be sent (pass --live to upload)")
	}

	attempted, uploaded := 0, 0
	for _, pl := range []struct {
		plat platform
		id   string
	}{{modrinth, r.mrID}, {curseforge, r.cfID}} {
		if pl.id == "" {
			continue
		}
		artifact := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.%s", filenameBase, pl.plat.short, pl.plat.ext))
		if _, err := os.Stat(artifact); err != nil {
			fmt.Printf("skipping %s: artifact %s not found (run 'publish build' first)\n", pl.plat.short, artifact)
			continue
		}
		attempted++
		data, err := os.ReadFile(artifact)
		if err != nil {
			fail(fmt.Sprintf("reading %s: %v", artifact, err))
		}
		fileName := filepath.Base(artifact)

		switch pl.plat.short {
		case "mr":
			uploadModrinth(r, pl.id, changelog, fileName, data, live)
		case "cf":
			uploadCurseforge(r, pl.id, changelog, fileName, data, live)
		}
		uploaded++
	}

	if attempted == 0 {
		fail(fmt.Sprintf("no artifacts found for '%s' in %s — run 'publish build' before 'publish upload'", r.subdirKey, artifactsDir))
	}
	mode := "validated (dry run)"
	if live {
		mode = "uploaded"
	}
	fmt.Printf("%d artifact(s) %s for %s\n", uploaded, mode, r.displayName)
}

func uploadModrinth(r pubResolved, projectID, changelog, fileName string, data []byte, live bool) {
	payload := map[string]any{
		"project_id":     projectID,
		"name":           r.displayName,
		"version_number": r.pVer,
		"changelog":      changelog,
		"dependencies":   []any{},
		"game_versions":  []string{r.mcVer},
		"version_type":   r.releaseType,
		"loaders":        []string{r.loader},
		"featured":       false,
		"file_parts":     []string{"file"},
		"primary_file":   "file",
	}

	fmt.Printf("modrinth: %s -> project %s | version %s | mc %s | loader %s | %d bytes\n",
		fileName, projectID, r.pVer, r.mcVer, r.loader, len(data))
	if !live {
		return
	}
	token := os.Getenv("MODRINTH_TOKEN")
	if token == "" {
		fail("MODRINTH_TOKEN not set")
	}

	meta, _ := json.Marshal(payload)
	contentType, body := buildMultipart([]mpart{
		{name: "data", contentType: "application/json", data: meta},
		{name: "file", fileName: fileName, contentType: "application/octet-stream", data: data},
	})

	doUpload("modrinth", modrinthAPI+"/version", map[string]string{
		"Authorization": token,
		"Content-Type":  contentType,
	}, body, r.pVer, projectID)
}

func uploadCurseforge(r pubResolved, projectID, changelog, fileName string, data []byte, live bool) {
	fmt.Printf("curseforge: %s -> project %s | version %s | mc %s | loader %s | %d bytes\n",
		fileName, projectID, r.pVer, r.mcVer, r.loader, len(data))
	if !live {
		return
	}
	token := os.Getenv("CURSEFORGE_TOKEN")
	if token == "" {
		fail("CURSEFORGE_TOKEN not set")
	}

	versionIDs := cfGameVersionIDs(token, r.mcVer, r.loader)
	meta, _ := json.Marshal(map[string]any{
		"changelog":     changelog,
		"changelogType": "markdown",
		"displayName":   r.displayName,
		"gameVersions":  versionIDs,
		"releaseType":   r.releaseType,
	})

	contentType, body := buildMultipart([]mpart{
		{name: "metadata", contentType: "application/json", data: meta},
		{name: "file", fileName: fileName, contentType: "application/octet-stream", data: data},
	})

	doUpload("curseforge", fmt.Sprintf("%s/projects/%s/upload-file", curseforgeAPI, projectID), map[string]string{
		"X-Api-Token":  token,
		"Content-Type": contentType,
	}, body, r.pVer, projectID)
}

func doUpload(label, url string, headers map[string]string, body []byte, pVer, projectID string) {
	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		fail(fmt.Sprintf("%s upload failed: %v", label, err))
	}
	for k, v := range headers {
		req.Header.Set(k, v)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		fail(fmt.Sprintf("%s upload failed: %v", label, err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		fail(fmt.Sprintf("%s upload failed (HTTP %d): %s", label, resp.StatusCode, string(detail)))
	}
	fmt.Printf("%s: uploaded %s to %s\n", label, pVer, projectID)
}

func cfGameVersionIDs(token, mcVer, loader string) []int64 {
	req, _ := http.NewRequest(http.MethodGet, curseforgeAPI+"/game/versions", nil)
	req.Header.Set("X-Api-Token", token)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		fail(fmt.Sprintf("CF game/versions lookup failed: %v", err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		fail(fmt.Sprintf("CF game/versions lookup failed (HTTP %d): %s", resp.StatusCode, string(detail)))
	}
	var list []map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&list); err != nil {
		fail(fmt.Sprintf("parsing CF game versions: %v", err))
	}

	loaderLC := strings.ToLower(loader)
	var ids []int64
	for _, entry := range list {
		name, _ := entry["name"].(string)
		slug, _ := entry["slug"].(string)
		idF, ok := entry["id"].(float64)
		if !ok {
			continue
		}
		if name == mcVer || slug == loaderLC || strings.ToLower(name) == loaderLC {
			ids = append(ids, int64(idF))
		}
	}
	if len(ids) < 2 {
		fail(fmt.Sprintf("could not resolve CF game-version IDs for mc '%s' + loader '%s' (matched %d of 2) — check the CF versions list", mcVer, loader, len(ids)))
	}
	return ids
}

func pubVerify(manifestPath, variant string) {
	r := pubResolve(manifestPath, variant)
	if r.mrID == "" {
		fail("verify currently checks Modrinth only, and this manifest has no modrinth_id")
	}
	url := fmt.Sprintf("%s/project/%s/version", modrinthAPI, r.mrID)
	resp, err := http.Get(url)
	if err != nil {
		fail(fmt.Sprintf("Modrinth version lookup failed: %v", err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		fail(fmt.Sprintf("Modrinth version lookup failed (HTTP %d): %s", resp.StatusCode, string(detail)))
	}
	var versions []map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&versions); err != nil {
		fail(fmt.Sprintf("parsing Modrinth version list: %v", err))
	}
	for _, v := range versions {
		if vn, _ := v["version_number"].(string); vn == r.pVer {
			vid, _ := v["id"].(string)
			published, _ := v["date_published"].(string)
			fmt.Printf("verified: %s %s is live on Modrinth (version id %s, published %s)\n", r.displayName, r.pVer, vid, published)
			return
		}
	}
	fail(fmt.Sprintf("version '%s' NOT found on Modrinth project '%s' (%d version(s) listed) — upload may have failed", r.pVer, r.mrID, len(versions)))
}

type mpart struct {
	name        string
	fileName    string
	contentType string
	data        []byte
}

func buildMultipart(parts []mpart) (contentType string, body []byte) {
	var buf bytes.Buffer
	w := multipart.NewWriter(&buf)
	for _, p := range parts {
		h := textproto.MIMEHeader{}
		if p.fileName != "" {
			h.Set("Content-Disposition", fmt.Sprintf(`form-data; name=%q; filename=%q`, p.name, p.fileName))
		} else {
			h.Set("Content-Disposition", fmt.Sprintf(`form-data; name=%q`, p.name))
		}
		h.Set("Content-Type", p.contentType)
		pw, err := w.CreatePart(h)
		if err != nil {
			fail(fmt.Sprintf("multipart build failed: %v", err))
		}
		if _, err := pw.Write(p.data); err != nil {
			fail(fmt.Sprintf("multipart build failed: %v", err))
		}
	}
	w.Close()
	return w.FormDataContentType(), buf.Bytes()
}
