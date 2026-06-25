package main

import (
	"fmt"
	"path/filepath"
)

func cmdBump(args []string) {
	if len(args) < 2 {
		failUsage(verbUsage["bump"])
	}
	packDir, newVer := absPath(args[0]), args[1]
	doConfigs := false
	for _, a := range args[2:] {
		if a == "--configs" {
			doConfigs = true
		}
	}
	if newVer == "" {
		failUsage("new version must not be empty\n" + verbUsage["bump"])
	}
	mfPath := filepath.Join(packDir, "manifest.json")

	m, err := ReadManifest(mfPath)
	if err != nil {
		failNotFound(fmt.Sprintf("failed to read %s: %v", mfPath, err))
	}
	old := m.Version
	m.Version = newVer
	if err := WriteManifest(mfPath, m); err != nil {
		fail(fmt.Sprintf("failed to write %s: %v", mfPath, err))
	}
	fmt.Printf("bumped %s: %s -> %s\n", mfPath, old, newVer)

	if doConfigs {
		packName := m.Name
		if packName == "" {
			packName = m.ID
		}
		if packName == "" {
			packName = filepath.Base(packDir)
		}
		updatePackConfigs(packDir, packName, newVer)
	}
}

