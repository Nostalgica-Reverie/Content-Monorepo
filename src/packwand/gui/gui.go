package gui

import (
	"bufio"
	"crypto/rand"
	"embed"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"sync"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/workspace"
	"github.com/skratchdot/open-golang/open"
	"github.com/spf13/cobra"
)

//go:embed static/*
var staticFiles embed.FS

type server struct {
	root string
	jobs *jobStore
}

type actionRequest struct {
	Action    string `json:"action"`
	Path      string `json:"path,omitempty"`
	Subdir    string `json:"subdir,omitempty"`
	Slug      string `json:"slug,omitempty"`
	NoRefresh bool   `json:"no_refresh,omitempty"`
	DryRun    bool   `json:"dry_run,omitempty"`
}

type actionResponse struct {
	JobID string `json:"job_id"`
}

type job struct {
	ID       string    `json:"id"`
	Action   string    `json:"action"`
	Args     []string  `json:"args"`
	Dir      string    `json:"dir"`
	Status   string    `json:"status"`
	Started  time.Time `json:"started"`
	Finished time.Time `json:"finished,omitempty"`
	ExitCode int       `json:"exit_code,omitempty"`
	Error    string    `json:"error,omitempty"`

	mu          sync.Mutex
	lines       []string
	subscribers map[chan string]struct{}
}

type jobStore struct {
	mu   sync.Mutex
	jobs map[string]*job
}

type modEntry struct {
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

type projectIndex struct {
	Projects []projectEntry `json:"projects"`
}

type projectEntry struct {
	ID  string `json:"id"`
	Dir string `json:"dir"`
}

type newPackRequest struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Type        string `json:"type"`
	Loader      string `json:"loader,omitempty"`
	MCVersion   string `json:"mc_version,omitempty"`
	Version     string `json:"version,omitempty"`
	ReleaseType string `json:"release_type,omitempty"`
	Description string `json:"description,omitempty"`
}

type featureCapability struct {
	Command     string `json:"command"`
	Use         string `json:"use"`
	Summary     string `json:"summary"`
	Group       string `json:"group,omitempty"`
	Runnable    bool   `json:"runnable"`
	GUIStatus   string `json:"gui_status"`
	GUIAction   string `json:"gui_action,omitempty"`
	Scope       string `json:"scope,omitempty"`
	Destructive bool   `json:"destructive,omitempty"`
}

type guiIntegration struct {
	action      string
	scope       string
	destructive bool
}

var guiIntegrations = map[string]guiIntegration{
	"build":             {action: "build", scope: "subdir"},
	"curseforge add":    {action: "add-mod", scope: "curseforge-subdir", destructive: true},
	"curseforge export": {action: "export-curseforge", scope: "curseforge-subdir"},
	"doctor":            {action: "doctor", scope: "workspace"},
	"lint":              {action: "lint", scope: "workspace"},
	"modrinth add":      {action: "add-mod", scope: "modrinth-subdir", destructive: true},
	"modrinth export":   {action: "export-modrinth", scope: "modrinth-subdir"},
	"packs index":       {action: "packs-index", scope: "workspace", destructive: true},
	"pin":               {action: "pin-mod", scope: "subdir", destructive: true},
	"refresh":           {action: "refresh", scope: "subdir", destructive: true},
	"rehash":            {action: "rehash", scope: "subdir", destructive: true},
	"remove":            {action: "remove-mod", scope: "subdir", destructive: true},
	"unpin":             {action: "unpin-mod", scope: "subdir", destructive: true},
	"update":            {action: "update-mod/update-all", scope: "subdir", destructive: true},
	"validate":          {action: "validate-all/validate-project", scope: "workspace/project"},
	"workspace refresh": {action: "workspace-refresh", scope: "workspace", destructive: true},
	"workspace status":  {action: "workspace-status", scope: "workspace"},
	"workspace sync":    {action: "workspace-sync", scope: "workspace", destructive: true},
	"workspace update":  {action: "workspace-update", scope: "workspace", destructive: true},
}

var projectIDPattern = regexp.MustCompile(`^[a-z0-9][a-z0-9_-]*$`)

