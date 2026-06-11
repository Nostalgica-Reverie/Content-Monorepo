package main

import (
	"archive/zip"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
)

type mrIndex struct {
	FormatVersion int               `json:"formatVersion"`
	VersionID     string            `json:"versionId"`
	Name          string            `json:"name"`
	Dependencies  map[string]string `json:"dependencies"`
	Files         []mrIndexFile     `json:"files"`
}

type mrIndexFile struct {
	Path      string            `json:"path"`
	Hashes    map[string]string `json:"hashes"`
	Env       map[string]string `json:"env"`
	Downloads []string          `json:"downloads"`
}

var mrCDNRe = regexp.MustCompile(`^https://cdn\.modrinth\.com/data/([^/]+)/versions/([^/]+)/`)

func cmdImport(args []string) {
	if len(args) < 1 {
		failUsage(verbUsage["import"])
	}
	source := args[0]
	customID := ""
	for i := 1; i < len(args); i++ {
		if args[i] == "--id" && i+1 < len(args) {
			customID = args[i+1]
			i++
		}
	}
	if _, err := exec.LookPath(packwizBin()); err != nil {
		failEnv("packwiz not found", "import scaffolds via packwiz init and refresh; install it or set PACKWIZ_BIN")
	}

	mrpackPath := source
	if strings.HasPrefix(source, "http://") || strings.HasPrefix(source, "https://") {
		var err error
		mrpackPath, err = downloadToTemp(source)
		if err != nil {
			fail(fmt.Sprintf("download failed: %v", err))
		}
		defer os.Remove(mrpackPath)
	} else if _, err := os.Stat(source); err != nil {
		failNotFound(fmt.Sprintf("no such file: %s", source))
	}

	zr, err := zip.OpenReader(mrpackPath)
	if err != nil {
		fail(fmt.Sprintf("not a readable zip/mrpack: %v", err))
	}
	defer zr.Close()

	idx, err := readMrIndex(&zr.Reader)
	if err != nil {
		fail(err.Error())
	}

	mc := idx.Dependencies["minecraft"]
	if mc == "" {
		fail("modrinth.index.json has no minecraft dependency")
	}
	loader, loaderVersion := detectLoader(idx.Dependencies)
	if loader == "" {
		fail("could not detect a mod loader in modrinth.index.json dependencies")
	}

	packID := customID
	if packID == "" {
		packID = slugify(idx.Name)
	}
	packDir := filepath.Join("modpacks", packID)
	if _, err := os.Stat(packDir); err == nil {
		fail(fmt.Sprintf("pack already exists: %s (use --id for a different name)", packDir))
	}
	subdir := filepath.Join(packDir, mc+"-mr")
	if err := os.MkdirAll(subdir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create %s: %v", subdir, err))
	}

	fmt.Printf("importing %q (%s, %s %s, mc %s) -> %s\n", idx.Name, idx.VersionID, loader, loaderVersion, mc, packDir)

	initFlag, _ := loaderLatestFlag(loader)
	cmd := exec.Command(packwizBin(), "init",
		"--name", idx.Name,
		"--author", placeholderAuthor,
		"--mc-version", mc,
		"--modloader", loader,
		initFlag,
		"--version", idx.VersionID,
		"-y",
	)
	cmd.Dir = subdir
	if out, err := cmd.CombinedOutput(); err != nil {
		fail(fmt.Sprintf("packwiz init failed in %s: %v\n%s", subdir, err, indent(string(out), "    ")))
	}
	if loaderVersion != "" {
		pinLoaderVersion(filepath.Join(subdir, "pack.toml"), loader, loaderVersion)
	}

	wrote, updatable := 0, 0
	for _, f := range idx.Files {
		if len(f.Downloads) == 0 {
			fmt.Fprintf(os.Stderr, "::warning::%s has no download URL; skipped\n", f.Path)
			continue
		}
		ok, hasUpdate := writeImportedToml(subdir, f)
		if ok {
			wrote++
			if hasUpdate {
				updatable++
			}
		}
	}

	overrides := extractOverrides(&zr.Reader, subdir)

	if err := os.WriteFile(filepath.Join(subdir, ".packwizignore"), []byte(packwizIgnore), 0o644); err != nil {
		fail(fmt.Sprintf("failed to write .packwizignore: %v", err))
	}

	refresh := exec.Command(packwizBin(), "refresh")
	refresh.Dir = subdir
	if out, err := refresh.CombinedOutput(); err != nil {
		fail(fmt.Sprintf("packwiz refresh failed in %s: %v\n%s", subdir, err, indent(string(out), "    ")))
	}

	writeJSON(filepath.Join(packDir, "manifest.json"), map[string]any{
		"$schema":      "../../tools/manifest/schema.json",
		"id":           packID,
		"name":         idx.Name,
		"type":         "modpack",
		"role":         "none",
		"release_type": "release",
		"version":      idx.VersionID,
		"mc_version":   mc,
		"loader":       loader,
		"modrinth_id":  "",
	})
	changelog := fmt.Sprintf("# %s\n\nImported from .mrpack (%s).\n", idx.Name, idx.VersionID)
	_ = os.WriteFile(filepath.Join(packDir, "changelog.md"), []byte(changelog), 0o644)

	fmt.Printf("\nimported %s:\n", packID)
	fmt.Printf("  %d file(s) written (%d with update metadata, %d pinned-by-URL only)\n", wrote, updatable, wrote-updatable)
	fmt.Printf("  %d override file(s) copied\n", overrides)
	fmt.Printf("  manifest.json scaffolded — fill modrinth_id/curseforge_id before publishing\n")
	if wrote > updatable {
		fmt.Printf("  note: files without cdn.modrinth.com URLs lack [update.modrinth]; 'somnus update' will leave them as-is\n")
	}
}

