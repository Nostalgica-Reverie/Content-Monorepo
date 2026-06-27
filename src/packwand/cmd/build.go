package cmd

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"packwand/manifest"
	"packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	llBuildCmd.Flags().StringP("pack", "p", "", "Build a specific pack by name (skip git diff detection)")
	rootCmd.AddCommand(llBuildCmd)

	rootCmd.AddCommand(llExportCmd)

	llPublishCmd.AddCommand(llPublishListCmd)
	llPublishCmd.AddCommand(llPublishBuildCmd)
	llPublishCmd.AddCommand(llPublishUploadCmd)
	llPublishCmd.AddCommand(llPublishVerifyCmd)
	rootCmd.AddCommand(llPublishCmd)
}

// — platform type —

type platform struct {
	short string
	cli   string
	ext   string
}

var (
	platModrinth   = platform{short: "mr", cli: "modrinth", ext: "mrpack"}
	platCurseforge = platform{short: "cf", cli: "curseforge", ext: "zip"}
)

func platformFromSuffix(s string) (platform, bool) {
	switch s {
	case "mr":
		return platModrinth, true
	case "cf":
		return platCurseforge, true
	default:
		return platform{}, false
	}
}

type buildTarget struct {
	category string
	pack     string
}

type exportJob struct {
	plat   platform
	dir    string
	subKey string
}

type queuedJob struct {
	label string
	done  <-chan error
}

// — build —

var llBuildCmd = &cobra.Command{
	Use:   "build [sha]",
	Short: "Build modpack exports and zip packs from git-changed targets (CI mode)",
	Args:  cobra.RangeArgs(0, 1),
	Run: func(cmd *cobra.Command, args []string) {
		targetedPack, _ := cmd.Flags().GetString("pack")
		shortSHA := "local"
		if len(args) > 0 {
			shortSHA = args[0]
		}
		llChdir()
		llRunBuild(targetedPack, shortSHA)
	},
}

var llExportCmd = &cobra.Command{
	Use:   "export [pack-name]",
	Short: "Export packs locally (like build but uses 'local' as the SHA suffix)",
	Args:  cobra.RangeArgs(0, 1),
	Run: func(cmd *cobra.Command, args []string) {
		pack := ""
		if len(args) > 0 {
			pack = args[0]
		}
		llChdir()
		llRunBuild(pack, "local")
	},
}

func llRunBuild(targetedPack, shortSHA string) {
	repoRoot, err := os.Getwd()
	if err != nil {
		llFail(fmt.Sprintf("failed to get current directory: %v", err))
	}
	artifactsDir := filepath.Join(repoRoot, "artifacts")
	if err := os.MkdirAll(artifactsDir, 0o755); err != nil {
		llFail(fmt.Sprintf("failed to create %s: %v", artifactsDir, err))
	}

	var changed []buildTarget
	if targetedPack != "" {
		t, err := resolvePack(targetedPack)
		if err != nil {
			llFail(err.Error())
		}
		changed = []buildTarget{t}
	} else {
		changed, err = detectChangedTargets()
		if err != nil {
			llFail(fmt.Sprintf("failed to detect changed targets: %v", err))
		}
		if len(changed) == 0 {
			fmt.Println("no packs detected in git diff.")
			return
		}
	}

	sched := workspace.NewScheduler(workspace.MaxConcurrent())
	slots := workspace.CacheSlotCount()

	type pending struct {
		label string
		done  <-chan error
	}
	var jobs []pending

	for _, t := range changed {
		switch t.category {
		case "modpacks":
			ps, err := queueModpackExports(sched, slots, t.pack, shortSHA, artifactsDir)
			if err != nil {
				llFail(fmt.Sprintf("modpack '%s' failed to enqueue: %v", t.pack, err))
			}
			for _, p := range ps {
				jobs = append(jobs, pending{label: p.label, done: p.done})
			}
		case "datapacks", "resourcepacks":
			p, err := queueZipPackBuild(sched, t.category, t.pack, shortSHA, artifactsDir)
			if err != nil {
				llFail(fmt.Sprintf("%s '%s' failed to enqueue: %v", t.category, t.pack, err))
			}
			jobs = append(jobs, pending{label: p.label, done: p.done})
		default:
			fmt.Printf("category '%s' does not require a build.\n", t.category)
		}
	}

	sched.Close()
	var failed int
	for _, j := range jobs {
		if err := <-j.done; err != nil {
			fmt.Fprintf(os.Stderr, "build failed: %s: %v\n", j.label, err)
			failed++
		}
	}
	if failed > 0 {
		llFail(fmt.Sprintf("%d build(s) failed", failed))
	}
	fmt.Println("all builds completed successfully.")
}

