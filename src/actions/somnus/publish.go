package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

func cmdPublish(args []string) {
	if len(args) < 2 {
		failUsage(verbUsage["publish"])
	}
	mode, manifestPath := args[0], args[1]
	var variant string
	live := false
	for _, a := range args[2:] {
		switch a {
		case "--live":
			live = true
		default:
			if !strings.HasPrefix(a, "-") && variant == "" {
				variant = a
			}
		}
	}
	switch mode {
	case "list":
		pubList(args[1:])
	case "build":
		pubBuild(manifestPath, variant)
	case "upload":
		pubUpload(manifestPath, variant, live)
	case "verify":
		pubVerify(manifestPath, variant)
	default:
		failUsage(fmt.Sprintf("unknown publish mode %q\n%s", mode, verbUsage["publish"]))
	}
}

type pubResolved struct {
	pName, rawName, pType, loader, releaseType string
	mrID, cfID, subdirKey, mcVer, pVer         string
	displayName                                string
	isExperimental                             bool
	builtMR, builtCF                           bool
}

func pubManifest(manifestPath string) map[string]any {
	data, err := os.ReadFile(manifestPath)
	if err != nil {
		failNotFound(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
	}
	var m map[string]any
	if err := json.Unmarshal(data, &m); err != nil {
		fail(fmt.Sprintf("invalid JSON in %s: %v", manifestPath, err))
	}
	return m
}

func reqStr(m map[string]any, key, where string) string {
	s, _ := m[key].(string)
	if s == "" {
		fail(fmt.Sprintf("missing or non-string '%s' in %s", key, where))
	}
	return s
}

func optStr(m map[string]any, key string) string {
	s, _ := m[key].(string)
	return s
}

func pubResolve(manifestPath, variant string) pubResolved {
	isExperimental := filepath.Base(manifestPath) == "manifest-experimental.json"
	m := pubManifest(manifestPath)

	rawName := reqStr(m, "name", manifestPath)
	r := pubResolved{
		rawName:        rawName,
		pName:          strings.ReplaceAll(rawName, " ", "-"),
		pType:          reqStr(m, "type", manifestPath),
		releaseType:    reqStr(m, "release_type", manifestPath),
		mrID:           optStr(m, "modrinth_id"),
		cfID:           optStr(m, "curseforge_id"),
		isExperimental: isExperimental,
	}
	id := reqStr(m, "id", manifestPath)
	if r.mrID == "" && r.cfID == "" {
		fail("manifest must set at least one of modrinth_id or curseforge_id")
	}

	packLoader := optStr(m, "loader")
	var variantName, variantVersion, variantLoader string

	if variant != "" {
		variants, ok := m["variants"].([]any)
		if !ok {
			fail(fmt.Sprintf("variant '%s' requested but manifest has no 'variants'", variant))
		}
		var found map[string]any
		for _, raw := range variants {
			v, ok := raw.(map[string]any)
			if !ok {
				continue
			}
			key := optStr(v, "id")
			if key == "" {
				key = optStr(v, "mc_version")
			}
			if key == variant {
				found = v
				break
			}
		}
		if found == nil {
			fail(fmt.Sprintf("variant '%s' not found in manifest", variant))
		}
		r.subdirKey = variant
		r.mcVer = optStr(found, "mc_version")
		if r.mcVer == "" {
			fail(fmt.Sprintf("variant '%s' missing mc_version", variant))
		}
		variantName = optStr(found, "name")
		variantVersion = optStr(found, "version")
		variantLoader = optStr(found, "loader")
	} else {
		r.mcVer = reqStr(m, "mc_version", manifestPath)
		r.subdirKey = r.mcVer
	}

	r.loader = variantLoader
	if r.loader == "" {
		r.loader = packLoader
	}
	if r.pType == "modpack" && r.loader == "" {
		fail(fmt.Sprintf("no loader resolved for '%s': set a pack-level 'loader' or a variant 'loader'", r.subdirKey))
	}

	if isExperimental {
		sha := os.Getenv("GITHUB_SHA")
		if sha == "" {
			fail("GITHUB_SHA not set; required for experimental")
		}
		short := sha
		if len(short) > 7 {
			short = short[:7]
		}
		cycle := time.Now().UTC().Format("06.01")
		if variant != "" {
			r.pVer = fmt.Sprintf("%s-%s-%s-%s", id, variant, cycle, short)
		} else {
			r.pVer = fmt.Sprintf("%s-%s-%s", id, cycle, short)
		}
	} else {
		baseVer := variantVersion
		if baseVer == "" {
			baseVer = optStr(m, "version")
		}
		if baseVer == "" {
			fail("missing 'version'")
		}
		if variant != "" {
			r.pVer = baseVer + "-" + variant
		} else {
			r.pVer = baseVer
		}
	}

	switch {
	case variantName != "":
		r.displayName = fmt.Sprintf("%s %s %s", rawName, variantName, r.pVer)
	case variant != "":
		r.displayName = fmt.Sprintf("%s %s %s", rawName, variant, r.pVer)
	default:
		r.displayName = fmt.Sprintf("%s %s", rawName, r.pVer)
	}
	if isExperimental {
		r.displayName = "[EXPERIMENTAL] " + r.displayName
	}
	return r
}

func pubList(manifestPaths []string) {
	entries := []map[string]any{}
	for _, manifestPath := range manifestPaths {
		m := pubManifest(manifestPath)
		if variants, ok := m["variants"].([]any); ok {
			for idx, raw := range variants {
				v, _ := raw.(map[string]any)
				key := optStr(v, "id")
				if key == "" {
					key = optStr(v, "mc_version")
				}
				if key == "" {
					fail("variant missing both 'id' and 'mc_version'")
				}
				entries = append(entries, map[string]any{"manifest": manifestPath, "variant": key, "order": idx})
			}
		} else {
			entries = append(entries, map[string]any{"manifest": manifestPath, "variant": nil, "order": 0})
		}
	}
	data, err := json.Marshal(entries)
	if err != nil {
		fail(fmt.Sprintf("failed to render entries: %v", err))
	}
	fmt.Println(string(data))
}

func pubBuild(manifestPath, variant string) {
	pDir := filepath.Dir(manifestPath)
	m := pubManifest(manifestPath)
	r := pubResolve(manifestPath, variant)

	workspace := os.Getenv("GITHUB_WORKSPACE")
	if workspace == "" {
		workspace = "."
	}
	artifactsDir := filepath.Join(workspace, pDir, "artifacts")
	if err := os.RemoveAll(artifactsDir); err != nil {
		fail(fmt.Sprintf("failed to clear %s: %v", artifactsDir, err))
	}
	if err := os.MkdirAll(artifactsDir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create %s: %v", artifactsDir, err))
	}

	var label string
	switch {
	case r.isExperimental && variant != "":
		label = fmt.Sprintf("EXPERIMENTAL %s [%s] (%s)", r.rawName, variant, r.pVer)
	case r.isExperimental:
		label = fmt.Sprintf("EXPERIMENTAL %s (%s)", r.rawName, r.pVer)
	case variant != "":
		label = fmt.Sprintf("%s [%s]", r.rawName, variant)
	default:
		label = r.rawName
	}
	fmt.Printf("::group::Building %s\n", label)

	switch r.pType {
	case "modpack":
		pubBuildModpack(pDir, artifactsDir, &r)
	case "datapack":
		pubBuildDatapack(pDir, artifactsDir, m, r.pVer)
		r.builtMR = r.mrID != ""
		r.builtCF = r.cfID != ""
	default:
		fail(fmt.Sprintf("unsupported pack type: %s", r.pType))
	}
	fmt.Println("::endgroup::")

	pubWriteOutputs(r, pDir)
}

func pubBuildModpack(pDir, artifactsDir string, r *pubResolved) {
	filenameBase := fmt.Sprintf("%s-%s-%s-%s", r.pName, r.mcVer, r.loader, r.pVer)
	built := 0

	for _, pl := range []struct {
		plat platform
		id   string
		flag *bool
	}{{modrinth, r.mrID, &r.builtMR}, {curseforge, r.cfID, &r.builtCF}} {
		if pl.id == "" {
			continue
		}
		targetPath := filepath.Join(pDir, r.subdirKey+"-"+pl.plat.short)
		if info, err := os.Stat(targetPath); err != nil || !info.IsDir() {
			fmt.Printf("skipping %s: folder %s not found (variant not published to this platform)\n", pl.plat.short, targetPath)
			continue
		}
		outFile := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.%s", filenameBase, pl.plat.short, pl.plat.ext))
		cmd := exec.Command(packwizBin(), pl.plat.cli, "export", "--output", outFile)
		cmd.Dir = targetPath
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			fail(fmt.Sprintf("packwiz export failed for %s", targetPath))
		}
		fmt.Printf("exported %s\n", outFile)
		*pl.flag = true
		built++
	}

	if built == 0 {
		fail(fmt.Sprintf("no platform folders found for subdir key '%s' (expected %s-mr / %s-cf)", r.subdirKey, r.subdirKey, r.subdirKey))
	}
}

