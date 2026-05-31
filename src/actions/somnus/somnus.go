package main

import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

func main() {
	if len(os.Args) < 2 {
		fail("usage: somnus <init|bump|export|sync|update|refresh|modlist|pages|test|lint|port> [args]")
	}

	switch os.Args[1] {
	case "export", "sync", "update", "refresh", "lint", "pages":
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
	case "update":
		cmdUpdate(os.Args[2:])
	case "refresh":
		cmdRefresh(os.Args[2:])
	case "modlist":
		cmdModlist(os.Args[2:])
	case "pages":
		cmdPages(os.Args[2:])
	case "test":
		cmdTest(os.Args[2:])
	case "lint":
		cmdLint(os.Args[2:])
	case "port":
		cmdPort(os.Args[2:])
	default:
		fail(fmt.Sprintf("unknown verb %q (expected init, bump, export, sync, update, refresh, modlist, pages, test, lint, or port)", os.Args[1]))
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

const (
	defaultMCVersion   = "1.21.1"
	defaultPackVersion = "0.0.0"
	placeholderAuthor  = "CHANGEME"
)

func cmdInit(args []string) {
	if len(args) < 2 {
		fail("usage: somnus init <category> <name> [--mc <version>] [--loader fabric|forge|neoforge|quilt] [--base | --consumes <id>] [--variants a,b,c]\n  category: modpacks | datapacks | resourcepacks")
	}
	category, name := args[0], args[1]
	switch category {
	case "modpacks", "datapacks", "resourcepacks":
	default:
		fail(fmt.Sprintf("invalid category %q (expected modpacks, datapacks, or resourcepacks)", category))
	}

	loader := "fabric"
	mcVersion := defaultMCVersion
	asBase := false
	consumesBase := ""
	var variants []string
	for i := 2; i < len(args); i++ {
		switch args[i] {
		case "--mc":
			if i+1 < len(args) {
				mcVersion = args[i+1]
				i++
			}
		case "--loader":
			if i+1 < len(args) {
				loader = args[i+1]
				i++
			}
		case "--base":
			asBase = true
		case "--consumes":
			if i+1 < len(args) {
				consumesBase = args[i+1]
				i++
			}
		case "--variants":
			if i+1 < len(args) {
				for _, v := range strings.Split(args[i+1], ",") {
					if v = strings.TrimSpace(v); v != "" {
						variants = append(variants, v)
					}
				}
				i++
			}
		}
	}
	if asBase && consumesBase != "" {
		fail("--base and --consumes are mutually exclusive (a pack is either a base or a consumer, not both)")
	}
	loaderFlag, ok := loaderLatestFlag(loader)
	if !ok {
		fail(fmt.Sprintf("invalid loader %q (expected fabric, forge, neoforge, or quilt)", loader))
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
		"version":      defaultPackVersion,
	}

	keys := []string{mcVersion}
	if len(variants) > 0 {
		keys = variants
	}

	switch {
	case asBase:
		manifest["role"] = "base"
	case consumesBase != "":
		var mappings []map[string]string
		for _, key := range keys {
			for _, plat := range []string{"mr", "cf"} {
				mappings = append(mappings, map[string]string{
					"source": "CHANGEME-" + plat,
					"target": key + "-" + plat,
				})
			}
		}
		manifest["role"] = map[string]any{
			"performance_base": map[string]any{
				"pack":     consumesBase,
				"mappings": mappings,
			},
		}
	default:
		manifest["role"] = "none"
	}

	if category == "modpacks" {
		manifest["loader"] = loader
		manifest["mc_version"] = mcVersion
	}
	manifest["modrinth_id"] = name

	if len(variants) > 0 {
		var vs []map[string]string
		for _, v := range variants {
			vs = append(vs, map[string]string{
				"id":         v,
				"mc_version": mcVersion,
				"name":       v,
			})
		}
		manifest["variants"] = vs
	}

	writeJSON(filepath.Join(packDir, "manifest.json"), manifest)

	changelog := fmt.Sprintf("# %s\n\nInitial scaffold. Describe the first release here.\n", name)
	if err := os.WriteFile(filepath.Join(packDir, "changelog.md"), []byte(changelog), 0o644); err != nil {
		fail(fmt.Sprintf("failed to write changelog.md: %v", err))
	}

	roleDesc := "none"
	if asBase {
		roleDesc = "base"
	} else if consumesBase != "" {
		roleDesc = "consumer of " + consumesBase + " (mappings are CHANGEME stubs \u2014 fill them in)"
	}
	fmt.Printf("scaffolded %s\n", packDir)
	fmt.Printf("  manifest.json (role: %s; fill in modrinth_id/curseforge_id, version, author)\n", roleDesc)
	fmt.Printf("  changelog.md\n")

	if category == "modpacks" {
		if _, err := exec.LookPath("packwiz"); err != nil {
			fmt.Println("note: packwiz not on PATH; skipped subdir init. Create the subdirs and run packwiz init manually.")
			return
		}
		for _, key := range keys {
			for _, plat := range []string{"mr", "cf"} {
				sub := filepath.Join(packDir, key+"-"+plat)
				if err := os.MkdirAll(sub, 0o755); err != nil {
					fail(fmt.Sprintf("failed to create %s: %v", sub, err))
				}
				fmt.Printf("  packwiz init in %s ...\n", sub)
				cmd := exec.Command("packwiz", "init",
					"--name", name,
					"--author", placeholderAuthor,
					"--mc-version", mcVersion,
					"--modloader", loader,
					loaderFlag,
					"--version", defaultPackVersion,
					"-y",
				)
				cmd.Dir = sub
				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr
				if err := cmd.Run(); err != nil {
					fail(fmt.Sprintf("packwiz init failed in %s: %v", sub, err))
				}
			}
		}
		fmt.Printf("ready: %s initialized %d subdir-pair(s) (%s, latest). Add mods with packwiz, then fill manifest placeholders.\n",
			packDir, len(keys), loader)
	} else {
		fmt.Printf("next: create %s/{version}/ and add the pack contents (pack.mcmeta at its root).\n", packDir)
	}
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

func loaderLatestFlag(loader string) (string, bool) {
	switch loader {
	case "fabric":
		return "--fabric-latest", true
	case "forge":
		return "--forge-latest", true
	case "neoforge":
		return "--neoforge-latest", true
	case "quilt":
		return "--quilt-latest", true
	default:
		return "", false
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

func cmdSync(args []string)    { maintainWrapper("sync") }
func cmdUpdate(args []string)  { maintainWrapper("update") }
func cmdRefresh(args []string) { maintainWrapper("refresh") }

func maintainWrapper(subcommand string) {
	bin := findBinary("MAINTAIN_BIN", "maintain", "./maintain-bin/maintain")
	if bin == "" {
		fail("maintain binary not found. Build it (go build -C src/actions/maintain -o maintain-bin/maintain .) or set $MAINTAIN_BIN")
	}
	fmt.Printf("somnus %s -> running maintain %s (%s)\n", subcommand, subcommand, bin)
	runPassthrough(bin, subcommand)
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
	side       string
	url        string
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

func cmdPages(args []string) {
	var packArg string
	if len(args) > 0 {
		packArg = args[0]
	}

	var subdirs []string
	if packArg != "" {
		subdirs = packModSubdirs(packArg)
		if len(subdirs) == 0 {
			fail(fmt.Sprintf("no mod subdirs found under %s", packArg))
		}
	} else {
		root := modpacksRoot()
		packs, err := os.ReadDir(root)
		if err != nil {
			fail(fmt.Sprintf("failed to read %s: %v", root, err))
		}
		for _, p := range packs {
			if p.IsDir() {
				subdirs = append(subdirs, packModSubdirs(filepath.Join(root, p.Name()))...)
			}
		}
		if len(subdirs) == 0 {
			fail("no mod subdirs found in any pack")
		}
	}

	written := 0
	for _, sub := range subdirs {
		n, err := writeModlistMD(sub)
		if err != nil {
			fmt.Fprintf(os.Stderr, "::warning::%s: %v\n", sub, err)
			continue
		}
		fmt.Printf("wrote %s/modlist.md (%d mods)\n", sub, n)
		written++
	}
	fmt.Printf("generated %d modlist.md file(s).\n", written)
}

func modpacksRoot() string {
	if d := os.Getenv("MODPACKS_DIR"); d != "" {
		return d
	}
	return "modpacks"
}

func packModSubdirs(packDir string) []string {
	var out []string
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return nil
	}
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		if _, err := os.Stat(filepath.Join(packDir, e.Name(), "mods")); err == nil {
			out = append(out, filepath.Join(packDir, e.Name()))
		}
	}
	return out
}

func writeModlistMD(subdir string) (int, error) {
	modsDir := filepath.Join(subdir, "mods")
	entries, err := os.ReadDir(modsDir)
	if err != nil {
		return 0, err
	}

	var client, shared, server []string
	count := 0
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".pw.toml") {
			continue
		}
		mod, err := parsePwToml(filepath.Join(modsDir, e.Name()))
		if err != nil {
			continue
		}
		count++
		line := fmt.Sprintf("- [%s](%s)", mod.name, modPageURL(mod))
		switch mod.side {
		case "client":
			client = append(client, line)
		case "server":
			server = append(server, line)
		default:
			shared = append(shared, line)
		}
	}

	var b strings.Builder
	b.WriteString("# Modlist\n")
	writeSection(&b, "Client Mods", client)
	writeSection(&b, "Shared Mods", shared)
	writeSection(&b, "Server Mods", server)

	out := filepath.Join(subdir, "modlist.md")
	if err := os.WriteFile(out, []byte(b.String()), 0o644); err != nil {
		return 0, err
	}
	return count, nil
}

func writeSection(b *strings.Builder, title string, lines []string) {
	if len(lines) == 0 {
		return
	}
	sort.Strings(lines)
	b.WriteString("\n## " + title + "\n\n")
	for _, l := range lines {
		b.WriteString(l + "\n")
	}
}

func modPageURL(m *pwMod) string {
	if m.mrModID != "" {
		return "https://modrinth.com/mod/" + m.mrModID
	}
	if m.cfFileID != nil && m.url != "" {
		return m.url
	}
	if m.url != "" {
		return m.url
	}
	return ""
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
	fmt.Printf("✓ all %d file(s) parsed OK\n", checked)
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

func cmdPort(args []string) {
	if len(args) < 2 {
		fail("usage: somnus port <mr-subdir> <cf-subdir> [--add]\n" +
			"  e.g. somnus port modpacks/rc-plus/26.1.2-mr modpacks/rc-plus/26.1.2-cf")
	}
	mrDir, cfDir := args[0], args[1]
	doAdd := false
	for _, a := range args[2:] {
		if a == "--add" {
			doAdd = true
		}
	}

	mrMods := filepath.Join(mrDir, "mods")
	if info, err := os.Stat(mrMods); err != nil || !info.IsDir() {
		fail(fmt.Sprintf("no mods/ in MR subdir %s", mrDir))
	}
	if _, err := os.Stat(filepath.Join(cfDir, "pack.toml")); err != nil {
		fail(fmt.Sprintf("CF subdir %s has no pack.toml (run packwiz/somnus init there first)", cfDir))
	}
	if doAdd {
		if _, err := exec.LookPath("packwiz"); err != nil {
			fail("packwiz not found in PATH")
		}
	}

	entries, err := os.ReadDir(mrMods)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", mrMods, err))
	}
	var mrNames []string
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".pw.toml") {
			mrNames = append(mrNames, strings.TrimSuffix(e.Name(), ".pw.toml"))
		}
	}

	cfMods := filepath.Join(cfDir, "mods")
	existing := make(map[string]bool)
	if cfEntries, err := os.ReadDir(cfMods); err == nil {
		for _, e := range cfEntries {
			if !e.IsDir() && strings.HasSuffix(e.Name(), ".pw.toml") {
				existing[strings.TrimSuffix(e.Name(), ".pw.toml")] = true
			}
		}
	}

	var missing []string
	for _, n := range mrNames {
		if !existing[n] {
			missing = append(missing, n)
		}
	}

	fmt.Printf("MR mods: %d | already on CF: %d | missing on CF: %d\n",
		len(mrNames), len(mrNames)-len(missing), len(missing))
	if len(missing) == 0 {
		fmt.Println("nothing to port \u2014 CF side already has matching slugs for every MR mod.")
		return
	}

	if !doAdd {
		fmt.Println("\nmods needing a CF entry (re-run with --add to add them interactively):")
		for _, n := range missing {
			fmt.Printf("  - %s\n", n)
		}
		fmt.Println("\nNote: 'missing' is matched by .pw.toml slug. Some may be Modrinth-only")
		fmt.Println("(no CF release) \u2014 those will simply not be found when you --add, which is fine.")
		return
	}

	fmt.Printf("\nadding %d mod(s) to %s via packwiz (you confirm each match; no -y)\n", len(missing), cfDir)
	var added, skipped, notFound []string
	for _, n := range missing {
		fmt.Printf("\n=== %s ===\n", n)
		cmd := exec.Command("packwiz", "curseforge", "add", n)
		cmd.Dir = cfDir
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "  (packwiz add did not complete for %s: %v)\n", n, err)
			notFound = append(notFound, n)
			continue
		}
		if _, err := os.Stat(filepath.Join(cfMods, n+".pw.toml")); err == nil {
			added = append(added, n)
		} else {
			skipped = append(skipped, n)
		}
	}

	fmt.Printf("\nport summary for %s:\n", cfDir)
	fmt.Printf("  added: %d\n", len(added))
	fmt.Printf("  skipped (declined/no file written): %d\n", len(skipped))
	fmt.Printf("  not found / errored: %d\n", len(notFound))
	if len(notFound) > 0 {
		fmt.Println("  these likely have no CurseForge release (Modrinth-only) \u2014 handle manually:")
		for _, n := range notFound {
			fmt.Printf("    - %s\n", n)
		}
	}
	fmt.Println("\nremember to run packwiz refresh in the CF subdir and verify the matches are correct.")
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
