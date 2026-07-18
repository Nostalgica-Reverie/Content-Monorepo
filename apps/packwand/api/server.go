// Package api implements Packwand's versioned HTTP API.
package api

import (
	"crypto/subtle"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
)

const Prefix = "/api/v1"

type Server struct {
	root  string
	token string
	jobs  *jobStore
}

type Options struct{ Token string }

func New(root string, options Options) (*Server, error) {
	absolute, err := filepath.Abs(root)
	if err != nil {
		return nil, err
	}
	return &Server{root: absolute, token: strings.TrimSpace(options.Token), jobs: newJobStore()}, nil
}

// Handler returns the API handler. fallback may serve a UI; nil produces a
// JSON not_found response for non-API paths.
func (s *Server) Handler(fallback http.Handler) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET "+Prefix+"/version", s.handleVersion)
	mux.HandleFunc("GET "+Prefix+"/capabilities", s.handleCapabilities)
	mux.HandleFunc("GET "+Prefix+"/openapi.json", s.handleOpenAPI)
	mux.HandleFunc("GET "+Prefix+"/packs", s.handlePacks)
	mux.HandleFunc("POST "+Prefix+"/packs", s.handleCreatePack)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/manifest", s.handleManifest)
	mux.HandleFunc("PUT "+Prefix+"/packs/{id}/manifest", s.handleSaveManifest)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/changelog", s.handleChangelog)
	mux.HandleFunc("PUT "+Prefix+"/packs/{id}/changelog", s.handleSaveChangelog)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/icon", s.handleIcon)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/mods", s.handleMods)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/subdirs/{sub}/registry/{kind}", s.handleRegistry)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/subdirs/{sub}/registry/{kind}/complete", s.handleRegistryComplete)
	mux.HandleFunc("POST "+Prefix+"/packs/{id}/subdirs/{sub}/check", s.handleCheck)
	mux.HandleFunc("POST "+Prefix+"/packs/{id}/subdirs/{sub}/kubejs/runtime-log", s.handleKubeJSRuntimeLog)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/subdirs/{sub}/tree", s.handleTree)
	mux.HandleFunc("GET "+Prefix+"/packs/{id}/subdirs/{sub}/file", s.handleReadFile)
	mux.HandleFunc("PUT "+Prefix+"/packs/{id}/subdirs/{sub}/file", s.handleWriteFile)
	mux.HandleFunc("POST "+Prefix+"/packs/{id}/subdirs/{sub}/files", s.handleCreateFile)
	mux.HandleFunc("GET "+Prefix+"/mods", s.handleMods) // v1 GUI compatibility
	mux.HandleFunc("GET "+Prefix+"/jobs", s.handleJobs)
	mux.HandleFunc("GET "+Prefix+"/jobs/{id}", s.handleJob)
	mux.HandleFunc("GET "+Prefix+"/jobs/{id}/events", s.handleJobEvents)
	mux.HandleFunc("POST "+Prefix+"/actions", s.handleCompatibilityAction)
	mux.HandleFunc("POST "+Prefix+"/webview/open", s.handleWebviewOpen)
	for _, action := range actions() {
		action := action
		mux.HandleFunc(action.Method+" "+action.Path, func(w http.ResponseWriter, r *http.Request) { s.startAction(w, r, action) })
	}
	if fallback != nil {
		mux.Handle("/", fallback)
	} else {
		mux.HandleFunc("/", func(w http.ResponseWriter, _ *http.Request) {
			writeError(w, http.StatusNotFound, "not_found", "route not found", "")
		})
	}
	handler := s.auth(mux)
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if strings.HasPrefix(r.URL.Path, "/api/") && !strings.HasPrefix(r.URL.Path, Prefix+"/") {
			writeError(w, http.StatusNotFound, "not_found", "API v1 route not found", "")
			return
		}
		handler.ServeHTTP(w, r)
	})
}