func resolvePack(name string) (buildTarget, error) {
	for _, category := range []string{"modpacks", "datapacks", "resourcepacks"} {
		if info, err := os.Stat(filepath.Join(category, name)); err == nil && info.IsDir() {
			return buildTarget{category: category, pack: name}, nil
		}
	}
	return buildTarget{}, fmt.Errorf("pack '%s' not found in modpacks/, datapacks/, or resourcepacks/", name)
}

func detectChangedTargets() ([]buildTarget, error) {
	fmt.Println("detecting changed files...")
	out, err := exec.Command("git", "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to invoke git: %w", err)
	}

	seen := make(map[buildTarget]struct{})
	var targets []buildTarget

	for _, line := range strings.Split(string(out), "\n") {
		if line == "" || strings.HasPrefix(line, "external/") || strings.HasPrefix(line, ".actions/") {
			continue
		}
		parts := strings.SplitN(line, "/", 3)
		if len(parts) < 2 {
			continue
		}
		t := buildTarget{category: parts[0], pack: parts[1]}
		if _, ok := seen[t]; !ok {
			seen[t] = struct{}{}
			targets = append(targets, t)
		}
	}
	return targets, nil
}

func subdirKeys(m *manifest.Manifest) []string {
	if len(m.Variants) == 0 {
		return nil
	}
	keys := make([]string, 0, len(m.Variants))
	for _, v := range m.Variants {
		if v.ID != "" {
			keys = append(keys, v.ID)
		} else {
			keys = append(keys, v.MCVersion)
		}
	}
	return keys
}

func queueModpackExports(sched *workspace.Scheduler, slots int, packID, sha, artifactsDir string) ([]queuedJob, error) {
	fmt.Printf("queueing modpack exports: %s\n", packID)
	packDir := filepath.Join("modpacks", packID)

	m, err := manifest.Read(filepath.Join(packDir, "manifest.json"))
	if err != nil {
		return nil, err
	}
	if m.Version == "" {
		return nil, fmt.Errorf("missing 'version' in %s/manifest.json", packDir)
	}

	var jobs []exportJob
	keys := subdirKeys(m)

	if keys != nil {
		for _, key := range keys {
			for _, plat := range []platform{platModrinth, platCurseforge} {
				dir := filepath.Join(packDir, key+"-"+plat.short)
				if info, err := os.Stat(dir); err == nil && info.IsDir() {
					jobs = append(jobs, exportJob{plat: plat, dir: dir, subKey: key})
				}
			}
		}
	} else {
		entries, err := os.ReadDir(packDir)
		if err != nil {
			return nil, fmt.Errorf("failed to read %s: %w", packDir, err)
		}
		for _, e := range entries {
			if !e.IsDir() {
				continue
			}
			name := e.Name()
			idx := strings.LastIndex(name, "-")
			if idx < 0 {
				continue
			}
			key, suffix := name[:idx], name[idx+1:]
			plat, ok := platformFromSuffix(suffix)
			if !ok {
				continue
			}
			jobs = append(jobs, exportJob{plat: plat, dir: filepath.Join(packDir, name), subKey: key})
		}
	}

	if len(jobs) == 0 {
		return nil, fmt.Errorf("no valid version dirs (expected '{key}-mr' or '{key}-cf') for %s", packID)
	}

	packName := strings.ReplaceAll(m.Name, " ", "-")
	out := make([]queuedJob, 0, len(jobs))
	for _, j := range jobs {
		j := j
		outputName := fmt.Sprintf("%s-%s-%s-%s-%s.%s",
			packName, j.subKey, j.plat.short, m.Version, sha, j.plat.ext)
		outputPath := filepath.Join(artifactsDir, outputName)
		done := sched.Submit(workspace.Task{
			Name: outputName,
			Needs: []workspace.Resource{
				workspace.Resource("export:" + j.dir),
				workspace.CacheSlot(j.dir, slots),
			},
			Run: func() error {
				c := exec.Command(workspace.SelfBin(), j.plat.cli, "export", "--output", outputPath)
				c.Dir = j.dir
				if out, err := c.CombinedOutput(); err != nil {
					return fmt.Errorf("packwand export failed for %s: %v\n%s", j.dir, err, out)
				}
				fmt.Printf("exported %s\n", outputName)
				return nil
			},
		})
		out = append(out, queuedJob{label: outputName, done: done})
	}
	return out, nil
}

