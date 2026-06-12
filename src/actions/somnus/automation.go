package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

type automation struct {
	AutoUpdate  *bool               `json:"auto_update"`
	ServerPromo *bool               `json:"server_promo"`
	SyncExclude []string            `json:"sync_exclude"`
	Freeze      map[string][]string `json:"freeze"`
}

type legacyOptOut struct {
	AutoUpdate  *bool    `json:"auto_update"`
	ServerPromo *bool    `json:"server_promo"`
	SyncExclude []string `json:"sync_exclude"`
	Freeze      []string `json:"freeze"`
}

func readAutomation(packDir string) automation {
	var a automation

	if data, err := os.ReadFile(filepath.Join(packDir, "manifest.json")); err == nil {
		var mf struct {
			Automation *automation `json:"automation"`
		}
		if err := json.Unmarshal(data, &mf); err == nil && mf.Automation != nil {
			a = *mf.Automation
		}
	}

	if data, err := os.ReadFile(filepath.Join(packDir, "opt-out.json")); err == nil {
		var legacy legacyOptOut
		if err := json.Unmarshal(data, &legacy); err != nil {
			fmt.Fprintf(os.Stderr, "::warning::invalid opt-out.json in %s: %v\n", packDir, err)
			return a
		}
		if a.AutoUpdate == nil {
			a.AutoUpdate = legacy.AutoUpdate
		}
		if a.ServerPromo == nil {
			a.ServerPromo = legacy.ServerPromo
		}
		a.SyncExclude = append(a.SyncExclude, legacy.SyncExclude...)
		if len(legacy.Freeze) > 0 {
			if a.Freeze == nil {
				a.Freeze = map[string][]string{}
			}
			for _, sub := range modSubdirsOf(packDir) {
				key := filepath.Base(sub)
				a.Freeze[key] = append(a.Freeze[key], legacy.Freeze...)
			}
		}
	}
	return a
}

func hasLegacyOptOut(packDir string) bool {
	_, err := os.Stat(filepath.Join(packDir, "opt-out.json"))
	return err == nil
}

func optedOutOfAutoUpdate(packDir string) (skip bool, legacy bool) {
	a := readAutomation(packDir)
	if a.AutoUpdate != nil && !*a.AutoUpdate {
		return true, hasLegacyOptOut(packDir)
	}
	return false, false
}

func setAutomationFreeze(packDir, subKey string, slugs []string) {
	mfPath := filepath.Join(packDir, "manifest.json")
	data, err := os.ReadFile(mfPath)
	if err != nil {
		failNotFound(fmt.Sprintf("failed to read %s: %v", mfPath, err))
	}
	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		fail(fmt.Sprintf("invalid JSON in %s: %v", mfPath, err))
	}

	auto, _ := raw["automation"].(map[string]any)
	if auto == nil {
		auto = map[string]any{}
	}
	freeze, _ := auto["freeze"].(map[string]any)
	if freeze == nil {
		freeze = map[string]any{}
	}

	if len(slugs) == 0 {
		delete(freeze, subKey)
	} else {
		freeze[subKey] = slugs
	}
	if len(freeze) == 0 {
		delete(auto, "freeze")
	} else {
		auto["freeze"] = freeze
	}
	if len(auto) == 0 {
		delete(raw, "automation")
	} else {
		raw["automation"] = auto
	}
	writeJSON(mfPath, raw)
}
