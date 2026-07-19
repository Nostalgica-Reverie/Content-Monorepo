package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"sync"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/clistyle"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/core"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	llValidateCmd.Flags().Bool("all", false, "Discover and validate every manifest under mods/, modpacks/, datapacks/, resourcepacks/")
	llValidateCmd.GroupID = GroupInfo
	rootCmd.AddCommand(llValidateCmd)

	llDoctorCmd.Flags().Bool("json", false, "Output as JSON")
	llDoctorCmd.GroupID = GroupInfo
	rootCmd.AddCommand(llDoctorCmd)

	llLintCmd.GroupID = GroupInfo
	rootCmd.AddCommand(llLintCmd)
}

// — validate —

var llValidateCmd = &cobra.Command{
	Use:     "validate [manifest.json...]",
	Short:   "Validate pack manifests — fields, subdirs, changelog, role, automation",
	Aliases: []string{"check-manifest"},
	Run: func(cmd *cobra.Command, args []string) {
		all, _ := cmd.Flags().GetBool("all")
		llChdir()

		var targets []string
		if all {
			targets = discoverManifestPaths()
			if len(targets) == 0 {
				llFail("--all found no manifests (run from the repo root)")
			}
		} else {
			if len(args) == 0 {
				llFail("provide manifest path(s) or use --all")
			}
			targets = args
		}

		if !all {
			for _, manifestPath := range targets {
				if err := validateManifestFileErr(manifestPath); err != nil {
					llFail(err.Error())
				}
			}
			return
		}

		// --all runs on every CI invocation: validate manifests in parallel via
		// the workspace scheduler, serialising within a pack subdir but fanning
		// out across packs. All failures are collected and reported, not just
		// the first.
		ctx := cmd.Context()
		sched := workspace.NewScheduler(core.MaxConcurrent())
		var mu sync.Mutex
		failures := make(map[string]string, len(targets))
		for _, manifestPath := range targets {
			if ctx != nil && ctx.Err() != nil {
				break
			}
			manifestPath := manifestPath
			sched.Submit(workspace.Task{
				Name:  manifestPath,
				Needs: []workspace.Resource{workspace.Resource("subdir:" + filepath.Dir(manifestPath))},
				Run: func() error {
					if err := validateManifestFileErr(manifestPath); err != nil {
						mu.Lock()
						failures[manifestPath] = err.Error()
						mu.Unlock()
					}
					return nil
				},
			})
		}
		sched.Close()
		sched.Wait()

		if len(failures) > 0 {
			paths := make([]string, 0, len(failures))
			for p := range failures {
				paths = append(paths, p)
			}
			sort.Strings(paths)
			for _, p := range paths {
				llErrFile(p, "%s", failures[p])
			}
			llFail(fmt.Sprintf("%d of %d manifest(s) failed validation", len(failures), len(targets)))
		}
		fmt.Printf(clistyle.IconOK+" all %d manifest(s) OK\n", len(targets))
	},
}

// validateFailure is the panic payload vFail uses to abort a single manifest's
// validation without terminating the process, so `validate --all` can keep
// going and report every broken manifest.
type validateFailure struct{ msg string }

func vFail(msg string) {
	panic(validateFailure{msg})
}

// validateManifestFileErr validates one manifest, converting vFail aborts into
// a returned error.
func validateManifestFileErr(manifestPath string) (err error) {
	defer func() {
		if r := recover(); r != nil {
			if f, ok := r.(validateFailure); ok {
				err = fmt.Errorf("%s", f.msg)
				return
			}
			panic(r)
		}
	}()
	validateManifestFile(manifestPath)
	return nil
}

func discoverManifestPaths() []string {
	var found []string
	for _, root := range []string{"mods", workspace.ModpacksDir(), "datapacks", "resourcepacks"} {
		entries, err := os.ReadDir(root)
		if err != nil {
			continue
		}
		for _, entry := range entries {
			if !entry.IsDir() {
				continue
			}
			for _, name := range []string{"manifest.json", "manifest-experimental.json"} {
				p := filepath.Join(root, entry.Name(), name)
				if info, err := os.Stat(p); err == nil && !info.IsDir() {
					found = append(found, p)
				}
			}
		}
	}
	return found
}

