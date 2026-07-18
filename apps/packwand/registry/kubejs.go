package registry

import (
	_ "embed"
	"encoding/json"
	"io/fs"
	"strings"
)

// kubejsEventsJSON seeds event-name completion (IDE.md §3.4). It is a curated
// subset of KubeJS 6 (Minecraft 1.20.1) events; per-version ProbeJS-style
// type definitions are a future source.
//
//go:embed schemas/kubejs/events.json
var kubejsEventsJSON []byte

type kubejsEvent struct {
	Name string `json:"name"`
	Type string `json:"type"` // startup, server, or client
}

// buildKubeJS indexes the kubejs/ tree of a modpack subdir: scripts by folder
// discipline, exported type dumps, the embedded event-name seeds, and the
// pack's mod slugs for dependency awareness (Platform.isLoaded completion).
// Directories without kubejs/ produce an empty registry.
func buildKubeJS(dir string, b *builder) {
	root := joinSlash(dir, "kubejs")
	if !dirExists(root) {
		return
	}
	for _, scriptType := range []string{"startup", "server", "client"} {
		folder := scriptType + "_scripts"
		scripts := joinSlash(root, folder)
		if !dirExists(scripts) {
			continue
		}
		scriptType := scriptType
		b.walkSource(sourceRoot{dir: scripts, origin: "kubejs/" + folder}, func(rel string, _ fs.FileInfo) {
			if strings.HasSuffix(rel, ".js") || strings.HasSuffix(rel, ".ts") {
				b.add(Entry{ID: rel, Kind: "script/" + scriptType, Origin: "kubejs/" + folder, Path: rel})
			}
		})
	}
	if exported := joinSlash(root, "exported"); dirExists(exported) {
		b.walkSource(sourceRoot{dir: exported, origin: "kubejs/exported"}, func(rel string, _ fs.FileInfo) {
			b.add(Entry{ID: rel, Kind: "type_dump", Origin: "kubejs/exported", Path: rel})
		})
	}
	if exports, err := ProbeJSExports(dir); err == nil {
		for _, exported := range exports {
			for _, symbol := range exported.Symbols {
				b.add(Entry{ID: symbol, Kind: "type/symbol", Origin: exported.Path})
			}
		}
	}

	var seeds struct {
		Events []kubejsEvent `json:"events"`
	}
	if err := json.Unmarshal(kubejsEventsJSON, &seeds); err == nil {
		b.reg.Sources = append(b.reg.Sources, "builtin")
		for _, event := range seeds.Events {
			b.add(Entry{ID: event.Name, Kind: "event/" + event.Type, Origin: "builtin"})
		}
	}

	if slugs := modSlugs(dir); len(slugs) > 0 {
		b.reg.Sources = append(b.reg.Sources, "mods")
		for _, slug := range slugs {
			b.add(Entry{ID: slug, Kind: "mod", Origin: "mods"})
		}
	}
}
