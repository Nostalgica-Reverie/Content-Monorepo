package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

func cmdBump(args []string) {
	if len(args) < 2 {
		fail("usage: somnus bump <pack-dir> <new-version> [--configs]\n  e.g. somnus bump modpacks/re-console-plus 26.06.1\n  --configs also updates in-pack version configs (main menu credits, loader dependency overrides)")
	}
	packDir, newVer := args[0], args[1]
	doConfigs := false
	for _, a := range args[2:] {
		if a == "--configs" {
			doConfigs = true
		}
	}
	if newVer == "" {
		fail("new version must not be empty")
	}
	mfPath := filepath.Join(packDir, "manifest.json")

	data, err := os.ReadFile(mfPath)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", mfPath, err))
	}
	var obj map[string]any
	if err := json.Unmarshal(data, &obj); err != nil {
		fail(fmt.Sprintf("invalid JSON in %s: %v", mfPath, err))
	}
	old, _ := obj["version"].(string)
	obj["version"] = newVer
	writeJSON(mfPath, obj)
	fmt.Printf("bumped %s: %s -> %s\n", mfPath, old, newVer)

	if doConfigs {
		packName, _ := obj["name"].(string)
		if packName == "" {
			packName, _ = obj["id"].(string)
		}
		if packName == "" {
			packName = filepath.Base(packDir)
		}
		updatePackConfigs(packDir, packName, newVer)
	}
}