func validateManifestFile(manifestPath string) {
	filename := filepath.Base(manifestPath)
	if filename != "manifest.json" && filename != "manifest-experimental.json" {
		vFail(fmt.Sprintf("unknown manifest filename: %s", filename))
	}
	isExperimental := filename == "manifest-experimental.json"

	m, err := manifest.Read(manifestPath)
	if err != nil {
		vFail(fmt.Sprintf("failed to parse %s: %v", manifestPath, err))
	}
	packDir := filepath.Dir(manifestPath)

	for _, field := range []struct{ name, value string }{
		{"id", m.ID},
		{"name", m.Name},
		{"type", m.Type},
		{"release_type", m.ReleaseType},
	} {
		if field.value == "" {
			vFail(fmt.Sprintf("manifest missing required field: %s", field.name))
		}
	}
	if m.Role.IsZero() {
		vFail("manifest missing required field: role")
	}

	if m.Type != "mod" && m.Type != "modpack" && m.Type != "datapack" && m.Type != "resourcepack" {
		vFail(fmt.Sprintf("invalid 'type': %s", m.Type))
	}

	variants := m.Variants

	if m.Type == "modpack" && len(variants) == 0 && strings.TrimSpace(m.Loader) == "" {
		vFail("modpack manifests must declare a 'loader'")
	}

	if (m.Type == "mod" || m.Type == "modpack") && len(variants) > 0 {
		validateVariants(m, variants)
	}
	if m.Type == "mod" && len(variants) == 0 {
		vFail("mod manifests must declare at least one Stonecutter variant")
	}

	hasMcVersion := m.MCVersion != nil
	hasVariants := len(m.Variants) > 0
	if hasMcVersion && hasVariants {
		vFail("manifest declares both 'mc_version' and 'variants' — use exactly one")
	}
	if !hasMcVersion && !hasVariants {
		vFail("manifest must declare either 'mc_version' or 'variants'")
	}

	if !isExperimental && m.Version == "" {
		vFail("manifest missing required field: version")
	}

	if m.ReleaseType != "release" && m.ReleaseType != "beta" && m.ReleaseType != "alpha" {
		vFail(fmt.Sprintf("invalid 'release_type': %s", m.ReleaseType))
	}
	if isExperimental && m.ReleaseType != "alpha" {
		llWarn("experimental manifest uses release_type='%s'; convention is 'alpha'", m.ReleaseType)
	}

	hasMr := strings.TrimSpace(m.ModrinthID) != ""
	hasCf := strings.TrimSpace(m.CurseforgeID) != ""
	hasGH := strings.TrimSpace(m.GitHubID) != ""
	hasGitea := strings.TrimSpace(m.GiteaID) != ""
	hasGL := strings.TrimSpace(m.GitLabID) != ""
	if !hasMr && !hasCf && !hasGH && !hasGitea && !hasGL {
		vFail("manifest must set at least one platform id (modrinth_id, curseforge_id, github_id, gitea_id, or gitlab_id)")
	}

	validLifecycles := map[string]bool{"": true, "active": true, "maintenance": true, "archived": true, "eol": true}
	if !validLifecycles[m.Lifecycle] {
		vFail(fmt.Sprintf("invalid 'lifecycle': %q (valid: active, maintenance, archived, eol)", m.Lifecycle))
	}
	if m.Lifecycle == "archived" || m.Lifecycle == "eol" {
		llWarn("%s is lifecycle=%s; excluded from workspace auto-update", m.ID, m.Lifecycle)
	}

	pb, roleLabel := validateRole(m, isExperimental)
	if m.Type == "mod" && m.Role.Label() != "none" {
		vFail("mod manifests must use role 'none'; performance-base relationships apply only to modpacks")
	}
	if pb != nil {
		validatePerformanceBase(m, pb, packDir)
	}

	if m.SharedAssets != "" {
		if m.SharedAssets == m.ID {
			vFail(fmt.Sprintf("'shared_assets' cannot reference the pack itself ('%s')", m.ID))
		}
		if loadReferencedManifest(m.SharedAssets) == nil {
			vFail(fmt.Sprintf("'shared_assets' references unknown pack '%s'", m.SharedAssets))
		}
	}

	if !isExperimental {
		validateChangelog(packDir)
	}

	if m.Type == "modpack" {
		validateModpackSubdirs(m, packDir, variants, hasMcVersion, hasMr, hasCf, hasGH || hasGitea || hasGL)
	} else if m.Type == "mod" {
		validateModStructure(packDir, variants)
	} else {
		validateZipPackStructure(packDir, m.Type)
	}

	validateManifestAutomation(manifestPath, m, packDir)

	optOutPath := filepath.Join(packDir, "opt-out.json")
	if fileExists(optOutPath) {
		llWarn("%s: opt-out.json is deprecated — migrate into manifest.json \"automation\"", optOutPath)
	}
	if fileExists(filepath.Join(packDir, "auto-update-ignore.json")) {
		llWarn("%s: legacy auto-update-ignore.json — migrate to manifest.json \"automation\"", packDir)
	}

	label := "production"
	if isExperimental {
		label = "EXPERIMENTAL"
	}
	version := m.Version
	if version == "" {
		version = "(generated)"
	}
	shape := "single-version"
	if hasVariants {
		shape = fmt.Sprintf("multi-variant (%d)", len(variants))
	}
	roleStr := roleLabel
	if pb != nil {
		roleStr = fmt.Sprintf("consumes %s (%d mappings)", pb.Pack, len(pb.Mappings))
	}
	sharedStr := ""
	if m.SharedAssets != "" {
		sharedStr = ", assets from " + m.SharedAssets
	}
	fmt.Printf(clistyle.IconOK+" %s %s (%s, %s, %s) [%s%s] — manifest OK\n",
		m.ID, version, m.ReleaseType, label, shape, roleStr, sharedStr)
}

