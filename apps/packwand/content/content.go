package content

import (
	"archive/zip"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	initCmd.Flags().String("mc", "", "Minecraft version (default: 26.1.2)")
	initCmd.Flags().String("loader", "fabric", "Mod loader: fabric, forge, neoforge, quilt")
	initCmd.Flags().Bool("base", false, "Scaffold this pack as a performance base")
	initCmd.Flags().String("consumes", "", "ID of the performance base this pack consumes")
	initCmd.Flags().String("variants", "", "Comma-separated variant IDs (multi-MC-version packs)")
	cmd.AddToGroup(initCmd, cmd.GroupPackManagement)

	addCmd.Flags().Bool("no-refresh", false, "Skip packwand refresh after add (batch adds)")
	cmd.AddToGroup(addCmd, cmd.GroupPackManagement)

	portCmd.Flags().Bool("add", false, "Interactively add missing CurseForge entries via packwand")
	portCmd.Flags().Bool("no-refresh", false, "Batch the refresh until the end")
	portCmd.Flags().Bool("json", false, "Output missing list as JSON (dry-run only)")
	cmd.AddToGroup(portCmd, cmd.GroupPackManagement)

	importCmd.Flags().String("id", "", "Override the pack ID derived from the archive name")
	cmd.AddToGroup(importCmd, cmd.GroupPackManagement)

	cmd.AddToGroup(testCmd, cmd.GroupInfo)
}

// — init —

const (
	defaultMCVersion   = "26.1.2"
	defaultPackVersion = "26.x"
	placeholderAuthor  = "CHANGEME"
	packwizIgnore      = "Logs\n*.zip\n*.mrpack\n"
)

var initCmd = &cobra.Command{
	Use:   "new <modpacks|datapacks|resourcepacks> <name>",
	Short: "Scaffold a new pack (manifest.json, changelog.md, packwiz subdirs)",
	Args:  cobra.ExactArgs(2),
	Run: func(c *cobra.Command, args []string) {
		category, name := args[0], args[1]
		switch category {
		case "modpacks", "datapacks", "resourcepacks":
		default:
			cmd.Fail(fmt.Sprintf("invalid category %q (expected modpacks, datapacks, or resourcepacks)", category))
		}

		mcVersion, _ := c.Flags().GetString("mc")
		if mcVersion == "" {
			mcVersion = defaultMCVersion
		}
		loader, _ := c.Flags().GetString("loader")
		asBase, _ := c.Flags().GetBool("base")
		consumesBase, _ := c.Flags().GetString("consumes")
		variantsStr, _ := c.Flags().GetString("variants")

		var variants []string
		for _, v := range strings.Split(variantsStr, ",") {
			if v = strings.TrimSpace(v); v != "" {
				variants = append(variants, v)
			}
		}

		if asBase && consumesBase != "" {
			cmd.Fail("--base and --consumes are mutually exclusive")
		}

		loaderFlag, ok := loaderLatestFlag(loader)
		if !ok {
			cmd.Fail(fmt.Sprintf("invalid loader %q (expected fabric, forge, neoforge, or quilt)", loader))
		}

		cmd.Chdir()

		packDir := filepath.Join(category, name)
		if _, err := os.Stat(packDir); err == nil {
			cmd.Fail(fmt.Sprintf("pack already exists: %s", packDir))
		}
		if err := os.MkdirAll(packDir, 0o755); err != nil {
			cmd.Fail(fmt.Sprintf("failed to create %s: %v", packDir, err))
		}

		mf := map[string]any{
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
			mf["role"] = "base"
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
			mf["role"] = map[string]any{
				"performance_base": map[string]any{
					"pack":     consumesBase,
					"mappings": mappings,
				},
			}
		default:
			mf["role"] = "none"
		}

		if category == "modpacks" {
			mf["loader"] = loader
			mf["mc_version"] = mcVersion
		}
		mf["modrinth_id"] = name

		if len(variants) > 0 {
			var vs []map[string]string
			for _, v := range variants {
				vs = append(vs, map[string]string{
					"id":         v,
					"mc_version": mcVersion,
					"name":       v,
				})
			}
			mf["variants"] = vs
		}

		cmd.WriteJSON(filepath.Join(packDir, "manifest.json"), mf)

		changelog := fmt.Sprintf("# %s\n\nInitial scaffold. Describe the first release here.\n", name)
		if err := os.WriteFile(filepath.Join(packDir, "changelog.md"), []byte(changelog), 0o644); err != nil {
			cmd.Fail(fmt.Sprintf("failed to write changelog.md: %v", err))
		}

		roleDesc := "none"
		if asBase {
			roleDesc = "base"
		} else if consumesBase != "" {
			roleDesc = "consumer of " + consumesBase + " (mappings are CHANGEME stubs — fill them in)"
		}
		fmt.Printf("scaffolded %s\n", packDir)
		fmt.Printf("  manifest.json (role: %s; fill in modrinth_id/curseforge_id, version, author)\n", roleDesc)
		fmt.Printf("  changelog.md\n")

		if category == "modpacks" {
			if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
				fmt.Println("note: packwand not on PATH; skipped subdir init. Create the subdirs and run packwand init manually.")
				return
			}
			for _, key := range keys {
				for _, plat := range []string{"mr", "cf"} {
					sub := filepath.Join(packDir, key+"-"+plat)
					if err := os.MkdirAll(sub, 0o755); err != nil {
						cmd.Fail(fmt.Sprintf("failed to create %s: %v", sub, err))
					}
					fmt.Printf("  packwand init in %s ...\n", sub)
					c := exec.Command(workspace.SelfBin(), "init",
						"--name", name,
						"--author", placeholderAuthor,
						"--mc-version", mcVersion,
						"--modloader", loader,
						loaderFlag,
						"--version", defaultPackVersion,
						"-y",
					)
					c.Dir = sub
					c.Stdout = os.Stdout
					c.Stderr = os.Stderr
					if err := c.Run(); err != nil {
						cmd.Fail(fmt.Sprintf("packwand init failed in %s: %v", sub, err))
					}
					if err := os.WriteFile(filepath.Join(sub, ".packwizignore"), []byte(packwizIgnore), 0o644); err != nil {
						cmd.Fail(fmt.Sprintf("failed to write .packwizignore in %s: %v", sub, err))
					}
				}
			}
			fmt.Printf("ready: %s initialized %d subdir-pair(s) (%s, latest) with .packwizignore. Add mods with packwand, then fill manifest placeholders.\n",
				packDir, len(keys), loader)
		} else {
			fmt.Printf("next: create %s/{version}/ and add the pack contents (pack.mcmeta at its root).\n", packDir)
		}
	},
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