func (s *Server) auth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if s.token == "" || r.URL.Path == Prefix+"/version" {
			next.ServeHTTP(w, r)
			return
		}
		provided := strings.TrimPrefix(r.Header.Get("Authorization"), "Bearer ")
		if len(provided) != len(s.token) || subtle.ConstantTimeCompare([]byte(provided), []byte(s.token)) != 1 {
			writeError(w, http.StatusUnauthorized, "unauthorized", "a valid bearer token is required", "")
			return
		}
		next.ServeHTTP(w, r)
	})
}

func (s *Server) handleVersion(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, map[string]any{"ok": true, "version": cmd.Version(), "api_version": "v1", "root": filepath.ToSlash(s.root)})
}

type capability struct {
	Name        string `json:"name"`
	Method      string `json:"method"`
	Path        string `json:"path"`
	Summary     string `json:"summary,omitempty"`
	Destructive bool   `json:"destructive"`
}

func (s *Server) handleCapabilities(w http.ResponseWriter, _ *http.Request) {
	registered := actions()
	items := make([]capability, len(registered))
	for i, action := range registered {
		items[i] = capability{action.Name, action.Method, action.Path, action.Summary, action.Destructive}
	}
	writeJSON(w, map[string]any{"packwand_version": cmd.Version(), "api_version": "v1", "actions": items, "features": legacyFeatures(items)})
}

func legacyFeatures(items []capability) []map[string]any {
	out := make([]map[string]any, 0, len(items))
	for _, item := range items {
		out = append(out, map[string]any{"command": item.Name, "use": item.Path, "summary": item.Summary, "runnable": true, "gui_status": "integrated", "gui_action": item.Name, "destructive": item.Destructive})
	}
	return out
}

type Pack struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Type        string    `json:"type"`
	Category    string    `json:"category"`
	Dir         string    `json:"dir"`
	Loader      string    `json:"loader,omitempty"`
	MCVersion   string    `json:"mc_version,omitempty"`
	Version     string    `json:"version,omitempty"`
	ReleaseType string    `json:"release_type,omitempty"`
	Description string    `json:"description,omitempty"`
	Lifecycle   string    `json:"lifecycle,omitempty"`
	Role        string    `json:"role,omitempty"`
	Variants    []Variant `json:"variants,omitempty"`
	Subdirs     []Subdir  `json:"subdirs,omitempty"`
}

type Variant struct {
	ID        string `json:"id,omitempty"`
	MCVersion string `json:"mc_version,omitempty"`
	Loader    string `json:"loader,omitempty"`
	Version   string `json:"version,omitempty"`
}

type Subdir struct {
	Key      string `json:"key,omitempty"`
	Path     string `json:"path"`
	Platform string `json:"platform,omitempty"`
}

func (s *Server) discoverPacks() ([]Pack, error) {
	entries, err := manifest.LoadAll(s.root)
	if err != nil {
		return nil, err
	}
	packs := make([]Pack, 0, len(entries))
	for _, entry := range entries {
		packs = append(packs, toAPIPack(s.root, entry))
	}
	sort.Slice(packs, func(i, j int) bool { return packs[i].ID < packs[j].ID })
	return packs, nil
}

func toAPIPack(root string, entry manifest.Entry) Pack {
	m := entry.Manifest
	pack := Pack{ID: entry.ID, Name: m.Name, Type: m.Type, Category: entry.Category, Dir: filepath.ToSlash(mustRel(root, entry.Dir)), Loader: m.Loader, Version: m.Version, ReleaseType: m.ReleaseType, Description: m.Description, Lifecycle: first(m.Lifecycle, "active"), Role: m.Role.Label()}
	if m.MCVersion != nil {
		pack.MCVersion = *m.MCVersion
	}
	for _, variant := range m.Variants {
		pack.Variants = append(pack.Variants, Variant{ID: variant.ID, MCVersion: variant.MCVersion, Loader: first(variant.Loader, m.Loader), Version: variant.Version})
	}
	for _, subdir := range manifest.SubDirsOf(entry.Dir) {
		platform := ""
		if strings.HasSuffix(subdir, "-mr") {
			platform = "modrinth"
		} else if strings.HasSuffix(subdir, "-cf") {
			platform = "curseforge"
		}
		pack.Subdirs = append(pack.Subdirs, Subdir{Key: subdirKey(filepath.Base(subdir)), Path: filepath.ToSlash(mustRel(root, subdir)), Platform: platform})
	}
	return pack
}