func pubBuildDatapack(pDir, artifactsDir string, m map[string]any, pVer string) {
	id := reqStr(m, "id", pDir)
	contentDir := filepath.Join(pDir, "content")
	if info, err := os.Stat(contentDir); err != nil || !info.IsDir() {
		fail(fmt.Sprintf("content directory not found at %s", contentDir))
	}
	outFile := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.zip", id, pVer))
	if err := zipContents(contentDir, outFile); err != nil {
		fail(fmt.Sprintf("zip failed: %v", err))
	}
}

func pubWriteOutputs(r pubResolved, pDir string) {
	outPath := os.Getenv("GITHUB_OUTPUT")
	if outPath == "" {
		return
	}
	f, err := os.OpenFile(outPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		fail(fmt.Sprintf("failed to open GITHUB_OUTPUT: %v", err))
	}
	defer f.Close()
	mrOut, cfOut := "", ""
	if r.builtMR {
		mrOut = r.mrID
	}
	if r.builtCF {
		cfOut = r.cfID
	}
	fmt.Fprintf(f, "mr_id=%s\n", mrOut)
	fmt.Fprintf(f, "cf_id=%s\n", cfOut)
	fmt.Fprintf(f, "name=%s\n", r.displayName)
	fmt.Fprintf(f, "ver=%s\n", r.pVer)
	fmt.Fprintf(f, "mc=%s\n", r.mcVer)
	fmt.Fprintf(f, "type=%s\n", r.pType)
	fmt.Fprintf(f, "loader=%s\n", r.loader)
	fmt.Fprintf(f, "release_type=%s\n", r.releaseType)
	fmt.Fprintf(f, "path=%s\n", pDir)
	fmt.Fprintf(f, "is_experimental=%t\n", r.isExperimental)
}
