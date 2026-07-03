package api

import (
	"errors"
	"net/http"
	"path/filepath"
	"reflect"
	"regexp"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/build"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
)

var slugPattern = regexp.MustCompile(`^[A-Za-z0-9][A-Za-z0-9._-]*$`)

type actionInput struct {
	Action    string `json:"action"`
	Path      string `json:"path,omitempty"`
	Subdir    string `json:"subdir,omitempty"`
	Slug      string `json:"slug,omitempty"`
	NoRefresh bool   `json:"no_refresh,omitempty"`
	DryRun    bool   `json:"dry_run,omitempty"`
}

func init() {
	workspaceAction("workspace.status", http.MethodGet, Prefix+"/workspace/status", false, []string{"workspace", "status", "--json"}, reflect.TypeOf([]cmd.WorkspaceStatus{}))
	workspaceAction("workspace.sync", http.MethodPost, Prefix+"/workspace/sync", true, []string{"workspace", "sync"}, nil)
	workspaceAction("workspace.refresh", http.MethodPost, Prefix+"/workspace/refresh", true, []string{"workspace", "refresh"}, nil)
	workspaceAction("workspace.update", http.MethodPost, Prefix+"/workspace/update", true, []string{"workspace", "update", "--all"}, nil)
	workspaceAction("validate", http.MethodGet, Prefix+"/validate", false, []string{"validate", "--all"}, nil)
	workspaceAction("lint", http.MethodGet, Prefix+"/lint", false, []string{"lint"}, nil)
	workspaceAction("doctor", http.MethodGet, Prefix+"/doctor", false, []string{"doctor", "--json"}, reflect.TypeOf(cmd.DoctorResult{}))

	packAction("packs.build", http.MethodPost, Prefix+"/packs/{id}/build", false, func(_ *http.Request) []string { return []string{"build"} })
	packAction("packs.rehash", http.MethodPost, Prefix+"/packs/{id}/rehash", true, func(_ *http.Request) []string { return []string{"rehash"} })
	Register(Action{Name: "packs.publish.plan", Method: http.MethodGet, Path: Prefix + "/packs/{id}/publish/plan", Summary: "Compute the publish plan", Result: reflect.TypeOf(build.PlanResult{}), Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.packDir(r.PathValue("id"))
		if err != nil {
			return "", nil, err
		}
		rel, err := filepath.Rel(s.root, dir)
		if err != nil {
			return "", nil, err
		}
		return s.root, []string{"publish", "plan", "--pack", filepath.ToSlash(rel)}, nil
	}})
	Register(Action{Name: "packs.publish.upload", Method: http.MethodPost, Path: Prefix + "/packs/{id}/publish/upload", Summary: "Upload a pack release", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.packDir(r.PathValue("id"))
		if err != nil {
			return "", nil, err
		}
		manifestPath := filepath.Join(dir, "manifest.json")
		args := []string{"publish", "upload", manifestPath}
		if variant := r.URL.Query().Get("variant"); variant != "" {
			args = append(args, variant)
		}
		return s.root, args, nil
	}})
	Register(Action{Name: "packs.mods.add", Method: http.MethodPost, Path: Prefix + "/packs/{id}/mods", Summary: "Add a mod", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		var body struct {
			Slug      string `json:"slug"`
			NoRefresh bool   `json:"no_refresh"`
		}
		if err := decodeBody(r, &body); err != nil {
			return "", nil, err
		}
		if !slugPattern.MatchString(body.Slug) {
			return "", nil, errors.New("invalid or missing slug")
		}
		provider := "modrinth"
		if strings.HasSuffix(filepath.Base(dir), "-cf") {
			provider = "curseforge"
		}
		args := []string{provider, "add", "-y", body.Slug}
		if body.NoRefresh {
			args = append(args, "--no-refresh")
		}
		return dir, args, nil
	}})
	packAction("packs.mods.remove", http.MethodDelete, Prefix+"/packs/{id}/mods/{slug}", true, func(r *http.Request) []string { return []string{"remove", r.PathValue("slug")} })
	packAction("packs.mods.pin", http.MethodPost, Prefix+"/packs/{id}/mods/{slug}/pin", true, func(r *http.Request) []string { return []string{"pin", r.PathValue("slug")} })
	packAction("packs.mods.unpin", http.MethodPost, Prefix+"/packs/{id}/mods/{slug}/unpin", true, func(r *http.Request) []string { return []string{"unpin", r.PathValue("slug")} })
	packAction("packs.mods.update", http.MethodPost, Prefix+"/packs/{id}/mods/{slug}/update", true, func(r *http.Request) []string { return []string{"update", r.PathValue("slug")} })
}

