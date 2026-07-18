// Package registry builds indexed, queryable models of everything
// referenceable within a pack directory (IDE.md §3): datapack resource
// locations, mod config files, resource pack assets, and KubeJS scripts.
// Registries back the IDE's autocomplete, type checking, and reference
// validation; they index vanilla-independent, pack-local sources only
// (vanilla per-version data and mod-jar scanning are future sources, see
// IDE.md §9.1).
package registry

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"hash"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// Kind selects which registry to build for a pack directory.
type Kind string

const (
	Datapack     Kind = "datapack"
	Config       Kind = "config"
	ResourcePack Kind = "resourcepack"
	KubeJS       Kind = "kubejs"
)

// Kinds returns every registry kind in build order.
func Kinds() []Kind { return []Kind{Datapack, Config, ResourcePack, KubeJS} }

// ParseKind resolves a user- or API-supplied kind name.
func ParseKind(value string) (Kind, error) {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "datapack":
		return Datapack, nil
	case "config":
		return Config, nil
	case "resourcepack", "rp":
		return ResourcePack, nil
	case "kubejs":
		return KubeJS, nil
	}
	return "", fmt.Errorf("unknown registry kind %q (want datapack, config, resourcepack, or kubejs)", value)
}

// Entry is one referenceable item: a resource location, a config file, a
// script, or a completion seed such as a KubeJS event name.
type Entry struct {
	// ID is the referenceable identifier: "ns:path" for resource locations,
	// a slash-relative file path for configs and scripts, an event name for
	// KubeJS events, or a mod slug.
	ID string `json:"id"`
	// Kind classifies the entry within its registry, e.g. "function",
	// "tag/function", "texture", "config_file", "script/server", "event/server".
	Kind string `json:"kind"`
	// Origin is the source root the entry came from, slash-relative to the
	// registry scope ("." for the scope itself, "builtin" for embedded data).
	Origin string `json:"origin"`
	// Path is the file path slash-relative to the origin root, when the entry
	// is backed by a file.
	Path string `json:"path,omitempty"`
	// Owner is the mod slug an entry belongs to, when it can be determined
	// (config registry only; empty means unowned/orphaned).
	Owner string `json:"owner,omitempty"`
	// SchemaRef points at the JSON Schema for the entry's document shape.
	// Reserved for the vendored per-pack-format schemas (IDE.md §3.1).
	SchemaRef string `json:"schema_ref,omitempty"`
}

// Registry indexes everything of one kind referenceable within a scope
// directory (IDE.md §3). Entries are sorted by ID for stable output, and
// Version is a content hash usable for cache invalidation.
type Registry struct {
	Scope   string   `json:"scope"`
	Kind    Kind     `json:"kind"`
	Version string   `json:"version"`
	Sources []string `json:"sources"`
	Entries []Entry  `json:"entries"`
}

// Build indexes dir for the given kind. dir may be a modpack subdir
// (pack.toml alongside config/, global_packs/, kubejs/, mods/) or a
// standalone datapack/resourcepack pack directory, whose content roots are
// located the same way content-lint finds them.
func Build(dir string, kind Kind) (*Registry, error) {
	info, err := os.Stat(dir)
	if err != nil {
		return nil, fmt.Errorf("registry scope %s: %w", dir, err)
	}
	if !info.IsDir() {
		return nil, fmt.Errorf("registry scope %s is not a directory", dir)
	}
	b := newBuilder(dir, kind)
	switch kind {
	case Datapack:
		buildDatapack(dir, b)
	case Config:
		buildConfig(dir, b)
	case ResourcePack:
		buildResourcePack(dir, b)
	case KubeJS:
		buildKubeJS(dir, b)
	default:
		return nil, fmt.Errorf("unknown registry kind %q", kind)
	}
	return b.finish(), nil
}

// BuildAll builds every registry kind for dir.
func BuildAll(dir string) ([]Registry, error) {
	out := make([]Registry, 0, len(Kinds()))
	for _, kind := range Kinds() {
		reg, err := Build(dir, kind)
		if err != nil {
			return nil, err
		}
		out = append(out, *reg)
	}
	return out, nil
}

// sourceRoot is one directory contributing entries, e.g. a bundled datapack
// under global_packs/ or the kubejs/ virtual pack.
type sourceRoot struct {
	dir    string // absolute path
	origin string // slash path relative to the registry scope
}

type builder struct {
	reg  *Registry
	hash hash.Hash
}

