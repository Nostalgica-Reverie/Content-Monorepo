package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

const (
	menuCreditsFile = "isxander-main-menu-credits.json"
	loaderDepsFile  = "fabric_loader_dependencies.json"
)

func updatePackConfigs(packDir, packName, version string) {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", packDir, err))
	}

	var touched []string
	updates := 0
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		name := e.Name()
		if !strings.HasSuffix(name, "-mr") && !strings.HasSuffix(name, "-cf") {
			continue
		}
		cfgDir := filepath.Join(packDir, name, "config")
		n := 0
		if updateMenuCredits(filepath.Join(cfgDir, menuCreditsFile), packName, version) {
			n++
		}
		if updateLoaderDeps(filepath.Join(cfgDir, loaderDepsFile), packName, version) {
			n++
		}
		if n > 0 {
			touched = append(touched, filepath.Join(packDir, name))
			updates += n
		}
	}

	if len(touched) == 0 {
		fmt.Printf("no version-bearing configs (%s, %s) found in any subdir of %s.\n", menuCreditsFile, loaderDepsFile, packDir)
		return
	}
	fmt.Printf("updated %d config file(s) across %d subdir(s)\n", updates, len(touched))

	if _, err := exec.LookPath(packwizBin()); err != nil {
		fmt.Println("note: packwiz not on PATH; run 'packwiz refresh' in each updated subdir to fix index hashes.")
		return
	}
	for _, dir := range touched {
		cmd := exec.Command(packwizBin(), "refresh")
		cmd.Dir = dir
		if out, err := cmd.CombinedOutput(); err != nil {
			fail(fmt.Sprintf("packwiz refresh failed in %s: %v\n%s", dir, err, indent(string(out), "    ")))
		}
		fmt.Printf("  refreshed %s\n", dir)
	}
}

func updateMenuCredits(path, packName, version string) bool {
	obj, ok := loadJSONMap(path)
	if !ok {
		return false
	}
	mainMenu, ok := obj["main_menu"].(map[string]any)
	if !ok {
		warnf("%s: no 'main_menu' object; skipped", path)
		return false
	}
	bottomRight, ok := mainMenu["bottom_right"].([]any)
	if !ok || len(bottomRight) == 0 {
		warnf("%s: no 'main_menu.bottom_right' entries; skipped", path)
		return false
	}
	first, ok := bottomRight[0].(map[string]any)
	if !ok {
		warnf("%s: bottom_right[0] is not an object; skipped", path)
		return false
	}
	first["text"] = packName + " " + version
	writeCompactJSON(path, obj)
	fmt.Printf("  %s -> %q\n", path, packName+" "+version)
	return true
}

func updateLoaderDeps(path, packName, version string) bool {
	obj, ok := loadJSONMap(path)
	if !ok {
		return false
	}
	overrides, ok := obj["overrides"].(map[string]any)
	if !ok {
		warnf("%s: no 'overrides' object; skipped", path)
		return false
	}
	minecraft, ok := overrides["minecraft"].(map[string]any)
	if !ok {
		warnf("%s: no 'overrides.minecraft' object; skipped", path)
		return false
	}
	recommends, ok := minecraft["+recommends"].(map[string]any)
	if !ok {
		warnf("%s: no 'overrides.minecraft.+recommends' object; skipped", path)
		return false
	}
	recommends[packName] = ">" + version
	writeCompactJSON(path, obj)
	fmt.Printf("  %s -> %s: %q\n", path, packName, ">"+version)
	return true
}

func loadJSONMap(path string) (map[string]any, bool) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, false
	}
	var obj map[string]any
	if err := json.Unmarshal(data, &obj); err != nil {
		warnf("invalid JSON in %s: %v; skipped", path, err)
		return nil, false
	}
	return obj, true
}

func writeCompactJSON(path string, v any) {
	var buf strings.Builder
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(v); err != nil {
		fail(fmt.Sprintf("failed to marshal %s: %v", path, err))
	}
	if err := os.WriteFile(path, []byte(buf.String()), 0o644); err != nil {
		fail(fmt.Sprintf("failed to write %s: %v", path, err))
	}
}
