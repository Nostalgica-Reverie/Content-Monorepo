package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

type modlistEntry struct {
	JarName        string `json:"jarName"`
	ModID          string `json:"modId,omitempty"`
	Name           string `json:"name"`
	Version        string `json:"version,omitempty"`
	CurseForgeHash *int64 `json:"curseForgeHash,omitempty"`
	ModrinthHash   string `json:"modrinthHash,omitempty"`
}

type pwMod struct {
	name       string
	filename   string
	side       string
	url        string
	hashFormat string
	hash       string
	cfFileID   *int64
	mrModID    string
}

func cmdModlist(args []string) {
	if len(args) < 1 {
		failUsage(verbUsage["modlist"])
	}
	subdir := absPath(args[0])
	modsDir := filepath.Join(subdir, "mods")
	if info, err := os.Stat(modsDir); err != nil || !info.IsDir() {
		failNotFound(fmt.Sprintf("no mods/ directory at %s", modsDir))
	}

	entries, err := os.ReadDir(modsDir)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", modsDir, err))
	}

	modlist := make(map[string]modlistEntry)
	var parsed, withCF, withMR int

	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".pw.toml") {
			continue
		}
		mod, err := parsePwToml(filepath.Join(modsDir, e.Name()))
		if err != nil {
			warnf("skipping %s: %v", e.Name(), err)
			continue
		}
		parsed++

		entry := modlistEntry{
			JarName: mod.filename,
			Name:    mod.name,
			Version: versionFromFilename(mod.filename),
		}
		if mod.mrModID != "" {
			entry.ModID = mod.mrModID
		}
		if mod.cfFileID != nil {
			entry.CurseForgeHash = mod.cfFileID
			withCF++
		}
		if mod.mrModID != "" && mod.hashFormat == "sha1" && mod.hash != "" {
			entry.ModrinthHash = mod.hash
			withMR++
		}

		modlist[mod.filename] = entry
	}

	outDir := filepath.Join(subdir, "config", "crash_assistant")
	if err := os.MkdirAll(outDir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create %s: %v", outDir, err))
	}
	outPath := filepath.Join(outDir, "modlist.json")
	data, err := json.MarshalIndent(modlist, "", "  ")
	if err != nil {
		fail(fmt.Sprintf("failed to marshal modlist: %v", err))
	}
	data = append(data, '\n')
	if err := os.WriteFile(outPath, data, 0o644); err != nil {
		fail(fmt.Sprintf("failed to write %s: %v", outPath, err))
	}

	fmt.Printf("wrote %s\n", outPath)
	fmt.Printf("  %d mod(s): %d with curseForgeHash, %d with modrinthHash(sha1)\n", parsed, withCF, withMR)
	if withMR < parsed {
		fmt.Printf("  note: %d mod(s) lack a usable modrinthHash (packwiz stores sha512, not the sha1 crash-assistant wants, or are MR-only). Names/versions are present.\n", parsed-withMR)
	}
}

func parsePwToml(path string) (*pwMod, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var m pwMod
	var section string

	for _, raw := range strings.Split(string(data), "\n") {
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			section = strings.Trim(line, "[]")
			continue
		}
		key, val, ok := splitKV(line)
		if !ok {
			continue
		}
		switch section {
		case "":
			switch key {
			case "name":
				m.name = val
			case "filename":
				m.filename = val
			case "side":
				m.side = val
			}
		case "download":
			switch key {
			case "hash-format":
				m.hashFormat = val
			case "hash":
				m.hash = val
			case "url":
				m.url = val
			}
		case "update.curseforge":
			if key == "file-id" {
				if n, err := parseInt64(val); err == nil {
					m.cfFileID = &n
				}
			}
		case "update.modrinth":
			if key == "mod-id" {
				m.mrModID = val
			}
		}
	}
	if m.filename == "" {
		return nil, fmt.Errorf("no filename field")
	}
	return &m, nil
}

func splitKV(line string) (key, val string, ok bool) {
	idx := strings.Index(line, "=")
	if idx < 0 {
		return "", "", false
	}
	key = strings.TrimSpace(line[:idx])
	val = strings.TrimSpace(line[idx+1:])
	val = strings.Trim(val, `"`)
	return key, val, true
}

func parseInt64(s string) (int64, error) {
	var n int64
	_, err := fmt.Sscanf(s, "%d", &n)
	return n, err
}

func versionFromFilename(filename string) string {
	v := strings.TrimSuffix(filename, ".jar")
	return v
}