// splitKV splits a TOML "key = value" line into key and unquoted value.
// Returns ok=false if no '=' is present.
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

// — add —

var addCmd = &cobra.Command{
	Use:   "add <mod-slug> [pack-dir|pack-subdir]",
	Short: "Add a mod to all (or a specific) pack's Modrinth and CurseForge subdirs",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(c *cobra.Command, args []string) {
		slug := args[0]
		noRefresh, _ := c.Flags().GetBool("no-refresh")

		var targetArg string
		if len(args) > 1 {
			targetArg = cmd.Abs(strings.TrimRight(args[1], "/"))
		}

		if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
			cmd.Fail("packwand not found on PATH — install it or set PACKWAND_BIN")
		}

		cmd.Chdir()

		targets := resolveAddTargets(targetArg)
		if len(targets) == 0 {
			fmt.Println("no pack subdirs found")
			return
		}

		fmt.Printf("adding %q to %d subdir(s)\n\n", slug, len(targets))
		added, failed, skipped := 0, 0, 0

		for _, dir := range targets {
			plat := cmd.PlatformSuffix(dir)
			var pwArgs []string
			switch plat {
			case "mr":
				pwArgs = []string{"modrinth", "add", "-y", slug}
			case "cf":
				pwArgs = []string{"curseforge", "add", "-y", slug}
			default:
				cmd.Warn("skipping %s — unrecognised suffix (need -mr or -cf)", dir)
				skipped++
				continue
			}
			if noRefresh {
				pwArgs = append(pwArgs, "--no-refresh")
			}

			fmt.Printf("[%s] %s\n", plat, dir)
			ex := exec.Command(workspace.SelfBin(), pwArgs...)
			ex.Dir = dir
			ex.Stdout = os.Stdout
			ex.Stderr = os.Stderr
			if err := ex.Run(); err != nil {
				cmd.Warn("%s: add failed — slug may not exist on %s under this name", dir, plat)
				failed++
				continue
			}
			added++
		}

		fmt.Printf("\nadd summary: %d added  %d not found/failed  %d skipped\n", added, failed, skipped)
		if failed > 0 && skipped < len(targets) {
			fmt.Printf("note: failures are expected when a mod has no release on that platform\n")
		}
		if added > 0 {
			workspace.AutoLintDirs(targets)
		}
	},
}

