package build

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

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	buildCmd.Flags().StringP("pack", "p", "", "Build a specific pack by name (skip git diff detection)")
	cmd.AddToGroup(buildCmd, cmd.GroupBuildExport)

	cmd.AddToGroup(exportCmd, cmd.GroupBuildExport)

	publishCmd.AddCommand(publishListCmd)
	publishCmd.AddCommand(publishBuildCmd)
	publishCmd.AddCommand(publishUploadCmd)
	publishCmd.AddCommand(publishVerifyCmd)
	cmd.AddToGroup(publishCmd, cmd.GroupBuildExport)
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

var buildCmd = &cobra.Command{
	Use:   "build [sha]",
	Short: "Build modpack exports and zip packs from git-changed targets (CI mode)",
	Args:  cobra.RangeArgs(0, 1),
	Run: func(c *cobra.Command, args []string) {
		targetedPack, _ := c.Flags().GetString("pack")
		shortSHA := "local"
		if len(args) > 0 {
			shortSHA = args[0]
		}
		cmd.Chdir()
		runBuild(targetedPack, shortSHA)
	},
}

var exportCmd = &cobra.Command{
	Use:   "export [pack-name]",
	Short: "Export packs locally (like build but uses 'local' as the SHA suffix)",
	Args:  cobra.RangeArgs(0, 1),
	Run: func(c *cobra.Command, args []string) {
		pack := ""
		if len(args) > 0 {
			pack = args[0]
		}
		cmd.Chdir()
		runBuild(pack, "local")
	},
}