var guiCmd = &cobra.Command{
	Use:   "gui",
	Short: "Run the local Packwand web GUI",
	Run: func(c *cobra.Command, args []string) {
		port, _ := c.Flags().GetInt("port")
		noOpen, _ := c.Flags().GetBool("no-open")
		root := workspace.FindRepoRoot()
		if root == "" {
			cmd.Fail("could not locate repo root (no .git or modpacks/ found walking up from here)")
		}
		if err := os.Chdir(root); err != nil {
			cmd.Fail(fmt.Sprintf("failed to enter repo root %s: %v", root, err))
		}
		addr, err := listenAddr(port)
		if err != nil {
			cmd.Fail(err.Error())
		}
		srv := &server{root: root, jobs: &jobStore{jobs: map[string]*job{}}}
		httpSrv := &http.Server{Addr: addr, Handler: srv.routes()}
		url := "http://" + addr + "/"
		fmt.Printf("packwand gui running at %s\n", url)
		if !noOpen {
			go func() {
				time.Sleep(200 * time.Millisecond)
				_ = open.Run(url)
			}()
		}
		if err := httpSrv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			cmd.Fail(fmt.Sprintf("gui server failed: %v", err))
		}
	},
}

func init() {
	guiCmd.Flags().IntP("port", "p", 0, "Port to bind; 0 chooses a free local port")
	guiCmd.Flags().Bool("no-open", false, "Do not open the browser automatically")
	cmd.AddToGroup(guiCmd, cmd.GroupOther)
}

func listenAddr(port int) (string, error) {
	if port > 0 {
		return fmt.Sprintf("127.0.0.1:%d", port), nil
	}
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return "", fmt.Errorf("failed to reserve local port: %w", err)
	}
	addr := ln.Addr().String()
	if err := ln.Close(); err != nil {
		return "", err
	}
	return addr, nil
}

func (s *server) routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/health", s.handleHealth)
	mux.HandleFunc("GET /api/features", s.handleFeatures)
	mux.HandleFunc("GET /api/projects", s.handleProjects)
	mux.HandleFunc("POST /api/projects", s.handleCreateProject)
	mux.HandleFunc("GET /api/project-icon/{id}", s.handleProjectIcon)
	mux.HandleFunc("GET /api/projects/{id}/changelog", s.handleProjectChangelog)
	mux.HandleFunc("GET /api/projects/{id}/manifest", s.handleProjectManifest)
	mux.HandleFunc("PUT /api/projects/{id}/manifest", s.handleSaveProjectManifest)
	mux.HandleFunc("GET /api/mods", s.handleMods)
	mux.HandleFunc("GET /api/jobs", s.handleJobs)
	mux.HandleFunc("GET /api/jobs/{id}", s.handleJob)
	mux.HandleFunc("GET /api/jobs/{id}/events", s.handleJobEvents)
	mux.HandleFunc("POST /api/actions", s.handleAction)
	mux.HandleFunc("POST /api/webview/open", s.handleWebviewOpen)

	static, _ := fs.Sub(staticFiles, "static")
	mux.Handle("/", http.FileServer(http.FS(static)))
	return mux
}

func (s *server) handleHealth(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, map[string]any{
		"ok":      true,
		"root":    filepath.ToSlash(s.root),
		"version": cmd.Version(),
	})
}

func (s *server) handleFeatures(w http.ResponseWriter, r *http.Request) {
	catalog := cmd.CommandCatalog()
	features := make([]featureCapability, 0, len(catalog))
	for _, command := range catalog {
		feature := featureCapability{
			Command:   command.Path,
			Use:       command.Use,
			Summary:   command.Summary,
			Group:     command.Group,
			Runnable:  command.Runnable,
			GUIStatus: "cli-only",
		}
		if integration, ok := guiIntegrations[command.Path]; ok {
			feature.GUIStatus = "integrated"
			feature.GUIAction = integration.action
			feature.Scope = integration.scope
			feature.Destructive = integration.destructive
		} else if !command.Runnable {
			feature.GUIStatus = "group"
		}
		features = append(features, feature)
	}
	writeJSON(w, map[string]any{
		"packwand_version": cmd.Version(),
		"features":         features,
	})
}

