package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

type Automation struct {
	AutoUpdate  *bool               `json:"auto_update,omitempty"`
	ServerPromo *bool               `json:"server_promo,omitempty"`
	SyncExclude []string            `json:"sync_exclude,omitempty"`
	Freeze      map[string][]string `json:"freeze,omitempty"`
}

type legacyOptOut struct {
	AutoUpdate  *bool    `json:"auto_update"`
	ServerPromo *bool    `json:"server_promo"`
	SyncExclude []string `json:"sync_exclude"`
	Freeze      []string `json:"freeze"`
}

func readAutomation(packDir string) Automation {
	var a Automation
	if m, err := ReadManifest(filepath.Join(packDir, "manifest.json")); err == nil && m.Automation != nil {
		a = *m.Automation
	}
	data, err := os.ReadFile(filepath.Join(packDir, "opt-out.json"))
	if err != nil {
		return a
	}
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
	return a
}

func cmdAutomation(args []string) {
	if len(args) < 2 || args[0] != "get" {
		failUsage(verbUsage["automation"])
	}
	a := readAutomation(absPath(args[1]))
	data, err := json.MarshalIndent(a, "", "  ")
	if err != nil {
		fail(fmt.Sprintf("failed to marshal automation: %v", err))
	}
	fmt.Println(string(data))
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
	m, err := ReadManifest(mfPath)
	if err != nil {
		failNotFound(fmt.Sprintf("failed to read %s: %v", mfPath, err))
	}
	if m.Automation == nil {
		m.Automation = &Automation{}
	}
	if len(slugs) == 0 {
		delete(m.Automation.Freeze, subKey)
	} else {
		if m.Automation.Freeze == nil {
			m.Automation.Freeze = map[string][]string{}
		}
		m.Automation.Freeze[subKey] = slugs
	}
	if len(m.Automation.Freeze) == 0 {
		m.Automation.Freeze = nil
	}
	if m.Automation.AutoUpdate == nil && m.Automation.ServerPromo == nil &&
		len(m.Automation.SyncExclude) == 0 && m.Automation.Freeze == nil {
		m.Automation = nil
	}
	if err := WriteManifest(mfPath, m); err != nil {
		fail(fmt.Sprintf("failed to write %s: %v", mfPath, err))
	}
}