func queueZipPackBuild(sched *workspace.Scheduler, category, packID, sha, artifactsDir string) (queuedJob, error) {
	packDir := filepath.Join(category, packID)

	versionDir, version, err := findSingleVersionDir(packDir)
	if err != nil {
		return queuedJob{}, err
	}

	dest := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s-%s.zip", packID, version, sha))
	label := filepath.Base(dest)

	done := sched.Submit(workspace.Task{
		Name:  label,
		Needs: []workspace.Resource{workspace.Resource("zip:" + packDir)},
		Run: func() error {
			if category == "resourcepacks" {
				if bin := packsquashBin(); bin != "" {
					fmt.Printf("squashing %s: %s (PackSquash)\n", category, packID)
					return squashContents(bin, packDir, versionDir, dest)
				}
				fmt.Printf("zipping %s: %s (packsquash not found; plain zip)\n", category, packID)
			} else {
				fmt.Printf("zipping %s: %s\n", category, packID)
			}
			return zipContents(versionDir, dest)
		},
	})
	return queuedJob{label: label, done: done}, nil
}

func packsquashBin() string {
	if b := os.Getenv("PACKSQUASH_BIN"); b != "" {
		return b
	}
	if p, err := exec.LookPath("packsquash"); err == nil {
		return p
	}
	return ""
}

func squashContents(bin, packDir, src, dest string) error {
	opts := fmt.Sprintf("pack_directory = %q\noutput_file_path = %q\nzip_spec_conformance_level = \"high\"\n", src, dest)
	if extra, err := os.ReadFile(filepath.Join(packDir, "packsquash.toml")); err == nil {
		fmt.Printf("  applying pack-level packsquash.toml\n")
		opts += "\n" + string(extra)
	}
	optFile, err := os.CreateTemp("", "packwand-packsquash-*.toml")
	if err != nil {
		return fmt.Errorf("failed to create packsquash options file: %w", err)
	}
	defer os.Remove(optFile.Name())
	if _, err := optFile.WriteString(opts); err != nil {
		return fmt.Errorf("failed to write packsquash options: %w", err)
	}
	optFile.Close()

	c := exec.Command(bin, optFile.Name())
	if out, err := c.CombinedOutput(); err != nil {
		return fmt.Errorf("packsquash failed for %s: %v\n%s", src, err, workspace.Indent(string(out), "    "))
	}
	fmt.Printf("squashed %s\n", filepath.Base(dest))
	return nil
}

func findSingleVersionDir(packDir string) (dir string, version string, err error) {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return "", "", fmt.Errorf("failed to read %s: %w", packDir, err)
	}
	var versionDirs []string
	for _, e := range entries {
		if e.IsDir() {
			versionDirs = append(versionDirs, e.Name())
		}
	}
	switch len(versionDirs) {
	case 0:
		return "", "", fmt.Errorf("no version directory found in %s", packDir)
	case 1:
		return filepath.Join(packDir, versionDirs[0]), versionDirs[0], nil
	default:
		return "", "", fmt.Errorf("expected exactly one version directory in %s, found %d: %v", packDir, len(versionDirs), versionDirs)
	}
}

func zipContents(src, dest string) error {
	f, err := os.Create(dest)
	if err != nil {
		return fmt.Errorf("failed to create %s: %w", dest, err)
	}
	defer f.Close()

	zw := zip.NewWriter(f)
	defer zw.Close()

	return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		if rel == "." {
			return nil
		}
		zipName := filepath.ToSlash(rel)
		if info.IsDir() {
			_, err := zw.Create(zipName + "/")
			return err
		}
		w, err := zw.Create(zipName)
		if err != nil {
			return err
		}
		in, err := os.Open(path)
		if err != nil {
			return fmt.Errorf("failed to open %s: %w", path, err)
		}
		defer in.Close()
		_, err = io.Copy(w, in)
		return err
	})
}

// — publish —

var llPublishCmd = &cobra.Command{
	Use:   "publish",
	Short: "Build, upload, verify, or list publish targets for a pack",
}

