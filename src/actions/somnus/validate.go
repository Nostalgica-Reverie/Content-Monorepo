package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

func cmdValidate(args []string) {
	if len(args) == 0 {
		failUsage(verbUsage["validate"])
	}

	all := args[0] == "--all"
	var targets []string
	if all {
		targets = discoverManifestPaths()
		if len(targets) == 0 {
			fail("--all found no manifests (run from the repo root)")
		}
	} else {
		targets = args
		for i, t := range targets {
			targets[i] = absPath(t)
		}
	}

	for _, manifestPath := range targets {
		validateManifestFile(manifestPath)
	}

	if all {
		fmt.Printf("✓ all %d manifest(s) OK\n", len(targets))
	}
}

func discoverManifestPaths() []string {
	var found []string
	for _, root := range []string{modpacksDir(), "datapacks", "resourcepacks"} {
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
		fail(fmt.Sprintf("unknown manifest filename: %s", filename))
	}
	isExperimental := filename == "manifest-experimental.json"

	m, err := ReadManifest(manifestPath)
	if err != nil {
		fail(fmt.Sprintf("failed to parse %s: %v", manifestPath, err))
	}
	packDir := filepath.Dir(manifestPath)

	for _, field := range []struct{ name, value string }{
		{"id", m.ID},
		{"name", m.Name},
		{"type", m.Type},
		{"release_type", m.ReleaseType},
	} {
		if field.value == "" {
			fail(fmt.Sprintf("manifest missing required field: %s", field.name))
		}
	}
	if m.Role.IsZero() {
		fail("manifest missing required field: role")
	}

	if m.Type != "modpack" && m.Type != "datapack" && m.Type != "resourcepack" {
		fail(fmt.Sprintf("invalid 'type': %s", m.Type))
	}

	variants := m.Variants

	if m.Type == "modpack" && len(variants) == 0 && strings.TrimSpace(m.Loader) == "" {
		fail("modpack manifests must declare a 'loader'")
	}

	if m.Type == "modpack" && len(variants) > 0 {
		validateVariants(m, variants)
	}

	hasMcVersion := m.MCVersion != nil
	hasVariants := len(m.Variants) > 0
	if hasMcVersion && hasVariants {
		fail("manifest declares both 'mc_version' and 'variants' — use exactly one")
	}
	if !hasMcVersion && !hasVariants {
		fail("manifest must declare either 'mc_version' or 'variants'")
	}

	if !isExperimental && m.Version == "" {
		fail("manifest missing required field: version")
	}

	if m.ReleaseType != "release" && m.ReleaseType != "beta" && m.ReleaseType != "alpha" {
		fail(fmt.Sprintf("invalid 'release_type': %s", m.ReleaseType))
	}
	if isExperimental && m.ReleaseType != "alpha" {
		vwarn(fmt.Sprintf("experimental manifest uses release_type='%s'; convention is 'alpha'", m.ReleaseType))
	}

	hasMr := strings.TrimSpace(m.ModrinthID) != ""
	hasCf := strings.TrimSpace(m.CurseforgeID) != ""
	if !hasMr && !hasCf {
		fail("manifest must set at least one of modrinth_id or curseforge_id")
	}

	pb, roleLabel := validateRole(m, isExperimental)
	if pb != nil {
		validatePerformanceBase(m, pb, packDir)
	}

	if m.SharedAssets != "" {
		if m.SharedAssets == m.ID {
			fail(fmt.Sprintf("'shared_assets' cannot reference the pack itself ('%s')", m.ID))
		}
		if loadReferencedManifest(m.SharedAssets) == nil {
			fail(fmt.Sprintf("'shared_assets' references unknown pack '%s'", m.SharedAssets))
		}
	}

	if !isExperimental {
		validateChangelog(packDir)
	}

	if m.Type == "modpack" {
		validateModpackSubdirs(m, packDir, variants, hasMcVersion, hasMr, hasCf)
	} else {
		validateZipPackStructure(packDir, m.Type)
	}

	validateManifestAutomation(manifestPath, m.Automation, packDir)

	optOutPath := filepath.Join(packDir, "opt-out.json")
	if fileExists(optOutPath) {
		vwarn(fmt.Sprintf("%s: opt-out.json is deprecated — migrate into manifest.json \"automation\"", optOutPath))
	}
	if fileExists(filepath.Join(packDir, "auto-update-ignore.json")) {
		vwarn(fmt.Sprintf("%s: legacy auto-update-ignore.json — migrate to manifest.json \"automation\"", packDir))
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
	fmt.Printf("✓ %s %s (%s, %s, %s) [%s%s] — manifest OK\n",
		m.ID, version, m.ReleaseType, label, shape, roleStr, sharedStr)
}

func validateVariants(m *Manifest, variants []Variant) {
	byVersion := map[string][]Variant{}
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
				fail(fmt.Sprintf("variant for mc_version '%s' shares that version with another variant and must declare a distinct 'id' (e.g. '%s-fabric')", mc, mc))
			}
		}
		ids := make([]string, 0, len(list))
		for _, v := range list {
			ids = append(ids, v.ID)
		}
		if dupes := duplicateValues(ids); len(dupes) > 0 {
			fail(fmt.Sprintf("duplicate variant id(s) for mc_version '%s': %s", mc, strings.Join(dupes, ", ")))
		}
		loaders := make([]string, 0, len(list))
		for _, v := range list {
			if strings.TrimSpace(v.Loader) != "" {
				loaders = append(loaders, v.Loader)
			}
		}
		if dupes := duplicateValues(loaders); len(dupes) > 0 {
			fail(fmt.Sprintf("two variants share both mc_version '%s' and loader '%s' — give them distinct ids or loaders", mc, strings.Join(dupes, ", ")))
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
			fail(fmt.Sprintf("variant '%s' has no loader: set a variant 'loader' or a pack-level 'loader'", key))
		}
	}
}