func validateVariants(m *manifest.Manifest, variants []manifest.Variant) {
	byVersion := map[string][]manifest.Variant{}
	order := []string{}
	for _, v := range variants {
		if _, ok := byVersion[v.MCVersion]; !ok {
			order = append(order, v.MCVersion)
		}
		byVersion[v.MCVersion] = append(byVersion[v.MCVersion], v)
	}

	for _, mc := range order {
		list := byVersion[mc]
		if len(list) <= 1 {
			continue
		}
		for _, v := range list {
			if strings.TrimSpace(v.ID) == "" {
				vFail(fmt.Sprintf("variant for mc_version '%s' shares that version with another variant and must declare a distinct 'id'", mc))
			}
		}
		var ids, loaders []string
		for _, v := range list {
			ids = append(ids, v.ID)
			if strings.TrimSpace(v.Loader) != "" {
				loaders = append(loaders, v.Loader)
			}
		}
		if dupes := duplicateValues(ids); len(dupes) > 0 {
			vFail(fmt.Sprintf("duplicate variant id(s) for mc_version '%s': %s", mc, strings.Join(dupes, ", ")))
		}
		if dupes := duplicateValues(loaders); len(dupes) > 0 {
			vFail(fmt.Sprintf("two variants share both mc_version '%s' and loader '%s'", mc, strings.Join(dupes, ", ")))
		}
	}

	for _, v := range variants {
		resolvedLoader := v.Loader
		if resolvedLoader == "" {
			resolvedLoader = m.Loader
		}
		if strings.TrimSpace(resolvedLoader) == "" {
			key := v.ID
			if key == "" {
				key = v.MCVersion
			}
			vFail(fmt.Sprintf("variant '%s' has no loader: set a variant 'loader' or a pack-level 'loader'", key))
		}
	}
}

func validateRole(m *manifest.Manifest, isExperimental bool) (*manifest.PerformanceBase, string) {
	if pb := m.Role.ConsumerBase(); pb != nil {
		return pb, ""
	}
	label := m.Role.Label()
	switch label {
	case "none", "base":
		if isExperimental && label == "base" {
			vFail("experimental manifests cannot have role 'base' (bases must be stable)")
		}
		return nil, label
	default:
		vFail(fmt.Sprintf("invalid 'role' string '%s' (expected 'none', 'base', or a performance_base object)", label))
		return nil, ""
	}
}