var llPublishListCmd = &cobra.Command{
	Use:   "list <manifest.json...>",
	Short: "Enumerate all (manifest, variant) publish pairs as JSON (for CI matrix)",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		pubList(args)
	},
}

var llPublishBuildCmd = &cobra.Command{
	Use:   "build <manifest.json> [variant]",
	Short: "Export the pack artifact(s) for publishing",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(cmd *cobra.Command, args []string) {
		manifestPath := llAbs(args[0])
		variant := ""
		if len(args) > 1 {
			variant = args[1]
		}
		llChdir()
		pubBuild(manifestPath, variant)
	},
}

var llPublishUploadCmd = &cobra.Command{
	Use:   "upload <manifest.json> [variant]",
	Short: "Upload pre-built artifacts to Modrinth and/or CurseForge",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(cmd *cobra.Command, args []string) {
		manifestPath := llAbs(args[0])
		variant := ""
		if len(args) > 1 {
			variant = args[1]
		}
		live, _ := cmd.Flags().GetBool("live")
		llChdir()
		pubUpload(manifestPath, variant, live)
	},
}

var llPublishVerifyCmd = &cobra.Command{
	Use:   "verify <manifest.json> [variant]",
	Short: "Verify a published version exists live on Modrinth",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(cmd *cobra.Command, args []string) {
		manifestPath := llAbs(args[0])
		variant := ""
		if len(args) > 1 {
			variant = args[1]
		}
		llChdir()
		pubVerify(manifestPath, variant)
	},
}

func init() {
	llPublishUploadCmd.Flags().Bool("live", false, "Actually upload (default: dry run)")
}

// — publish types and helpers —

type pubResolved struct {
	pName, rawName, pType, loader, releaseType string
	mrID, cfID, subdirKey, mcVer, pVer         string
	displayName                                 string
	isExperimental                              bool
	builtMR, builtCF                           bool
}