func (s *server) handleProjects(w http.ResponseWriter, r *http.Request) {
	path := filepath.Join(s.root, "docs", "docs", "public", "projects.json")
	data, err := os.ReadFile(path)
	if err != nil {
		http.Error(w, "projects index not found; run packwand packs index", http.StatusNotFound)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	_, _ = w.Write(data)
}

func (s *server) handleCreateProject(w http.ResponseWriter, r *http.Request) {
	var req newPackRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid JSON", http.StatusBadRequest)
		return
	}
	req.ID = strings.TrimSpace(req.ID)
	req.Name = strings.TrimSpace(req.Name)
	if !projectIDPattern.MatchString(req.ID) {
		http.Error(w, "id must be lowercase letters, numbers, hyphens, or underscores", http.StatusBadRequest)
		return
	}
	if req.Name == "" {
		http.Error(w, "name is required", http.StatusBadRequest)
		return
	}
	root, typ, err := rootForProjectType(req.Type)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	dir := filepath.Join(s.root, root, req.ID)
	if _, err := os.Stat(dir); err == nil {
		http.Error(w, "project directory already exists", http.StatusConflict)
		return
	} else if !os.IsNotExist(err) {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	m := &manifest.Manifest{
		Schema:      "../../tools/manifest/schema.json",
		ID:          req.ID,
		Name:        req.Name,
		Type:        typ,
		Loader:      strings.TrimSpace(req.Loader),
		Version:     firstNonEmpty(strings.TrimSpace(req.Version), "0.1.0"),
		ReleaseType: firstNonEmpty(strings.TrimSpace(req.ReleaseType), "alpha"),
		Description: strings.TrimSpace(req.Description),
		Role:        manifest.StringRole("none"),
	}
	if mc := strings.TrimSpace(req.MCVersion); mc != "" {
		m.MCVersion = &mc
	}
	if err := manifest.Write(filepath.Join(dir, "manifest.json"), m); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	changelog := "# Changelog\n\n## " + m.Version + "\n\n- Initial project scaffold.\n"
	if err := os.WriteFile(filepath.Join(dir, "changelog.md"), []byte(changelog), 0o644); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if err := s.regenerateProjectsIndex(); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, map[string]string{
		"id":  req.ID,
		"dir": filepath.ToSlash(filepath.Join(root, req.ID)),
	})
}

func (s *server) handleProjectIcon(w http.ResponseWriter, r *http.Request) {
	dir, err := s.projectDir(r.PathValue("id"))
	if err != nil {
		http.NotFound(w, r)
		return
	}
	path := filepath.Join(dir, "icon.png")
	info, err := os.Stat(path)
	if err != nil || info.IsDir() {
		http.NotFound(w, r)
		return
	}
	w.Header().Set("Cache-Control", "no-cache")
	http.ServeFile(w, r, path)
}

func (s *server) handleProjectChangelog(w http.ResponseWriter, r *http.Request) {
	dir, err := s.projectDir(r.PathValue("id"))
	if err != nil {
		http.NotFound(w, r)
		return
	}
	data, err := os.ReadFile(filepath.Join(dir, "changelog.md"))
	if err != nil {
		if os.IsNotExist(err) {
			writeJSON(w, map[string]string{"path": filepath.ToSlash(filepath.Join(dir, "changelog.md")), "content": ""})
			return
		}
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, map[string]string{"path": filepath.ToSlash(filepath.Join(dir, "changelog.md")), "content": string(data)})
}

func (s *server) handleProjectManifest(w http.ResponseWriter, r *http.Request) {
	dir, err := s.projectDir(r.PathValue("id"))
	if err != nil {
		http.NotFound(w, r)
		return
	}
	data, err := os.ReadFile(filepath.Join(dir, "manifest.json"))
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, map[string]string{"path": filepath.ToSlash(filepath.Join(dir, "manifest.json")), "content": string(data)})
}