func validatePerformanceBase(m *manifest.Manifest, pb *manifest.PerformanceBase, packDir string) {
	if pb.Pack == "" || len(pb.Mappings) == 0 {
		vFail("role.performance_base must have a 'pack' and a non-empty 'mappings' array")
	}
	if pb.Pack == m.ID {
		vFail(fmt.Sprintf("performance_base.pack cannot reference the pack itself ('%s')", m.ID))
	}

	base := loadReferencedManifest(pb.Pack)
	if base == nil {
		vFail(fmt.Sprintf("performance_base.pack references unknown pack '%s'", pb.Pack))
	}
	if !base.Role.IsBase() {
		vFail(fmt.Sprintf("performance_base.pack references '%s', but that pack's role is not 'base'", pb.Pack))
	}

	basePackDir := filepath.Join(workspace.ModpacksDir(), pb.Pack)
	for _, mp := range pb.Mappings {
		if mp.Source == "" || mp.Target == "" {
			vFail("each performance_base mapping needs both 'source' and 'target'")
		}
		sSuffix := llPlatformSuffix(mp.Source)
		tSuffix := llPlatformSuffix(mp.Target)
		if sSuffix == "" {
			vFail(fmt.Sprintf("mapping source '%s' must end in '-mr' or '-cf'", mp.Source))
		}
		if tSuffix == "" {
			vFail(fmt.Sprintf("mapping target '%s' must end in '-mr' or '-cf'", mp.Target))
		}
		if sSuffix != tSuffix {
			vFail(fmt.Sprintf("FORBIDDEN cross-platform mapping: source '%s' (%s) → target '%s' (%s). MR/CF must never cross (license risk).", mp.Source, sSuffix, mp.Target, tSuffix))
		}
		if !dirExists(filepath.Join(basePackDir, mp.Source)) {
			vFail(fmt.Sprintf("mapping source '%s' does not exist in base pack '%s'", mp.Source, pb.Pack))
		}
		if !dirExists(filepath.Join(packDir, mp.Target)) {
			vFail(fmt.Sprintf("mapping target '%s' does not exist in this pack", mp.Target))
		}
	}
}

func validateChangelog(packDir string) {
	changelogPath := filepath.Join(packDir, "changelog.md")
	data, err := os.ReadFile(changelogPath)
	if err != nil {
		vFail(fmt.Sprintf("changelog.md is missing at %s", changelogPath))
	}
	content := strings.TrimSpace(string(data))
	if content == "" {
		vFail(fmt.Sprintf("changelog.md is empty at %s", changelogPath))
	}
	for _, line := range strings.Split(content, "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed != "" && !strings.HasPrefix(trimmed, "#") {
			return
		}
	}
	vFail(fmt.Sprintf("changelog.md has headers but no content at %s", changelogPath))
}

func validateModpackSubdirs(m *manifest.Manifest, packDir string, variants []manifest.Variant, hasMcVersion, hasMr, hasCf, hasForge bool) {
	if !hasMr && !hasCf && hasForge {
		return
	}

	if hasMcVersion {
		mc := *m.MCVersion
		mr := filepath.Join(packDir, mc+"-mr")
		cf := filepath.Join(packDir, mc+"-cf")
		if hasMr && !dirExists(mr) {
			vFail(fmt.Sprintf("modrinth_id is set but %s does not exist", mr))
		}
		if hasCf && !dirExists(cf) {
			vFail(fmt.Sprintf("curseforge_id is set but %s does not exist", cf))
		}
		if dirExists(mr) && !hasMr {
			llWarn("%s exists but modrinth_id is not set", mr)
		}
		if dirExists(cf) && !hasCf {
			llWarn("%s exists but curseforge_id is not set", cf)
		}
		return
	}

	for _, v := range variants {
		key := v.ID
		if key == "" {
			key = v.MCVersion
		}
		mr := filepath.Join(packDir, key+"-mr")
		cf := filepath.Join(packDir, key+"-cf")
		mrPresent := dirExists(mr)
		cfPresent := dirExists(cf)
		if hasMr && !mrPresent && !cfPresent {
			vFail(fmt.Sprintf("variant %s: has neither %s nor %s", key, filepath.Base(mr), filepath.Base(cf)))
		}
		if hasMr && !mrPresent {
			llWarn("variant %s: %s absent — this variant will NOT publish to Modrinth", key, mr)
		}
		if hasCf && !cfPresent {
			llWarn("variant %s: %s absent — this variant will NOT publish to CurseForge", key, cf)
		}
	}
}

var gradleProjectRe = regexp.MustCompile(`^[A-Za-z0-9._-]+$`)