func resolveAddTargets(targetArg string) []string {
	if targetArg == "" {
		root := workspace.ModpacksDir()
		entries, err := os.ReadDir(root)
		if err != nil {
			cmd.Fail(fmt.Sprintf("failed to read %s: %v", root, err))
		}
		var out []string
		for _, e := range entries {
			if e.IsDir() {
				out = append(out, manifest.SubDirsOf(filepath.Join(root, e.Name()))...)
			}
		}
		return out
	}

	base := filepath.Base(targetArg)
	if strings.HasSuffix(base, "-mr") || strings.HasSuffix(base, "-cf") {
		if _, err := os.Stat(targetArg); err != nil {
			cmd.Fail(fmt.Sprintf("subdir not found: %s", targetArg))
		}
		return []string{targetArg}
	}

	if _, err := os.Stat(filepath.Join(targetArg, "manifest.json")); err != nil {
		cmd.Fail(fmt.Sprintf("no manifest.json in %s — pass a pack dir, a subdir, or nothing for all packs", targetArg))
	}
	return manifest.SubDirsOf(targetArg)
}

// — port —

type portResult struct {
	MRTotal   int      `json:"mr_total"`
	CFMatched int      `json:"cf_matched"`
	Missing   []string `json:"missing"`
}

var portCmd = &cobra.Command{
	Use:   "port <mr-subdir> <cf-subdir>",
	Short: "Compare MR and CF subdirs and port missing mods from Modrinth to CurseForge",
	Args:  cobra.ExactArgs(2),
	Run: func(c *cobra.Command, args []string) {
		mrDir := cmd.Abs(args[0])
		cfDir := cmd.Abs(args[1])
		doAdd, _ := c.Flags().GetBool("add")
		noRefresh, _ := c.Flags().GetBool("no-refresh")
		asJSON, _ := c.Flags().GetBool("json")

		cmd.Chdir()

		mrMods := filepath.Join(mrDir, "mods")
		if info, err := os.Stat(mrMods); err != nil || !info.IsDir() {
			cmd.Fail(fmt.Sprintf("no mods/ in MR subdir %s", mrDir))
		}
		if _, err := os.Stat(filepath.Join(cfDir, "pack.toml")); err != nil {
			cmd.Fail(fmt.Sprintf("CF subdir %s has no pack.toml (run packwand init there first)", cfDir))
		}
		if doAdd {
			if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
				cmd.Fail("packwand not found; install it or set PACKWAND_BIN")
			}
		}

		entries, err := os.ReadDir(mrMods)
		if err != nil {
			cmd.Fail(fmt.Sprintf("failed to read %s: %v", mrMods, err))
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
			fmt.Println("nothing to port — CF side already has matching slugs for every MR mod.")
			return
		}

		if !doAdd {
			if asJSON {
				res := portResult{
					MRTotal:   len(mrNames),
					CFMatched: len(mrNames) - len(missing),
					Missing:   missing,
				}
				if res.Missing == nil {
					res.Missing = []string{}
				}
				data, _ := json.MarshalIndent(res, "", "  ")
				fmt.Println(string(data))
				return
			}
			fmt.Println("\nmods needing a CF entry (re-run with --add to add them interactively):")
			for _, n := range missing {
				fmt.Printf("  - %s\n", n)
			}
			fmt.Println("\nNote: 'missing' is matched by .pw.toml slug. Some may be Modrinth-only")
			fmt.Println("(no CF release) — those will simply not be found when you --add, which is fine.")
			return
		}

		fmt.Printf("\nadding %d mod(s) to %s via packwand (you confirm each match; no -y)\n", len(missing), cfDir)
		var added, skipped, notFound []string
		for _, n := range missing {
			fmt.Printf("\n=== %s ===\n", n)
			addArgs := []string{"curseforge", "add", n}
			if noRefresh {
				addArgs = append(addArgs, "--no-refresh")
			}
			ex := exec.Command(workspace.SelfBin(), addArgs...)
			ex.Dir = cfDir
			ex.Stdin = os.Stdin
			ex.Stdout = os.Stdout
			ex.Stderr = os.Stderr
			if err := ex.Run(); err != nil {
				fmt.Fprintf(os.Stderr, "  (packwand add did not complete for %s: %v)\n", n, err)
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
			fmt.Println("  these likely have no CurseForge release (Modrinth-only) — handle manually:")
			for _, n := range notFound {
				fmt.Printf("    - %s\n", n)
			}
		}
		if noRefresh && len(added) > 0 {
			fmt.Printf("\nrunning a single packwand refresh in %s ...\n", cfDir)
			ex := exec.Command(workspace.SelfBin(), "refresh")
			ex.Dir = cfDir
			if out, err := ex.CombinedOutput(); err != nil {
				cmd.Fail(fmt.Sprintf("final refresh failed in %s: %v\n%s", cfDir, err, out))
			}
			fmt.Println("index finalized; verify the matches are correct.")
			return
		}
		fmt.Println("\nremember to run packwand refresh in the CF subdir and verify the matches are correct.")
	},
}