func mustRel(root, path string) string { rel, _ := filepath.Rel(root, path); return rel }

func subdirKey(name string) string {
	switch {
	case strings.HasSuffix(name, "-mr"), strings.HasSuffix(name, "-cf"):
		return name[:len(name)-3]
	default:
		return name
	}
}

func (s *Server) handlePacks(w http.ResponseWriter, _ *http.Request) {
	packs, err := s.discoverPacks()
	if err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	writeJSON(w, map[string]any{"projects": packs, "packs": packs})
}

var idPattern = regexp.MustCompile(`^[a-z0-9][a-z0-9_-]*$`)

type createPackRequest struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Type        string `json:"type"`
	Loader      string `json:"loader"`
	MCVersion   string `json:"mc_version"`
	Version     string `json:"version"`
	ReleaseType string `json:"release_type"`
	Description string `json:"description"`
}

func (s *Server) handleCreatePack(w http.ResponseWriter, r *http.Request) {
	var request createPackRequest
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	request.ID, request.Name = strings.TrimSpace(request.ID), strings.TrimSpace(request.Name)
	if !idPattern.MatchString(request.ID) {
		writeError(w, 400, "invalid_argument", "id must be lowercase letters, numbers, hyphens, or underscores", "id")
		return
	}
	if request.Name == "" {
		writeError(w, 400, "invalid_argument", "name is required", "name")
		return
	}
	category, typ, ok := packCategory(request.Type)
	if !ok {
		writeError(w, 400, "invalid_argument", "type must be modpack, datapack, or resourcepack", "type")
		return
	}
	dir := filepath.Join(s.root, category, request.ID)
	if _, err := os.Stat(dir); err == nil {
		writeError(w, 409, "conflict", "pack already exists", "id")
		return
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	m := &manifest.Manifest{Schema: "../../tools/manifest/schema.json", ID: request.ID, Name: request.Name, Type: typ, Loader: request.Loader, Version: first(request.Version, "0.1.0"), ReleaseType: first(request.ReleaseType, "alpha"), Description: request.Description, Role: manifest.StringRole("none")}
	if request.MCVersion != "" {
		m.MCVersion = &request.MCVersion
	}
	if err := manifest.Write(filepath.Join(dir, "manifest.json"), m); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	_ = os.WriteFile(filepath.Join(dir, "changelog.md"), []byte("# Changelog\n\n## "+m.Version+"\n\n- Initial project scaffold.\n"), 0o644)
	writeJSONStatus(w, http.StatusCreated, map[string]string{"id": request.ID, "dir": filepath.ToSlash(mustRel(s.root, dir))})
}

func packCategory(value string) (string, string, bool) {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "modpack", "modpacks":
		return "modpacks", "modpack", true
	case "datapack", "datapacks":
		return "datapacks", "datapack", true
	case "resourcepack", "resourcepacks", "resource-pack":
		return "resourcepacks", "resourcepack", true
	}
	return "", "", false
}

func (s *Server) packDir(id string) (string, error) {
	if !idPattern.MatchString(id) {
		return "", errors.New("invalid pack id")
	}
	for _, category := range []string{"modpacks", "datapacks", "resourcepacks"} {
		root := filepath.Join(s.root, category)
		entries, _ := os.ReadDir(root)
		for _, entry := range entries {
			if !entry.IsDir() {
				continue
			}
			dir := filepath.Join(root, entry.Name())
			pack, err := manifest.Read(filepath.Join(dir, "manifest.json"))
			if err == nil && (pack.ID == id || entry.Name() == id) {
				return dir, nil
			}
		}
	}
	return "", os.ErrNotExist
}

