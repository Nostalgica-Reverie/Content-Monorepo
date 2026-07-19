package registry

import (
	"encoding/json"
	"fmt"
	"path"
	"slices"
	"strings"

	"github.com/BurntSushi/toml"
)

// Diagnostic is one problem found in a document buffer (IDE.md §4.1).
// Line and Col are 1-based; semantic diagnostics point at the first
// occurrence of the offending value when it can be located.
type Diagnostic struct {
	Severity string `json:"severity"` // "error" or "warning"
	Line     int    `json:"line"`
	Col      int    `json:"col"`
	Message  string `json:"message"`
	Code     string `json:"code,omitempty"` // "syntax", "structure", or "reference"
}

// CheckDocument validates a document buffer as if it lived at rel (a slash
// path relative to the scope dir), so editors can check unsaved content.
// Checks are two-tier (IDE.md §4.1): structural (shape of the document for
// its kind) and referential (IDs must exist in the relevant registry).
// Referential checks skip vanilla and unknown namespaces, like content-lint.
func CheckDocument(dir, rel string, content []byte) []Diagnostic {
	return NewDocCheckSession(dir).CheckDocument(rel, content)
}

// DocCheckSession caches the lazily-built registries across many document
// checks against the same scope dir. Callers looping over a pack's files
// (e.g. preflight's reference step) should create one session for the loop
// instead of calling the package-level CheckDocument per file, which would
// rebuild the full registry (a filesystem walk) for every document.
type DocCheckSession struct {
	cache *registryCache
}

// NewDocCheckSession creates a session scoped to dir. It must not outlive the
// operation it serves: registries are built once and never invalidated.
func NewDocCheckSession(dir string) *DocCheckSession {
	return &DocCheckSession{cache: &registryCache{dir: dir}}
}

// CheckDocument validates one document buffer as if it lived at rel, reusing
// registries already built by earlier calls on this session.
func (s *DocCheckSession) CheckDocument(rel string, content []byte) []Diagnostic {
	rel = strings.TrimPrefix(path.Clean(strings.ReplaceAll(rel, "\\", "/")), "./")
	switch {
	case strings.HasSuffix(rel, ".json") || strings.HasSuffix(rel, ".mcmeta"):
		return checkJSONDocument(s.cache, rel, content)
	case strings.HasSuffix(rel, ".toml"):
		return checkTOMLDocument(content)
	case strings.HasSuffix(rel, ".js") || strings.HasSuffix(rel, ".ts"):
		return checkKubeJSScript(s.cache, rel, content)
	case strings.HasSuffix(rel, ".mcfunction"):
		return checkFunctionDocument(s.cache, rel, content)
	}
	return []Diagnostic{}
}

func checkTOMLDocument(content []byte) []Diagnostic {
	var value any
	if err := toml.Unmarshal(content, &value); err != nil {
		diag := Diagnostic{Severity: "error", Line: 1, Col: 1, Message: "invalid TOML: " + err.Error(), Code: "syntax"}
		var parseErr toml.ParseError
		if pe, ok := err.(toml.ParseError); ok {
			parseErr = pe
		}
		if parseErr.Position.Line > 0 {
			diag.Line = parseErr.Position.Line
			diag.Message = "invalid TOML: " + parseErr.Message
		}
		return []Diagnostic{diag}
	}
	return []Diagnostic{}
}

func checkJSONDocument(cache *registryCache, rel string, content []byte) []Diagnostic {
	var document any
	if err := json.Unmarshal(content, &document); err != nil {
		line, col := 1, 1
		if syntaxErr, ok := err.(*json.SyntaxError); ok {
			line, col = offsetToLineCol(content, int(syntaxErr.Offset))
		}
		return []Diagnostic{{Severity: "error", Line: line, Col: col, Message: "invalid JSON: " + err.Error(), Code: "syntax"}}
	}

	checker := &docChecker{registryCache: cache, content: content, diags: []Diagnostic{}}
	base := path.Base(rel)
	dataRel, inData := subPathFrom(rel, "data")
	assetRel, inAssets := subPathFrom(rel, "assets")
	switch {
	case base == "pack.mcmeta":
		checker.checkPackMcmeta(document)
	case inData:
		checker.checkDataDocument(dataRel, document)
	case inAssets:
		checker.checkAssetDocument(assetRel, document)
	}
	return checker.diags
}

// subPathFrom returns the slash path starting at the first path segment
// equal to top ("data" or "assets"), so nested content roots like
// global_packs/required_data/X/data/ns/... classify the same as data/ns/...
func subPathFrom(rel, top string) (string, bool) {
	parts := strings.Split(rel, "/")
	for i, part := range parts {
		if part == top && i < len(parts)-1 {
			return strings.Join(parts[i:], "/"), true
		}
	}
	return "", false
}

// registryCache lazily builds and holds the per-kind registries for a scope
// dir. It is shared across every document checked by one DocCheckSession, so
// the full-tree walk happens at most once per kind per session.
type registryCache struct {
	dir          string
	datapack     *Registry
	resourcepack *Registry
	kubejs       *Registry
}