// — import —

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

type cfManifest struct {
	ManifestType string      `json:"manifestType"`
	Name         string      `json:"name"`
	Version      string      `json:"version"`
	Author       string      `json:"author"`
	Minecraft    cfMinecraft `json:"minecraft"`
	Files        []cfFile    `json:"files"`
	Overrides    string      `json:"overrides"`
}

type cfMinecraft struct {
	Version    string        `json:"version"`
	ModLoaders []cfModLoader `json:"modLoaders"`
}

type cfModLoader struct {
	ID      string `json:"id"`
	Primary bool   `json:"primary"`
}

type cfFile struct {
	ProjectID int  `json:"projectID"`
	FileID    int  `json:"fileID"`
	Required  bool `json:"required"`
}

var importCmd = &cobra.Command{
	Use:   "import <mrpack-or-cf-zip|URL>",
	Short: "Import an .mrpack or CurseForge zip as a new modpack",
	Args:  cobra.ExactArgs(1),
	Run: func(c *cobra.Command, args []string) {
		source := args[0]
		if !strings.HasPrefix(source, "http://") && !strings.HasPrefix(source, "https://") {
			source = cmd.Abs(source)
		}
		customID, _ := c.Flags().GetString("id")

		if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
			cmd.Fail("packwand not found; ll-import scaffolds via packwand init and refresh — install it or set PACKWAND_BIN")
		}

		cmd.Chdir()

		archivePath := source
		if strings.HasPrefix(source, "http://") || strings.HasPrefix(source, "https://") {
			var err error
			archivePath, err = downloadToTemp(source)
			if err != nil {
				cmd.Fail(fmt.Sprintf("download failed: %v", err))
			}
			defer os.Remove(archivePath)
		} else if _, err := os.Stat(source); err != nil {
			cmd.Fail(fmt.Sprintf("no such file: %s", source))
		}

		zr, err := zip.OpenReader(archivePath)
		if err != nil {
			cmd.Fail(fmt.Sprintf("not a readable zip: %v", err))
		}
		defer zr.Close()

		if _, found := findZipEntry(&zr.Reader, "modrinth.index.json"); found {
			idx, err := readMrIndex(&zr.Reader)
			if err != nil {
				cmd.Fail(err.Error())
			}
			importMrpack(archivePath, &zr.Reader, idx, customID)
			return
		}
		if data, found := findZipEntry(&zr.Reader, "manifest.json"); found {
			var cfm cfManifest
			if err := json.Unmarshal(data, &cfm); err != nil {
				cmd.Fail(fmt.Sprintf("manifest.json is not valid JSON: %v", err))
			}
			if cfm.ManifestType != "minecraftModpack" {
				cmd.Fail(fmt.Sprintf("unrecognised manifest type %q in manifest.json", cfm.ManifestType))
			}
			importCFZip(archivePath, &zr.Reader, &cfm, customID)
			return
		}
		cmd.Fail("not an mrpack or curseforge modpack zip (no modrinth.index.json or manifest.json found)")
	},
}

func findZipEntry(zr *zip.Reader, name string) ([]byte, bool) {
	for _, f := range zr.File {
		if f.Name != name {
			continue
		}
		rc, err := f.Open()
		if err != nil {
			return nil, false
		}
		defer rc.Close()
		data, err := io.ReadAll(rc)
		if err != nil {
			return nil, false
		}
		return data, true
	}
	return nil, false
}

