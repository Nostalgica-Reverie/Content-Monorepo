package cmd

import (
	"archive/zip"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	"packwand/manifest"
	"packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	llInitCmd.Flags().String("mc", "", "Minecraft version (default: 26.1.2)")
	llInitCmd.Flags().String("loader", "fabric", "Mod loader: fabric, forge, neoforge, quilt")
	llInitCmd.Flags().Bool("base", false, "Scaffold this pack as a performance base")
	llInitCmd.Flags().String("consumes", "", "ID of the performance base this pack consumes")
	llInitCmd.Flags().String("variants", "", "Comma-separated variant IDs (multi-MC-version packs)")
	rootCmd.AddCommand(llInitCmd)

	llAddCmd.Flags().Bool("no-refresh", false, "Skip packwand refresh after add (batch adds)")
	rootCmd.AddCommand(llAddCmd)

	llPortCmd.Flags().Bool("add", false, "Interactively add missing CurseForge entries via packwand")
	llPortCmd.Flags().Bool("no-refresh", false, "Batch the refresh until the end")
	llPortCmd.Flags().Bool("json", false, "Output missing list as JSON (dry-run only)")
	rootCmd.AddCommand(llPortCmd)

	llImportCmd.Flags().String("id", "", "Override the pack ID derived from the archive name")
	rootCmd.AddCommand(llImportCmd)

	rootCmd.AddCommand(llTestCmd)
}

// — init —

const (
	defaultMCVersion   = "26.1.2"
	defaultPackVersion = "26.x"
	placeholderAuthor  = "CHANGEME"
	packwizIgnore      = "Logs\n*.zip\n*.mrpack\n"
)