func (s *server) handleSaveProjectManifest(w http.ResponseWriter, r *http.Request) {
	dir, err := s.projectDir(r.PathValue("id"))
	if err != nil {
		http.NotFound(w, r)
		return
	}
	var req struct {
		Content string `json:"content"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid JSON", http.StatusBadRequest)
		return
	}
	var m manifest.Manifest
	if err := json.Unmarshal([]byte(req.Content), &m); err != nil {
		http.Error(w, "invalid manifest JSON: "+err.Error(), http.StatusBadRequest)
		return
	}
	if m.ID == "" || m.Name == "" || m.Type == "" {
		http.Error(w, "manifest must include id, name, and type", http.StatusBadRequest)
		return
	}
	if _, _, err := rootForProjectType(m.Type); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	if err := manifest.Write(filepath.Join(dir, "manifest.json"), &m); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if err := s.regenerateProjectsIndex(); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, map[string]string{"status": "saved"})
}

func (s *server) regenerateProjectsIndex() error {
	c := exec.Command(workspace.SelfBin(), "packs", "index")
	c.Dir = s.root
	workspace.ConfigureSubprocess(c)
	out, err := c.CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to regenerate projects index: %v: %s", err, strings.TrimSpace(string(out)))
	}
	return nil
}

func (s *server) projectDir(id string) (string, error) {
	if id == "" || strings.ContainsAny(id, "\\/\r\n\t") {
		return "", errors.New("invalid project id")
	}
	data, err := os.ReadFile(filepath.Join(s.root, "docs", "docs", "public", "projects.json"))
	if err != nil {
		return "", err
	}
	var index projectIndex
	if err := json.Unmarshal(data, &index); err != nil {
		return "", err
	}
	for _, project := range index.Projects {
		if project.ID == id {
			return s.cleanRepoPath(project.Dir)
		}
	}
	return "", os.ErrNotExist
}

func rootForProjectType(typ string) (root, normalized string, err error) {
	switch strings.ToLower(strings.TrimSpace(typ)) {
	case "modpack", "modpacks":
		return "modpacks", "modpack", nil
	case "resourcepack", "resourcepacks", "resource-pack", "resource-packs":
		return "resourcepacks", "resourcepack", nil
	case "datapack", "datapacks":
		return "datapacks", "datapack", nil
	default:
		return "", "", fmt.Errorf("invalid project type %q", typ)
	}
}

func (s *server) handleMods(w http.ResponseWriter, r *http.Request) {
	subdir, err := s.cleanRepoPath(r.URL.Query().Get("subdir"))
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	mods, err := readMods(filepath.Join(subdir, "mods"))
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	writeJSON(w, mods)
}

func (s *server) handleJobs(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, s.jobs.list())
}

func (s *server) handleJob(w http.ResponseWriter, r *http.Request) {
	j := s.jobs.get(r.PathValue("id"))
	if j == nil {
		http.NotFound(w, r)
		return
	}
	writeJSON(w, j.snapshot())
}

func (s *server) handleJobEvents(w http.ResponseWriter, r *http.Request) {
	j := s.jobs.get(r.PathValue("id"))
	if j == nil {
		http.NotFound(w, r)
		return
	}
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming unsupported", http.StatusInternalServerError)
		return
	}

	// Take the replay snapshot and subscribe under one lock acquisition so
	// lines appended in between are neither dropped nor sent twice.
	replay, ch := j.subscribe()
	defer j.unsubscribe(ch)
	for _, line := range replay {
		writeEvent(w, line)
	}
	flusher.Flush()

	for {
		select {
		case <-r.Context().Done():
			return
		case line, ok := <-ch:
			if !ok {
				return
			}
			writeEvent(w, line)
			flusher.Flush()
		}
	}
}

func (s *server) handleAction(w http.ResponseWriter, r *http.Request) {
	var req actionRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid JSON", http.StatusBadRequest)
		return
	}
	dir, args, err := s.resolveAction(req)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	j := s.jobs.create(req.Action, args, dir)
	go s.runJob(j)
	writeJSON(w, actionResponse{JobID: j.ID})
}

func (s *server) resolveAction(req actionRequest) (string, []string, error) {
	switch req.Action {
	case "packs-index":
		return s.root, []string{"packs", "index"}, nil
	case "validate-all":
		return s.root, []string{"validate", "--all"}, nil
	case "validate-project":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		return dir, []string{"validate", "manifest.json"}, err
	case "doctor":
		return s.root, []string{"doctor"}, nil
	case "lint":
		return s.root, []string{"lint"}, nil
	case "workspace-status":
		return s.root, []string{"workspace", "status"}, nil
	case "workspace-sync":
		args := []string{"workspace", "sync"}
		if req.DryRun {
			args = append(args, "--dry-run")
		}
		return s.root, args, nil
	case "workspace-refresh":
		return s.root, []string{"workspace", "refresh"}, nil
	case "workspace-update-check":
		return s.root, []string{"workspace", "update", "--all", "--check"}, nil
	case "workspace-update":
		return s.root, []string{"workspace", "update", "--all"}, nil
	case "refresh":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		return dir, []string{"refresh"}, err
	case "add-mod":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		if err != nil {
			return "", nil, err
		}
		slug, err := cleanSlug(req.Slug)
		if err != nil {
			return "", nil, err
		}
		args, err := addArgsForSubdir(dir, slug)
		if err != nil {
			return "", nil, err
		}
		if req.NoRefresh {
			args = append(args, "--no-refresh")
		}
		return dir, args, nil
	case "remove-mod":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		if err != nil {
			return "", nil, err
		}
		slug, err := cleanSlug(req.Slug)
		if err != nil {
			return "", nil, err
		}
		return dir, []string{"remove", slug}, nil
	case "pin-mod":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		if err != nil {
			return "", nil, err
		}
		slug, err := cleanSlug(req.Slug)
		if err != nil {
			return "", nil, err
		}
		return dir, []string{"pin", slug}, nil
	case "unpin-mod":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		if err != nil {
			return "", nil, err
		}
		slug, err := cleanSlug(req.Slug)
		if err != nil {
			return "", nil, err
		}
		return dir, []string{"unpin", slug}, nil
	case "update-mod":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		if err != nil {
			return "", nil, err
		}
		slug, err := cleanSlug(req.Slug)
		if err != nil {
			return "", nil, err
		}
		return dir, []string{"update", slug}, nil
	case "update-all":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		if err != nil {
			return "", nil, err
		}
		return dir, []string{"update", "--all", "-y"}, nil
	case "build":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		return dir, []string{"build"}, err
	case "rehash":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		return dir, []string{"rehash"}, err
	case "export-modrinth":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		return dir, []string{"modrinth", "export"}, err
	case "export-curseforge":
		dir, err := s.cleanRepoPath(firstNonEmpty(req.Subdir, req.Path))
		return dir, []string{"curseforge", "export"}, err
	default:
		return "", nil, fmt.Errorf("unknown action %q", req.Action)
	}
}

func addArgsForSubdir(dir, slug string) ([]string, error) {
	switch {
	case strings.HasSuffix(filepath.Base(dir), "-mr"):
		return []string{"modrinth", "add", "-y", slug}, nil
	case strings.HasSuffix(filepath.Base(dir), "-cf"):
		return []string{"curseforge", "add", "-y", slug}, nil
	default:
		return nil, errors.New("add-mod requires a -mr or -cf pack subdir")
	}
}

func cleanSlug(slug string) (string, error) {
	slug = strings.TrimSpace(slug)
	if slug == "" {
		return "", errors.New("slug is required")
	}
	if strings.ContainsAny(slug, "\\/\r\n\t") {
		return "", errors.New("slug must not contain path separators or whitespace controls")
	}
	return slug, nil
}

func firstNonEmpty(values ...string) string {
	for _, v := range values {
		if v != "" {
			return v
		}
	}
	return ""
}

func (s *server) cleanRepoPath(p string) (string, error) {
	if p == "" {
		return "", errors.New("path is required")
	}
	if filepath.IsAbs(p) {
		return "", errors.New("absolute paths are not accepted")
	}
	clean := filepath.Clean(filepath.FromSlash(p))
	full, err := filepath.Abs(filepath.Join(s.root, clean))
	if err != nil {
		return "", err
	}
	rel, err := filepath.Rel(s.root, full)
	if err != nil {
		return "", err
	}
	if rel == "." || strings.HasPrefix(rel, ".."+string(filepath.Separator)) || rel == ".." {
		return "", errors.New("path must stay inside the repository")
	}
	info, err := os.Stat(full)
	if err != nil {
		return "", err
	}
	if !info.IsDir() {
		return "", errors.New("path must be a directory")
	}
	return full, nil
}

func readMods(modsDir string) ([]modEntry, error) {
	entries, err := os.ReadDir(modsDir)
	if err != nil {
		if os.IsNotExist(err) {
			return []modEntry{}, nil
		}
		return nil, err
	}
	mods := make([]modEntry, 0, len(entries))
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".pw.toml") {
			continue
		}
		mod, err := readModMeta(filepath.Join(modsDir, entry.Name()))
		if err != nil {
			continue
		}
		mod.Slug = strings.TrimSuffix(entry.Name(), ".pw.toml")
		mods = append(mods, mod)
	}
	return mods, nil
}

func readModMeta(path string) (modEntry, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return modEntry{}, err
	}
	var mod modEntry
	var section string
	for _, raw := range strings.Split(string(data), "\n") {
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			section = strings.Trim(line, "[]")
			continue
		}
		key, value, ok := splitKV(line)
		if !ok {
			continue
		}
		switch section {
		case "":
			switch key {
			case "name":
				mod.Name = value
			case "filename":
				mod.Filename = value
			case "side":
				mod.Side = value
			case "pin":
				mod.Pin = value == "true"
			}
		case "download":
			if key == "url" {
				mod.DownloadURL = value
			}
		case "update.modrinth":
			mod.Platform = "modrinth"
			switch key {
			case "mod-id":
				mod.ProjectID = value
			case "version":
				mod.VersionID = value
			}
		case "update.curseforge":
			mod.Platform = "curseforge"
			switch key {
			case "project-id":
				mod.ProjectID = value
			case "file-id":
				mod.VersionID = value
			}
		case "update.github":
			mod.Platform = "github"
			if key == "slug" {
				mod.ProjectID = value
			}
		case "update.forgejo":
			mod.Platform = "forgejo"
			if key == "slug" {
				mod.ProjectID = value
			}
		case "update.gitlab":
			mod.Platform = "gitlab"
			if key == "slug" {
				mod.ProjectID = value
			}
		}
	}
	if mod.Side == "" {
		mod.Side = "both"
	}
	return mod, nil
}

func splitKV(line string) (key, value string, ok bool) {
	i := strings.Index(line, "=")
	if i < 0 {
		return "", "", false
	}
	key = strings.TrimSpace(line[:i])
	value = strings.TrimSpace(line[i+1:])
	value = strings.Trim(value, `"`)
	return key, value, true
}

func (s *server) runJob(j *job) {
	j.append("$ packwand " + strings.Join(j.Args, " "))
	c := exec.Command(workspace.SelfBin(), j.Args...)
	c.Dir = j.Dir
	workspace.ConfigureSubprocess(c)
	stdout, err := c.StdoutPipe()
	if err != nil {
		j.finish(-1, err)
		return
	}
	stderr, err := c.StderrPipe()
	if err != nil {
		j.finish(-1, err)
		return
	}
	if err := c.Start(); err != nil {
		j.finish(-1, err)
		return
	}

	var wg sync.WaitGroup
	wg.Add(2)
	go streamLines(stdout, func(line string) { j.append(line) }, &wg)
	go streamLines(stderr, func(line string) { j.append(line) }, &wg)
	wg.Wait()
	err = c.Wait()
	exitCode := 0
	if c.ProcessState != nil {
		exitCode = c.ProcessState.ExitCode()
	}
	j.finish(exitCode, err)
}

func streamLines(r io.Reader, emit func(string), wg *sync.WaitGroup) {
	defer wg.Done()
	scanner := bufio.NewScanner(r)
	buf := make([]byte, 0, 64*1024)
	scanner.Buffer(buf, 1024*1024)
	for scanner.Scan() {
		emit(scanner.Text())
	}
	if err := scanner.Err(); err != nil {
		emit("stream error: " + err.Error())
	}
}

func newJobID() string {
	var b [8]byte
	if _, err := rand.Read(b[:]); err != nil {
		return fmt.Sprintf("%d", time.Now().UnixNano())
	}
	return hex.EncodeToString(b[:])
}

func (s *jobStore) create(action string, args []string, dir string) *job {
	j := &job{
		ID:          newJobID(),
		Action:      action,
		Args:        args,
		Dir:         filepath.ToSlash(dir),
		Status:      "running",
		Started:     time.Now(),
		subscribers: map[chan string]struct{}{},
	}
	s.mu.Lock()
	s.jobs[j.ID] = j
	s.mu.Unlock()
	return j
}

func (s *jobStore) get(id string) *job {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.jobs[id]
}

func (s *jobStore) list() []jobSnapshot {
	s.mu.Lock()
	jobs := make([]*job, 0, len(s.jobs))
	for _, j := range s.jobs {
		jobs = append(jobs, j)
	}
	s.mu.Unlock()
	out := make([]jobSnapshot, len(jobs))
	for i, j := range jobs {
		out[i] = j.snapshot()
	}
	return out
}

type jobSnapshot struct {
	ID       string    `json:"id"`
	Action   string    `json:"action"`
	Args     []string  `json:"args"`
	Dir      string    `json:"dir"`
	Status   string    `json:"status"`
	Started  time.Time `json:"started"`
	Finished time.Time `json:"finished,omitempty"`
	ExitCode int       `json:"exit_code,omitempty"`
	Error    string    `json:"error,omitempty"`
	Lines    []string  `json:"lines,omitempty"`
}

func (j *job) snapshot() jobSnapshot {
	j.mu.Lock()
	defer j.mu.Unlock()
	lines := append([]string(nil), j.lines...)
	return jobSnapshot{
		ID:       j.ID,
		Action:   j.Action,
		Args:     append([]string(nil), j.Args...),
		Dir:      j.Dir,
		Status:   j.Status,
		Started:  j.Started,
		Finished: j.Finished,
		ExitCode: j.ExitCode,
		Error:    j.Error,
		Lines:    lines,
	}
}

func (j *job) append(line string) {
	j.mu.Lock()
	j.lines = append(j.lines, line)
	for ch := range j.subscribers {
		select {
		case ch <- line:
		default:
		}
	}
	j.mu.Unlock()
}

func (j *job) finish(exitCode int, err error) {
	status := "completed"
	errText := ""
	if err != nil {
		status = "failed"
		errText = err.Error()
	}
	j.mu.Lock()
	j.Status = status
	j.ExitCode = exitCode
	j.Error = errText
	j.Finished = time.Now()
	j.mu.Unlock()
	if err != nil {
		j.append(fmt.Sprintf("failed: %v", err))
	} else {
		j.append("completed")
	}
	j.mu.Lock()
	for ch := range j.subscribers {
		close(ch)
	}
	j.subscribers = map[chan string]struct{}{}
	j.mu.Unlock()
}

func (j *job) subscribe() ([]string, chan string) {
	ch := make(chan string, 128)
	j.mu.Lock()
	replay := append([]string(nil), j.lines...)
	if j.Status == "running" {
		j.subscribers[ch] = struct{}{}
	} else {
		close(ch)
	}
	j.mu.Unlock()
	return replay, ch
}

func (j *job) unsubscribe(ch chan string) {
	j.mu.Lock()
	if _, ok := j.subscribers[ch]; ok {
		delete(j.subscribers, ch)
		close(ch)
	}
	j.mu.Unlock()
}

func writeJSON(w http.ResponseWriter, v any) {
	w.Header().Set("Content-Type", "application/json")
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	_ = enc.Encode(v)
}

func writeEvent(w io.Writer, line string) {
	data, _ := json.Marshal(line)
	fmt.Fprintf(w, "data: %s\n\n", data)
}