func validateRole(m *Manifest, isExperimental bool) (*PerformanceBase, string) {
	if pb := m.Role.ConsumerBase(); pb != nil {
		return pb, ""
	}
	label := m.Role.Label()
	switch label {
	case "none", "base":
		if isExperimental && label == "base" {
			fail("experimental manifests cannot have role 'base' (bases must be stable)")
		}
		return nil, label
	default:
		fail(fmt.Sprintf("invalid 'role' string '%s' (expected 'none', 'base', or a performance_base object)", label))
		return nil, ""
	}
}

func validatePerformanceBase(m *Manifest, pb *PerformanceBase, packDir string) {
	if pb.Pack == "" || len(pb.Mappings) == 0 {
		fail("role.performance_base must have a 'pack' and a non-empty 'mappings' array")
	}
	if pb.Pack == m.ID {
		fail(fmt.Sprintf("performance_base.pack cannot reference the pack itself ('%s')", m.ID))
	}

	base := loadReferencedManifest(pb.Pack)
	if base == nil {
		fail(fmt.Sprintf("performance_base.pack references unknown pack '%s' (no manifest.json at %s/%s/)", pb.Pack, modpacksDir(), pb.Pack))
	}
	if !base.Role.IsBase() {
		fail(fmt.Sprintf("performance_base.pack references '%s', but that pack's role is not 'base'", pb.Pack))
	}

	basePackDir := filepath.Join(modpacksDir(), pb.Pack)
	for _, mp := range pb.Mappings {
		if mp.Source == "" || mp.Target == "" {
			fail("each performance_base mapping needs both 'source' and 'target'")
		}
		sSuffix := platformSuffix(mp.Source)
		tSuffix := platformSuffix(mp.Target)
		if sSuffix == "" {
			fail(fmt.Sprintf("mapping source '%s' must end in '-mr' or '-cf'", mp.Source))
		}
		if tSuffix == "" {
			fail(fmt.Sprintf("mapping target '%s' must end in '-mr' or '-cf'", mp.Target))
		}
		if sSuffix != tSuffix {
			fail(fmt.Sprintf("FORBIDDEN cross-platform mapping: source '%s' (%s) → target '%s' (%s). Modrinth and CurseForge substrates must never cross (license risk).", mp.Source, sSuffix, mp.Target, tSuffix))
		}
		if !dirExists(filepath.Join(basePackDir, mp.Source)) {
			fail(fmt.Sprintf("mapping source '%s' does not exist in base pack '%s'", mp.Source, pb.Pack))
		}
		if !dirExists(filepath.Join(packDir, mp.Target)) {
			fail(fmt.Sprintf("mapping target '%s' does not exist in this pack", mp.Target))
		}
	}
}

