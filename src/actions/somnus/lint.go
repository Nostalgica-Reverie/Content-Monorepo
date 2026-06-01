package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

func cmdLint(args []string) {
	var files []string
	if len(args) > 0 {
		files = args
	} else {
		files = gitChangedFiles()
	}

	var lintable []string
	for _, f := range files {
		if strings.HasSuffix(f, ".json") || strings.HasSuffix(f, ".toml") {
			lintable = append(lintable, f)
		}
	}
	if len(lintable) == 0 {
		fmt.Println("no JSON/TOML files to lint.")
		return
	}

	fmt.Printf("linting %d file(s)...\n", len(lintable))
	checked, failed := 0, 0
	for _, f := range lintable {
		if _, err := os.Stat(f); err != nil {
			continue
		}
		checked++
		if err := lintOne(f); err != nil {
			fmt.Fprintf(os.Stderr, "::error file=%s::%v\n", f, err)
			failed++
		}
	}

	if failed > 0 {
		fail(fmt.Sprintf("%d of %d file(s) failed syntax linting", failed, checked))
	}
	fmt.Printf("\u2713 all %d file(s) parsed OK\n", checked)
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
	if strings.HasSuffix(path, ".pw.toml") {
		return lintTomlStructure(string(data))
	}
	return nil
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
		fmt.Fprintf(os.Stderr, "::warning::could not read git diff-tree: %v\n", err)
		return nil
	}
	var files []string
	for _, l := range strings.Split(string(out), "\n") {
		if l = strings.TrimSpace(l); l != "" {
			files = append(files, l)
		}
	}
	return files
}
