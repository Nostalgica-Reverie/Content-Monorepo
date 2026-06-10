package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

type optOut struct {
	AutoUpdate  *bool    `json:"auto_update"`
	ServerPromo *bool    `json:"server_promo"`
	SyncExclude []string `json:"sync_exclude"`
}

func readOptOut(packDir string) optOut {
	var o optOut
	data, err := os.ReadFile(filepath.Join(packDir, "opt-out.json"))
	if err != nil {
		return o
	}
	if err := json.Unmarshal(data, &o); err != nil {
		fmt.Fprintf(os.Stderr, "::warning::invalid opt-out.json in %s: %v\n", packDir, err)
	}
	return o
}

func optedOutOfAutoUpdate(packDir string) (skip bool, legacy bool) {
	o := readOptOut(packDir)
	if o.AutoUpdate != nil && !*o.AutoUpdate {
		return true, false
	}
	if _, err := os.Stat(filepath.Join(packDir, "auto-update-ignore.json")); err == nil {
		return true, true
	}
	return false, false
}