func workspaceAction(name, method, path string, destructive bool, args []string, result reflect.Type) {
	Register(Action{Name: name, Method: method, Path: path, Summary: name, Destructive: destructive, Result: result, Build: func(s *Server, r *http.Request) (string, []string, error) {
		out := append([]string(nil), args...)
		if name == "workspace.sync" && queryBool(r, "dry_run") {
			out = append(out, "--dry-run")
		}
		if name == "workspace.update" && queryBool(r, "check") {
			out = append(out, "--check")
		}
		return s.root, out, nil
	}})
}

func packAction(name, method, path string, destructive bool, arguments func(*http.Request) []string) {
	Register(Action{Name: name, Method: method, Path: path, Summary: name, Destructive: destructive, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		args := arguments(r)
		if len(args) == 0 {
			return "", nil, errors.New("request body is missing required values")
		}
		return dir, args, nil
	}})
}

func queryBool(r *http.Request, name string) bool {
	value := r.URL.Query().Get(name)
	return value == "1" || strings.EqualFold(value, "true")
}

func (s *Server) handleCompatibilityAction(w http.ResponseWriter, r *http.Request) {
	var input actionInput
	if err := decodeBody(r, &input); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	action := Action{Name: input.Action, Method: http.MethodPost, Path: Prefix + "/actions", Destructive: true}
	dir, args, err := s.resolveCompatibilityAction(input)
	if err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "action")
		return
	}
	job := s.jobs.create(action, args, dir)
	go s.runJob(job)
	writeJSONStatus(w, 202, map[string]string{"job_id": job.ID})
}

func (s *Server) resolveCompatibilityAction(input actionInput) (string, []string, error) {
	path := first(input.Subdir, input.Path)
	workspaceArgs := map[string][]string{
		"packs-index": {"packs", "index"}, "validate-all": {"validate", "--all"}, "doctor": {"doctor"}, "lint": {"lint"},
		"workspace-status": {"workspace", "status"}, "workspace-refresh": {"workspace", "refresh"}, "workspace-update": {"workspace", "update", "--all"}, "workspace-update-check": {"workspace", "update", "--all", "--check"},
	}
	if args, ok := workspaceArgs[input.Action]; ok {
		return s.root, args, nil
	}
	if input.Action == "workspace-sync" {
		args := []string{"workspace", "sync"}
		if input.DryRun {
			args = append(args, "--dry-run")
		}
		return s.root, args, nil
	}
	dir, err := s.cleanRepoPath(path)
	if err != nil {
		return "", nil, err
	}
	slug := strings.TrimSpace(input.Slug)
	switch input.Action {
	case "validate-project":
		return dir, []string{"validate", "manifest.json"}, nil
	case "refresh":
		return dir, []string{"refresh"}, nil
	case "build":
		return dir, []string{"build"}, nil
	case "rehash":
		return dir, []string{"rehash"}, nil
	case "export-modrinth":
		return dir, []string{"modrinth", "export"}, nil
	case "export-curseforge":
		return dir, []string{"curseforge", "export"}, nil
	case "update-all":
		return dir, []string{"update", "--all", "-y"}, nil
	case "add-mod":
		if slug == "" {
			return "", nil, errors.New("slug is required")
		}
		provider := "modrinth"
		if strings.HasSuffix(filepath.Base(dir), "-cf") {
			provider = "curseforge"
		}
		args := []string{provider, "add", "-y", slug}
		if input.NoRefresh {
			args = append(args, "--no-refresh")
		}
		return dir, args, nil
	case "remove-mod", "pin-mod", "unpin-mod", "update-mod":
		if slug == "" {
			return "", nil, errors.New("slug is required")
		}
		command := strings.TrimSuffix(input.Action, "-mod")
		if command == "remove" || command == "pin" || command == "unpin" || command == "update" {
			return dir, []string{command, slug}, nil
		}
	}
	return "", nil, errors.New("unknown action")
}
