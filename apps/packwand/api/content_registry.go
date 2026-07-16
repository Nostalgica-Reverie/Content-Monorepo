package api

import (
	"errors"
	"net/http"
	"os"
	"path/filepath"
	"reflect"
	"strconv"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/registry"
)

// Content registry routes (IDE.md §3 API surface). Reads are served
// in-process because building a registry is a local file walk; the rebuild
// endpoint runs as a job so clients get SSE progress and a capabilities
// entry like every other action.
func init() {
	Register(Action{
		Name:    "packs.registry.rebuild",
		Method:  http.MethodPost,
		Path:    Prefix + "/packs/{id}/subdirs/{sub}/registry/rebuild",
		Summary: "Rebuild all content registries for a pack subdir",
		Result:  reflect.TypeOf([]registry.Registry{}),
		Build: func(s *Server, r *http.Request) (string, []string, error) {
			dir, err := s.subdirDir(r)
			if err != nil {
				return "", nil, err
			}
			return dir, []string{"registry", "all", "--json"}, nil
		},
	})
}

// subdirDir resolves the {id}/{sub} pair of a registry route to a directory.
// sub is a subdir base name (e.g. "1.21-mr"); the literal "root" addresses
// the pack directory itself, for datapack/resourcepack packs that have no
// -mr/-cf subdirs.
func (s *Server) subdirDir(r *http.Request) (string, error) {
	dir, err := s.packDir(r.PathValue("id"))
	if err != nil {
		return "", err
	}
	sub := r.PathValue("sub")
	if sub == "root" {
		return dir, nil
	}
	for _, candidate := range manifest.SubDirsOf(dir) {
		if filepath.Base(candidate) == sub {
			return candidate, nil
		}
	}
	return "", os.ErrNotExist
}

func (s *Server) registryFor(w http.ResponseWriter, r *http.Request) (*registry.Registry, string, bool) {
	dir, err := s.subdirDir(r)
	if err != nil {
		writeError(w, 404, "not_found", "pack or subdir not found", "sub")
		return nil, "", false
	}
	kind, err := registry.ParseKind(r.PathValue("kind"))
	if err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "kind")
		return nil, "", false
	}
	reg, err := registry.Build(dir, kind)
	if err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return nil, "", false
	}
	reg.Scope = filepath.ToSlash(mustRel(s.root, dir))
	return reg, dir, true
}

func (s *Server) handleRegistry(w http.ResponseWriter, r *http.Request) {
	reg, _, ok := s.registryFor(w, r)
	if !ok {
		return
	}
	writeJSON(w, reg)
}

func (s *Server) handleRegistryComplete(w http.ResponseWriter, r *http.Request) {
	reg, dir, ok := s.registryFor(w, r)
	if !ok {
		return
	}
	params := r.URL.Query()
	query := params.Get("q")
	var kinds []string
	if raw := strings.TrimSpace(params.Get("kinds")); raw != "" {
		for _, kind := range strings.Split(raw, ",") {
			if kind = strings.TrimSpace(kind); kind != "" {
				kinds = append(kinds, kind)
			}
		}
	}
	limit, _ := strconv.Atoi(params.Get("limit"))

	// With no explicit query, derive the token and kind filter from the
	// document position the editor is completing at.
	if query == "" && params.Get("file") != "" {
		full, err := safeChild(dir, params.Get("file"))
		if err != nil {
			writeError(w, 400, "invalid_argument", err.Error(), "file")
			return
		}
		offset, err := strconv.Atoi(params.Get("offset"))
		if err != nil {
			writeError(w, 400, "invalid_argument", "offset must be an integer byte offset", "offset")
			return
		}
		inferredQuery, inferredKinds, err := registry.InferFromFile(full, offset)
		if err != nil {
			writeError(w, 400, "invalid_argument", err.Error(), "file")
			return
		}
		query = inferredQuery
		if len(kinds) == 0 {
			kinds = inferredKinds
		}
	}

	items := reg.Complete(query, kinds, limit)
	writeJSON(w, map[string]any{
		"query":            query,
		"kinds":            kinds,
		"items":            items,
		"registry_version": reg.Version,
	})
}

// safeChild resolves a slash-relative file path inside dir, rejecting
// absolute paths and traversal outside dir.
func safeChild(dir, rel string) (string, error) {
	if rel == "" || filepath.IsAbs(rel) {
		return "", errors.New("file must be a relative path inside the subdir")
	}
	full := filepath.Join(dir, filepath.Clean(filepath.FromSlash(rel)))
	back, err := filepath.Rel(dir, full)
	if err != nil || back == ".." || strings.HasPrefix(back, ".."+string(filepath.Separator)) {
		return "", errors.New("file must stay inside the subdir")
	}
	return full, nil
}
