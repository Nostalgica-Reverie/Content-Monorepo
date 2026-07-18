package api

import (
	"errors"
	"net/http"
	"os"
	"path/filepath"
	"reflect"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/content"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/registry"
)

// IDE editor services (IDE.md §4): buffer checking, the editor file tree,
// file read/write, structured paste/duplicate operations, and the preflight
// gate. Reads and checks are served in-process; preflight runs as a job so
// the gate streams progress like every other action.
func init() {
	Register(Action{
		Name:    "packs.preflight",
		Method:  http.MethodPost,
		Path:    Prefix + "/packs/{id}/subdirs/{sub}/preflight",
		Summary: "Run the pre-launch validation gate for a pack subdir",
		Result:  reflect.TypeOf(content.PreflightResult{}),
		Build: func(s *Server, r *http.Request) (string, []string, error) {
			dir, err := s.subdirDir(r)
			if err != nil {
				return "", nil, err
			}
			return dir, []string{"preflight", ".", "--json"}, nil
		},
	})
	Register(Action{
		Name:    "packs.local-ci",
		Method:  http.MethodPost,
		Path:    Prefix + "/packs/{id}/subdirs/{sub}/ci-local",
		Summary: "Run CI-equivalent validation stages for a pack subdir",
		Result:  reflect.TypeOf(content.LocalCIResult{}),
		Build: func(s *Server, r *http.Request) (string, []string, error) {
			dir, err := s.subdirDir(r)
			if err != nil {
				return "", nil, err
			}
			return dir, []string{"ci-local", ".", "--json"}, nil
		},
	})
}

const maxEditorFileSize = 2 << 20

// editableExtensions are the file types the IDE opens as text.
var editableExtensions = map[string]bool{
	".json": true, ".mcmeta": true, ".toml": true, ".properties": true,
	".cfg": true, ".txt": true, ".md": true, ".js": true, ".ts": true,
	".mcfunction": true, ".snbt": true, ".lang": true, ".json5": true,
}

// handleCheck validates an editor buffer without saving it (IDE.md §4.1).
func (s *Server) handleCheck(w http.ResponseWriter, r *http.Request) {
	dir, err := s.subdirDir(r)
	if err != nil {
		writeError(w, 404, "not_found", "pack or subdir not found", "sub")
		return
	}
	var request struct {
		File    string `json:"file"`
		Content string `json:"content"`
	}
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	if _, err := safeChild(dir, request.File); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "file")
		return
	}
	diagnostics := registry.CheckDocument(dir, request.File, []byte(request.Content))
	valid := true
	for _, diagnostic := range diagnostics {
		if diagnostic.Severity == "error" {
			valid = false
			break
		}
	}
	writeJSON(w, map[string]any{"file": request.File, "valid": valid, "diagnostics": diagnostics})
}

// handleKubeJSRuntimeLog accepts a launcher-tailed KubeJS log chunk and turns
// stack locations into Problems diagnostics. The Tauri shell may post each
// chunk as it arrives; no game log is written by the browser.
func (s *Server) handleKubeJSRuntimeLog(w http.ResponseWriter, r *http.Request) {
	if _, err := s.subdirDir(r); err != nil {
		writeError(w, 404, "not_found", "pack or subdir not found", "sub")
		return
	}
	var request struct {
		Content string `json:"content"`
	}
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	writeJSON(w, map[string]any{"diagnostics": registry.ParseKubeJSLog(request.Content)})
}

type treeFile struct {
	Path     string `json:"path"`
	RefID    string `json:"ref_id,omitempty"`
	Kind     string `json:"kind"`
	Owner    string `json:"owner,omitempty"`
	Editable bool   `json:"editable"`
}

type treeGroup struct {
	Name  string     `json:"name"`
	Files []treeFile `json:"files"`
}

// handleTree returns the editor's file tree, grouped by content domain
// (IDE.md §5). It is derived from the registries so tree entries carry the
// referenceable ID ("copy as reference", §4.3) alongside the file path.
func (s *Server) handleTree(w http.ResponseWriter, r *http.Request) {
	dir, err := s.subdirDir(r)
	if err != nil {
		writeError(w, 404, "not_found", "pack or subdir not found", "sub")
		return
	}
	groups := []treeGroup{packMetadataGroup(dir)}
	for _, source := range []struct {
		name string
		kind registry.Kind
	}{
		{"Configs", registry.Config},
		{"Datapacks", registry.Datapack},
		{"Resource packs", registry.ResourcePack},
		{"KubeJS", registry.KubeJS},
	} {
		reg, err := registry.Build(dir, source.kind)
		if err != nil {
			writeError(w, 500, "internal", err.Error(), "")
			return
		}
		group := treeGroup{Name: source.name, Files: []treeFile{}}
		for _, entry := range reg.Entries {
			if entry.Path == "" {
				continue
			}
			rel := entry.Path
			if entry.Origin != "." && entry.Origin != "" {
				rel = entry.Origin + "/" + entry.Path
			}
			group.Files = append(group.Files, treeFile{
				Path:     rel,
				RefID:    entry.ID,
				Kind:     entry.Kind,
				Owner:    entry.Owner,
				Editable: editableExtensions[strings.ToLower(filepath.Ext(rel))],
			})
		}
		sort.Slice(group.Files, func(i, j int) bool { return group.Files[i].Path < group.Files[j].Path })
		groups = append(groups, group)
	}
	writeJSON(w, map[string]any{"groups": groups})
}