func validateChangelog(packDir string) {
	changelogPath := filepath.Join(packDir, "changelog.md")
	data, err := os.ReadFile(changelogPath)
	if err != nil {
		fail(fmt.Sprintf("changelog.md is missing at %s", changelogPath))
	}
	content := strings.TrimSpace(string(data))
	if content == "" {
		fail(fmt.Sprintf("changelog.md is empty at %s", changelogPath))
	}
	for line := range strings.SplitSeq(content, "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed != "" && !strings.HasPrefix(trimmed, "#") {
			return
		}
	}
	fail(fmt.Sprintf("changelog.md has headers but no content at %s", changelogPath))
}

func validateModpackSubdirs(m *Manifest, packDir string, variants []Variant, hasMcVersion, hasMr, hasCf bool) {
	if hasMcVersion {
		mc := *m.MCVersion
		mr := filepath.Join(packDir, mc+"-mr")
		cf := filepath.Join(packDir, mc+"-cf")
		if hasMr && !dirExists(mr) {
			fail(fmt.Sprintf("modrinth_id is set but %s does not exist", mr))
		}
		if hasCf && !dirExists(cf) {
			fail(fmt.Sprintf("curseforge_id is set but %s does not exist", cf))
		}
		if dirExists(mr) && !hasMr {
			vwarn(fmt.Sprintf("%s exists but modrinth_id is not set", mr))
		}
		if dirExists(cf) && !hasCf {
			vwarn(fmt.Sprintf("%s exists but curseforge_id is not set", cf))
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
			fail(fmt.Sprintf("variant %s: has neither %s nor %s — nothing to publish", key, filepath.Base(mr), filepath.Base(cf)))
		}
		if hasMr && !mrPresent {
			vwarn(fmt.Sprintf("variant %s: %s absent — this variant will NOT publish to Modrinth", key, mr))
		}
		if hasCf && !cfPresent {
			vwarn(fmt.Sprintf("variant %s: %s absent — this variant will NOT publish to CurseForge", key, cf))
		}
	}
}

func validateZipPackStructure(packDir, packType string) {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		fail(fmt.Sprintf("failed to read %s: %v", packDir, err))
	}
	var versionDirs []string
	for _, entry := range entries {
		if entry.IsDir() {
			versionDirs = append(versionDirs, entry.Name())
		}
	}
	base := filepath.Base(packDir)
	if len(versionDirs) == 0 {
		fail(fmt.Sprintf("%s '%s' has no version directory (expected %s/{version}/)", packType, base, base))
	}
	if len(versionDirs) > 1 {
		fail(fmt.Sprintf("%s '%s' must have exactly one version directory, found %d: %s", packType, base, len(versionDirs), strings.Join(versionDirs, ", ")))
	}
	versionDir := filepath.Join(packDir, versionDirs[0])
	if !fileExists(filepath.Join(versionDir, "pack.mcmeta")) {
		vwarn(fmt.Sprintf("%s version dir %s has no pack.mcmeta at its root (Minecraft requires it)", packType, versionDir))
	}
}

func validateManifestAutomation(manifestPath string, auto *Automation, packDir string) {
	if auto == nil {
		return
	}
	for sub := range auto.Freeze {
		if !dirExists(filepath.Join(packDir, sub)) {
			vwarn(fmt.Sprintf("%s: automation.freeze references subdir '%s' which does not exist", manifestPath, sub))
		}
	}
}

func loadReferencedManifest(packID string) *Manifest {
	m, err := ReadManifest(filepath.Join(modpacksDir(), packID, "manifest.json"))
	if err != nil {
		return nil
	}
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

func vwarn(msg string) {
	fmt.Fprintf(os.Stderr, "::warning::%s\n", msg)
}