func (s *Server) handleManifest(w http.ResponseWriter, r *http.Request) {
	s.serveTextFile(w, r, "manifest.json", false)
}
func (s *Server) handleChangelog(w http.ResponseWriter, r *http.Request) {
	s.serveTextFile(w, r, "changelog.md", true)
}
func (s *Server) serveTextFile(w http.ResponseWriter, r *http.Request, name string, missingOK bool) {
	dir, err := s.packDir(r.PathValue("id"))
	if err != nil {
		writeError(w, 404, "not_found", "pack not found", "id")
		return
	}
	path := filepath.Join(dir, name)
	data, err := os.ReadFile(path)
	if err != nil && missingOK && os.IsNotExist(err) {
		data = []byte{}
	} else if err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	writeJSON(w, map[string]string{"path": filepath.ToSlash(mustRel(s.root, path)), "content": string(data)})
}

func (s *Server) handleSaveManifest(w http.ResponseWriter, r *http.Request) {
	dir, err := s.packDir(r.PathValue("id"))
	if err != nil {
		writeError(w, 404, "not_found", "pack not found", "id")
		return
	}
	var request struct {
		Content string `json:"content"`
	}
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	var m manifest.Manifest
	if err := json.Unmarshal([]byte(request.Content), &m); err != nil {
		writeError(w, 400, "invalid_argument", "invalid manifest JSON: "+err.Error(), "content")
		return
	}
	if m.ID == "" || m.Name == "" || m.Type == "" {
		writeError(w, 400, "invalid_argument", "manifest must include id, name, and type", "content")
		return
	}
	if err := manifest.Write(filepath.Join(dir, "manifest.json"), &m); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	writeJSON(w, map[string]string{"status": "saved"})
}

func (s *Server) handleSaveChangelog(w http.ResponseWriter, r *http.Request) {
	dir, err := s.packDir(r.PathValue("id"))
	if err != nil {
		writeError(w, 404, "not_found", "pack not found", "id")
		return
	}
	var request struct {
		Content string `json:"content"`
	}
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	if err := os.WriteFile(filepath.Join(dir, "changelog.md"), []byte(request.Content), 0o644); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	writeJSON(w, map[string]string{"status": "saved"})
}

func (s *Server) handleIcon(w http.ResponseWriter, r *http.Request) {
	dir, err := s.packDir(r.PathValue("id"))
	if err != nil {
		writeError(w, 404, "not_found", "pack not found", "id")
		return
	}
	path := filepath.Join(dir, "icon.png")
	if _, err := os.Stat(path); err != nil {
		writeError(w, 404, "not_found", "icon not found", "")
		return
	}
	w.Header().Set("Cache-Control", "no-cache")
	http.ServeFile(w, r, path)
}

func decodeBody(r *http.Request, target any) error {
	decoder := json.NewDecoder(io.LimitReader(r.Body, 1<<20))
	decoder.DisallowUnknownFields()
	return decoder.Decode(target)
}
func first(values ...string) string {
	for _, value := range values {
		if value != "" {
			return value
		}
	}
	return ""
}

func (s *Server) cleanRepoPath(value string) (string, error) {
	if value == "" || filepath.IsAbs(value) {
		return "", errors.New("relative path is required")
	}
	full, err := filepath.Abs(filepath.Join(s.root, filepath.Clean(filepath.FromSlash(value))))
	if err != nil {
		return "", err
	}
	rel, err := filepath.Rel(s.root, full)
	if err != nil || rel == "." || rel == ".." || strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
		return "", errors.New("path must stay inside the repository")
	}
	info, err := os.Stat(full)
	if err != nil || !info.IsDir() {
		return "", errors.New("path must be an existing directory")
	}
	return full, nil
}

func (s *Server) actionDir(r *http.Request) (string, error) {
	if value := r.URL.Query().Get("subdir"); value != "" {
		return s.cleanRepoPath(value)
	}
	dir, err := s.packDir(r.PathValue("id"))
	if err != nil {
		return "", err
	}
	subdirs := manifest.SubDirsOf(dir)
	if len(subdirs) == 1 {
		return subdirs[0], nil
	}
	if len(subdirs) > 1 {
		return "", errors.New("subdir query parameter is required for packs with multiple variants")
	}
	return dir, nil
}