func importMrpack(archivePath string, zr *zip.Reader, idx *mrIndex, customID string) {
	mc := idx.Dependencies["minecraft"]
	if mc == "" {
		cmd.Fail("modrinth.index.json has no minecraft dependency")
	}
	loader, loaderVersion := detectLoader(idx.Dependencies)
	if loader == "" {
		cmd.Fail("could not detect a mod loader in modrinth.index.json dependencies")
	}

	packID := customID
	if packID == "" {
		packID = slugify(idx.Name)
	}
	packDir := filepath.Join("modpacks", packID)
	if _, err := os.Stat(packDir); err == nil {
		cmd.Fail(fmt.Sprintf("pack already exists: %s (use --id for a different name)", packDir))
	}
	subdir := filepath.Join(packDir, mc+"-mr")
	if err := os.MkdirAll(subdir, 0o755); err != nil {
		cmd.Fail(fmt.Sprintf("failed to create %s: %v", subdir, err))
	}

	fmt.Printf("importing %q (%s, %s %s, mc %s) -> %s\n", idx.Name, idx.VersionID, loader, loaderVersion, mc, packDir)

	initFlag, _ := loaderLatestFlag(loader)
	ex := exec.Command(workspace.SelfBin(), "init",
		"--name", idx.Name,
		"--author", placeholderAuthor,
		"--mc-version", mc,
		"--modloader", loader,
		initFlag,
		"--version", idx.VersionID,
		"-y",
	)
	ex.Dir = subdir
	if out, err := ex.CombinedOutput(); err != nil {
		cmd.Fail(fmt.Sprintf("packwand init failed in %s: %v\n%s", subdir, err, workspace.Indent(string(out), "    ")))
	}
	if loaderVersion != "" {
		pinLoaderVersion(filepath.Join(subdir, "pack.toml"), loader, loaderVersion)
	}

	wrote, updatable := 0, 0
	for _, f := range idx.Files {
		if len(f.Downloads) == 0 {
			cmd.Warn("%s has no download URL; skipped", f.Path)
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

	// Files already written as .pw.toml metadata must not also be extracted
	// from overrides, or refresh would index the raw jar a second time.
	indexedPaths := make(map[string]bool, len(idx.Files))
	for _, f := range idx.Files {
		indexedPaths[strings.ToLower(path.Clean(f.Path))] = true
	}
	overrides, jarOverrides := extractOverrides(zr, subdir, indexedPaths)

	if err := os.WriteFile(filepath.Join(subdir, ".packwizignore"), []byte(packwizIgnore), 0o644); err != nil {
		cmd.Fail(fmt.Sprintf("failed to write .packwizignore: %v", err))
	}

	refresh := exec.Command(workspace.SelfBin(), "refresh")
	refresh.Dir = subdir
	if out, err := refresh.CombinedOutput(); err != nil {
		cmd.Fail(fmt.Sprintf("packwand refresh failed in %s: %v\n%s", subdir, err, workspace.Indent(string(out), "    ")))
	}

	cmd.WriteJSON(filepath.Join(packDir, "manifest.json"), map[string]any{
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
	warnJarOverrides(jarOverrides)
	fmt.Printf("  manifest.json scaffolded — fill modrinth_id/curseforge_id before publishing\n")
	if wrote > updatable {
		fmt.Printf("  note: files without cdn.modrinth.com URLs lack [update.modrinth]; 'packwand workspace update' will leave them as-is\n")
	}
}

func importCFZip(archivePath string, zr *zip.Reader, cfm *cfManifest, customID string) {
	mc := cfm.Minecraft.Version
	if mc == "" {
		cmd.Fail("CF manifest.json has no minecraft.version")
	}
	loader, loaderVersion := detectCFLoader(cfm.Minecraft.ModLoaders)
	if loader == "" {
		cmd.Fail("could not detect a mod loader in CF manifest.json modLoaders")
	}

	packID := customID
	if packID == "" {
		packID = slugify(cfm.Name)
	}
	packDir := filepath.Join("modpacks", packID)
	if _, err := os.Stat(packDir); err == nil {
		cmd.Fail(fmt.Sprintf("pack already exists: %s (use --id for a different name)", packDir))
	}
	subdir := filepath.Join(packDir, mc+"-cf")
	if err := os.MkdirAll(subdir, 0o755); err != nil {
		cmd.Fail(fmt.Sprintf("failed to create %s: %v", subdir, err))
	}

	fmt.Printf("importing CF pack %q (%s %s, mc %s) -> %s\n", cfm.Name, loader, loaderVersion, mc, packDir)

	ex := exec.Command(workspace.SelfBin(), "curseforge", "import", archivePath)
	ex.Dir = subdir
	ex.Stdout = os.Stdout
	ex.Stderr = os.Stderr
	cfImported := true
	if err := ex.Run(); err != nil {
		cfImported = false
		fmt.Printf("packwand curseforge import not available (%v); scaffolding pack structure only\n", err)
		initFlag, _ := loaderLatestFlag(loader)
		init2 := exec.Command(workspace.SelfBin(), "init",
			"--name", cfm.Name,
			"--author", placeholderAuthor,
			"--mc-version", mc,
			"--modloader", loader,
			initFlag,
			"--version", cfm.Version,
			"-y",
		)
		init2.Dir = subdir
		if out, err2 := init2.CombinedOutput(); err2 != nil {
			cmd.Fail(fmt.Sprintf("packwand init failed in %s: %v\n%s", subdir, err2, workspace.Indent(string(out), "    ")))
		}
		fmt.Printf("  note: %d mod file(s) require manual 'packwand curseforge add' — see cf-pending.txt\n", len(cfm.Files))
		var pending strings.Builder
		for _, f := range cfm.Files {
			if f.Required {
				fmt.Fprintf(&pending, "packwand curseforge install --project-id %d --file-id %d\n", f.ProjectID, f.FileID)
			}
		}
		_ = os.WriteFile(filepath.Join(subdir, "cf-pending.txt"), []byte(pending.String()), 0o644)
	}

	// When curseforge import succeeded it has already copied the override
	// files, skipping those referenced by mod metadata — extracting them
	// again here would reintroduce the skipped jars as stray files.
	overrides := 0
	var jarOverrides []string
	if !cfImported {
		overrideDir := cfm.Overrides
		if overrideDir == "" {
			overrideDir = "overrides"
		}
		overrides, jarOverrides = extractOverridesPrefix(zr, subdir, overrideDir+"/", nil)
	}

	if err := os.WriteFile(filepath.Join(subdir, ".packwizignore"), []byte(packwizIgnore), 0o644); err != nil {
		cmd.Fail(fmt.Sprintf("failed to write .packwizignore: %v", err))
	}

	cmd.WriteJSON(filepath.Join(packDir, "manifest.json"), map[string]any{
		"$schema":       "../../tools/manifest/schema.json",
		"id":            packID,
		"name":          cfm.Name,
		"type":          "modpack",
		"role":          "none",
		"release_type":  "release",
		"version":       cfm.Version,
		"mc_version":    mc,
		"loader":        loader,
		"curseforge_id": "",
	})
	changelog := fmt.Sprintf("# %s\n\nImported from CurseForge zip (%s).\n", cfm.Name, cfm.Version)
	_ = os.WriteFile(filepath.Join(packDir, "changelog.md"), []byte(changelog), 0o644)

	fmt.Printf("\nimported %s:\n", packID)
	if cfImported {
		fmt.Printf("  override files handled by curseforge import\n")
	} else {
		fmt.Printf("  %d override file(s) copied\n", overrides)
		warnJarOverrides(jarOverrides)
	}
	fmt.Printf("  manifest.json scaffolded — fill curseforge_id/modrinth_id before publishing\n")
}

func detectCFLoader(loaders []cfModLoader) (loader, version string) {
	for _, l := range loaders {
		id := l.ID
		for _, prefix := range []string{"fabric-", "quilt-", "neoforge-", "forge-"} {
			if strings.HasPrefix(id, prefix) {
				return strings.TrimSuffix(prefix, "-"), strings.TrimPrefix(id, prefix)
			}
		}
	}
	return "", ""
}

// extractOverridesPrefix copies every archive entry under prefix (excluding
// ones already tracked via indexedPaths) into subdir, and returns both the
// count copied and the relative paths of any that look like mod jars — an
// override that's actually a .jar means the source pack's export couldn't
// resolve that mod to a hosted file, so it will never be update-tracked.
func extractOverridesPrefix(zr *zip.Reader, subdir, prefix string, indexedPaths map[string]bool) (int, []string) {
	count := 0
	var jarOverrides []string
	for _, f := range zr.File {
		// Normalize the backslash separators written by some Windows zip
		// tools (e.g. Compress-Archive) so prefix matching works.
		name := strings.ReplaceAll(f.Name, "\\", "/")
		if !strings.HasPrefix(name, prefix) || strings.HasSuffix(name, "/") || f.FileInfo().IsDir() {
			continue
		}
		rel := strings.TrimPrefix(name, prefix)
		if indexedPaths[strings.ToLower(path.Clean(rel))] {
			cmd.Warn("skipping override %s (already referenced by the index)", f.Name)
			continue
		}
		dest := filepath.Join(subdir, filepath.FromSlash(rel))
		if !strings.HasPrefix(filepath.Clean(dest), filepath.Clean(subdir)+string(os.PathSeparator)) {
			cmd.Warn("skipping suspicious archive path %s", f.Name)
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
			if strings.HasSuffix(strings.ToLower(rel), ".jar") {
				jarOverrides = append(jarOverrides, rel)
			}
		}
	}
	return count, jarOverrides
}

// warnJarOverrides prints a distinct summary for override files that look
// like mod jars rather than configs/resourcepacks — these came from the
// source pack's export failing to resolve them to a hosted file, so they'll
// sit as static, un-update-tracked binaries unless re-added properly.
func warnJarOverrides(jarOverrides []string) {
	if len(jarOverrides) == 0 {
		return
	}
	sort.Strings(jarOverrides)
	fmt.Printf("  warning: %d override file(s) look like mod jars, not configs — they will NOT be update-tracked:\n", len(jarOverrides))
	for _, j := range jarOverrides {
		fmt.Printf("    %s\n", j)
	}
	fmt.Printf("  note: re-add these via 'packwand add' (if hosted on Modrinth/CurseForge) or accept they'll stay static\n")
}

func downloadToTemp(url string) (string, error) {
	resp, err := http.Get(url) //nolint:gosec
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("HTTP %d from %s", resp.StatusCode, url)
	}
	tmp, err := os.CreateTemp("", "packwand-import-*.mrpack")
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
	data, err := os.ReadFile(packToml)
	if err != nil {
		cmd.Warn("could not pin loader version: %v", err)
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
		if k, _, ok := splitKV(line); ok && k == loader {
			lines[i] = fmt.Sprintf("%s = %q", loader, version)
			if err := os.WriteFile(packToml, []byte(strings.Join(lines, "\n")), 0o644); err != nil {
				cmd.Warn("could not pin loader version: %v", err)
			}
			return
		}
	}
	cmd.Warn("no %q key under [versions] in %s; loader left at latest", loader, packToml)
}

func writeImportedToml(subdir string, f mrIndexFile) (ok bool, hasUpdate bool) {
	base := filepath.Base(f.Path)
	metaPath := filepath.Join(subdir, filepath.Dir(f.Path), strings.TrimSuffix(base, filepath.Ext(base))+".pw.toml")
	if err := os.MkdirAll(filepath.Dir(metaPath), 0o755); err != nil {
		cmd.Warn("%s: %v; skipped", f.Path, err)
		return false, false
	}

	hashFormat, hash := "sha512", f.Hashes["sha512"]
	if hash == "" {
		hashFormat, hash = "sha1", f.Hashes["sha1"]
	}
	if hash == "" {
		cmd.Warn("%s has no sha512/sha1 hash; skipped", f.Path)
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
		cmd.Warn("failed to write %s: %v", metaPath, err)
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

func extractOverrides(zr *zip.Reader, subdir string, indexedPaths map[string]bool) (int, []string) {
	count := 0
	var jarOverrides []string
	for _, prefix := range []string{"overrides/", "client-overrides/"} {
		n, jars := extractOverridesPrefix(zr, subdir, prefix, indexedPaths)
		count += n
		jarOverrides = append(jarOverrides, jars...)
	}
	return count, jarOverrides
}

var slugifyRe = regexp.MustCompile(`[^a-z0-9]+`)

func slugify(name string) string {
	s := strings.ToLower(name)
	s = slugifyRe.ReplaceAllString(s, "-")
	return strings.Trim(s, "-")
}

const defaultInstallerURL = "https://github.com/packwiz/packwiz-installer-bootstrap/releases/latest/download/packwiz-installer-bootstrap.jar"

// resolveInstallerJar finds a packwiz installer jar to run. It returns the
// jar path and whether it is our own locally-built fork (in which case its
// Main-Class is a RequiresBootstrap guard that refuses direct `-jar`
// execution, so callers must invoke it via `-cp jar
// link.infra.packwiz.installer.Main`, matching cmd/packwiz-bootstrap) versus
// the legacy self-updating bootstrap jar (invoked via `-jar` as before).
//
// A repo checkout's own `task build-installer` output is preferred over the
// cached/downloaded upstream bootstrap jar, so `packwand test` (used for dev
// boot testing) exercises this repo's actual patched installer rather than
// silently falling back to an unrelated upstream build.
func resolveInstallerJar() (path string, direct bool, err error) {
	for _, name := range []string{"PACKWAND_INSTALLER_JAR", "PACKWIZ_INSTALLER_JAR"} {
		if p := os.Getenv(name); p != "" {
			if _, err := os.Stat(p); err != nil {
				return "", false, fmt.Errorf("%s points to %s: %w", name, p, err)
			}
			return p, false, nil
		}
	}
	if root := workspace.FindRepoRoot(); root != "" {
		local := filepath.Join(root, "apps", "packwand-installer", "build", "dist", "packwiz-installer.jar")
		if _, err := os.Stat(local); err == nil {
			return local, true, nil
		}
	}
	cache, err := core.GetPackwandCache()
	if err != nil {
		return "", false, err
	}
	path = filepath.Join(cache, "installer", "packwiz-installer-bootstrap.jar")
	if _, err := os.Stat(path); err == nil {
		return path, false, nil
	}
	url := os.Getenv("PACKWAND_INSTALLER_URL")
	if url == "" {
		url = defaultInstallerURL
	}
	if err := downloadInstallerJar(path, url, os.Getenv("PACKWAND_INSTALLER_SHA256")); err != nil {
		return "", false, err
	}
	return path, false, nil
}

func downloadInstallerJar(destination, source, expectedSHA256 string) error {
	response, err := core.GetWithUA(source, "application/java-archive, application/octet-stream")
	if err != nil {
		return err
	}
	defer response.Body.Close()
	if response.StatusCode != http.StatusOK {
		return fmt.Errorf("HTTP %d from %s", response.StatusCode, source)
	}
	if err := os.MkdirAll(filepath.Dir(destination), 0o755); err != nil {
		return err
	}
	tmp, err := os.CreateTemp(filepath.Dir(destination), "installer-*.jar.tmp")
	if err != nil {
		return err
	}
	defer os.Remove(tmp.Name())
	hasher := sha256.New()
	if _, err := io.Copy(io.MultiWriter(tmp, hasher), response.Body); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	if expectedSHA256 != "" {
		actual := hex.EncodeToString(hasher.Sum(nil))
		if !strings.EqualFold(actual, expectedSHA256) {
			return fmt.Errorf("installer SHA-256 mismatch: expected %s, got %s", expectedSHA256, actual)
		}
	}
	return os.Rename(tmp.Name(), destination)
}

// — test —

const servePort = "8080"

var testCmd = &cobra.Command{
	Use:   "test <pack-subdir>",
	Short: "Spin up packwand serve and run packwiz-installer against it to validate a pack",
	Args:  cobra.ExactArgs(1),
	Run: func(c *cobra.Command, args []string) {
		packSubdir := cmd.Abs(args[0])

		if _, err := os.Stat(filepath.Join(packSubdir, "pack.toml")); err != nil {
			cmd.Fail(fmt.Sprintf("no pack.toml in %s", packSubdir))
		}
		if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
			cmd.Fail("packwand not found; install it or set PACKWAND_BIN")
		}
		if _, err := exec.LookPath("java"); err != nil {
			cmd.Fail("java not found in PATH; packwiz-installer requires a JRE/JDK")
		}
		installerJar, installerDirect, err := resolveInstallerJar()
		if err != nil {
			cmd.Fail(fmt.Sprintf("could not provision packwiz installer: %v", err))
		}
		instanceDir := os.Getenv("PACKWAND_TEST_INSTANCE")
		if instanceDir == "" {
			instanceDir = "./.packwand-test-instance"
			fmt.Printf("PACKWAND_TEST_INSTANCE unset; using default %s\n", instanceDir)
		}
		if err := os.MkdirAll(instanceDir, 0o755); err != nil {
			cmd.Fail(fmt.Sprintf("failed to create instance dir %s: %v", instanceDir, err))
		}
		absInstance, err := filepath.Abs(instanceDir)
		if err != nil {
			cmd.Fail(fmt.Sprintf("failed to resolve instance dir: %v", err))
		}
		absJar, err := filepath.Abs(installerJar)
		if err != nil {
			cmd.Fail(fmt.Sprintf("failed to resolve jar path: %v", err))
		}

		fmt.Printf("starting packwand serve in %s ...\n", packSubdir)
		serve := exec.Command(workspace.SelfBin(), "serve", "--port", servePort)
		serve.Dir = packSubdir
		serve.Stdout = os.Stderr
		serve.Stderr = os.Stderr
		if err := serve.Start(); err != nil {
			cmd.Fail(fmt.Sprintf("failed to start packwand serve: %v", err))
		}
		defer func() {
			if serve.Process != nil {
				_ = serve.Process.Kill()
				_, _ = serve.Process.Wait()
			}
		}()

		if !waitForPort("127.0.0.1:"+servePort, 10*time.Second) {
			cmd.Fail("packwand serve did not become ready on port " + servePort)
		}
		fmt.Println("packwand serve is up.")

		packURL := fmt.Sprintf("http://localhost:%s/pack.toml", servePort)
		fmt.Printf("installing pack into %s ...\n", absInstance)
		var installer *exec.Cmd
		if installerDirect {
			// Our own build's Main-Class is a RequiresBootstrap guard that
			// refuses direct `-jar` execution; invoke the real entry point
			// via the classpath instead, matching cmd/packwiz-bootstrap.
			installer = exec.Command("java", "-cp", absJar, "link.infra.packwiz.installer.Main", "-g", "--continue-on-error", packURL)
		} else {
			installer = exec.Command("java", "-jar", absJar, "-g", "--continue-on-error", packURL)
		}
		installer.Dir = absInstance
		installer.Stdout = os.Stdout
		installer.Stderr = os.Stderr
		if err := installer.Run(); err != nil {
			cmd.Fail(fmt.Sprintf("packwiz-installer failed: %v", err))
		}

		fmt.Printf("\ntest instance ready at %s\n", absInstance)
		fmt.Println("point your launcher (MultiMC/Prism) at it, or launch from there. (packwand does not launch the game.)")
	},
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
