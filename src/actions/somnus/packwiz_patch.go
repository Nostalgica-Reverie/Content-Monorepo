package main

import (
	"embed"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
)

//go:embed patches
var patchesFS embed.FS

const packwizPinnedSHA = "dfd8b68a4796c763e25bad50265ea1f1233e24f1"
const packwizRepo = "https://github.com/packwiz/packwiz"

func cmdPackwiz(args []string) {
	if len(args) < 1 || args[0] != "build" {
		failUsage(verbUsage["packwiz"])
	}

	output := filepath.Join("packwiz-bin", "packwiz")
	if runtime.GOOS == "windows" {
		output += ".exe"
	}
	for i := 1; i < len(args)-1; i++ {
		if args[i] == "--output" {
			output = args[i+1]
			break
		}
	}

	tmpDir, err := os.MkdirTemp("", "somnus-packwiz-*")
	if err != nil {
		fail(fmt.Sprintf("failed to create temp dir: %v", err))
	}
	defer os.RemoveAll(tmpDir)

	repoDir := filepath.Join(tmpDir, "packwiz")

	runIn := func(dir, name string, cmdArgs ...string) {
		cmd := exec.Command(name, cmdArgs...)
		cmd.Dir = dir
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			fail(fmt.Sprintf("%s %v: %v", name, cmdArgs, err))
		}
	}

	fmt.Printf("cloning packwiz @ %s...\n", packwizPinnedSHA[:8])
	runIn("", "git", "clone", "--filter=blob:none", packwizRepo, repoDir)
	runIn("", "git", "-C", repoDir, "checkout", packwizPinnedSHA)

	entries, err := patchesFS.ReadDir("patches")
	if err != nil {
		fail(fmt.Sprintf("failed to read embedded patches: %v", err))
	}
	names := make([]string, 0, len(entries))
	for _, e := range entries {
		if !e.IsDir() {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)

	patchTmp, err := os.MkdirTemp("", "somnus-patches-*")
	if err != nil {
		fail(fmt.Sprintf("failed to create patch staging dir: %v", err))
	}
	defer os.RemoveAll(patchTmp)

	for _, name := range names {
		data, err := patchesFS.ReadFile("patches/" + name)
		if err != nil {
			fail(fmt.Sprintf("failed to read embedded patch %s: %v", name, err))
		}
		patchPath := filepath.Join(patchTmp, name)
		if err := os.WriteFile(patchPath, data, 0o644); err != nil {
			fail(fmt.Sprintf("failed to stage patch %s: %v", name, err))
		}
		fmt.Printf("applying %s...\n", name)
		runIn("", "git", "-C", repoDir, "apply", patchPath)
	}

	if err := os.MkdirAll(filepath.Dir(output), 0o755); err != nil {
		fail(fmt.Sprintf("failed to create output dir: %v", err))
	}
	absOutput, _ := filepath.Abs(output)

	fmt.Printf("building packwiz -> %s...\n", absOutput)
	runIn(repoDir, "go", "build", "-o", absOutput, ".")

	fmt.Printf("done: %s\n", absOutput)
}
