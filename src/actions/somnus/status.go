package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

type packStatus struct {
	ID         string        `json:"id"`
	Name       string        `json:"name"`
	Version    string        `json:"version"`
	MCVersion  string        `json:"mc_version,omitempty"`
	Loader     string        `json:"loader,omitempty"`
	AutoUpdate bool          `json:"auto_update"`
	Subdirs    []subdirStat  `json:"subdirs"`
	TotalMods  int           `json:"total_mods"`
	FrozenMods int           `json:"frozen_mods"`
}

type subdirStat struct {
	Key      string   `json:"key"`
	Platform string   `json:"platform"`
	ModCount int      `json:"mod_count"`
	Frozen   []string `json:"frozen,omitempty"`
}

func cmdStatus(args []string) {
	asJSON := false
	for _, a := range args {
		if a == "--json" {
			asJSON = true
		}
	}

	root := modpacksDir()
	entries, err := os.ReadDir(root)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", root, err))
	}

	var statuses []packStatus
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		packPath := filepath.Join(root, e.Name())
		m, err := ReadManifest(filepath.Join(packPath, "manifest.json"))
		if err != nil {
			continue
		}

		auto := readAutomation(packPath)
		autoUpdate := auto.AutoUpdate == nil || *auto.AutoUpdate

		var subdirs []subdirStat
		totalMods, totalFrozen := 0, 0

		for _, sub := range modSubdirsOf(packPath) {
			key := filepath.Base(sub)
			plat := "?"
			if strings.HasSuffix(key, "-mr") {
				plat = "mr"
			} else if strings.HasSuffix(key, "-cf") {
				plat = "cf"
			}

			modsDir := filepath.Join(sub, "mods")
			modEntries, _ := os.ReadDir(modsDir)
			modCount := 0
			for _, me := range modEntries {
				if !me.IsDir() && strings.HasSuffix(me.Name(), ".pw.toml") {
					modCount++
				}
			}

			frozen := auto.Freeze[key]
			totalMods += modCount
			totalFrozen += len(frozen)

			subdirs = append(subdirs, subdirStat{
				Key:      key,
				Platform: plat,
				ModCount: modCount,
				Frozen:   frozen,
			})
		}

		mcVersion := ""
		if m.MCVersion != nil {
			mcVersion = *m.MCVersion
		}

		statuses = append(statuses, packStatus{
			ID:         m.ID,
			Name:       m.Name,
			Version:    m.Version,
			MCVersion:  mcVersion,
			Loader:     m.Loader,
			AutoUpdate: autoUpdate,
			Subdirs:    subdirs,
			TotalMods:  totalMods,
			FrozenMods: totalFrozen,
		})
	}

	if asJSON {
		data, _ := json.MarshalIndent(statuses, "", "  ")
		fmt.Println(string(data))
		return
	}

	if len(statuses) == 0 {
		fmt.Println("no packs found")
		return
	}

	for _, s := range statuses {
		autoStr := "auto-update"
		if !s.AutoUpdate {
			autoStr = "no-update"
		}
		frozenNote := ""
		if s.FrozenMods > 0 {
			frozenNote = fmt.Sprintf("  %d frozen", s.FrozenMods)
		}
		fmt.Printf("%s  v%s  mc%s  %s  [%s]  %d mods%s\n",
			s.ID, s.Version, s.MCVersion, s.Loader, autoStr, s.TotalMods, frozenNote)
		for _, sub := range s.Subdirs {
			subFrozen := ""
			if len(sub.Frozen) > 0 {
				subFrozen = fmt.Sprintf("  (%d frozen)", len(sub.Frozen))
			}
			fmt.Printf("    %-32s [%s]  %d mods%s\n", sub.Key, sub.Platform, sub.ModCount, subFrozen)
		}
	}
	fmt.Printf("\n%d pack(s)\n", len(statuses))
}
