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
	Raw      map[string]any
}

func cmdPacks(args []string) {
	if len(args) == 0 {
		failUsage(verbUsage["packs"])
	}
	switch args[0] {
	case "list":
		packsList()
	case "get":
		if len(args) < 2 {
			failUsage(verbUsage["packs"])
		}
		field := ""
		if len(args) > 2 {
			field = args[2]
		}
		packsGet(args[1], field)
	case "set":
		if len(args) < 4 {
			failUsage(verbUsage["packs"])
		}
		packsSet(args[1], args[2], args[3])
	default:
		failUsage(fmt.Sprintf("unknown packs subcommand %q\n%s", args[0], verbUsage["packs"]))
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
			data, err := os.ReadFile(filepath.Join(dir, "manifest.json"))
			if err != nil {
				continue
			}
			var raw map[string]any
			if err := json.Unmarshal(data, &raw); err != nil {
				fmt.Fprintf(os.Stderr, "::warning::packs: invalid JSON in %s/manifest.json; skipped\n", dir)
				continue
			}
			id, _ := raw["id"].(string)
			if id == "" {
				id = e.Name()
			}
			out = append(out, packRef{Category: cat, Dir: dir, ID: id, Raw: raw})
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

func packsList() {
	packs := loadAllPacks()
	if len(packs) == 0 {
		failNotFound("no packs found — run somnus from the repo root")
	}
	idW, verW := 4, 7
	for _, p := range packs {
		if len(p.ID) > idW {
			idW = len(p.ID)
		}
		if v, _ := p.Raw["version"].(string); len(v) > verW {
			verW = len(v)
		}
	}
	fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  platforms\n", idW, "id", "type", verW, "version", "loader", "role")
	for _, p := range packs {
		typ, _ := p.Raw["type"].(string)
		ver, _ := p.Raw["version"].(string)
		loader, _ := p.Raw["loader"].(string)
		if loader == "" {
			loader = "-"
		}
		role := roleLabel(p.Raw["role"])
		var plats []string
		if mr, _ := p.Raw["modrinth_id"].(string); mr != "" {
			plats = append(plats, "mr")
		}
		if cf, _ := p.Raw["curseforge_id"].(string); cf != "" {
			plats = append(plats, "cf")
		}
		platStr := strings.Join(plats, "+")
		if platStr == "" {
			platStr = "-"
		}
		if variants, ok := p.Raw["variants"].([]any); ok && len(variants) > 0 {
			typ = fmt.Sprintf("%s(%dv)", typ, len(variants))
		}
		fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  %s\n", idW, p.ID, typ, verW, ver, loader, role, platStr)
	}
	fmt.Printf("\n%d pack(s) registered\n", len(packs))
}

func roleLabel(role any) string {
	switch r := role.(type) {
	case string:
		if r == "" {
			return "none"
		}
		return r
	case map[string]any:
		if _, ok := r["performance_base"]; ok {
			return "consumer"
		}
	}
	return "none"
}

func packsGet(id, field string) {
	p := findPack(id)
	if field == "" {
		data, err := json.MarshalIndent(p.Raw, "", "  ")
		if err != nil {
			fail(fmt.Sprintf("failed to render manifest: %v", err))
		}
		fmt.Println(string(data))
		return
	}
	val, ok := p.Raw[field]
	if !ok {
		failNotFound(fmt.Sprintf("pack %q has no field %q", p.ID, field))
	}
	switch v := val.(type) {
	case string:
		fmt.Println(v)
	default:
		data, _ := json.MarshalIndent(v, "", "  ")
		fmt.Println(string(data))
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
	old, _ := p.Raw[field].(string)
	p.Raw[field] = value
	writeJSON(filepath.Join(p.Dir, "manifest.json"), p.Raw)
	fmt.Printf("%s: %s: %q -> %q\n", p.ID, field, old, value)
	if field == "version" {
		fmt.Println("note: 'somnus bump' is the richer path for versions (supports --configs for in-pack version files)")
	}
}
