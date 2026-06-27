package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync/atomic"
)

func cmdLint(args []string) {
	var files []string
	if len(args) > 0 {
		files = make([]string, len(args))
		for i, f := range args {
			files[i] = absPath(f)
		}
	} else {
		files = gitChangedFiles()
	}

	var lintable []string
	for _, f := range files {
		if strings.HasSuffix(f, ".json") || strings.HasSuffix(f, ".pw.toml") {
			lintable = append(lintable, f)
		}
	}
	if len(lintable) == 0 {
		fmt.Println("no JSON or .pw.toml files to lint.")
		return
	}

	fmt.Printf("linting %d file(s)...\n", len(lintable))
	failed, checked := runLintFiles(lintable)
	if failed > 0 {
		fail(fmt.Sprintf("%d of %d file(s) failed syntax linting", failed, checked))
	}
	fmt.Printf("\u2713 all %d file(s) parsed OK\n", checked)
}

// runLintFiles lints a list of files concurrently and returns (failed, checked) counts.
func runLintFiles(files []string) (failed, checked int64) {
	sched := NewScheduler(maxConcurrent())
	dones := make([]<-chan error, 0, len(files))
	for _, f := range files {
		if _, err := os.Stat(f); err != nil {
			continue
		}
		atomic.AddInt64(&checked, 1)
		dones = append(dones, sched.Submit(Task{
			Name: f,
			Run: func() error {
				if err := lintOne(f); err != nil {
					errf(f, "%v", err)
					atomic.AddInt64(&failed, 1)
				}
				return nil
			},
		}))
	}
	sched.Close()
	for _, c := range dones {
		<-c
	}
	return failed, checked
}

// autoLintDirs lints all .pw.toml files found under mods/ in each given subdir.
// Warnings are printed but the function never exits \u2014 it's a best-effort post-op check.
func autoLintDirs(dirs []string) {
	seen := map[string]bool{}
	var files []string
	for _, dir := range dirs {
		modsDir := filepath.Join(dir, "mods")
		entries, _ := os.ReadDir(modsDir)
		for _, e := range entries {
			if !e.IsDir() && strings.HasSuffix(e.Name(), ".pw.toml") {
				p := filepath.Join(modsDir, e.Name())
				if !seen[p] {
					files = append(files, p)
					seen[p] = true
				}
			}
		}
	}
	if len(files) == 0 {
		return
	}
	fmt.Printf("linting %d mod file(s)...\n", len(files))
	failed, checked := runLintFiles(files)
	if failed > 0 {
		warnf("%d of %d mod file(s) failed lint \u2014 run 'somnus lint' for details", failed, checked)
	} else {
		fmt.Printf("\u2713 %d mod file(s) lint clean\n", checked)
	}
}

func lintOne(path string) error {
	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("could not read: %w", err)
	}
	if strings.HasSuffix(path, ".json") {
		var v any
		if err := json.Unmarshal(data, &v); err != nil {
			return fmt.Errorf("INVALID JSON: %w", err)
		}
		return nil
	}
	return lintTomlStructure(string(data))
}

func lintTomlStructure(content string) error {
	for i, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		if strings.HasPrefix(line, "[") {
			if !strings.HasSuffix(line, "]") {
				return fmt.Errorf("line %d: malformed section header: %q", i+1, line)
			}
			continue
		}
		if !strings.Contains(line, "=") {
			return fmt.Errorf("line %d: not a section or key=value: %q", i+1, line)
		}
	}
	return nil
}

func gitChangedFiles() []string {
	out, err := exec.Command("git", "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").Output()
	if err != nil {
		warnf("could not read git diff-tree: %v", err)
		return nil
	}
	var files []string
	for l := range strings.SplitSeq(string(out), "\n") {
		if l = strings.TrimSpace(l); l != "" {
			files = append(files, l)
		}
	}
	return files
}