func newBuilder(dir string, kind Kind) *builder {
	return &builder{
		reg:  &Registry{Scope: filepath.ToSlash(dir), Kind: kind, Sources: []string{}, Entries: []Entry{}},
		hash: sha256.New(),
	}
}

func (b *builder) add(entry Entry) { b.reg.Entries = append(b.reg.Entries, entry) }

// walkSource visits every regular file under src.dir, feeding file metadata
// into the version hash and passing slash-relative paths to visit.
func (b *builder) walkSource(src sourceRoot, visit func(rel string, info fs.FileInfo)) {
	b.reg.Sources = append(b.reg.Sources, src.origin)
	_ = filepath.WalkDir(src.dir, func(p string, d fs.DirEntry, err error) error {
		if err != nil || d.IsDir() {
			return nil
		}
		rel, err := filepath.Rel(src.dir, p)
		if err != nil {
			return nil
		}
		info, err := d.Info()
		if err != nil {
			return nil
		}
		slashRel := filepath.ToSlash(rel)
		fmt.Fprintf(b.hash, "%s|%s|%d|%d\n", src.origin, slashRel, info.Size(), info.ModTime().UnixNano())
		visit(slashRel, info)
		return nil
	})
}

func (b *builder) finish() *Registry {
	entries := b.reg.Entries
	sort.Slice(entries, func(i, j int) bool {
		if entries[i].ID != entries[j].ID {
			return entries[i].ID < entries[j].ID
		}
		if entries[i].Kind != entries[j].Kind {
			return entries[i].Kind < entries[j].Kind
		}
		return entries[i].Origin < entries[j].Origin
	})
	for _, e := range entries {
		fmt.Fprintf(b.hash, "%s|%s|%s|%s\n", e.ID, e.Kind, e.Origin, e.Path)
	}
	b.reg.Version = hex.EncodeToString(b.hash.Sum(nil))
	return b.reg
}

// isModpackSubdir reports whether dir is a packwiz-style modpack subdir
// (e.g. modpacks/<pack>/1.20.1-mr) rather than a standalone content pack.
func isModpackSubdir(dir string) bool { return fileExists(filepath.Join(dir, "pack.toml")) }

func fileExists(path string) bool {
	info, err := os.Stat(path)
	return err == nil && !info.IsDir()
}

func dirExists(path string) bool {
	info, err := os.Stat(path)
	return err == nil && info.IsDir()
}

// contentRoots locates the roots holding <top>/ (data or assets) inside a
// standalone pack directory: at the pack root, or nested one level inside
// version directories — mirroring content-lint's findContentRoots.
func contentRoots(dir, top string) []sourceRoot {
	if dirExists(filepath.Join(dir, top)) || fileExists(filepath.Join(dir, "pack.mcmeta")) {
		return []sourceRoot{{dir: dir, origin: "."}}
	}
	var out []sourceRoot
	entries, _ := os.ReadDir(dir)
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		sub := filepath.Join(dir, e.Name())
		if dirExists(filepath.Join(sub, top)) || fileExists(filepath.Join(sub, "pack.mcmeta")) {
			out = append(out, sourceRoot{dir: sub, origin: e.Name()})
		}
	}
	return out
}

// childPacks returns the children of dir/<base> that contain <top>/ — the
// bundled packs under global_packs/ groups or a subdir's resourcepacks/.
func childPacks(dir, base, top string) []sourceRoot {
	var out []sourceRoot
	entries, _ := os.ReadDir(filepath.Join(dir, filepath.FromSlash(base)))
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		child := filepath.Join(dir, filepath.FromSlash(base), e.Name())
		if dirExists(filepath.Join(child, top)) {
			out = append(out, sourceRoot{dir: child, origin: base + "/" + e.Name()})
		}
	}
	return out
}

// modSlugs lists the mod slugs of a modpack subdir from mods/*.pw.toml names.
func modSlugs(dir string) []string {
	entries, _ := os.ReadDir(filepath.Join(dir, "mods"))
	var out []string
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".pw.toml") {
			continue
		}
		out = append(out, strings.TrimSuffix(e.Name(), ".pw.toml"))
	}
	sort.Strings(out)
	return out
}

// joinSlash joins dir with a slash-separated relative path.
func joinSlash(dir, rel string) string { return filepath.Join(dir, filepath.FromSlash(rel)) }

// stripExt removes the final extension from a slash path.
func stripExt(p string) string {
	if ext := filepath.Ext(p); ext != "" {
		return strings.TrimSuffix(p, ext)
	}
	return p
}
