package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type packRef struct {
	Category string
	Dir      string
	ID       string
	M        *Manifest
}

func cmdPacks(args []string) {
	asJSON := false
	var rest []string
	for _, a := range args {
		if a == "--json" {
			asJSON = true
		} else {
			rest = append(rest, a)
		}
	}
	if len(rest) == 0 {
		failUsage(verbUsage["packs"])
	}
	switch rest[0] {
	case "list":
		packsList(asJSON)
	case "get":
		if len(rest) < 2 {
			failUsage(verbUsage["packs"])
		}
		field := ""
		if len(rest) > 2 {
			field = rest[2]
		}
		packsGet(rest[1], field)
	case "set":
		if len(rest) < 4 {
			failUsage(verbUsage["packs"])
		}
		packsSet(rest[1], rest[2], rest[3])
	case "index":
		if _, err := writeProjectsIndex(); err != nil {
			fail(fmt.Sprintf("index generation failed: %v", err))
		}
	default:
		failUsage(fmt.Sprintf("unknown packs subcommand %q\n%s", rest[0], verbUsage["packs"]))
	}
}

func loadAllPacks() []packRef {
	var out []packRef
	for _, cat := range []string{"modpacks", "datapacks", "resourcepacks"} {
		entries, err := os.ReadDir(cat)
		if err != nil {
			continue
		}
		for _, e := range entries {
			if !e.IsDir() {
				continue
			}
			dir := filepath.Join(cat, e.Name())
			m, err := ReadManifest(filepath.Join(dir, "manifest.json"))
			if err != nil {
				warnf("packs: %v; skipped", err)
				continue
			}
			id := m.ID
			if id == "" {
				id = e.Name()
			}
			out = append(out, packRef{Category: cat, Dir: dir, ID: id, M: m})
		}
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Category != out[j].Category {
			return out[i].Category < out[j].Category
		}
		return out[i].ID < out[j].ID
	})
	return out
}

func findPack(id string) packRef {
	packs := loadAllPacks()
	for _, p := range packs {
		if p.ID == id || p.Dir == id || filepath.Base(p.Dir) == id {
			return p
		}
	}
	var known []string
	for _, p := range packs {
		known = append(known, p.ID)
	}
	failNotFound(fmt.Sprintf("no pack %q (known: %s)", id, strings.Join(known, ", ")))
	return packRef{} // unreachable
}

func packsList(asJSON bool) {
	packs := loadAllPacks()
	if len(packs) == 0 {
		failNotFound("no packs found — run somnus from the repo root")
	}
	if asJSON {
		type jsonEntry struct {
			ID       string    `json:"id"`
			Category string    `json:"category"`
			Dir      string    `json:"dir"`
			Manifest *Manifest `json:"manifest"`
		}
		out := make([]jsonEntry, len(packs))
		for i, p := range packs {
			out[i] = jsonEntry{ID: p.ID, Category: p.Category, Dir: p.Dir, Manifest: p.M}
		}
		data, _ := json.MarshalIndent(out, "", "  ")
		fmt.Println(string(data))
		return
	}
	idW, verW := 4, 7
	for _, p := range packs {
		if len(p.ID) > idW {
			idW = len(p.ID)
		}
		if len(p.M.Version) > verW {
			verW = len(p.M.Version)
		}
	}
	fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  platforms\n", idW, "id", "type", verW, "version", "loader", "role")
	for _, p := range packs {
		m := p.M
		loader := m.Loader
		if loader == "" {
			loader = "-"
		}
		role := m.Role.Label()
		var plats []string
		if m.ModrinthID != "" {
			plats = append(plats, "mr")
		}
		if m.CurseforgeID != "" {
			plats = append(plats, "cf")
		}
		platStr := strings.Join(plats, "+")
		if platStr == "" {
			platStr = "-"
		}
		typ := m.Type
		if len(m.Variants) > 0 {
			typ = fmt.Sprintf("%s(%dv)", typ, len(m.Variants))
		}
		fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  %s\n", idW, p.ID, typ, verW, m.Version, loader, role, platStr)
	}
	fmt.Printf("\n%d pack(s) registered\n", len(packs))
}

func packsGet(id, field string) {
	p := findPack(id)
	if field == "" {
		data, err := json.MarshalIndent(p.M, "", "  ")
		if err != nil {
			fail(fmt.Sprintf("failed to render manifest: %v", err))
		}
		fmt.Println(string(data))
		return
	}
	data, _ := json.Marshal(p.M)
	var raw map[string]any
	json.Unmarshal(data, &raw) //nolint:errcheck
	val, ok := raw[field]
	if !ok {
		failNotFound(fmt.Sprintf("pack %q has no field %q", p.ID, field))
	}
	switch v := val.(type) {
	case string:
		fmt.Println(v)
	default:
		out, _ := json.MarshalIndent(v, "", "  ")
		fmt.Println(string(out))
	}
}

var settableFields = map[string]bool{
	"name": true, "version": true, "release_type": true, "description": true,
	"modrinth_id": true, "curseforge_id": true, "mc_version": true, "loader": true,
}

func packsSet(id, field, value string) {
	if !settableFields[field] {
		var allowed []string
		for f := range settableFields {
			allowed = append(allowed, f)
		}
		sort.Strings(allowed)
		failUsage(fmt.Sprintf("field %q is not settable via packs set (allowed: %s)\nstructured fields (role, variants) should be edited in the manifest directly", field, strings.Join(allowed, ", ")))
	}
	p := findPack(id)
	m := p.M
	var old string
	switch field {
	case "name":
		old, m.Name = m.Name, value
	case "version":
		old, m.Version = m.Version, value
	case "release_type":
		old, m.ReleaseType = m.ReleaseType, value
	case "description":
		old, m.Description = m.Description, value
	case "modrinth_id":
		old, m.ModrinthID = m.ModrinthID, value
	case "curseforge_id":
		old, m.CurseforgeID = m.CurseforgeID, value
	case "mc_version":
		if m.MCVersion != nil {
			old = *m.MCVersion
		}
		m.MCVersion = &value
	case "loader":
		old, m.Loader = m.Loader, value
	}
	if err := WriteManifest(filepath.Join(p.Dir, "manifest.json"), m); err != nil {
		fail(fmt.Sprintf("failed to write manifest: %v", err))
	}
	fmt.Printf("%s: %s: %q -> %q\n", p.ID, field, old, value)
	if field == "version" {
		fmt.Println("note: 'somnus bump' is the richer path for versions (supports --configs for in-pack version files)")
	}
}