func validateModStructure(packDir string, variants []manifest.Variant) {
	for _, file := range []string{"settings.gradle", "gradlew", "gradlew.bat"} {
		if !fileExists(filepath.Join(packDir, file)) {
			vFail(fmt.Sprintf("mod source is missing %s", filepath.Join(packDir, file)))
		}
	}

	seen := map[string]bool{}
	for _, variant := range variants {
		key := variant.ID
		if key == "" {
			key = variant.MCVersion
		}
		project := strings.TrimSpace(variant.GradleProject)
		if project == "" {
			vFail(fmt.Sprintf("mod variant '%s' is missing gradle_project", key))
		}
		if !gradleProjectRe.MatchString(project) {
			vFail(fmt.Sprintf("mod variant '%s' has invalid gradle_project %q", key, project))
		}
		if seen[project] {
			vFail(fmt.Sprintf("multiple mod variants use gradle_project %q", project))
		}
		seen[project] = true
		projectDir := filepath.Join(packDir, "versions", project)
		if !dirExists(projectDir) {
			vFail(fmt.Sprintf("mod variant '%s' references missing Stonecutter project directory %s", key, projectDir))
		}
	}
}

func validateZipPackStructure(packDir, packType string) {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		vFail(fmt.Sprintf("failed to read %s: %v", packDir, err))
	}
	var versionDirs []string
	for _, entry := range entries {
		if entry.IsDir() {
			versionDirs = append(versionDirs, entry.Name())
		}
	}
	base := filepath.Base(packDir)
	if len(versionDirs) == 0 {
		vFail(fmt.Sprintf("%s '%s' has no version directory", packType, base))
	}
	if len(versionDirs) > 1 {
		vFail(fmt.Sprintf("%s '%s' must have exactly one version directory, found %d: %s", packType, base, len(versionDirs), strings.Join(versionDirs, ", ")))
	}
	versionDir := filepath.Join(packDir, versionDirs[0])
	if !fileExists(filepath.Join(versionDir, "pack.mcmeta")) {
		llWarn("%s version dir %s has no pack.mcmeta at its root", packType, versionDir)
	}
}

var calVerRe = regexp.MustCompile(`^\d{2}\.\d{2}(\.\d+)?$`)

func validateManifestAutomation(manifestPath string, m *manifest.Manifest, packDir string) {
	auto := m.Automation
	if auto == nil {
		return
	}
	if m.Type == "mod" && (auto.SyncOnBuild || len(auto.SyncVariants) > 0) {
		vFail(fmt.Sprintf("%s: mods cannot configure automation.sync_on_build or automation.sync_variants", manifestPath))
	}
	for sub := range auto.Freeze {
		if !dirExists(filepath.Join(packDir, sub)) {
			llWarn("%s: automation.freeze references subdir '%s' which does not exist", manifestPath, sub)
		}
	}
	if auto.FullAuto != nil && auto.FullAuto.Enabled {
		if m.Lifecycle == "archived" || m.Lifecycle == "eol" {
			vFail(fmt.Sprintf("%s: automation.full_auto.enabled is true but lifecycle is %q — full automation cannot run on an archived/eol pack", manifestPath, m.Lifecycle))
		}
		if !calVerRe.MatchString(m.Version) {
			vFail(fmt.Sprintf("%s: automation.full_auto.enabled requires a CalVer 'version' (e.g. '26.06' or '26.06.1'), got %q", manifestPath, m.Version))
		}
	}
}

// referencedManifests memoizes loadReferencedManifest: a shared base pack's
// manifest is referenced by many consumers in a single `validate --all` run,
// and re-parsing it per reference is wasted I/O. Goroutine-safe.
var referencedManifests sync.Map // packID -> *manifest.Manifest (nil-able)

func loadReferencedManifest(packID string) *manifest.Manifest {
	if cached, ok := referencedManifests.Load(packID); ok {
		m, _ := cached.(*manifest.Manifest)
		return m
	}
	m, err := manifest.Read(filepath.Join(workspace.ModpacksDir(), packID, "manifest.json"))
	if err != nil {
		m = nil
	}
	referencedManifests.Store(packID, m)
	return m
}

func duplicateValues(values []string) []string {
	counts := map[string]int{}
	for _, v := range values {
		counts[v]++
	}
	var dupes []string
	seen := map[string]bool{}
	for _, v := range values {
		if counts[v] > 1 && !seen[v] {
			seen[v] = true
			dupes = append(dupes, v)
		}
	}
	return dupes
}

func dirExists(path string) bool {
	info, err := os.Stat(path)
	return err == nil && info.IsDir()
}

