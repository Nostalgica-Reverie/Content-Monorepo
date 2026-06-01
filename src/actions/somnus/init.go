package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

const (
	defaultMCVersion   = "1.21.1"
	defaultPackVersion = "0.0.0"
	placeholderAuthor  = "CHANGEME"
)

func cmdInit(args []string) {
	if len(args) < 2 {
		fail("usage: somnus init <category> <name> [--mc <version>] [--loader fabric|forge|neoforge|quilt] [--base | --consumes <id>] [--variants a,b,c]\n  category: modpacks | datapacks | resourcepacks")
	}
	category, name := args[0], args[1]
	switch category {
	case "modpacks", "datapacks", "resourcepacks":
	default:
		fail(fmt.Sprintf("invalid category %q (expected modpacks, datapacks, or resourcepacks)", category))
	}

	loader := "fabric"
	mcVersion := defaultMCVersion
	asBase := false
	consumesBase := ""
	var variants []string
	for i := 2; i < len(args); i++ {
		switch args[i] {
		case "--mc":
			if i+1 < len(args) {
				mcVersion = args[i+1]
				i++
			}
		case "--loader":
			if i+1 < len(args) {
				loader = args[i+1]
				i++
			}
		case "--base":
			asBase = true
		case "--consumes":
			if i+1 < len(args) {
				consumesBase = args[i+1]
				i++
			}
		case "--variants":
			if i+1 < len(args) {
				for _, v := range strings.Split(args[i+1], ",") {
					if v = strings.TrimSpace(v); v != "" {
						variants = append(variants, v)
					}
				}
				i++
			}
		}
	}
	if asBase && consumesBase != "" {
		fail("--base and --consumes are mutually exclusive (a pack is either a base or a consumer, not both)")
	}
	loaderFlag, ok := loaderLatestFlag(loader)
	if !ok {
		fail(fmt.Sprintf("invalid loader %q (expected fabric, forge, neoforge, or quilt)", loader))
	}

	packDir := filepath.Join(category, name)
	if _, err := os.Stat(packDir); err == nil {
		fail(fmt.Sprintf("pack already exists: %s", packDir))
	}
	if err := os.MkdirAll(packDir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create %s: %v", packDir, err))
	}

	mf := map[string]any{
		"$schema":      "../../tools/manifest/schema.json",
		"id":           name,
		"name":         name,
		"type":         categoryType(category),
		"release_type": "release",
		"version":      defaultPackVersion,
	}

	keys := []string{mcVersion}
	if len(variants) > 0 {
		keys = variants
	}

	switch {
	case asBase:
		mf["role"] = "base"
	case consumesBase != "":
		var mappings []map[string]string
		for _, key := range keys {
			for _, plat := range []string{"mr", "cf"} {
				mappings = append(mappings, map[string]string{
					"source": "CHANGEME-" + plat,
					"target": key + "-" + plat,
				})
			}
		}
		mf["role"] = map[string]any{
			"performance_base": map[string]any{
				"pack":     consumesBase,
				"mappings": mappings,
			},
		}
	default:
		mf["role"] = "none"
	}

	if category == "modpacks" {
		mf["loader"] = loader
		mf["mc_version"] = mcVersion
	}
	mf["modrinth_id"] = name

	if len(variants) > 0 {
		var vs []map[string]string
		for _, v := range variants {
			vs = append(vs, map[string]string{
				"id":         v,
				"mc_version": mcVersion,
				"name":       v,
			})
		}
		mf["variants"] = vs
	}

	writeJSON(filepath.Join(packDir, "manifest.json"), mf)

	changelog := fmt.Sprintf("# %s\n\nInitial scaffold. Describe the first release here.\n", name)
	if err := os.WriteFile(filepath.Join(packDir, "changelog.md"), []byte(changelog), 0o644); err != nil {
		fail(fmt.Sprintf("failed to write changelog.md: %v", err))
	}

	roleDesc := "none"
	if asBase {
		roleDesc = "base"
	} else if consumesBase != "" {
		roleDesc = "consumer of " + consumesBase + " (mappings are CHANGEME stubs \u2014 fill them in)"
	}
	fmt.Printf("scaffolded %s\n", packDir)
	fmt.Printf("  manifest.json (role: %s; fill in modrinth_id/curseforge_id, version, author)\n", roleDesc)
	fmt.Printf("  changelog.md\n")

	if category == "modpacks" {
		if _, err := exec.LookPath("packwiz"); err != nil {
			fmt.Println("note: packwiz not on PATH; skipped subdir init. Create the subdirs and run packwiz init manually.")
			return
		}
		for _, key := range keys {
			for _, plat := range []string{"mr", "cf"} {
				sub := filepath.Join(packDir, key+"-"+plat)
				if err := os.MkdirAll(sub, 0o755); err != nil {
					fail(fmt.Sprintf("failed to create %s: %v", sub, err))
				}
				fmt.Printf("  packwiz init in %s ...\n", sub)
				cmd := exec.Command("packwiz", "init",
					"--name", name,
					"--author", placeholderAuthor,
					"--mc-version", mcVersion,
					"--modloader", loader,
					loaderFlag,
					"--version", defaultPackVersion,
					"-y",
				)
				cmd.Dir = sub
				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr
				if err := cmd.Run(); err != nil {
					fail(fmt.Sprintf("packwiz init failed in %s: %v", sub, err))
				}
			}
		}
		fmt.Printf("ready: %s initialized %d subdir-pair(s) (%s, latest). Add mods with packwiz, then fill manifest placeholders.\n",
			packDir, len(keys), loader)
	} else {
		fmt.Printf("next: create %s/{version}/ and add the pack contents (pack.mcmeta at its root).\n", packDir)
	}
}

func categoryType(category string) string {
	switch category {
	case "datapacks":
		return "datapack"
	case "resourcepacks":
		return "resourcepack"
	default:
		return "modpack"
	}
}

func loaderLatestFlag(loader string) (string, bool) {
	switch loader {
	case "fabric":
		return "--fabric-latest", true
	case "forge":
		return "--forge-latest", true
	case "neoforge":
		return "--neoforge-latest", true
	case "quilt":
		return "--quilt-latest", true
	default:
		return "", false
	}
}