func downloadToTemp(url string) (string, error) {
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("HTTP %d from %s", resp.StatusCode, url)
	}
	tmp, err := os.CreateTemp("", "somnus-import-*.mrpack")
	if err != nil {
		return "", err
	}
	defer tmp.Close()
	n, err := io.Copy(tmp, resp.Body)
	if err != nil {
		os.Remove(tmp.Name())
		return "", err
	}
	fmt.Printf("downloaded %.1f MB\n", float64(n)/1e6)
	return tmp.Name(), nil
}

func readMrIndex(zr *zip.Reader) (*mrIndex, error) {
	for _, f := range zr.File {
		if f.Name != "modrinth.index.json" {
			continue
		}
		rc, err := f.Open()
		if err != nil {
			return nil, fmt.Errorf("failed to open modrinth.index.json: %w", err)
		}
		defer rc.Close()
		var idx mrIndex
		if err := json.NewDecoder(rc).Decode(&idx); err != nil {
			return nil, fmt.Errorf("invalid modrinth.index.json: %w", err)
		}
		return &idx, nil
	}
	return nil, fmt.Errorf("no modrinth.index.json in archive — not an mrpack?")
}

func detectLoader(deps map[string]string) (loader, version string) {
	for key, l := range map[string]string{
		"fabric-loader": "fabric", "quilt-loader": "quilt",
		"forge": "forge", "neoforge": "neoforge",
	} {
		if v, ok := deps[key]; ok {
			return l, v
		}
	}
	return "", ""
}

func pinLoaderVersion(packToml, loader, version string) {
	key := loader
	if loader == "fabric" {
		key = "fabric"
	}
	data, err := os.ReadFile(packToml)
	if err != nil {
		fmt.Fprintf(os.Stderr, "::warning::could not pin loader version: %v\n", err)
		return
	}
	lines := strings.Split(string(data), "\n")
	inVersions := false
	for i, raw := range lines {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[") {
			inVersions = line == "[versions]"
			continue
		}
		if !inVersions {
			continue
		}
		if k, _, ok := splitKV(line); ok && k == key {
			lines[i] = fmt.Sprintf("%s = %q", key, version)
			if err := os.WriteFile(packToml, []byte(strings.Join(lines, "\n")), 0o644); err != nil {
				fmt.Fprintf(os.Stderr, "::warning::could not pin loader version: %v\n", err)
			}
			return
		}
	}
	fmt.Fprintf(os.Stderr, "::warning::no %q key under [versions] in %s; loader left at latest\n", key, packToml)
}