type docChecker struct {
	*registryCache
	content []byte

	diags []Diagnostic
}

func (c *docChecker) errorf(code, locate string, format string, a ...any) {
	c.report("error", code, locate, format, a...)
}

func (c *docChecker) warnf(code, locate string, format string, a ...any) {
	c.report("warning", code, locate, format, a...)
}

// report appends a diagnostic anchored at the first occurrence of locate
// (usually the offending value) in the buffer, or 1:1 when absent.
func (c *docChecker) report(severity, code, locate string, format string, a ...any) {
	line, col := 1, 1
	if locate != "" {
		if offset := strings.Index(string(c.content), `"`+locate+`"`); offset >= 0 {
			line, col = offsetToLineCol(c.content, offset)
		}
	}
	c.diags = append(c.diags, Diagnostic{Severity: severity, Line: line, Col: col, Message: fmt.Sprintf(format, a...), Code: code})
}

func offsetToLineCol(content []byte, offset int) (int, int) {
	if offset > len(content) {
		offset = len(content)
	}
	line, col := 1, 1
	for _, b := range content[:offset] {
		if b == '\n' {
			line++
			col = 1
			continue
		}
		col++
	}
	return line, col
}

func (c *registryCache) datapackRegistry() *Registry {
	if c.datapack == nil {
		reg, err := Build(c.dir, Datapack)
		if err != nil {
			reg = &Registry{Entries: []Entry{}}
		}
		c.datapack = reg
	}
	return c.datapack
}

func (c *registryCache) resourcepackRegistry() *Registry {
	if c.resourcepack == nil {
		reg, err := Build(c.dir, ResourcePack)
		if err != nil {
			reg = &Registry{Entries: []Entry{}}
		}
		c.resourcepack = reg
	}
	return c.resourcepack
}

func (c *registryCache) kubejsRegistry() (*Registry, error) {
	if c.kubejs == nil {
		reg, err := Build(c.dir, KubeJS)
		if err != nil {
			return nil, err
		}
		c.kubejs = reg
	}
	return c.kubejs, nil
}

// has reports whether the registry contains id with any of the given kinds.
func has(reg *Registry, id string, kinds ...string) bool {
	for _, entry := range reg.Entries {
		if entry.ID != id {
			continue
		}
		if slices.Contains(kinds, entry.Kind) {
			return true
		}
	}
	return false
}

// namespaces returns every namespace the registry ships entries for, so
// references into namespaces we cannot see (vanilla, mods) are skipped.
func namespaces(reg *Registry) map[string]bool {
	out := map[string]bool{}
	for _, entry := range reg.Entries {
		if i := strings.IndexByte(entry.ID, ':'); i > 0 {
			out[entry.ID[:i]] = true
		}
	}
	return out
}

func splitRef(ref string) (string, string) {
	if ns, rest, found := strings.Cut(ref, ":"); found {
		return ns, rest
	}
	return "minecraft", ref
}

func (c *docChecker) checkPackMcmeta(document any) {
	object, ok := document.(map[string]any)
	if !ok {
		c.errorf("structure", "", "pack.mcmeta must be a JSON object")
		return
	}
	pack, ok := object["pack"].(map[string]any)
	if !ok {
		c.errorf("structure", "", "missing 'pack' object")
		return
	}
	if _, ok := pack["pack_format"].(float64); !ok {
		c.errorf("structure", "pack", "missing or non-numeric 'pack.pack_format'")
	}
	if _, ok := pack["description"]; !ok {
		c.warnf("structure", "pack", "missing 'pack.description'")
	}
}

func (c *docChecker) checkDataDocument(dataRel string, document any) {
	entry, ok := classifyData(dataRel)
	if !ok {
		return
	}
	switch {
	case entry.Kind == "tag/function":
		c.checkFunctionTag(document)
	case strings.HasPrefix(entry.Kind, "tag/"):
		c.checkTagShape(document)
	case entry.Kind == "advancement":
		c.checkAdvancement(document)
	case entry.Kind == "recipe":
		c.checkRecipe(document)
	case entry.Kind == "predicate" || entry.Kind == "loot_table":
		if _, ok := document.(map[string]any); !ok {
			c.errorf("structure", "", "%s must be a JSON object", entry.Kind)
		}
	}
}

// tagValues extracts the entries of a tag document's values array, each as
// (reference, required).
func (c *docChecker) tagValues(document any) ([]struct {
	ref      string
	required bool
}, bool) {
	object, ok := document.(map[string]any)
	if !ok {
		c.errorf("structure", "", "tag must be a JSON object")
		return nil, false
	}
	rawValues, ok := object["values"].([]any)
	if !ok {
		c.errorf("structure", "", "tag has no 'values' array")
		return nil, false
	}
	out := make([]struct {
		ref      string
		required bool
	}, 0, len(rawValues))
	for _, value := range rawValues {
		ref, required := "", true
		switch typed := value.(type) {
		case string:
			ref = typed
		case map[string]any:
			ref, _ = typed["id"].(string)
			if req, ok := typed["required"].(bool); ok {
				required = req
			}
		default:
			c.errorf("structure", "", "tag value has unexpected type %T", value)
			continue
		}
		if ref == "" {
			c.errorf("structure", "", "tag value is missing an id")
			continue
		}
		out = append(out, struct {
			ref      string
			required bool
		}{ref, required})
	}
	return out, true
}