func pubResolve(manifestPath, variant string) pubResolved {
	isExperimental := filepath.Base(manifestPath) == "manifest-experimental.json"
	m, err := manifest.Read(manifestPath)
	if err != nil {
		llFail(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
	}

	if m.Name == "" {
		llFail(fmt.Sprintf("missing 'name' in %s", manifestPath))
	}
	if m.Type == "" {
		llFail(fmt.Sprintf("missing 'type' in %s", manifestPath))
	}
	if m.ReleaseType == "" {
		llFail(fmt.Sprintf("missing 'release_type' in %s", manifestPath))
	}
	if m.ID == "" {
		llFail(fmt.Sprintf("missing 'id' in %s", manifestPath))
	}

	rawName := m.Name
	r := pubResolved{
		rawName:        rawName,
		pName:          strings.ReplaceAll(rawName, " ", "-"),
		pType:          m.Type,
		releaseType:    m.ReleaseType,
		mrID:           m.ModrinthID,
		cfID:           m.CurseforgeID,
		isExperimental: isExperimental,
	}
	if r.mrID == "" && r.cfID == "" {
		llFail("manifest must set at least one of modrinth_id or curseforge_id")
	}

	packLoader := m.Loader
	var variantName, variantVersion, variantLoader string

	if variant != "" {
		if len(m.Variants) == 0 {
			llFail(fmt.Sprintf("variant '%s' requested but manifest has no 'variants'", variant))
		}
		var found *manifest.Variant
		for i := range m.Variants {
			v := &m.Variants[i]
			key := v.ID
			if key == "" {
				key = v.MCVersion
			}
			if key == variant {
				found = v
				break
			}
		}
		if found == nil {
			llFail(fmt.Sprintf("variant '%s' not found in manifest", variant))
		}
		r.subdirKey = variant
		r.mcVer = found.MCVersion
		if r.mcVer == "" {
			llFail(fmt.Sprintf("variant '%s' missing mc_version", variant))
		}
		variantName = found.Name
		variantVersion = found.Version
		variantLoader = found.Loader
	} else {
		if m.MCVersion == nil || *m.MCVersion == "" {
			llFail(fmt.Sprintf("missing 'mc_version' in %s", manifestPath))
		}
		r.mcVer = *m.MCVersion
		r.subdirKey = r.mcVer
	}

	r.loader = variantLoader
	if r.loader == "" {
		r.loader = packLoader
	}
	if r.pType == "modpack" && r.loader == "" {
		llFail(fmt.Sprintf("no loader resolved for '%s': set a pack-level 'loader' or a variant 'loader'", r.subdirKey))
	}

	if isExperimental {
		sha := os.Getenv("GITHUB_SHA")
		if sha == "" {
			llFail("GITHUB_SHA not set; required for experimental builds")
		}
		short := sha
		if len(short) > 7 {
			short = short[:7]
		}
		cycle := time.Now().UTC().Format("06.01")
		if variant != "" {
			r.pVer = fmt.Sprintf("%s-%s-%s-%s", m.ID, variant, cycle, short)
		} else {
			r.pVer = fmt.Sprintf("%s-%s-%s", m.ID, cycle, short)
		}
	} else {
		baseVer := variantVersion
		if baseVer == "" {
			baseVer = m.Version
		}
		if baseVer == "" {
			llFail("missing 'version'")
		}
		if variant != "" {
			r.pVer = baseVer + "-" + variant
		} else {
			r.pVer = baseVer
		}
	}

	switch {
	case variantName != "":
		r.displayName = fmt.Sprintf("%s %s %s", rawName, variantName, r.pVer)
	case variant != "":
		r.displayName = fmt.Sprintf("%s %s %s", rawName, variant, r.pVer)
	default:
		r.displayName = fmt.Sprintf("%s %s", rawName, r.pVer)
	}
	if isExperimental {
		r.displayName = "[EXPERIMENTAL] " + r.displayName
	}
	return r
}

func pubList(manifestPaths []string) {
	entries := []map[string]any{}
	for _, manifestPath := range manifestPaths {
		m, err := manifest.Read(manifestPath)
		if err != nil {
			llFail(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
		}
		if len(m.Variants) > 0 {
			for idx, v := range m.Variants {
				key := v.ID
				if key == "" {
					key = v.MCVersion
				}
				if key == "" {
					llFail("variant missing both 'id' and 'mc_version'")
				}
				entries = append(entries, map[string]any{"manifest": manifestPath, "variant": key, "order": idx})
			}
		} else {
			entries = append(entries, map[string]any{"manifest": manifestPath, "variant": nil, "order": 0})
		}
	}
	data, err := json.Marshal(entries)
	if err != nil {
		llFail(fmt.Sprintf("failed to render entries: %v", err))
	}
	fmt.Println(string(data))
}

func pubBuild(manifestPath, variant string) {
	pDir := filepath.Dir(manifestPath)
	m, err := manifest.Read(manifestPath)
	if err != nil {
		llFail(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
	}
	r := pubResolve(manifestPath, variant)

	ghWorkspace := os.Getenv("GITHUB_WORKSPACE")
	if ghWorkspace == "" {
		ghWorkspace = "."
	}
	artifactsDir := filepath.Join(ghWorkspace, pDir, "artifacts")
	if err := os.RemoveAll(artifactsDir); err != nil {
		llFail(fmt.Sprintf("failed to clear %s: %v", artifactsDir, err))
	}
	if err := os.MkdirAll(artifactsDir, 0o755); err != nil {
		llFail(fmt.Sprintf("failed to create %s: %v", artifactsDir, err))
	}

	var label string
	switch {
	case r.isExperimental && variant != "":
		label = fmt.Sprintf("EXPERIMENTAL %s [%s] (%s)", r.rawName, variant, r.pVer)
	case r.isExperimental:
		label = fmt.Sprintf("EXPERIMENTAL %s (%s)", r.rawName, r.pVer)
	case variant != "":
		label = fmt.Sprintf("%s [%s]", r.rawName, variant)
	default:
		label = r.rawName
	}
	fmt.Printf("::group::Building %s\n", label)

	switch r.pType {
	case "modpack":
		pubBuildModpack(pDir, artifactsDir, &r)
	case "datapack":
		pubBuildDatapack(pDir, artifactsDir, m.ID, r.pVer)
		r.builtMR = r.mrID != ""
		r.builtCF = r.cfID != ""
	default:
		llFail(fmt.Sprintf("unsupported pack type: %s", r.pType))
	}
	fmt.Println("::endgroup::")

	pubWriteOutputs(r, pDir)
}

func pubBuildModpack(pDir, artifactsDir string, r *pubResolved) {
	filenameBase := fmt.Sprintf("%s-%s-%s-%s", r.pName, r.mcVer, r.loader, r.pVer)

	type exportPlan struct {
		plat       platform
		targetPath string
		outFile    string
		flag       *bool
	}
	var plans []exportPlan

	for _, pl := range []struct {
		plat platform
		id   string
		flag *bool
	}{{platModrinth, r.mrID, &r.builtMR}, {platCurseforge, r.cfID, &r.builtCF}} {
		if pl.id == "" {
			continue
		}
		targetPath := filepath.Join(pDir, r.subdirKey+"-"+pl.plat.short)
		if info, err := os.Stat(targetPath); err != nil || !info.IsDir() {
			fmt.Printf("skipping %s: folder %s not found (variant not published to this platform)\n", pl.plat.short, targetPath)
			continue
		}
		plans = append(plans, exportPlan{
			plat:       pl.plat,
			targetPath: targetPath,
			outFile:    filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.%s", filenameBase, pl.plat.short, pl.plat.ext)),
			flag:       pl.flag,
		})
	}

	if len(plans) == 0 {
		llFail(fmt.Sprintf("no platform folders found for subdir key '%s' (expected %s-mr / %s-cf)", r.subdirKey, r.subdirKey, r.subdirKey))
	}

	sched := workspace.NewScheduler(workspace.MaxConcurrent())
	slots := workspace.CacheSlotCount()
	dones := make([]<-chan error, len(plans))
	for i, p := range plans {
		p := p
		dones[i] = sched.Submit(workspace.Task{
			Name: filepath.Base(p.outFile),
			Needs: []workspace.Resource{
				workspace.Resource("export:" + p.targetPath),
				workspace.CacheSlot(p.targetPath, slots),
			},
			Run: func() error {
				c := exec.Command(workspace.SelfBin(), p.plat.cli, "export", "--output", p.outFile)
				c.Dir = p.targetPath
				if out, err := c.CombinedOutput(); err != nil {
					return fmt.Errorf("packwand export failed for %s: %v\n%s", p.targetPath, err, workspace.Indent(string(out), "    "))
				}
				fmt.Printf("exported %s\n", p.outFile)
				*p.flag = true
				return nil
			},
		})
	}
	sched.Close()
	for _, c := range dones {
		if err := <-c; err != nil {
			llFail(err.Error())
		}
	}
}

func pubBuildDatapack(pDir, artifactsDir, id, pVer string) {
	contentDir := filepath.Join(pDir, "content")
	if info, err := os.Stat(contentDir); err != nil || !info.IsDir() {
		llFail(fmt.Sprintf("content directory not found at %s", contentDir))
	}
	outFile := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.zip", id, pVer))
	if err := zipContents(contentDir, outFile); err != nil {
		llFail(fmt.Sprintf("zip failed: %v", err))
	}
}

func pubWriteOutputs(r pubResolved, pDir string) {
	outPath := os.Getenv("GITHUB_OUTPUT")
	if outPath == "" {
		return
	}
	f, err := os.OpenFile(outPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		llFail(fmt.Sprintf("failed to open GITHUB_OUTPUT: %v", err))
	}
	defer f.Close()
	mrOut, cfOut := "", ""
	if r.builtMR {
		mrOut = r.mrID
	}
	if r.builtCF {
		cfOut = r.cfID
	}
	fmt.Fprintf(f, "mr_id=%s\n", mrOut)
	fmt.Fprintf(f, "cf_id=%s\n", cfOut)
	fmt.Fprintf(f, "name=%s\n", r.displayName)
	fmt.Fprintf(f, "ver=%s\n", r.pVer)
	fmt.Fprintf(f, "mc=%s\n", r.mcVer)
	fmt.Fprintf(f, "type=%s\n", r.pType)
	fmt.Fprintf(f, "loader=%s\n", r.loader)
	fmt.Fprintf(f, "release_type=%s\n", r.releaseType)
	fmt.Fprintf(f, "path=%s\n", pDir)
	fmt.Fprintf(f, "is_experimental=%t\n", r.isExperimental)
}

// — upload / verify —

const (
	modrinthAPI   = "https://api.modrinth.com/v2"
	curseforgeAPI = "https://minecraft.curseforge.com/api"
)

func pubUpload(manifestPath, variant string, live bool) {
	pDir := filepath.Dir(manifestPath)
	r := pubResolve(manifestPath, variant)

	if r.pType != "modpack" {
		llFail(fmt.Sprintf("upload currently supports modpacks only (got '%s')", r.pType))
	}

	changelog := fmt.Sprintf("Update for %s", r.rawName)
	if data, err := os.ReadFile(filepath.Join(pDir, "changelog.md")); err == nil {
		changelog = string(data)
	}

	ghWorkspace := os.Getenv("GITHUB_WORKSPACE")
	if ghWorkspace == "" {
		ghWorkspace = "."
	}
	artifactsDir := filepath.Join(ghWorkspace, pDir, "artifacts")
	filenameBase := fmt.Sprintf("%s-%s-%s-%s", r.pName, r.mcVer, r.loader, r.pVer)

	if !live {
		fmt.Println("[DRY RUN] publish upload — nothing will be sent (pass --live to upload)")
	}

	attempted, uploaded := 0, 0
	for _, pl := range []struct {
		plat platform
		id   string
	}{{platModrinth, r.mrID}, {platCurseforge, r.cfID}} {
		if pl.id == "" {
			continue
		}
		artifact := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.%s", filenameBase, pl.plat.short, pl.plat.ext))
		if _, err := os.Stat(artifact); err != nil {
			fmt.Printf("skipping %s: artifact %s not found (run 'publish build' first)\n", pl.plat.short, artifact)
			continue
		}
		attempted++
		data, err := os.ReadFile(artifact)
		if err != nil {
			llFail(fmt.Sprintf("reading %s: %v", artifact, err))
		}
		fileName := filepath.Base(artifact)

		switch pl.plat.short {
		case "mr":
			uploadModrinth(r, pl.id, changelog, fileName, data, live)
		case "cf":
			uploadCurseforge(r, pl.id, changelog, fileName, data, live)
		}
		uploaded++
	}

	if attempted == 0 {
		llFail(fmt.Sprintf("no artifacts found for '%s' in %s — run 'publish build' before 'publish upload'", r.subdirKey, artifactsDir))
	}
	mode := "validated (dry run)"
	if live {
		mode = "uploaded"
	}
	fmt.Printf("%d artifact(s) %s for %s\n", uploaded, mode, r.displayName)
}

func uploadModrinth(r pubResolved, projectID, changelog, fileName string, data []byte, live bool) {
	payload := map[string]any{
		"project_id":     projectID,
		"name":           r.displayName,
		"version_number": r.pVer,
		"changelog":      changelog,
		"dependencies":   []any{},
		"game_versions":  []string{r.mcVer},
		"version_type":   r.releaseType,
		"loaders":        []string{r.loader},
		"featured":       false,
		"file_parts":     []string{"file"},
		"primary_file":   "file",
	}

	fmt.Printf("modrinth: %s -> project %s | version %s | mc %s | loader %s | %d bytes\n",
		fileName, projectID, r.pVer, r.mcVer, r.loader, len(data))
	if !live {
		return
	}
	token := os.Getenv("MODRINTH_TOKEN")
	if token == "" {
		llFail("MODRINTH_TOKEN not set")
	}

	meta, _ := json.Marshal(payload)
	contentType, body := buildMultipart([]mpart{
		{name: "data", contentType: "application/json", data: meta},
		{name: "file", fileName: fileName, contentType: "application/octet-stream", data: data},
	})

	doUpload("modrinth", modrinthAPI+"/version", map[string]string{
		"Authorization": token,
		"Content-Type":  contentType,
	}, body, r.pVer, projectID)
}

func uploadCurseforge(r pubResolved, projectID, changelog, fileName string, data []byte, live bool) {
	fmt.Printf("curseforge: %s -> project %s | version %s | mc %s | loader %s | %d bytes\n",
		fileName, projectID, r.pVer, r.mcVer, r.loader, len(data))
	if !live {
		return
	}
	token := os.Getenv("CURSEFORGE_TOKEN")
	if token == "" {
		llFail("CURSEFORGE_TOKEN not set")
	}

	versionIDs := cfGameVersionIDs(token, r.mcVer, r.loader)
	meta, _ := json.Marshal(map[string]any{
		"changelog":     changelog,
		"changelogType": "markdown",
		"displayName":   r.displayName,
		"gameVersions":  versionIDs,
		"releaseType":   r.releaseType,
	})

	contentType, body := buildMultipart([]mpart{
		{name: "metadata", contentType: "application/json", data: meta},
		{name: "file", fileName: fileName, contentType: "application/octet-stream", data: data},
	})

	doUpload("curseforge", fmt.Sprintf("%s/projects/%s/upload-file", curseforgeAPI, projectID), map[string]string{
		"X-Api-Token":  token,
		"Content-Type": contentType,
	}, body, r.pVer, projectID)
}

func doUpload(label, url string, headers map[string]string, body []byte, pVer, projectID string) {
	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		llFail(fmt.Sprintf("%s upload failed: %v", label, err))
	}
	for k, v := range headers {
		req.Header.Set(k, v)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		llFail(fmt.Sprintf("%s upload failed: %v", label, err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		llFail(fmt.Sprintf("%s upload failed (HTTP %d): %s", label, resp.StatusCode, string(detail)))
	}
	fmt.Printf("%s: uploaded %s to %s\n", label, pVer, projectID)
}

func cfGameVersionIDs(token, mcVer, loader string) []int64 {
	req, _ := http.NewRequest(http.MethodGet, curseforgeAPI+"/game/versions", nil)
	req.Header.Set("X-Api-Token", token)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		llFail(fmt.Sprintf("CF game/versions lookup failed: %v", err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		llFail(fmt.Sprintf("CF game/versions lookup failed (HTTP %d): %s", resp.StatusCode, string(detail)))
	}
	var list []map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&list); err != nil {
		llFail(fmt.Sprintf("parsing CF game versions: %v", err))
	}

	loaderLC := strings.ToLower(loader)
	var ids []int64
	for _, entry := range list {
		name, _ := entry["name"].(string)
		slug, _ := entry["slug"].(string)
		idF, ok := entry["id"].(float64)
		if !ok {
			continue
		}
		if name == mcVer || slug == loaderLC || strings.ToLower(name) == loaderLC {
			ids = append(ids, int64(idF))
		}
	}
	if len(ids) < 2 {
		llFail(fmt.Sprintf("could not resolve CF game-version IDs for mc '%s' + loader '%s' (matched %d of 2)", mcVer, loader, len(ids)))
	}
	return ids
}

func pubVerify(manifestPath, variant string) {
	r := pubResolve(manifestPath, variant)
	if r.mrID == "" {
		llFail("verify currently checks Modrinth only, and this manifest has no modrinth_id")
	}
	url := fmt.Sprintf("%s/project/%s/version", modrinthAPI, r.mrID)
	resp, err := http.Get(url) //nolint:gosec
	if err != nil {
		llFail(fmt.Sprintf("Modrinth version lookup failed: %v", err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		llFail(fmt.Sprintf("Modrinth version lookup failed (HTTP %d): %s", resp.StatusCode, string(detail)))
	}
	var versions []map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&versions); err != nil {
		llFail(fmt.Sprintf("parsing Modrinth version list: %v", err))
	}
	for _, v := range versions {
		if vn, _ := v["version_number"].(string); vn == r.pVer {
			vid, _ := v["id"].(string)
			published, _ := v["date_published"].(string)
			fmt.Printf("verified: %s %s is live on Modrinth (version id %s, published %s)\n", r.displayName, r.pVer, vid, published)
			return
		}
	}
	llFail(fmt.Sprintf("version '%s' NOT found on Modrinth project '%s' (%d version(s) listed)", r.pVer, r.mrID, len(versions)))
}

type mpart struct {
	name        string
	fileName    string
	contentType string
	data        []byte
}

func buildMultipart(parts []mpart) (contentType string, body []byte) {
	var buf bytes.Buffer
	w := multipart.NewWriter(&buf)
	for _, p := range parts {
		h := textproto.MIMEHeader{}
		if p.fileName != "" {
			h.Set("Content-Disposition", fmt.Sprintf(`form-data; name=%q; filename=%q`, p.name, p.fileName))
		} else {
			h.Set("Content-Disposition", fmt.Sprintf(`form-data; name=%q`, p.name))
		}
		h.Set("Content-Type", p.contentType)
		pw, err := w.CreatePart(h)
		if err != nil {
			llFail(fmt.Sprintf("multipart build failed: %v", err))
		}
		if _, err := pw.Write(p.data); err != nil {
			llFail(fmt.Sprintf("multipart build failed: %v", err))
		}
	}
	w.Close()
	return w.FormDataContentType(), buf.Bytes()
}