func writeImportedToml(subdir string, f mrIndexFile) (ok bool, hasUpdate bool) {
	base := filepath.Base(f.Path)
	metaPath := filepath.Join(subdir, filepath.Dir(f.Path), strings.TrimSuffix(base, filepath.Ext(base))+".pw.toml")
	if err := os.MkdirAll(filepath.Dir(metaPath), 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "::warning::%s: %v; skipped\n", f.Path, err)
		return false, false
	}

	hashFormat, hash := "sha512", f.Hashes["sha512"]
	if hash == "" {
		hashFormat, hash = "sha1", f.Hashes["sha1"]
	}
	if hash == "" {
		fmt.Fprintf(os.Stderr, "::warning::%s has no sha512/sha1 hash; skipped\n", f.Path)
		return false, false
	}

	var b strings.Builder
	fmt.Fprintf(&b, "name = %q\n", strings.TrimSuffix(base, filepath.Ext(base)))
	fmt.Fprintf(&b, "filename = %q\n", base)
	fmt.Fprintf(&b, "side = %q\n", sideFromEnv(f.Env))
	fmt.Fprintf(&b, "\n[download]\n")
	fmt.Fprintf(&b, "url = %q\n", f.Downloads[0])
	fmt.Fprintf(&b, "hash-format = %q\n", hashFormat)
	fmt.Fprintf(&b, "hash = %q\n", hash)

	if m := mrCDNRe.FindStringSubmatch(f.Downloads[0]); m != nil {
		fmt.Fprintf(&b, "\n[update]\n[update.modrinth]\n")
		fmt.Fprintf(&b, "mod-id = %q\n", m[1])
		fmt.Fprintf(&b, "version = %q\n", m[2])
		hasUpdate = true
	}

	if err := os.WriteFile(metaPath, []byte(b.String()), 0o644); err != nil {
		fmt.Fprintf(os.Stderr, "::warning::failed to write %s: %v\n", metaPath, err)
		return false, false
	}
	return true, hasUpdate
}

func sideFromEnv(env map[string]string) string {
	client := env["client"] != "unsupported"
	server := env["server"] != "unsupported"
	switch {
	case len(env) == 0, client && server:
		return "both"
	case client:
		return "client"
	default:
		return "server"
	}
}

func extractOverrides(zr *zip.Reader, subdir string) int {
	count := 0
	for _, prefix := range []string{"overrides/", "client-overrides/"} {
		for _, f := range zr.File {
			if !strings.HasPrefix(f.Name, prefix) || strings.HasSuffix(f.Name, "/") {
				continue
			}
			rel := strings.TrimPrefix(f.Name, prefix)
			dest := filepath.Join(subdir, filepath.FromSlash(rel))
			if !strings.HasPrefix(filepath.Clean(dest), filepath.Clean(subdir)+string(os.PathSeparator)) {
				fmt.Fprintf(os.Stderr, "::warning::skipping suspicious archive path %s\n", f.Name)
				continue
			}
			if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
				continue
			}
			rc, err := f.Open()
			if err != nil {
				continue
			}
			out, err := os.Create(dest)
			if err != nil {
				rc.Close()
				continue
			}
			_, err = io.Copy(out, rc)
			out.Close()
			rc.Close()
			if err == nil {
				count++
			}
		}
	}
	return count
}

func slugify(name string) string {
	s := strings.ToLower(name)
	s = regexp.MustCompile(`[^a-z0-9]+`).ReplaceAllString(s, "-")
	return strings.Trim(s, "-")
}