func (c *docChecker) checkTagShape(document any) {
	c.tagValues(document)
}

func (c *docChecker) checkFunctionTag(document any) {
	values, ok := c.tagValues(document)
	if !ok {
		return
	}
	reg := c.datapackRegistry()
	known := namespaces(reg)
	for _, value := range values {
		isTag := strings.HasPrefix(value.ref, "#")
		ns, _ := splitRef(strings.TrimPrefix(value.ref, "#"))
		if !known[ns] {
			continue
		}
		id := strings.TrimPrefix(value.ref, "#")
		resolved := false
		if isTag {
			resolved = has(reg, id, "tag/function")
		} else {
			resolved = has(reg, id, "function")
		}
		if !resolved && value.required {
			kind := "function"
			if isTag {
				kind = "function tag"
			}
			c.errorf("reference", strings.TrimPrefix(value.ref, "#"), "missing %s reference %q", kind, value.ref)
		}
	}
}

func (c *docChecker) checkAdvancement(document any) {
	object, ok := document.(map[string]any)
	if !ok {
		c.errorf("structure", "", "advancement must be a JSON object")
		return
	}
	if _, ok := object["criteria"].(map[string]any); !ok {
		c.errorf("structure", "", "advancement has no 'criteria' object")
	}
	reg := c.datapackRegistry()
	known := namespaces(reg)
	if parent, ok := object["parent"].(string); ok && parent != "" {
		ns, _ := splitRef(parent)
		if known[ns] && !has(reg, parent, "advancement") {
			c.errorf("reference", parent, "missing advancement reference %q", parent)
		}
	}
}

func (c *docChecker) checkRecipe(document any) {
	object, ok := document.(map[string]any)
	if !ok {
		c.errorf("structure", "", "recipe must be a JSON object")
		return
	}
	if kind, ok := object["type"].(string); !ok || kind == "" {
		c.errorf("structure", "", "recipe has no 'type'")
	}
}

func (c *docChecker) checkAssetDocument(assetRel string, document any) {
	entry, ok := classifyAsset(assetRel)
	if !ok {
		return
	}
	switch entry.Kind {
	case "model":
		c.checkModel(document)
	case "blockstate":
		c.checkBlockstate(document)
	case "lang":
		c.checkLang(document)
	}
}

func (c *docChecker) checkAssetRef(ref, kind string) {
	if ref == "" || strings.HasPrefix(ref, "#") {
		return
	}
	reg := c.resourcepackRegistry()
	ns, _ := splitRef(ref)
	if ns == "minecraft" || !namespaces(reg)[ns] {
		return
	}
	if !has(reg, ref, kind) {
		c.errorf("reference", ref, "missing %s reference %q", kind, ref)
	}
}

func (c *docChecker) checkModel(document any) {
	object, ok := document.(map[string]any)
	if !ok {
		c.errorf("structure", "", "model must be a JSON object")
		return
	}
	if parent, ok := object["parent"].(string); ok {
		c.checkAssetRef(parent, "model")
	}
	if textures, ok := object["textures"].(map[string]any); ok {
		for slot, value := range textures {
			texture, ok := value.(string)
			if !ok {
				c.errorf("structure", slot, "texture slot %q must be a string", slot)
				continue
			}
			c.checkAssetRef(texture, "texture")
		}
	}
}

func (c *docChecker) checkBlockstate(document any) {
	object, ok := document.(map[string]any)
	if !ok {
		c.errorf("structure", "", "blockstate must be a JSON object")
		return
	}
	check := func(apply any) {
		switch typed := apply.(type) {
		case map[string]any:
			if model, ok := typed["model"].(string); ok {
				c.checkAssetRef(model, "model")
			}
		case []any:
			for _, alternative := range typed {
				if variant, ok := alternative.(map[string]any); ok {
					if model, ok := variant["model"].(string); ok {
						c.checkAssetRef(model, "model")
					}
				}
			}
		}
	}
	if variants, ok := object["variants"].(map[string]any); ok {
		for _, variant := range variants {
			check(variant)
		}
	} else if multipart, ok := object["multipart"].([]any); ok {
		for _, part := range multipart {
			if typed, ok := part.(map[string]any); ok {
				check(typed["apply"])
			}
		}
	} else {
		c.errorf("structure", "", "blockstate has neither 'variants' nor 'multipart'")
	}
}

func (c *docChecker) checkLang(document any) {
	object, ok := document.(map[string]any)
	if !ok {
		c.errorf("structure", "", "lang file must be a JSON object")
		return
	}
	for key, value := range object {
		if _, ok := value.(string); !ok {
			c.errorf("structure", key, "lang value for %q must be a string", key)
		}
	}
}