func packMetadataGroup(dir string) treeGroup {
	group := treeGroup{Name: "Pack", Files: []treeFile{}}
	for _, name := range []string{"pack.toml", "index.toml", "pack.mcmeta"} {
		if _, err := os.Stat(filepath.Join(dir, name)); err == nil {
			group.Files = append(group.Files, treeFile{Path: name, Kind: "pack_metadata", Editable: true})
		}
	}
	return group
}

func (s *Server) handleReadFile(w http.ResponseWriter, r *http.Request) {
	dir, err := s.subdirDir(r)
	if err != nil {
		writeError(w, 404, "not_found", "pack or subdir not found", "sub")
		return
	}
	full, err := safeChild(dir, r.URL.Query().Get("path"))
	if err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "path")
		return
	}
	info, err := os.Stat(full)
	if err != nil {
		writeError(w, 404, "not_found", "file not found", "path")
		return
	}
	if info.IsDir() || info.Size() > maxEditorFileSize {
		writeError(w, 400, "invalid_argument", "path must be a file no larger than 2 MiB", "path")
		return
	}
	data, err := os.ReadFile(full)
	if err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	if strings.ContainsRune(string(data), 0) {
		writeError(w, 400, "invalid_argument", "file is binary and cannot be opened as text", "path")
		return
	}
	writeJSON(w, map[string]string{"path": r.URL.Query().Get("path"), "content": string(data)})
}

func (s *Server) handleWriteFile(w http.ResponseWriter, r *http.Request) {
	dir, err := s.subdirDir(r)
	if err != nil {
		writeError(w, 404, "not_found", "pack or subdir not found", "sub")
		return
	}
	var request struct {
		Path    string `json:"path"`
		Content string `json:"content"`
	}
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	full, err := safeChild(dir, request.Path)
	if err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "path")
		return
	}
	if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	if err := os.WriteFile(full, []byte(request.Content), 0o644); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	writeJSON(w, map[string]string{"status": "saved", "path": request.Path})
}

// handleCreateFile implements the structured paste/duplicate operations
// (IDE.md §4.3): paste new content into the pack, or copy a file from a
// sibling subdir of the same pack. Paths under data/ and assets/ are
// normalized to lowercase, matching the resource-location charset rules
// content-lint enforces.
func (s *Server) handleCreateFile(w http.ResponseWriter, r *http.Request) {
	dir, err := s.subdirDir(r)
	if err != nil {
		writeError(w, 404, "not_found", "pack or subdir not found", "sub")
		return
	}
	var request struct {
		Path      string `json:"path"`
		Content   string `json:"content"`
		FromSub   string `json:"from_sub,omitempty"`
		FromPath  string `json:"from_path,omitempty"`
		Overwrite bool   `json:"overwrite,omitempty"`
	}
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	content := request.Content
	if request.FromSub != "" {
		source, err := s.siblingSubdir(r, request.FromSub)
		if err != nil {
			writeError(w, 404, "not_found", "source subdir not found", "from_sub")
			return
		}
		fromPath := first(request.FromPath, request.Path)
		fullSource, err := safeChild(source, fromPath)
		if err != nil {
			writeError(w, 400, "invalid_argument", err.Error(), "from_path")
			return
		}
		data, err := os.ReadFile(fullSource)
		if err != nil {
			writeError(w, 404, "not_found", "source file not found", "from_path")
			return
		}
		content = string(data)
		if request.Path == "" {
			request.Path = fromPath
		}
	}
	normalized := normalizeContentPath(request.Path)
	full, err := safeChild(dir, normalized)
	if err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "path")
		return
	}
	if _, err := os.Stat(full); err == nil && !request.Overwrite {
		writeError(w, 409, "conflict", "file already exists (pass overwrite to replace it)", "path")
		return
	}
	if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	if err := os.WriteFile(full, []byte(content), 0o644); err != nil {
		writeError(w, 500, "internal", err.Error(), "")
		return
	}
	writeJSONStatus(w, http.StatusCreated, map[string]string{"status": "created", "path": normalized})
}

// siblingSubdir resolves another subdir of the pack in the current request.
func (s *Server) siblingSubdir(r *http.Request, sub string) (string, error) {
	dir, err := s.packDir(r.PathValue("id"))
	if err != nil {
		return "", err
	}
	for _, candidate := range manifest.SubDirsOf(dir) {
		if filepath.Base(candidate) == sub {
			return candidate, nil
		}
	}
	return "", errors.New("subdir not found")
}

// normalizeContentPath lowercases the path segments below data/ or assets/
// so pasted files respect the resource-location charset.
func normalizeContentPath(rel string) string {
	rel = strings.ReplaceAll(rel, "\\", "/")
	parts := strings.Split(rel, "/")
	normalize := false
	for i, part := range parts {
		if normalize {
			parts[i] = strings.ToLower(part)
			continue
		}
		if part == "data" || part == "assets" {
			normalize = true
		}
	}
	return strings.Join(parts, "/")
}
