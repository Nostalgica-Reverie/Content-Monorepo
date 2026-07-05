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
	Version   string `json:"version,omitempty"`
	Configs   bool   `json:"configs,omitempty"`
	Side      string `json:"side,omitempty"`
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

	packAction("packs.run", http.MethodPost, Prefix+"/packs/{id}/run/{script}", true, func(r *http.Request) []string { return []string{"run", r.PathValue("script")} })

	packAction("packs.migrate.format", http.MethodPost, Prefix+"/packs/{id}/migrate/format", true, func(_ *http.Request) []string { return []string{"migrate", "format"} })
	Register(Action{Name: "packs.migrate.loader", Method: http.MethodPost, Path: Prefix + "/packs/{id}/migrate/loader", Summary: "Migrate loader version(s)", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		var body struct {
			Target string `json:"target"`
		}
		if err := decodeBody(r, &body); err != nil {
			return "", nil, err
		}
		if body.Target == "" {
			return "", nil, errors.New("target is required (a version, \"latest\", or \"recommended\")")
		}
		return dir, []string{"migrate", "loader", body.Target}, nil
	}})
	Register(Action{Name: "packs.migrate.minecraft", Method: http.MethodPost, Path: Prefix + "/packs/{id}/migrate/minecraft", Summary: "Migrate the Minecraft version", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		var body struct {
			Version string `json:"version"`
		}
		if err := decodeBody(r, &body); err != nil {
			return "", nil, err
		}
		if body.Version == "" {
			return "", nil, errors.New("version is required")
		}
		return dir, []string{"migrate", "minecraft", body.Version}, nil
	}})

	Register(Action{Name: "packs.docs.modlist", Method: http.MethodPost, Path: Prefix + "/packs/{id}/docs/modlist", Summary: "Write a crash-assistant modlist.json for a pack subdir", Destructive: true, Result: reflect.TypeOf(cmd.ModlistResult{}), Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		return s.root, []string{"modlist", dir, "--json"}, nil
	}})
	Register(Action{Name: "packs.docs.pages", Method: http.MethodPost, Path: Prefix + "/packs/{id}/docs/pages", Summary: "Regenerate modlist.md files for a pack (and the projects index)", Destructive: true, Result: reflect.TypeOf(cmd.PagesResult{}), Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.packDir(r.PathValue("id"))
		if err != nil {
			return "", nil, err
		}
		return s.root, []string{"pages", "--pack", dir, "--json"}, nil
	}})
	Register(Action{Name: "docs.diff", Method: http.MethodGet, Path: Prefix + "/diff", Summary: "Diff mod additions, removals, and updates between two git refs", Result: reflect.TypeOf(cmd.DiffResult{}), Build: func(s *Server, r *http.Request) (string, []string, error) {
		oldRef, newRef := r.URL.Query().Get("old"), r.URL.Query().Get("new")
		if oldRef == "" || newRef == "" {
			return "", nil, errors.New("old and new query parameters are required")
		}
		args := []string{"diff", oldRef, newRef, "--json"}
		if prefix := r.URL.Query().Get("prefix"); prefix != "" {
			args = append(args, prefix)
		}
		return s.root, args, nil
	}})

	Register(Action{Name: "packs.bump", Method: http.MethodPost, Path: Prefix + "/packs/{id}/bump", Summary: "Bump the manifest version", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.packDir(r.PathValue("id"))
		if err != nil {
			return "", nil, err
		}
		var body struct {
			Version string `json:"version"`
			Configs bool   `json:"configs"`
		}
		if err := decodeBody(r, &body); err != nil {
			return "", nil, err
		}
		if body.Version == "" {
			return "", nil, errors.New("version is required")
		}
		args := []string{"bump", dir, body.Version}
		if body.Configs {
			args = append(args, "--configs")
		}
		return s.root, args, nil
	}})
	Register(Action{Name: "packs.freeze.list", Method: http.MethodGet, Path: Prefix + "/packs/{id}/freeze", Summary: "List frozen mods in a pack subdir", Result: reflect.TypeOf([]string{}), Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		return s.root, []string{"freeze", dir, "--json"}, nil
	}})
	Register(Action{Name: "packs.freeze.apply", Method: http.MethodPost, Path: Prefix + "/packs/{id}/freeze", Summary: "Freeze mods so updates skip them", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		slugs, err := decodeSlugs(r)
		if err != nil {
			return "", nil, err
		}
		return s.root, append([]string{"freeze", dir}, slugs...), nil
	}})
	Register(Action{Name: "packs.unfreeze", Method: http.MethodPost, Path: Prefix + "/packs/{id}/unfreeze", Summary: "Unfreeze mods so updates apply to them again", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.actionDir(r)
		if err != nil {
			return "", nil, err
		}
		slugs, err := decodeSlugs(r)
		if err != nil {
			return "", nil, err
		}
		return s.root, append([]string{"unfreeze", dir}, slugs...), nil
	}})

	Register(Action{Name: "packs.mods.side.get", Method: http.MethodGet, Path: Prefix + "/packs/{id}/mods/{slug}/side", Summary: "Show a mod's side across all subdirs in a pack", Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.packDir(r.PathValue("id"))
		if err != nil {
			return "", nil, err
		}
		return s.root, []string{"side", dir, r.PathValue("slug")}, nil
	}})
	Register(Action{Name: "packs.mods.side.set", Method: http.MethodPut, Path: Prefix + "/packs/{id}/mods/{slug}/side", Summary: "Set a mod's side across all subdirs in a pack", Destructive: true, Build: func(s *Server, r *http.Request) (string, []string, error) {
		dir, err := s.packDir(r.PathValue("id"))
		if err != nil {
			return "", nil, err
		}
		var body struct {
			Side string `json:"side"`
		}
		if err := decodeBody(r, &body); err != nil {
			return "", nil, err
		}
		if !validSideValues[body.Side] {
			return "", nil, errors.New("side must be one of: client, server, both, either")
		}
		return s.root, []string{"side", dir, r.PathValue("slug"), body.Side}, nil
	}})
}

var validSideValues = map[string]bool{"client": true, "server": true, "both": true, "either": true}

// decodeSlugs reads a {"slugs": [...]} request body and requires at least one entry.
func decodeSlugs(r *http.Request) ([]string, error) {
	var body struct {
		Slugs []string `json:"slugs"`
	}
	if err := decodeBody(r, &body); err != nil {
		return nil, err
	}
	if len(body.Slugs) == 0 {
		return nil, errors.New("slugs is required and must not be empty")
	}
	return body.Slugs, nil
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
		"docs-pages": {"pages"},
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
	case "bump":
		version := strings.TrimSpace(input.Version)
		if version == "" {
			return "", nil, errors.New("version is required")
		}
		args := []string{"bump", ".", version}
		if input.Configs {
			args = append(args, "--configs")
		}
		return dir, args, nil
	case "freeze-mod", "unfreeze-mod":
		if slug == "" {
			return "", nil, errors.New("slug is required")
		}
		return dir, []string{strings.TrimSuffix(input.Action, "-mod"), ".", slug}, nil
	case "set-side":
		if slug == "" {
			return "", nil, errors.New("slug is required")
		}
		side := strings.TrimSpace(input.Side)
		if !validSideValues[side] {
			return "", nil, errors.New("side must be one of: client, server, both, either")
		}
		return dir, []string{"side", ".", slug, side}, nil
	case "nix-gen":
		return dir, []string{"nix", "gen"}, nil
	case "docs-modlist":
		return dir, []string{"modlist", "."}, nil
	}
	return "", nil, errors.New("unknown action")
}
