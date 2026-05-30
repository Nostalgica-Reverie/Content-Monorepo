package main

import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

func main() {
	if len(os.Args) < 2 {
		fail("usage: somnus <init|bump|export|sync|modlist|test> [args]")
	}

	switch os.Args[1] {
	case "export", "sync":
		if root := findRepoRoot(); root != "" {
			if err := os.Chdir(root); err != nil {
				fail(fmt.Sprintf("failed to enter repo root %s: %v", root, err))
			}
		} else {
			fail("could not locate repo root (no .git or modpacks/ found walking up from here)")
		}
	}

	switch os.Args[1] {
	case "init":
		cmdInit(os.Args[2:])
	case "bump":
		cmdBump(os.Args[2:])
	case "export":
		cmdExport(os.Args[2:])
	case "sync":
		cmdSync(os.Args[2:])
	case "modlist":
		cmdModlist(os.Args[2:])
	case "test":
		cmdTest(os.Args[2:])
	default:
		fail(fmt.Sprintf("unknown verb %q (expected init, bump, export, sync, modlist, or test)", os.Args[1]))
	}
}

func findRepoRoot() string {
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for {
		if _, err := os.Stat(filepath.Join(dir, ".git")); err == nil {
			return dir
		}
		if info, err := os.Stat(filepath.Join(dir, "modpacks")); err == nil && info.IsDir() {
			return dir
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return ""
		}
		dir = parent
	}
}

func cmdInit(args []string) {
	if len(args) < 2 {
		fail("usage: somnus init <category> <name>\n  category: modpacks | datapacks | resourcepacks")
	}
	category, name := args[0], args[1]
	switch category {
	case "modpacks", "datapacks", "resourcepacks":
	default:
		fail(fmt.Sprintf("invalid category %q (expected modpacks, datapacks, or resourcepacks)", category))
	}

	packDir := filepath.Join(category, name)
	if _, err := os.Stat(packDir); err == nil {
		fail(fmt.Sprintf("pack already exists: %s", packDir))
	}
	if err := os.MkdirAll(packDir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create %s: %v", packDir, err))
	}

	manifest := map[string]any{
		"$schema":      "../../tools/manifest/schema.json",
		"id":           name,
		"name":         name,
		"type":         categoryType(category),
		"release_type": "release",
		"version":      "0.0.0",
		"role":         "none",
	}
	if category == "modpacks" {
		manifest["loader"] = "fabric"
		manifest["mc_version"] = "1.21.1"
	}
	manifest["modrinth_id"] = name

	writeJSON(filepath.Join(packDir, "manifest.json"), manifest)

	changelog := fmt.Sprintf("# %s\n\nInitial scaffold. Describe the first release here.\n", name)
	if err := os.WriteFile(filepath.Join(packDir, "changelog.md"), []byte(changelog), 0o644); err != nil {
		fail(fmt.Sprintf("failed to write changelog.md: %v", err))
	}

	fmt.Printf("scaffolded %s\n", packDir)
	fmt.Printf("  manifest.json (role: none, fill in modrinth_id/curseforge_id and version)\n")
	fmt.Printf("  changelog.md\n")
	fmt.Printf("next: add platform subdirs (e.g. %s/1.21.1-mr) and run packwiz init there.\n", packDir)
}

func categoryType(category string) string {
	switch category {
	case "datapacks":
		return "datapack"
	case "resourcepacks":
		return "resourcepack"
	default:
		return "modpack"
	}
}

func cmdBump(args []string) {
	if len(args) < 2 {
		fail("usage: somnus bump <pack-dir> <new-version>\n  e.g. somnus bump modpacks/rc-plus 26.06.1")
	}
	packDir, newVer := args[0], args[1]
	mfPath := filepath.Join(packDir, "manifest.json")

	data, err := os.ReadFile(mfPath)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", mfPath, err))
	}
	var obj map[string]any
	if err := json.Unmarshal(data, &obj); err != nil {
		fail(fmt.Sprintf("invalid JSON in %s: %v", mfPath, err))
	}
	old, _ := obj["version"].(string)
	obj["version"] = newVer
	writeJSON(mfPath, obj)
	fmt.Printf("bumped %s: %s -> %s\n", mfPath, old, newVer)
}

func cmdExport(args []string) {
	bin := findBinary("BUILDER_BIN", "builder", "./builder-bin/builder")
	if bin == "" {
		fail("builder binary not found. Build it (go build -C src/actions/builder -o builder-bin/builder .) or set $BUILDER_BIN")
	}

	if len(args) > 0 {
		pack := args[0]
		fmt.Printf("somnus export -> running builder --pack %s (%s)\n", pack, bin)
		runPassthrough(bin, "--pack", pack, "local")
		return
	}
	fmt.Printf("somnus export -> running builder (%s)\n", bin)
	runPassthrough(bin, "local")
}

func cmdSync(args []string) {
	bin := findBinary("MAINTAIN_BIN", "maintain", "./maintain-bin/maintain")
	if bin == "" {
		fail("maintain binary not found. Build it (go build -C src/actions/maintain -o maintain-bin/maintain .) or set $MAINTAIN_BIN")
	}
	fmt.Printf("somnus sync -> running maintain sync (%s)\n", bin)
	runPassthrough(bin, "sync")
}

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
	hashFormat string
	hash       string
	cfFileID   *int64
	mrModID    string
}

