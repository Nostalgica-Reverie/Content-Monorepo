package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

type doctorReport struct {
	Version  string      `json:"version"`
	Tools    []toolCheck `json:"tools"`
	Repo     string      `json:"repo,omitempty"`
	Projects []catCheck  `json:"projects"`
	Problems int         `json:"problems"`
	Warnings int         `json:"warnings"`
	Healthy  bool        `json:"healthy"`
}

type toolCheck struct {
	Name   string `json:"name"`
	Status string `json:"status"` // "ok", "missing", "warn"
	Path   string `json:"path,omitempty"`
	Note   string `json:"note,omitempty"`
}

type catCheck struct {
	Category string   `json:"category"`
	Count    int      `json:"count"`
	Errors   []string `json:"errors,omitempty"`
	Warnings []string `json:"warnings,omitempty"`
}

func cmdDoctor(args []string) {
	asJSON := false
	for _, a := range args {
		if a == "--json" {
			asJSON = true
		}
	}

	report := doctorReport{Version: somnusVersion}
	problems, warnings := 0, 0

	checkTool := func(name, why string, required bool) {
		if p, err := exec.LookPath(name); err == nil {
			report.Tools = append(report.Tools, toolCheck{Name: name, Status: "ok", Path: p})
		} else if required {
			report.Tools = append(report.Tools, toolCheck{Name: name, Status: "missing", Note: why})
			problems++
		} else {
			report.Tools = append(report.Tools, toolCheck{Name: name, Status: "warn", Note: "optional: " + why})
			warnings++
		}
	}
	checkTool("git", "change detection, changelogs, sync anchoring", true)
	checkTool(packwizBin(), "every pack operation", true)
	checkTool("java", "only needed for 'somnus test'", false)
	checkTool("zip", "datapack/resourcepack builds via the publisher", false)
	checkTool("packsquash", "optimized resource pack builds (plain zip used when absent)", false)

	root := findRepoRoot()
	if root == "" {
		report.Problems = problems + 1
		report.Healthy = false
		if asJSON {
			data, _ := json.MarshalIndent(report, "", "  ")
			fmt.Println(string(data))
			return
		}
		fmt.Printf("somnus %s doctor\n\n", somnusVersion)
		for _, t := range report.Tools {
			printToolLine(t)
		}
		fmt.Println("  MISS  repo      no .git or modpacks/ found walking up from here")
		fail("doctor found problems — run somnus from inside the monorepo")
	}
	report.Repo = root

	total, broken := 0, 0
	for _, cat := range []string{"modpacks", "datapacks", "resourcepacks"} {
		dir := filepath.Join(root, cat)
		entries, err := os.ReadDir(dir)
		if err != nil {
			continue
		}
		cc := catCheck{Category: cat}
		for _, e := range entries {
			if !e.IsDir() {
				continue
			}
			mf := filepath.Join(dir, e.Name(), "manifest.json")
			if _, err := os.Stat(mf); err != nil {
				continue
			}
			cc.Count++
			packPath := filepath.Join(dir, e.Name())
			if _, err := ReadManifest(mf); err != nil {
				cc.Errors = append(cc.Errors, fmt.Sprintf("unparsable manifest %s: %v", mf, err))
				broken++
			}
			if _, err := os.Stat(filepath.Join(packPath, "opt-out.json")); err == nil {
				cc.Warnings = append(cc.Warnings, fmt.Sprintf("legacy opt-out.json in %s — migrate to manifest.json automation", packPath))
				warnings++
			}
			if _, err := os.Stat(filepath.Join(packPath, "auto-update-ignore.json")); err == nil {
				cc.Warnings = append(cc.Warnings, fmt.Sprintf("legacy auto-update-ignore.json in %s — migrate to manifest.json automation", packPath))
				warnings++
			}
			if frozen := readAutomation(packPath).Freeze; len(frozen) > 0 {
				for _, p := range pinDrift(packPath, frozen) {
					cc.Warnings = append(cc.Warnings, fmt.Sprintf("freeze drift: %s declared frozen but not pinned — re-run somnus freeze", p))
					warnings++
				}
			}
			if subs, err := os.ReadDir(packPath); err == nil {
				for _, s := range subs {
					if s.IsDir() {
						if _, err := os.Stat(filepath.Join(packPath, s.Name(), "sync-exclude.json")); err == nil {
							cc.Warnings = append(cc.Warnings, fmt.Sprintf("legacy sync-exclude.json in %s — migrate to manifest.json automation", filepath.Join(packPath, s.Name())))
							warnings++
						}
					}
				}
			}
		}
		total += cc.Count
		if cc.Count > 0 || len(cc.Errors) > 0 || len(cc.Warnings) > 0 {
			report.Projects = append(report.Projects, cc)
		}
	}

	report.Problems = problems + broken
	report.Warnings = warnings
	report.Healthy = report.Problems == 0

	if asJSON {
		data, _ := json.MarshalIndent(report, "", "  ")
		fmt.Println(string(data))
		return
	}

	fmt.Printf("somnus %s doctor\n\n", somnusVersion)
	for _, t := range report.Tools {
		printToolLine(t)
	}
	fmt.Printf("  ok    repo      %s\n", root)
	for _, cc := range report.Projects {
		fmt.Printf("  ok    %-9s %d manifest(s)\n", cc.Category, cc.Count)
		for _, e := range cc.Errors {
			fmt.Printf("  BAD   manifest  %s\n", e)
		}
		for _, w := range cc.Warnings {
			fmt.Printf("  warn  legacy    %s\n", w)
		}
	}

	fmt.Printf("\n%d project manifest(s) found, %d unparsable\n", total, broken)
	if report.Problems > 0 {
		fail(fmt.Sprintf("doctor found %d problem(s)", report.Problems))
	}
	fmt.Println("environment looks healthy.")
}

func printToolLine(t toolCheck) {
	switch t.Status {
	case "ok":
		fmt.Printf("  ok    %-9s %s\n", t.Name, t.Path)
	case "missing":
		fmt.Printf("  MISS  %-9s required: %s\n", t.Name, t.Note)
	case "warn":
		fmt.Printf("  warn  %-9s %s\n", t.Name, t.Note)
	}
}
