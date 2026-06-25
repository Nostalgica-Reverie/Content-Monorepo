package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"time"
)

type indexVariant struct {
	ID        string `json:"id,omitempty"`
	MCVersion string `json:"mc_version"`
	Loader    string `json:"loader,omitempty"`
}

type indexEntry struct {
	ID           string         `json:"id"`
	Name         string         `json:"name"`
	Type         string         `json:"type"`
	Loader       string         `json:"loader,omitempty"`
	MCVersion    string         `json:"mc_version,omitempty"`
	Version      string         `json:"version,omitempty"`
	ReleaseType  string         `json:"release_type,omitempty"`
	Description  string         `json:"description,omitempty"`
	ModrinthID   string         `json:"modrinth_id,omitempty"`
	CurseforgeID string         `json:"curseforge_id,omitempty"`
	DocsPath     string         `json:"docs_path,omitempty"`
	Variants     []indexVariant `json:"variants,omitempty"`
}

type indexFile struct {
	Generated string       `json:"generated"`
	Projects  []indexEntry `json:"projects"`
}

func projectsIndexOutPath() string {
	if p := os.Getenv("PROJECTS_INDEX_OUT"); p != "" {
		return p
	}
	return filepath.Join("docs", "docs", "public", "projects.json")
}

func docsPathFor(typ, id string) string {
	switch typ {
	case "modpack":
		return "/modpacks/" + id + "/"
	case "datapack":
		return "/datapacks/" + id + "/"
	case "resourcepack":
		return "/resource-packs/" + id + "/"
	}
	return ""
}

func writeCategoryIndexes(entries []indexEntry) {
	byCat := map[string][]indexEntry{}
	for _, e := range entries {
		cat := e.Type + "s"
		byCat[cat] = append(byCat[cat], e)
	}
	for cat, list := range byCat {
		if _, err := os.Stat(cat); err != nil {
			continue
		}
		out := filepath.Join(cat, "Project.json")
		writeJSON(out, map[string]any{
			"_generated": "Used by Somnus do not touch pls thx",
			"generated":  time.Now().UTC().Format(time.RFC3339),
			"projects":   list,
		})
		fmt.Printf("wrote %s (%d project(s))\n", out, len(list))
	}
}

func writeProjectsIndex() (int, error) {
	var entries []indexEntry
	seen := map[string]bool{}

	for _, root := range []string{"modpacks", "datapacks", "resourcepacks"} {
		packs, err := os.ReadDir(root)
		if err != nil {
			continue
		}
		for _, p := range packs {
			if !p.IsDir() {
				continue
			}
			path := filepath.Join(root, p.Name(), "manifest.json")
			m, err := ReadManifest(path)
			if err != nil {
				fmt.Fprintf(os.Stderr, "::warning::index: %v\n", err)
				continue
			}
			if m.ID == "" || m.Name == "" || seen[m.ID] {
				continue
			}
			seen[m.ID] = true

			e := indexEntry{
				ID:           m.ID,
				Name:         m.Name,
				Type:         m.Type,
				Version:      m.Version,
				ReleaseType:  m.ReleaseType,
				Description:  m.Description,
				ModrinthID:   m.ModrinthID,
				CurseforgeID: m.CurseforgeID,
				DocsPath:     docsPathFor(m.Type, m.ID),
			}
			if len(m.Variants) > 0 {
				for _, v := range m.Variants {
					loader := v.Loader
					if loader == "" {
						loader = m.Loader
					}
					e.Variants = append(e.Variants, indexVariant{
						ID:        v.ID,
						MCVersion: v.MCVersion,
						Loader:    loader,
					})
				}
			} else {
				if m.MCVersion != nil {
					e.MCVersion = *m.MCVersion
				}
				e.Loader = m.Loader
			}
			entries = append(entries, e)
		}
	}

	sort.Slice(entries, func(i, j int) bool {
		if entries[i].Type != entries[j].Type {
			return entries[i].Type < entries[j].Type
		}
		return entries[i].Name < entries[j].Name
	})

	out := projectsIndexOutPath()
	if err := os.MkdirAll(filepath.Dir(out), 0o755); err != nil {
		return 0, fmt.Errorf("creating %s: %w", filepath.Dir(out), err)
	}
	writeJSON(out, indexFile{
		Generated: time.Now().UTC().Format(time.RFC3339),
		Projects:  entries,
	})
	fmt.Printf("wrote %s (%d project(s))\n", out, len(entries))
	writeCategoryIndexes(entries)
	return len(entries), nil
}