func cmdModlist(args []string) {
	if len(args) < 1 {
		fail("usage: somnus modlist <pack-subdir>\n  e.g. somnus modlist modpacks/rc-plus/26.1.2-mr")
	}
	subdir := args[0]
	modsDir := filepath.Join(subdir, "mods")
	if info, err := os.Stat(modsDir); err != nil || !info.IsDir() {
		fail(fmt.Sprintf("no mods/ directory at %s", modsDir))
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
			fmt.Fprintf(os.Stderr, "::warning::skipping %s: %v\n", e.Name(), err)
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
			}
		case "download":
			switch key {
			case "hash-format":
				m.hashFormat = val
			case "hash":
				m.hash = val
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

const servePort = "8080"

func cmdTest(args []string) {
	if len(args) < 1 {
		fail("usage: somnus test <pack-subdir>\n  e.g. somnus test modpacks/rc-plus/26.1.2-mr")
	}
	packSubdir := args[0]

	if _, err := os.Stat(filepath.Join(packSubdir, "pack.toml")); err != nil {
		fail(fmt.Sprintf("no pack.toml in %s", packSubdir))
	}
	if _, err := exec.LookPath("packwiz"); err != nil {
		fail("packwiz not found in PATH")
	}
	if _, err := exec.LookPath("java"); err != nil {
		fail("java not found in PATH (packwiz-installer is a Java jar)")
	}
	installerJar := os.Getenv("PACKWIZ_INSTALLER_JAR")
	if installerJar == "" {
		fail("set $PACKWIZ_INSTALLER_JAR to the packwiz-installer-bootstrap.jar path\n  (download from https://github.com/packwiz/packwiz-installer-bootstrap/releases)")
	}
	if _, err := os.Stat(installerJar); err != nil {
		fail(fmt.Sprintf("packwiz-installer jar not found at %s", installerJar))
	}

	instanceDir := os.Getenv("SOMNUS_TEST_INSTANCE")
	if instanceDir == "" {
		instanceDir = "./.somnus-test-instance"
		fmt.Printf("SOMNUS_TEST_INSTANCE unset; using default %s\n", instanceDir)
	}
	if err := os.MkdirAll(instanceDir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create instance dir %s: %v", instanceDir, err))
	}
	absInstance, err := filepath.Abs(instanceDir)
	if err != nil {
		fail(fmt.Sprintf("failed to resolve instance dir: %v", err))
	}
	absJar, err := filepath.Abs(installerJar)
	if err != nil {
		fail(fmt.Sprintf("failed to resolve jar path: %v", err))
	}

	fmt.Printf("starting packwiz serve in %s ...\n", packSubdir)
	serve := exec.Command("packwiz", "serve", "--port", servePort)
	serve.Dir = packSubdir
	serve.Stdout = os.Stderr
	serve.Stderr = os.Stderr
	if err := serve.Start(); err != nil {
		fail(fmt.Sprintf("failed to start packwiz serve: %v", err))
	}
	defer func() {
		if serve.Process != nil {
			_ = serve.Process.Kill()
			_, _ = serve.Process.Wait()
		}
	}()

	if !waitForPort("127.0.0.1:"+servePort, 10*time.Second) {
		fail("packwiz serve did not become ready on port " + servePort)
	}
	fmt.Println("packwiz serve is up.")

	packURL := fmt.Sprintf("http://localhost:%s/pack.toml", servePort)
	fmt.Printf("installing pack into %s ...\n", absInstance)
	installer := exec.Command("java", "-jar", absJar, "-g", packURL)
	installer.Dir = absInstance
	installer.Stdout = os.Stdout
	installer.Stderr = os.Stderr
	if err := installer.Run(); err != nil {
		fail(fmt.Sprintf("packwiz-installer failed: %v", err))
	}

	fmt.Printf("\ntest instance ready at %s\n", absInstance)
	fmt.Println("point your launcher (MultiMC/Prism) at it, or launch from there. (somnus does not launch the game.)")
}

func waitForPort(addr string, timeout time.Duration) bool {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		conn, err := net.DialTimeout("tcp", addr, 500*time.Millisecond)
		if err == nil {
			_ = conn.Close()
			return true
		}
		time.Sleep(200 * time.Millisecond)
	}
	return false
}

func findBinary(envVar, name, fallback string) string {
	if v := os.Getenv(envVar); v != "" {
		return v
	}
	if p, err := exec.LookPath(name); err == nil {
		return p
	}
	if _, err := os.Stat(fallback); err == nil {
		return fallback
	}
	return ""
}

func runPassthrough(bin string, args ...string) {
	cmd := exec.Command(bin, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			os.Exit(exitErr.ExitCode())
		}
		fail(fmt.Sprintf("failed to run %s: %v", bin, err))
	}
}

func writeJSON(path string, v any) {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		fail(fmt.Sprintf("failed to marshal JSON: %v", err))
	}
	data = append(data, '\n')
	if err := os.WriteFile(path, data, 0o644); err != nil {
		fail(fmt.Sprintf("failed to write %s: %v", path, err))
	}
}

func fail(msg string) {
	fmt.Fprintf(os.Stderr, "::error::%s\n", msg)
	os.Exit(1)
}