func fileExists(path string) bool {
	info, err := os.Stat(path)
	return err == nil && !info.IsDir()
}
func checkPackFormat(subDir string, cc *catCheck, warnings *int) {
	packToml, err := os.ReadFile(filepath.Join(subDir, "pack.toml"))
	if err != nil {
		return // subdir may not be a packwand pack
	}
	content := string(packToml)
	if strings.Contains(content, "pack-format") && !strings.Contains(content, "packwand:") {
		cc.Warnings = append(cc.Warnings, fmt.Sprintf("pack-format is not packwand: in %s/pack.toml (run: packwand workspace migrate format)", subDir))
		(*warnings)++
	}
	indexToml, err := os.ReadFile(filepath.Join(subDir, "index.toml"))
	if err != nil {
		return
	}
	if strings.Contains(string(indexToml), "hash-format") && strings.Contains(string(indexToml), "sha256") {
		cc.Warnings = append(cc.Warnings, fmt.Sprintf("index uses sha256 in %s (run: packwand refresh to upgrade to sha512)", subDir))
		(*warnings)++
	}
}

// — doctor —

type DoctorResult struct {
	Version  string      `json:"version"`
	Tools    []toolCheck `json:"tools"`
	Repo     string      `json:"repo,omitempty"`
	Projects []catCheck  `json:"projects"`
	Problems int         `json:"problems"`
	Warnings int         `json:"warnings"`
	Healthy  bool        `json:"healthy"`
}

type toolCheck struct {
	Name   string `json:"name"`
	Status string `json:"status"`
	Path   string `json:"path,omitempty"`
	Note   string `json:"note,omitempty"`
}

type catCheck struct {
	Category string   `json:"category"`
	Count    int      `json:"count"`
	Errors   []string `json:"errors,omitempty"`
	Warnings []string `json:"warnings,omitempty"`
}