var llInitCmd = &cobra.Command{
	Use:   "new <modpacks|datapacks|resourcepacks> <name>",
	Short: "Scaffold a new pack (manifest.json, changelog.md, packwiz subdirs)",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		category, name := args[0], args[1]
		switch category {
		case "modpacks", "datapacks", "resourcepacks":
		default:
			llFail(fmt.Sprintf("invalid category %q (expected modpacks, datapacks, or resourcepacks)", category))
		}

		mcVersion, _ := cmd.Flags().GetString("mc")
		if mcVersion == "" {
			mcVersion = defaultMCVersion
		}
		loader, _ := cmd.Flags().GetString("loader")
		asBase, _ := cmd.Flags().GetBool("base")
		consumesBase, _ := cmd.Flags().GetString("consumes")
		variantsStr, _ := cmd.Flags().GetString("variants")

		var variants []string
		for _, v := range strings.Split(variantsStr, ",") {
			if v = strings.TrimSpace(v); v != "" {
				variants = append(variants, v)
			}
		}

		if asBase && consumesBase != "" {
			llFail("--base and --consumes are mutually exclusive")
		}

		loaderFlag, ok := loaderLatestFlag(loader)
		if !ok {
			llFail(fmt.Sprintf("invalid loader %q (expected fabric, forge, neoforge, or quilt)", loader))
		}

		llChdir()

		packDir := filepath.Join(category, name)
		if _, err := os.Stat(packDir); err == nil {
			llFail(fmt.Sprintf("pack already exists: %s", packDir))
		}
		if err := os.MkdirAll(packDir, 0o755); err != nil {
			llFail(fmt.Sprintf("failed to create %s: %v", packDir, err))
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

		llWriteJSON(filepath.Join(packDir, "manifest.json"), mf)

		changelog := fmt.Sprintf("# %s\n\nInitial scaffold. Describe the first release here.\n", name)
		if err := os.WriteFile(filepath.Join(packDir, "changelog.md"), []byte(changelog), 0o644); err != nil {
			llFail(fmt.Sprintf("failed to write changelog.md: %v", err))
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
						llFail(fmt.Sprintf("failed to create %s: %v", sub, err))
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
						llFail(fmt.Sprintf("packwand init failed in %s: %v", sub, err))
					}
					if err := os.WriteFile(filepath.Join(sub, ".packwizignore"), []byte(packwizIgnore), 0o644); err != nil {
						llFail(fmt.Sprintf("failed to write .packwizignore in %s: %v", sub, err))
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

// — add —

var llAddCmd = &cobra.Command{
	Use:   "add <mod-slug> [pack-dir|pack-subdir]",
	Short: "Add a mod to all (or a specific) pack's Modrinth and CurseForge subdirs",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(cmd *cobra.Command, args []string) {
		slug := args[0]
		noRefresh, _ := cmd.Flags().GetBool("no-refresh")

		var targetArg string
		if len(args) > 1 {
			targetArg = llAbs(strings.TrimRight(args[1], "/"))
		}

		if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
			llFail("packwand not found on PATH — install it or set PACKWIZ_BIN")
		}

		llChdir()

		targets := resolveAddTargets(targetArg)
		if len(targets) == 0 {
			fmt.Println("no pack subdirs found")
			return
		}

		fmt.Printf("adding %q to %d subdir(s)\n\n", slug, len(targets))
		added, failed, skipped := 0, 0, 0

		for _, dir := range targets {
			plat := llPlatformSuffix(dir)
			var pwArgs []string
			switch plat {
			case "mr":
				pwArgs = []string{"modrinth", "add", "-y", slug}
			case "cf":
				pwArgs = []string{"curseforge", "add", "-y", slug}
			default:
				llWarn("skipping %s — unrecognised suffix (need -mr or -cf)", dir)
				skipped++
				continue
			}
			if noRefresh {
				pwArgs = append(pwArgs, "--no-refresh")
			}

			fmt.Printf("[%s] %s\n", plat, dir)
			c := exec.Command(workspace.SelfBin(), pwArgs...)
			c.Dir = dir
			c.Stdout = os.Stdout
			c.Stderr = os.Stderr
			if err := c.Run(); err != nil {
				llWarn("%s: add failed — slug may not exist on %s under this name", dir, plat)
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
			llFail(fmt.Sprintf("failed to read %s: %v", root, err))
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
			llFail(fmt.Sprintf("subdir not found: %s", targetArg))
		}
		return []string{targetArg}
	}

	if _, err := os.Stat(filepath.Join(targetArg, "manifest.json")); err != nil {
		llFail(fmt.Sprintf("no manifest.json in %s — pass a pack dir, a subdir, or nothing for all packs", targetArg))
	}
	return manifest.SubDirsOf(targetArg)
}

// — port —

type portResult struct {
	MRTotal   int      `json:"mr_total"`
	CFMatched int      `json:"cf_matched"`
	Missing   []string `json:"missing"`
}

var llPortCmd = &cobra.Command{
	Use:   "port <mr-subdir> <cf-subdir>",
	Short: "Compare MR and CF subdirs and port missing mods from Modrinth to CurseForge",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		mrDir := llAbs(args[0])
		cfDir := llAbs(args[1])
		doAdd, _ := cmd.Flags().GetBool("add")
		noRefresh, _ := cmd.Flags().GetBool("no-refresh")
		asJSON, _ := cmd.Flags().GetBool("json")

		llChdir()

		mrMods := filepath.Join(mrDir, "mods")
		if info, err := os.Stat(mrMods); err != nil || !info.IsDir() {
			llFail(fmt.Sprintf("no mods/ in MR subdir %s", mrDir))
		}
		if _, err := os.Stat(filepath.Join(cfDir, "pack.toml")); err != nil {
			llFail(fmt.Sprintf("CF subdir %s has no pack.toml (run packwand init there first)", cfDir))
		}
		if doAdd {
			if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
				llFail("packwand not found; install it or set PACKWIZ_BIN")
			}
		}

		entries, err := os.ReadDir(mrMods)
		if err != nil {
			llFail(fmt.Sprintf("failed to read %s: %v", mrMods, err))
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
			c := exec.Command(workspace.SelfBin(), addArgs...)
			c.Dir = cfDir
			c.Stdin = os.Stdin
			c.Stdout = os.Stdout
			c.Stderr = os.Stderr
			if err := c.Run(); err != nil {
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
			c := exec.Command(workspace.SelfBin(), "refresh")
			c.Dir = cfDir
			if out, err := c.CombinedOutput(); err != nil {
				llFail(fmt.Sprintf("final refresh failed in %s: %v\n%s", cfDir, err, out))
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

var llImportCmd = &cobra.Command{
	Use:   "import <mrpack-or-cf-zip|URL>",
	Short: "Import an .mrpack or CurseForge zip as a new modpack",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		source := args[0]
		if !strings.HasPrefix(source, "http://") && !strings.HasPrefix(source, "https://") {
			source = llAbs(source)
		}
		customID, _ := cmd.Flags().GetString("id")

		if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
			llFail("packwand not found; ll-import scaffolds via packwand init and refresh — install it or set PACKWIZ_BIN")
		}

		llChdir()

		archivePath := source
		if strings.HasPrefix(source, "http://") || strings.HasPrefix(source, "https://") {
			var err error
			archivePath, err = downloadToTemp(source)
			if err != nil {
				llFail(fmt.Sprintf("download failed: %v", err))
			}
			defer os.Remove(archivePath)
		} else if _, err := os.Stat(source); err != nil {
			llFail(fmt.Sprintf("no such file: %s", source))
		}

		zr, err := zip.OpenReader(archivePath)
		if err != nil {
			llFail(fmt.Sprintf("not a readable zip: %v", err))
		}
		defer zr.Close()

		if _, found := findZipEntry(&zr.Reader, "modrinth.index.json"); found {
			idx, err := readMrIndex(&zr.Reader)
			if err != nil {
				llFail(err.Error())
			}
			importMrpack(archivePath, &zr.Reader, idx, customID)
			return
		}
		if data, found := findZipEntry(&zr.Reader, "manifest.json"); found {
			var cfm cfManifest
			if err := json.Unmarshal(data, &cfm); err != nil {
				llFail(fmt.Sprintf("manifest.json is not valid JSON: %v", err))
			}
			if cfm.ManifestType != "minecraftModpack" {
				llFail(fmt.Sprintf("unrecognised manifest type %q in manifest.json", cfm.ManifestType))
			}
			importCFZip(archivePath, &zr.Reader, &cfm, customID)
			return
		}
		llFail("not an mrpack or curseforge modpack zip (no modrinth.index.json or manifest.json found)")
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
		llFail("modrinth.index.json has no minecraft dependency")
	}
	loader, loaderVersion := detectLoader(idx.Dependencies)
	if loader == "" {
		llFail("could not detect a mod loader in modrinth.index.json dependencies")
	}

	packID := customID
	if packID == "" {
		packID = slugify(idx.Name)
	}
	packDir := filepath.Join("modpacks", packID)
	if _, err := os.Stat(packDir); err == nil {
		llFail(fmt.Sprintf("pack already exists: %s (use --id for a different name)", packDir))
	}
	subdir := filepath.Join(packDir, mc+"-mr")
	if err := os.MkdirAll(subdir, 0o755); err != nil {
		llFail(fmt.Sprintf("failed to create %s: %v", subdir, err))
	}

	fmt.Printf("importing %q (%s, %s %s, mc %s) -> %s\n", idx.Name, idx.VersionID, loader, loaderVersion, mc, packDir)

	initFlag, _ := loaderLatestFlag(loader)
	c := exec.Command(workspace.SelfBin(), "init",
		"--name", idx.Name,
		"--author", placeholderAuthor,
		"--mc-version", mc,
		"--modloader", loader,
		initFlag,
		"--version", idx.VersionID,
		"-y",
	)
	c.Dir = subdir
	if out, err := c.CombinedOutput(); err != nil {
		llFail(fmt.Sprintf("packwand init failed in %s: %v\n%s", subdir, err, workspace.Indent(string(out), "    ")))
	}
	if loaderVersion != "" {
		pinLoaderVersion(filepath.Join(subdir, "pack.toml"), loader, loaderVersion)
	}

	wrote, updatable := 0, 0
	for _, f := range idx.Files {
		if len(f.Downloads) == 0 {
			llWarn("%s has no download URL; skipped", f.Path)
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

	overrides := extractOverrides(zr, subdir)

	if err := os.WriteFile(filepath.Join(subdir, ".packwizignore"), []byte(packwizIgnore), 0o644); err != nil {
		llFail(fmt.Sprintf("failed to write .packwizignore: %v", err))
	}

	refresh := exec.Command(workspace.SelfBin(), "refresh")
	refresh.Dir = subdir
	if out, err := refresh.CombinedOutput(); err != nil {
		llFail(fmt.Sprintf("packwand refresh failed in %s: %v\n%s", subdir, err, workspace.Indent(string(out), "    ")))
	}

	llWriteJSON(filepath.Join(packDir, "manifest.json"), map[string]any{
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
		fmt.Printf("  note: files without cdn.modrinth.com URLs lack [update.modrinth]; 'packwand workspace update' will leave them as-is\n")
	}
}

func importCFZip(archivePath string, zr *zip.Reader, cfm *cfManifest, customID string) {
	mc := cfm.Minecraft.Version
	if mc == "" {
		llFail("CF manifest.json has no minecraft.version")
	}
	loader, loaderVersion := detectCFLoader(cfm.Minecraft.ModLoaders)
	if loader == "" {
		llFail("could not detect a mod loader in CF manifest.json modLoaders")
	}

	packID := customID
	if packID == "" {
		packID = slugify(cfm.Name)
	}
	packDir := filepath.Join("modpacks", packID)
	if _, err := os.Stat(packDir); err == nil {
		llFail(fmt.Sprintf("pack already exists: %s (use --id for a different name)", packDir))
	}
	subdir := filepath.Join(packDir, mc+"-cf")
	if err := os.MkdirAll(subdir, 0o755); err != nil {
		llFail(fmt.Sprintf("failed to create %s: %v", subdir, err))
	}

	fmt.Printf("importing CF pack %q (%s %s, mc %s) -> %s\n", cfm.Name, loader, loaderVersion, mc, packDir)

	c := exec.Command(workspace.SelfBin(), "curseforge", "import", archivePath)
	c.Dir = subdir
	c.Stdout = os.Stdout
	c.Stderr = os.Stderr
	if err := c.Run(); err != nil {
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
			llFail(fmt.Sprintf("packwand init failed in %s: %v\n%s", subdir, err2, workspace.Indent(string(out), "    ")))
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

	overrideDir := cfm.Overrides
	if overrideDir == "" {
		overrideDir = "overrides"
	}
	overrides := extractOverridesPrefix(zr, subdir, overrideDir+"/")

	if err := os.WriteFile(filepath.Join(subdir, ".packwizignore"), []byte(packwizIgnore), 0o644); err != nil {
		llFail(fmt.Sprintf("failed to write .packwizignore: %v", err))
	}

	llWriteJSON(filepath.Join(packDir, "manifest.json"), map[string]any{
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
	fmt.Printf("  %d override file(s) copied\n", overrides)
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

func extractOverridesPrefix(zr *zip.Reader, subdir, prefix string) int {
	count := 0
	for _, f := range zr.File {
		if !strings.HasPrefix(f.Name, prefix) || strings.HasSuffix(f.Name, "/") {
			continue
		}
		rel := strings.TrimPrefix(f.Name, prefix)
		dest := filepath.Join(subdir, filepath.FromSlash(rel))
		if !strings.HasPrefix(filepath.Clean(dest), filepath.Clean(subdir)+string(os.PathSeparator)) {
			llWarn("skipping suspicious archive path %s", f.Name)
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
	return count
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
		llWarn("could not pin loader version: %v", err)
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
				llWarn("could not pin loader version: %v", err)
			}
			return
		}
	}
	llWarn("no %q key under [versions] in %s; loader left at latest", loader, packToml)
}

func writeImportedToml(subdir string, f mrIndexFile) (ok bool, hasUpdate bool) {
	base := filepath.Base(f.Path)
	metaPath := filepath.Join(subdir, filepath.Dir(f.Path), strings.TrimSuffix(base, filepath.Ext(base))+".pw.toml")
	if err := os.MkdirAll(filepath.Dir(metaPath), 0o755); err != nil {
		llWarn("%s: %v; skipped", f.Path, err)
		return false, false
	}

	hashFormat, hash := "sha512", f.Hashes["sha512"]
	if hash == "" {
		hashFormat, hash = "sha1", f.Hashes["sha1"]
	}
	if hash == "" {
		llWarn("%s has no sha512/sha1 hash; skipped", f.Path)
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
		llWarn("failed to write %s: %v", metaPath, err)
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
				llWarn("skipping suspicious archive path %s", f.Name)
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

// — test —

const servePort = "8080"

var llTestCmd = &cobra.Command{
	Use:   "test <pack-subdir>",
	Short: "Spin up packwand serve and run packwiz-installer against it to validate a pack",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		packSubdir := llAbs(args[0])

		if _, err := os.Stat(filepath.Join(packSubdir, "pack.toml")); err != nil {
			llFail(fmt.Sprintf("no pack.toml in %s", packSubdir))
		}
		if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
			llFail("packwand not found; install it or set PACKWIZ_BIN")
		}
		if _, err := exec.LookPath("java"); err != nil {
			llFail("java not found in PATH; packwiz-installer requires a JRE/JDK")
		}
		installerJar := os.Getenv("PACKWIZ_INSTALLER_JAR")
		if installerJar == "" {
			llFail("PACKWIZ_INSTALLER_JAR is not set; download packwiz-installer-bootstrap.jar and export the path")
		}
		if _, err := os.Stat(installerJar); err != nil {
			llFail(fmt.Sprintf("packwiz-installer jar not found at %s", installerJar))
		}

		instanceDir := os.Getenv("PACKWAND_TEST_INSTANCE")
		if instanceDir == "" {
			instanceDir = "./.packwand-test-instance"
			fmt.Printf("PACKWAND_TEST_INSTANCE unset; using default %s\n", instanceDir)
		}
		if err := os.MkdirAll(instanceDir, 0o755); err != nil {
			llFail(fmt.Sprintf("failed to create instance dir %s: %v", instanceDir, err))
		}
		absInstance, err := filepath.Abs(instanceDir)
		if err != nil {
			llFail(fmt.Sprintf("failed to resolve instance dir: %v", err))
		}
		absJar, err := filepath.Abs(installerJar)
		if err != nil {
			llFail(fmt.Sprintf("failed to resolve jar path: %v", err))
		}

		fmt.Printf("starting packwand serve in %s ...\n", packSubdir)
		serve := exec.Command(workspace.SelfBin(), "serve", "--port", servePort)
		serve.Dir = packSubdir
		serve.Stdout = os.Stderr
		serve.Stderr = os.Stderr
		if err := serve.Start(); err != nil {
			llFail(fmt.Sprintf("failed to start packwand serve: %v", err))
		}
		defer func() {
			if serve.Process != nil {
				_ = serve.Process.Kill()
				_, _ = serve.Process.Wait()
			}
		}()

		if !waitForPort("127.0.0.1:"+servePort, 10*time.Second) {
			llFail("packwand serve did not become ready on port " + servePort)
		}
		fmt.Println("packwand serve is up.")

		packURL := fmt.Sprintf("http://localhost:%s/pack.toml", servePort)
		fmt.Printf("installing pack into %s ...\n", absInstance)
		installer := exec.Command("java", "-jar", absJar, "-g", packURL)
		installer.Dir = absInstance
		installer.Stdout = os.Stdout
		installer.Stderr = os.Stderr
		if err := installer.Run(); err != nil {
			llFail(fmt.Sprintf("packwiz-installer failed: %v", err))
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
