package api

import "net/http"

func (s *Server) handleOpenAPI(w http.ResponseWriter, _ *http.Request) {
	paths := map[string]any{
		"/version":              map[string]any{"get": operation("Get API version", "200")},
		"/capabilities":         map[string]any{"get": operation("List capabilities", "200")},
		"/openapi.json":         map[string]any{"get": operation("Get this OpenAPI document", "200")},
		"/packs":                map[string]any{"get": operation("List packs", "200"), "post": operation("Create a pack", "201")},
		"/packs/{id}/manifest":  map[string]any{"get": operation("Read a manifest", "200"), "put": operation("Update a manifest", "200")},
		"/packs/{id}/changelog": map[string]any{"get": operation("Read a changelog", "200"), "put": operation("Update a changelog", "200")},
		"/packs/{id}/icon":      map[string]any{"get": operation("Read a pack icon", "200")},
		"/packs/{id}/mods":      map[string]any{"get": operation("List pack mods", "200")},
		"/packs/{id}/subdirs/{sub}/registry/{kind}":          map[string]any{"get": operation("Read a content registry (datapack, config, resourcepack, or kubejs)", "200")},
		"/packs/{id}/subdirs/{sub}/registry/{kind}/complete": map[string]any{"get": operation("Complete registry entries at a query or document position", "200")},
		"/packs/{id}/subdirs/{sub}/check":                    map[string]any{"post": operation("Check an editor buffer for structural and reference problems", "200")},
		"/packs/{id}/subdirs/{sub}/kubejs/runtime-log":       map[string]any{"post": operation("Parse a tailed KubeJS runtime log chunk into diagnostics", "200")},
		"/packs/{id}/subdirs/{sub}/ci-local":                 map[string]any{"post": operation("Run CI-equivalent validation stages for a subdir", "202")},
		"/packs/{id}/subdirs/{sub}/tree":                     map[string]any{"get": operation("Read the editor file tree grouped by content domain", "200")},
		"/packs/{id}/subdirs/{sub}/file":                     map[string]any{"get": operation("Read a file in a pack subdir", "200"), "put": operation("Save a file in a pack subdir", "200")},
		"/packs/{id}/subdirs/{sub}/files":                    map[string]any{"post": operation("Paste a new file or duplicate one from a sibling subdir", "201")},
		"/jobs":                                              map[string]any{"get": operation("List jobs", "200")},
		"/jobs/{id}":                                         map[string]any{"get": operation("Get a job", "200")},
		"/jobs/{id}/events":                                  map[string]any{"get": operation("Stream job events", "200")},
		"/webview/open":                                      map[string]any{"post": operation("Open the mod browser", "202")},
	}
	for _, action := range actions() {
		path := action.Path[len(Prefix):]
		method := lower(action.Method)
		operation := map[string]any{
			"operationId": action.Name,
			"summary":     action.Summary,
			"responses":   map[string]any{"202": map[string]any{"description": "Job accepted"}, "400": map[string]any{"description": "Invalid argument"}, "401": map[string]any{"description": "Unauthorized"}},
		}
		entry, _ := paths[path].(map[string]any)
		if entry == nil {
			entry = map[string]any{}
			paths[path] = entry
		}
		entry[method] = operation
	}
	writeJSON(w, map[string]any{
		"openapi":    "3.1.0",
		"info":       map[string]string{"title": "Packwand API", "version": "v1"},
		"servers":    []map[string]string{{"url": Prefix}},
		"paths":      paths,
		"components": map[string]any{"securitySchemes": map[string]any{"bearerAuth": map[string]string{"type": "http", "scheme": "bearer"}}},
	})
}

func lower(value string) string {
	out := make([]byte, len(value))
	for i, b := range []byte(value) {
		if b >= 'A' && b <= 'Z' {
			b += 'a' - 'A'
		}
		out[i] = b
	}
	return string(out)
}

func operation(summary, status string) map[string]any {
	return map[string]any{"summary": summary, "responses": map[string]any{status: map[string]any{"description": "Success"}, "401": map[string]any{"description": "Unauthorized"}}}
}
