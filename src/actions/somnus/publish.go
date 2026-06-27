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
	mode, manifestPath := args[0], absPath(args[1])
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

func pubResolve(manifestPath, variant string) pubResolved {
	isExperimental := filepath.Base(manifestPath) == "manifest-experimental.json"
	m, err := ReadManifest(manifestPath)
	if err != nil {
		failNotFound(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
	}

	if m.Name == "" {
		fail(fmt.Sprintf("missing or non-string 'name' in %s", manifestPath))
	}
	if m.Type == "" {
		fail(fmt.Sprintf("missing or non-string 'type' in %s", manifestPath))
	}
	if m.ReleaseType == "" {
		fail(fmt.Sprintf("missing or non-string 'release_type' in %s", manifestPath))
	}
	if m.ID == "" {
		fail(fmt.Sprintf("missing or non-string 'id' in %s", manifestPath))
	}

	rawName := m.Name
	r := pubResolved{
		rawName:        rawName,
		pName:          strings.ReplaceAll(rawName, " ", "-"),
		pType:          m.Type,
		releaseType:    m.ReleaseType,
		mrID:           m.ModrinthID,
		cfID:           m.CurseforgeID,
		isExperimental: isExperimental,
	}
	if r.mrID == "" && r.cfID == "" {
		fail("manifest must set at least one of modrinth_id or curseforge_id")
	}

	packLoader := m.Loader
	var variantName, variantVersion, variantLoader string

	if variant != "" {
		if len(m.Variants) == 0 {
			fail(fmt.Sprintf("variant '%s' requested but manifest has no 'variants'", variant))
		}
		var found *Variant
		for i := range m.Variants {
			v := &m.Variants[i]
			key := v.ID
			if key == "" {
				key = v.MCVersion
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
		r.mcVer = found.MCVersion
		if r.mcVer == "" {
			fail(fmt.Sprintf("variant '%s' missing mc_version", variant))
		}
		variantName = found.Name
		variantVersion = found.Version
		variantLoader = found.Loader
	} else {
		if m.MCVersion == nil || *m.MCVersion == "" {
			fail(fmt.Sprintf("missing or non-string 'mc_version' in %s", manifestPath))
		}
		r.mcVer = *m.MCVersion
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
			r.pVer = fmt.Sprintf("%s-%s-%s-%s", m.ID, variant, cycle, short)
		} else {
			r.pVer = fmt.Sprintf("%s-%s-%s", m.ID, cycle, short)
		}
	} else {
		baseVer := variantVersion
		if baseVer == "" {
			baseVer = m.Version
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
		m, err := ReadManifest(manifestPath)
		if err != nil {
			failNotFound(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
		}
		if len(m.Variants) > 0 {
			for idx, v := range m.Variants {
				key := v.ID
				if key == "" {
					key = v.MCVersion
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
	m, err := ReadManifest(manifestPath)
	if err != nil {
		failNotFound(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
	}
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
		pubBuildDatapack(pDir, artifactsDir, m.ID, r.pVer)
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

	type exportPlan struct {
		plat       platform
		targetPath string
		outFile    string
		flag       *bool
	}
	var plans []exportPlan

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
		plans = append(plans, exportPlan{
			plat:       pl.plat,
			targetPath: targetPath,
			outFile:    filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.%s", filenameBase, pl.plat.short, pl.plat.ext)),
			flag:       pl.flag,
		})
	}

	if len(plans) == 0 {
		fail(fmt.Sprintf("no platform folders found for subdir key '%s' (expected %s-mr / %s-cf)", r.subdirKey, r.subdirKey, r.subdirKey))
	}

	sched := NewScheduler(maxConcurrent())
	slots := cacheSlotCount()
	dones := make([]<-chan error, len(plans))
	for i, p := range plans {
		dones[i] = sched.Submit(Task{
			Name: filepath.Base(p.outFile),
			Needs: []Resource{
				Resource("export:" + p.targetPath),
				CacheSlot(p.targetPath, slots),
			},
			Run: func() error {
				cmd := exec.Command(packwizBin(), p.plat.cli, "export", "--output", p.outFile)
				cmd.Dir = p.targetPath
				if out, err := cmd.CombinedOutput(); err != nil {
					return fmt.Errorf("packwiz export failed for %s: %v\n%s", p.targetPath, err, indent(string(out), "    "))
				}
				fmt.Printf("exported %s\n", p.outFile)
				*p.flag = true
				return nil
			},
		})
	}
	sched.Close()
	for _, c := range dones {
		if err := <-c; err != nil {
			fail(err.Error())
		}
	}
}

func pubBuildDatapack(pDir, artifactsDir, id, pVer string) {
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