var llDoctorCmd = &cobra.Command{
	Use:     "doctor",
	Short:   "Check that tools, repo root, and manifests are all healthy",
	Aliases: []string{"check"},
	Run: func(cmd *cobra.Command, args []string) {
		asJSON, _ := cmd.Flags().GetBool("json")

		report := DoctorResult{Version: "packwand"}
		problems, warnings := 0, 0

		addTool := func(name, why string, required bool) {
			if p, err := exec.LookPath(name); err == nil {
				report.Tools = append(report.Tools, toolCheck{Name: name, Status: "ok", Path: p})
			} else if required {
				report.Tools = append(report.Tools, toolCheck{Name: name, Status: "missing", Note: why})
				problems++
			} else {
				report.Tools = append(report.Tools, toolCheck{Name: name, Status: "warn", Note: "optional: " + why})
				warnings++
			}
		}
		addTool("git", "change detection, changelogs, sync anchoring", true)
		addTool(workspace.SelfBin(), "every pack operation", true)
		addTool("java", "only needed for 'packwand test'", false)
		addTool("zip", "datapack/resourcepack builds via the publisher", false)
		addTool("packsquash", "optimized resource pack builds (plain zip used when absent)", false)

		root := workspace.FindRepoRoot()
		if root == "" {
			report.Problems = problems + 1
			report.Healthy = false
			if asJSON {
				data, _ := json.MarshalIndent(report, "", "  ")
				fmt.Println(string(data))
				return
			}
			fmt.Printf("packwand doctor\n\n")
			for _, t := range report.Tools {
				printToolLine(t)
			}
			fmt.Println("  MISS  repo      no .git or modpacks/ found walking up from here")
			llFail("doctor found problems — run packwand from inside the monorepo")
		}
		report.Repo = root

		if err := os.Chdir(root); err != nil {
			llFail(fmt.Sprintf("failed to chdir to repo root: %v", err))
		}

		total, broken := 0, 0
		for _, cat := range []string{"mods", "modpacks", "datapacks", "resourcepacks"} {
			dir := filepath.Join(root, cat)
			entries, err := os.ReadDir(dir)
			if err != nil {
				continue
			}
			cc := catCheck{Category: cat}
			for _, e := range entries {
				if !e.IsDir() {
					continue
				}
				mf := filepath.Join(dir, e.Name(), "manifest.json")
				if _, err := os.Stat(mf); err != nil {
					continue
				}
				cc.Count++
				packPath := filepath.Join(dir, e.Name())
				if _, err := manifest.Read(mf); err != nil {
					cc.Errors = append(cc.Errors, fmt.Sprintf("unparsable manifest %s: %v", mf, err))
					broken++
				}
				if _, err := os.Stat(filepath.Join(packPath, "opt-out.json")); err == nil {
					cc.Warnings = append(cc.Warnings, fmt.Sprintf("legacy opt-out.json in %s", packPath))
					warnings++
				}
				if _, err := os.Stat(filepath.Join(packPath, "auto-update-ignore.json")); err == nil {
					cc.Warnings = append(cc.Warnings, fmt.Sprintf("legacy auto-update-ignore.json in %s", packPath))
					warnings++
				}
				if frozen := manifest.ReadAutomation(packPath).Freeze; len(frozen) > 0 {
					for _, p := range pinDrift(packPath, frozen) {
						cc.Warnings = append(cc.Warnings, fmt.Sprintf("freeze drift: %s declared frozen but not pinned", p))
						warnings++
					}
				}
				// lifecycle check
				if m, rErr := manifest.Read(mf); rErr == nil {
					lc := m.Lifecycle
					validLC := map[string]bool{"": true, "active": true, "maintenance": true, "archived": true, "eol": true}
					if !validLC[lc] {
						cc.Errors = append(cc.Errors, fmt.Sprintf("invalid lifecycle %q in %s", lc, mf))
						broken++
					}
					// pack-format check across subdirs
					for _, sub := range manifest.SubDirsOf(packPath) {
						checkPackFormat(sub, &cc, &warnings)
					}
				}
				if subs, err := os.ReadDir(packPath); err == nil {
					for _, s := range subs {
						if s.IsDir() {
							if _, err := os.Stat(filepath.Join(packPath, s.Name(), "sync-exclude.json")); err == nil {
								cc.Warnings = append(cc.Warnings, fmt.Sprintf("legacy sync-exclude.json in %s", filepath.Join(packPath, s.Name())))
								warnings++
							}
						}
					}
				}
			}
			total += cc.Count
			if cc.Count > 0 || len(cc.Errors) > 0 || len(cc.Warnings) > 0 {
				report.Projects = append(report.Projects, cc)
			}
		}

		report.Problems = problems + broken
		report.Warnings = warnings
		report.Healthy = report.Problems == 0

		if asJSON {
			data, _ := json.MarshalIndent(report, "", "  ")
			fmt.Println(string(data))
			return
		}

		fmt.Printf("packwand doctor\n\n")
		for _, t := range report.Tools {
			printToolLine(t)
		}
		fmt.Printf("  ok    repo      %s\n", root)
		for _, cc := range report.Projects {
			fmt.Printf("  ok    %-9s %d manifest(s)\n", cc.Category, cc.Count)
			for _, e := range cc.Errors {
				fmt.Printf("  BAD   manifest  %s\n", e)
			}
			for _, w := range cc.Warnings {
				fmt.Printf("  warn  legacy    %s\n", w)
			}
		}

		fmt.Printf("\n%d project manifest(s) found, %d unparsable\n", total, broken)
		if report.Problems > 0 {
			llFail(fmt.Sprintf("doctor found %d problem(s)", report.Problems))
		}
		fmt.Println("environment looks healthy.")
	},
}

func printToolLine(t toolCheck) {
	switch t.Status {
	case "ok":
		fmt.Printf("  ok    %-9s %s\n", t.Name, t.Path)
	case "missing":
		fmt.Printf("  MISS  %-9s required: %s\n", t.Name, t.Note)
	case "warn":
		fmt.Printf("  warn  %-9s %s\n", t.Name, t.Note)
	}
}

// — lint —

var llLintCmd = &cobra.Command{
	Use:   "lint [files...]",
	Short: "Check JSON and .pw.toml files for syntax errors (no args: lints git-changed files)",
	Run: func(cmd *cobra.Command, args []string) {
		var files []string
		if len(args) > 0 {
			files = make([]string, len(args))
			for i, f := range args {
				files[i] = llAbs(f)
			}
		} else {
			files = workspace.GitChangedFiles()
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
		failed, checked := workspace.RunLintFiles(lintable)
		if failed > 0 {
			llFail(fmt.Sprintf("%d of %d file(s) failed syntax linting", failed, checked))
		}
		fmt.Printf(clistyle.IconOK+" all %d file(s) parsed OK\n", checked)
	},
}