func (s *Server) startAction(w http.ResponseWriter, r *http.Request, action Action) {
	dir, args, err := action.Build(s, r)
	if err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "")
		return
	}
	job := s.jobs.create(action, args, dir)
	go s.runJob(job)
	writeJSONStatus(w, http.StatusAccepted, map[string]string{"job_id": job.ID})
}

func (s *Server) handleJobs(w http.ResponseWriter, _ *http.Request) { writeJSON(w, s.jobs.list()) }
func (s *Server) handleJob(w http.ResponseWriter, r *http.Request) {
	job := s.jobs.get(r.PathValue("id"))
	if job == nil {
		writeError(w, 404, "not_found", "job not found", "id")
		return
	}
	writeJSON(w, job)
}
func (s *Server) handleJobEvents(w http.ResponseWriter, r *http.Request) {
	job := s.jobs.get(r.PathValue("id"))
	if job == nil {
		writeError(w, 404, "not_found", "job not found", "id")
		return
	}
	flusher, ok := w.(http.Flusher)
	if !ok {
		writeError(w, 500, "internal", "streaming unsupported", "")
		return
	}
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	replay, channel := job.subscribe()
	defer job.unsubscribe(channel)
	for _, line := range replay {
		writeEvent(w, line)
	}
	flusher.Flush()
	for {
		select {
		case <-r.Context().Done():
			return
		case line, ok := <-channel:
			if !ok {
				return
			}
			writeEvent(w, line)
			flusher.Flush()
		}
	}
}
func writeEvent(w io.Writer, line string) {
	data, _ := json.Marshal(line)
	_, _ = fmt.Fprintf(w, "data: %s\n\n", data)
}

func (s *Server) handleMods(w http.ResponseWriter, r *http.Request) {
	dir, err := s.actionDir(r)
	if r.PathValue("id") == "" {
		dir, err = s.cleanRepoPath(r.URL.Query().Get("subdir"))
	}
	if err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "subdir")
		return
	}
	mods, err := readMods(filepath.Join(dir, "mods"))
	if err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	writeJSON(w, mods)
}

type Mod struct {
	Slug        string `json:"slug"`
	Name        string `json:"name,omitempty"`
	Filename    string `json:"filename,omitempty"`
	Side        string `json:"side,omitempty"`
	Pin         bool   `json:"pin"`
	Platform    string `json:"platform,omitempty"`
	ProjectID   string `json:"project_id,omitempty"`
	VersionID   string `json:"version_id,omitempty"`
	DownloadURL string `json:"download_url,omitempty"`
}

func readMods(dir string) ([]Mod, error) {
	entries, err := os.ReadDir(dir)
	if os.IsNotExist(err) {
		return []Mod{}, nil
	}
	if err != nil {
		return nil, err
	}
	mods := []Mod{}
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".pw.toml") {
			continue
		}
		mod := Mod{Slug: strings.TrimSuffix(entry.Name(), ".pw.toml"), Side: "both"}
		data, _ := os.ReadFile(filepath.Join(dir, entry.Name()))
		section := ""
		for _, raw := range strings.Split(string(data), "\n") {
			line := strings.TrimSpace(raw)
			if strings.HasPrefix(line, "[") {
				section = strings.Trim(line, "[]")
				continue
			}
			pair := strings.SplitN(line, "=", 2)
			if len(pair) != 2 {
				continue
			}
			key, value := strings.TrimSpace(pair[0]), strings.Trim(strings.TrimSpace(pair[1]), `"`)
			switch section + "/" + key {
			case "/name":
				mod.Name = value
			case "/filename":
				mod.Filename = value
			case "/side":
				mod.Side = value
			case "/pin":
				mod.Pin = value == "true"
			case "download/url":
				mod.DownloadURL = value
			case "update.modrinth/mod-id":
				mod.Platform, mod.ProjectID = "modrinth", value
			case "update.modrinth/version":
				mod.VersionID = value
			case "update.curseforge/project-id":
				mod.Platform, mod.ProjectID = "curseforge", value
			case "update.curseforge/file-id":
				mod.VersionID = value
			}
		}
		mods = append(mods, mod)
	}
	sort.Slice(mods, func(i, j int) bool { return mods[i].Name < mods[j].Name })
	return mods, nil
}