func runBuild(targetedPack, shortSHA string) {
	repoRoot, err := os.Getwd()
	if err != nil {
		cmd.Fail(fmt.Sprintf("failed to get current directory: %v", err))
	}
	artifactsDir := filepath.Join(repoRoot, "artifacts")
	if err := os.MkdirAll(artifactsDir, 0o755); err != nil {
		cmd.Fail(fmt.Sprintf("failed to create %s: %v", artifactsDir, err))
	}

	var changed []buildTarget
	if targetedPack != "" {
		t, err := resolvePack(targetedPack)
		if err != nil {
			cmd.Fail(err.Error())
		}
		changed = []buildTarget{t}
	} else {
		changed, err = detectChangedTargets()
		if err != nil {
			cmd.Fail(fmt.Sprintf("failed to detect changed targets: %v", err))
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
				cmd.Fail(fmt.Sprintf("modpack '%s' failed to enqueue: %v", t.pack, err))
			}
			for _, p := range ps {
				jobs = append(jobs, pending{label: p.label, done: p.done})
			}
		case "datapacks", "resourcepacks":
			p, err := queueZipPackBuild(sched, t.category, t.pack, shortSHA, artifactsDir)
			if err != nil {
				cmd.Fail(fmt.Sprintf("%s '%s' failed to enqueue: %v", t.category, t.pack, err))
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
		cmd.Fail(fmt.Sprintf("%d build(s) failed", failed))
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
				workspace.ConfigureSubprocess(c)
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

var publishCmd = &cobra.Command{
	Use:   "publish",
	Short: "Build, upload, verify, or list publish targets for a pack",
}

var publishListCmd = &cobra.Command{
	Use:   "list <manifest.json...>",
	Short: "Enumerate all (manifest, variant) publish pairs as JSON (for CI matrix)",
	Args:  cobra.MinimumNArgs(1),
	Run: func(c *cobra.Command, args []string) {
		cmd.Chdir()
		pubList(args)
	},
}

var publishBuildCmd = &cobra.Command{
	Use:   "build <manifest.json> [variant]",
	Short: "Export the pack artifact(s) for publishing",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(c *cobra.Command, args []string) {
		manifestPath := cmd.Abs(args[0])
		variant := ""
		if len(args) > 1 {
			variant = args[1]
		}
		cmd.Chdir()
		pubBuild(manifestPath, variant)
	},
}

var publishUploadCmd = &cobra.Command{
	Use:   "upload <manifest.json> [variant]",
	Short: "Upload pre-built artifacts to Modrinth and/or CurseForge",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(c *cobra.Command, args []string) {
		manifestPath := cmd.Abs(args[0])
		variant := ""
		if len(args) > 1 {
			variant = args[1]
		}
		live, _ := c.Flags().GetBool("live")
		changelogFile, _ := c.Flags().GetString("changelog-file")
		if changelogFile != "" {
			changelogFile = cmd.Abs(changelogFile)
		}
		cmd.Chdir()
		pubUpload(manifestPath, variant, live, changelogFile)
	},
}

var publishVerifyCmd = &cobra.Command{
	Use:   "verify <manifest.json> [variant]",
	Short: "Verify a published version exists live on Modrinth",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(c *cobra.Command, args []string) {
		manifestPath := cmd.Abs(args[0])
		variant := ""
		if len(args) > 1 {
			variant = args[1]
		}
		cmd.Chdir()
		pubVerify(manifestPath, variant)
	},
}

func init() {
	publishUploadCmd.Flags().Bool("live", false, "Actually upload (default: dry run)")
	publishUploadCmd.Flags().String("changelog-file", "", "Read release notes from this file instead of the pack's changelog.md")
}

// — publish types and helpers —

type pubResolved struct {
	pName, rawName, pType, loader, releaseType string
	mrID, cfID, subdirKey, mcVer, pVer         string
	displayName, packID                        string
	isExperimental                             bool
	builtMR, builtCF                           bool
}

func pubResolve(manifestPath, variant string) pubResolved {
	isExperimental := filepath.Base(manifestPath) == "manifest-experimental.json"
	m, err := manifest.Read(manifestPath)
	if err != nil {
		cmd.Fail(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
	}

	if m.Name == "" {
		cmd.Fail(fmt.Sprintf("missing 'name' in %s", manifestPath))
	}
	if m.Type == "" {
		cmd.Fail(fmt.Sprintf("missing 'type' in %s", manifestPath))
	}
	if m.ReleaseType == "" {
		cmd.Fail(fmt.Sprintf("missing 'release_type' in %s", manifestPath))
	}
	if m.ID == "" {
		cmd.Fail(fmt.Sprintf("missing 'id' in %s", manifestPath))
	}

	rawName := m.Name
	r := pubResolved{
		rawName:        rawName,
		pName:          strings.ReplaceAll(rawName, " ", "-"),
		pType:          m.Type,
		releaseType:    m.ReleaseType,
		mrID:           m.ModrinthID,
		cfID:           m.CurseforgeID,
		packID:         m.ID,
		isExperimental: isExperimental,
	}
	if r.mrID == "" && r.cfID == "" {
		cmd.Fail("manifest must set at least one of modrinth_id or curseforge_id")
	}

	packLoader := m.Loader
	var variantName, variantVersion, variantLoader string

	if variant != "" {
		if len(m.Variants) == 0 {
			cmd.Fail(fmt.Sprintf("variant '%s' requested but manifest has no 'variants'", variant))
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
			cmd.Fail(fmt.Sprintf("variant '%s' not found in manifest", variant))
		}
		r.subdirKey = variant
		r.mcVer = found.MCVersion
		if r.mcVer == "" {
			cmd.Fail(fmt.Sprintf("variant '%s' missing mc_version", variant))
		}
		variantName = found.Name
		variantVersion = found.Version
		variantLoader = found.Loader
	} else {
		if m.MCVersion == nil || *m.MCVersion == "" {
			cmd.Fail(fmt.Sprintf("missing 'mc_version' in %s", manifestPath))
		}
		r.mcVer = *m.MCVersion
		r.subdirKey = r.mcVer
	}

	r.loader = variantLoader
	if r.loader == "" {
		r.loader = packLoader
	}
	if r.pType == "modpack" && r.loader == "" {
		cmd.Fail(fmt.Sprintf("no loader resolved for '%s': set a pack-level 'loader' or a variant 'loader'", r.subdirKey))
	}

	if isExperimental {
		sha := os.Getenv("GITHUB_SHA")
		if sha == "" {
			cmd.Fail("GITHUB_SHA not set; required for experimental builds")
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
			cmd.Fail("missing 'version'")
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
			cmd.Fail(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
		}
		if len(m.Variants) > 0 {
			for idx, v := range m.Variants {
				key := v.ID
				if key == "" {
					key = v.MCVersion
				}
				if key == "" {
					cmd.Fail("variant missing both 'id' and 'mc_version'")
				}
				entries = append(entries, map[string]any{"manifest": manifestPath, "variant": key, "order": idx})
			}
		} else {
			entries = append(entries, map[string]any{"manifest": manifestPath, "variant": nil, "order": 0})
		}
	}
	data, err := json.Marshal(entries)
	if err != nil {
		cmd.Fail(fmt.Sprintf("failed to render entries: %v", err))
	}
	fmt.Println(string(data))
}

func pubBuild(manifestPath, variant string) {
	pDir := filepath.Dir(manifestPath)
	m, err := manifest.Read(manifestPath)
	if err != nil {
		cmd.Fail(fmt.Sprintf("failed to read %s: %v", manifestPath, err))
	}
	r := pubResolve(manifestPath, variant)

	ghWorkspace := os.Getenv("GITHUB_WORKSPACE")
	if ghWorkspace == "" {
		ghWorkspace = "."
	}
	artifactsDir := filepath.Join(ghWorkspace, pDir, "artifacts")
	if err := os.RemoveAll(artifactsDir); err != nil {
		cmd.Fail(fmt.Sprintf("failed to clear %s: %v", artifactsDir, err))
	}
	if err := os.MkdirAll(artifactsDir, 0o755); err != nil {
		cmd.Fail(fmt.Sprintf("failed to create %s: %v", artifactsDir, err))
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
		cmd.Fail(fmt.Sprintf("unsupported pack type: %s", r.pType))
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
		cmd.Fail(fmt.Sprintf("no platform folders found for subdir key '%s' (expected %s-mr / %s-cf)", r.subdirKey, r.subdirKey, r.subdirKey))
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
				workspace.ConfigureSubprocess(c)
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
			cmd.Fail(err.Error())
		}
	}
}

func pubBuildDatapack(pDir, artifactsDir, id, pVer string) {
	contentDir := filepath.Join(pDir, "content")
	if info, err := os.Stat(contentDir); err != nil || !info.IsDir() {
		cmd.Fail(fmt.Sprintf("content directory not found at %s", contentDir))
	}
	outFile := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s.zip", id, pVer))
	if err := zipContents(contentDir, outFile); err != nil {
		cmd.Fail(fmt.Sprintf("zip failed: %v", err))
	}
}

func pubWriteOutputs(r pubResolved, pDir string) {
	outPath := os.Getenv("GITHUB_OUTPUT")
	if outPath == "" {
		return
	}
	f, err := os.OpenFile(outPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		cmd.Fail(fmt.Sprintf("failed to open GITHUB_OUTPUT: %v", err))
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

func pubUpload(manifestPath, variant string, live bool, changelogFile string) {
	pDir := filepath.Dir(manifestPath)
	r := pubResolve(manifestPath, variant)

	if r.pType != "modpack" && r.pType != "datapack" {
		cmd.Fail(fmt.Sprintf("upload supports modpacks and datapacks (got '%s')", r.pType))
	}

	changelog := fmt.Sprintf("Update for %s", r.rawName)
	if changelogFile != "" {
		data, err := os.ReadFile(changelogFile)
		if err != nil {
			cmd.Fail(fmt.Sprintf("reading --changelog-file %s: %v", changelogFile, err))
		}
		changelog = string(data)
	} else if data, err := os.ReadFile(filepath.Join(pDir, "changelog.md")); err == nil {
		changelog = string(data)
	}

	ghWorkspace := os.Getenv("GITHUB_WORKSPACE")
	if ghWorkspace == "" {
		ghWorkspace = "."
	}
	artifactsDir := filepath.Join(ghWorkspace, pDir, "artifacts")

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
		artifact := filepath.Join(artifactsDir, pubArtifactName(r, pl.plat))
		if _, err := os.Stat(artifact); err != nil {
			fmt.Printf("skipping %s: artifact %s not found (run 'publish build' first)\n", pl.plat.short, artifact)
			continue
		}
		attempted++
		data, err := os.ReadFile(artifact)
		if err != nil {
			cmd.Fail(fmt.Sprintf("reading %s: %v", artifact, err))
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
		cmd.Fail(fmt.Sprintf("no artifacts found for '%s' in %s — run 'publish build' before 'publish upload'", r.subdirKey, artifactsDir))
	}
	mode := "validated (dry run)"
	if live {
		mode = "uploaded"
	}
	fmt.Printf("%d artifact(s) %s for %s\n", uploaded, mode, r.displayName)
}

// pubArtifactName mirrors the naming used by pubBuildModpack / pubBuildDatapack.
func pubArtifactName(r pubResolved, pl platform) string {
	if r.pType == "datapack" {
		return fmt.Sprintf("%s-%s.zip", r.packID, r.pVer)
	}
	return fmt.Sprintf("%s-%s-%s-%s-%s.%s", r.pName, r.mcVer, r.loader, r.pVer, pl.short, pl.ext)
}

// modrinthLoaders resolves the loader tags for the Modrinth version payload.
// Non-modpack types without an explicit loader publish under the generic
// "minecraft" tag, matching what the previous mc-publish workflow sent.
func modrinthLoaders(r pubResolved) []string {
	if r.loader != "" {
		return []string{r.loader}
	}
	return []string{"minecraft"}
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
		"loaders":        modrinthLoaders(r),
		"featured":       false,
		"file_parts":     []string{"file"},
		"primary_file":   "file",
	}

	fmt.Printf("modrinth: %s -> project %s | version %s | mc %s | loaders %v | %d bytes\n",
		fileName, projectID, r.pVer, r.mcVer, modrinthLoaders(r), len(data))
	if !live {
		return
	}
	token := os.Getenv("MODRINTH_TOKEN")
	if token == "" {
		cmd.Fail("MODRINTH_TOKEN not set")
	}

	// The version-creation endpoint requires the project's base62 ID and,
	// unlike Modrinth's lookup endpoints, rejects a slug outright (manifests
	// configure modrinth_id as the human-readable slug, e.g. "re-console").
	payload["project_id"] = modrinthResolveProjectID(projectID)

	meta, _ := json.Marshal(payload)
	contentType, body := buildMultipart([]mpart{
		{name: "data", contentType: "application/json", data: meta},
		{name: "file", fileName: fileName, contentType: "application/octet-stream", data: data},
	})

	status, detail := postWithRetry("modrinth", modrinthAPI+"/version", map[string]string{
		"Authorization": token,
		"Content-Type":  contentType,
	}, body)
	if status < 200 || status >= 300 {
		cmd.Fail(fmt.Sprintf("modrinth upload failed (HTTP %d): %s", status, string(detail)))
	}
	fmt.Printf("modrinth: uploaded %s to %s\n", r.pVer, projectID)
}

// modrinthResolveProjectID resolves a Modrinth project slug or ID to its
// canonical base62 ID via the (slug-tolerant) project lookup endpoint.
func modrinthResolveProjectID(idOrSlug string) string {
	resp, err := http.Get(modrinthAPI + "/project/" + idOrSlug) //nolint:gosec
	if err != nil {
		cmd.Fail(fmt.Sprintf("Modrinth project lookup failed for '%s': %v", idOrSlug, err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		cmd.Fail(fmt.Sprintf("Modrinth project lookup failed for '%s' (HTTP %d): %s", idOrSlug, resp.StatusCode, string(detail)))
	}
	var proj struct {
		ID string `json:"id"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&proj); err != nil {
		cmd.Fail(fmt.Sprintf("parsing Modrinth project response for '%s': %v", idOrSlug, err))
	}
	if proj.ID == "" {
		cmd.Fail(fmt.Sprintf("Modrinth project lookup for '%s' returned no id", idOrSlug))
	}
	return proj.ID
}

func uploadCurseforge(r pubResolved, projectID, changelog, fileName string, data []byte, live bool) {
	fmt.Printf("curseforge: %s -> project %s | version %s | mc %s | loader %s | %d bytes\n",
		fileName, projectID, r.pVer, r.mcVer, r.loader, len(data))
	if !live {
		return
	}
	token := os.Getenv("CURSEFORGE_TOKEN")
	if token == "" {
		cmd.Fail("CURSEFORGE_TOKEN not set")
	}

	gameIDs, loaderIDs := cfResolveVersionIDs(token, r.mcVer, r.loader)

	// CurseForge rejects loader IDs on non-mod project types with errorCode
	// 1009 (invalid game version), so fall back to game-version IDs alone.
	variants := [][]int64{append(append([]int64{}, gameIDs...), loaderIDs...)}
	if len(loaderIDs) > 0 {
		variants = append(variants, gameIDs)
	}

	url := fmt.Sprintf("%s/projects/%s/upload-file", curseforgeAPI, projectID)
	for i, ids := range variants {
		meta, _ := json.Marshal(map[string]any{
			"changelog":     changelog,
			"changelogType": "markdown",
			"displayName":   r.displayName,
			"gameVersions":  ids,
			"releaseType":   r.releaseType,
		})
		contentType, body := buildMultipart([]mpart{
			{name: "metadata", contentType: "application/json", data: meta},
			{name: "file", fileName: fileName, contentType: "application/octet-stream", data: data},
		})

		status, detail := postWithRetry("curseforge", url, map[string]string{
			"X-Api-Token":  token,
			"Content-Type": contentType,
		}, body)
		if status >= 200 && status < 300 {
			fmt.Printf("curseforge: uploaded %s to %s\n", r.pVer, projectID)
			return
		}
		if cfIsInvalidGameVersionError(detail) && i+1 < len(variants) {
			fmt.Printf("curseforge: rejected game-version IDs %v (errorCode %d), retrying without loader IDs\n", ids, cfErrorCodeInvalidGameVersion)
			continue
		}
		cmd.Fail(fmt.Sprintf("curseforge upload failed (HTTP %d): %s", status, string(detail)))
	}
}

const (
	uploadMaxAttempts = 3
	uploadRetryDelay  = 2 * time.Second

	// cfErrorCodeInvalidGameVersion is returned by the CurseForge upload API
	// when a submitted game-version ID is invalid for the project type.
	cfErrorCodeInvalidGameVersion = 1009
)

// cfIsInvalidGameVersionError reports whether a CurseForge error response
// body carries errorCode 1009 (invalid game version ID).
func cfIsInvalidGameVersionError(body []byte) bool {
	var e struct {
		ErrorCode    int    `json:"errorCode"`
		ErrorMessage string `json:"errorMessage"`
	}
	return json.Unmarshal(body, &e) == nil && e.ErrorCode == cfErrorCodeInvalidGameVersion
}

// postWithRetry POSTs body to url, retrying transient failures (network
// errors, HTTP 429, HTTP 5xx) with doubling backoff. Non-transient responses
// are returned to the caller for interpretation; a network error on the final
// attempt is fatal.
func postWithRetry(label, url string, headers map[string]string, body []byte) (int, []byte) {
	delay := uploadRetryDelay
	for attempt := 1; ; attempt++ {
		status, detail, err := postOnce(url, headers, body)
		transient := err != nil || status == http.StatusTooManyRequests || status >= 500
		if !transient || attempt >= uploadMaxAttempts {
			if err != nil {
				cmd.Fail(fmt.Sprintf("%s upload failed: %v", label, err))
			}
			return status, detail
		}
		if err != nil {
			fmt.Printf("%s: attempt %d/%d failed (%v), retrying in %s\n", label, attempt, uploadMaxAttempts, err, delay)
		} else {
			fmt.Printf("%s: attempt %d/%d got HTTP %d, retrying in %s\n", label, attempt, uploadMaxAttempts, status, delay)
		}
		time.Sleep(delay)
		delay *= 2
	}
}

func postOnce(url string, headers map[string]string, body []byte) (int, []byte, error) {
	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return 0, nil, err
	}
	for k, v := range headers {
		req.Header.Set(k, v)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return 0, nil, err
	}
	defer resp.Body.Close()
	detail, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, detail, nil
}

// cfResolveVersionIDs resolves CurseForge numeric game-version IDs for the
// given Minecraft version and (optional) loader. Versions are filtered by
// their game-version *type* (slug prefix "minecraft" for game versions,
// "modloader" for loaders) so a loader name can never match a game version
// entry or vice versa.
func cfResolveVersionIDs(token, mcVer, loader string) (gameIDs, loaderIDs []int64) {
	var types []struct {
		ID   int64  `json:"id"`
		Slug string `json:"slug"`
	}
	cfGetJSON(token, "/game/version-types", &types)

	mcTypes := map[int64]bool{}
	loaderTypes := map[int64]bool{}
	for _, t := range types {
		switch {
		case strings.HasPrefix(t.Slug, "minecraft"):
			mcTypes[t.ID] = true
		case strings.HasPrefix(t.Slug, "modloader"):
			loaderTypes[t.ID] = true
		}
	}

	var versions []struct {
		ID                int64  `json:"id"`
		GameVersionTypeID int64  `json:"gameVersionTypeID"`
		Name              string `json:"name"`
		Slug              string `json:"slug"`
	}
	cfGetJSON(token, "/game/versions", &versions)

	for _, v := range versions {
		if mcTypes[v.GameVersionTypeID] && strings.EqualFold(v.Name, mcVer) {
			gameIDs = append(gameIDs, v.ID)
		}
		if loader != "" && loaderTypes[v.GameVersionTypeID] &&
			(strings.EqualFold(v.Name, loader) || strings.EqualFold(v.Slug, loader)) {
			loaderIDs = append(loaderIDs, v.ID)
		}
	}

	if len(gameIDs) == 0 {
		cmd.Fail(fmt.Sprintf("could not resolve a CF game-version ID for mc '%s'", mcVer))
	}
	if loader != "" && len(loaderIDs) == 0 {
		cmd.Fail(fmt.Sprintf("could not resolve a CF game-version ID for loader '%s'", loader))
	}
	return gameIDs, loaderIDs
}

func cfGetJSON(token, path string, target any) {
	req, err := http.NewRequest(http.MethodGet, curseforgeAPI+path, nil)
	if err != nil {
		cmd.Fail(fmt.Sprintf("CF %s lookup failed: %v", path, err))
	}
	req.Header.Set("X-Api-Token", token)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		cmd.Fail(fmt.Sprintf("CF %s lookup failed: %v", path, err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		cmd.Fail(fmt.Sprintf("CF %s lookup failed (HTTP %d): %s", path, resp.StatusCode, string(detail)))
	}
	if err := json.NewDecoder(resp.Body).Decode(target); err != nil {
		cmd.Fail(fmt.Sprintf("parsing CF %s response: %v", path, err))
	}
}

func pubVerify(manifestPath, variant string) {
	r := pubResolve(manifestPath, variant)
	if r.mrID == "" {
		cmd.Fail("verify currently checks Modrinth only, and this manifest has no modrinth_id")
	}
	url := fmt.Sprintf("%s/project/%s/version", modrinthAPI, r.mrID)
	resp, err := http.Get(url) //nolint:gosec
	if err != nil {
		cmd.Fail(fmt.Sprintf("Modrinth version lookup failed: %v", err))
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		detail, _ := io.ReadAll(resp.Body)
		cmd.Fail(fmt.Sprintf("Modrinth version lookup failed (HTTP %d): %s", resp.StatusCode, string(detail)))
	}
	var versions []map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&versions); err != nil {
		cmd.Fail(fmt.Sprintf("parsing Modrinth version list: %v", err))
	}
	for _, v := range versions {
		if vn, _ := v["version_number"].(string); vn == r.pVer {
			vid, _ := v["id"].(string)
			published, _ := v["date_published"].(string)
			fmt.Printf("verified: %s %s is live on Modrinth (version id %s, published %s)\n", r.displayName, r.pVer, vid, published)
			return
		}
	}
	cmd.Fail(fmt.Sprintf("version '%s' NOT found on Modrinth project '%s' (%d version(s) listed)", r.pVer, r.mrID, len(versions)))
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
			cmd.Fail(fmt.Sprintf("multipart build failed: %v", err))
		}
		if _, err := pw.Write(p.data); err != nil {
			cmd.Fail(fmt.Sprintf("multipart build failed: %v", err))
		}
	}
	w.Close()
	return w.FormDataContentType(), buf.Bytes()
}
