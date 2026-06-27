package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	// bump
	llBumpCmd.Flags().Bool("configs", false, "Also update in-pack version files and refresh")
	llBumpCmd.GroupID = GroupBuildExport
	rootCmd.AddCommand(llBumpCmd)

	// freeze
	llFreezeCmd.Flags().Bool("json", false, "Output frozen list as JSON")
	llFreezeCmd.GroupID = GroupPackManagement
	rootCmd.AddCommand(llFreezeCmd)

	// unfreeze
	llUnfreezeCmd.GroupID = GroupPackManagement
	rootCmd.AddCommand(llUnfreezeCmd)

	// side
	llSideCmd.GroupID = GroupPackManagement
	rootCmd.AddCommand(llSideCmd)

	// packs
	llPacksCmd.PersistentFlags().Bool("json", false, "Output as JSON")
	llPacksCmd.AddCommand(llPacksListCmd)
	llPacksCmd.AddCommand(llPacksGetCmd)
	llPacksCmd.AddCommand(llPacksSetCmd)
	llPacksCmd.AddCommand(llPacksIndexCmd)
	llPacksCmd.GroupID = GroupWorkspace
	rootCmd.AddCommand(llPacksCmd)

	// automation
	llAutomationCmd.AddCommand(llAutomationGetCmd)
	llAutomationCmd.GroupID = GroupOther
	rootCmd.AddCommand(llAutomationCmd)
}

// â€” bump â€”

var llBumpCmd = &cobra.Command{
	Use:   "bump <pack-dir> <new-version>",
	Short: "Bump the manifest version (--configs also updates in-pack version files)",
	Args:  cobra.RangeArgs(2, 99),
	Run: func(cmd *cobra.Command, args []string) {
		packDir := llAbs(args[0])
		newVer := args[1]
		doConfigs, _ := cmd.Flags().GetBool("configs")

		if newVer == "" {
			llFail("new version must not be empty")
		}

		llChdir()

		mfPath := filepath.Join(packDir, "manifest.json")
		m, err := manifest.Read(mfPath)
		if err != nil {
			llFail(fmt.Sprintf("failed to read %s: %v", mfPath, err))
		}
		old := m.Version
		m.Version = newVer
		if err := manifest.Write(mfPath, m); err != nil {
			llFail(fmt.Sprintf("failed to write %s: %v", mfPath, err))
		}
		fmt.Printf("bumped %s: %s -> %s\n", mfPath, old, newVer)

		if doConfigs {
			packName := m.Name
			if packName == "" {
				packName = m.ID
			}
			if packName == "" {
				packName = filepath.Base(packDir)
			}
			updatePackConfigs(packDir, packName, newVer)
		}
	},
}

func updatePackConfigs(packDir, packName, version string) {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		llFail(fmt.Sprintf("failed to read %s: %v", packDir, err))
	}

	var touched []string
	updates := 0
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		name := e.Name()
		if !strings.HasSuffix(name, "-mr") && !strings.HasSuffix(name, "-cf") {
			continue
		}
		cfgDir := filepath.Join(packDir, name, "config")
		n := 0
		if updateMenuCredits(filepath.Join(cfgDir, "isxander-main-menu-credits.json"), packName, version) {
			n++
		}
		if updateLoaderDeps(filepath.Join(cfgDir, "fabric_loader_dependencies.json"), packName, version) {
			n++
		}
		if n > 0 {
			touched = append(touched, filepath.Join(packDir, name))
			updates += n
		}
	}

	if len(touched) == 0 {
		fmt.Printf("no version-bearing configs found in any subdir of %s.\n", packDir)
		return
	}
	fmt.Printf("updated %d config file(s) across %d subdir(s)\n", updates, len(touched))

	if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
		fmt.Println("note: packwand not found via SelfBin; run 'packwand refresh' in each updated subdir to fix index hashes.")
		return
	}
	for _, dir := range touched {
		c := exec.Command(workspace.SelfBin(), "refresh")
		c.Dir = dir
		if out, err := c.CombinedOutput(); err != nil {
			llFail(fmt.Sprintf("packwand refresh failed in %s: %v\n%s", dir, err, workspace.Indent(string(out), "    ")))
		}
		fmt.Printf("  refreshed %s\n", dir)
	}
}

func updateMenuCredits(path, packName, version string) bool {
	obj, ok := loadJSONMap(path)
	if !ok {
		return false
	}
	mainMenu, ok := obj["main_menu"].(map[string]any)
	if !ok {
		return false
	}
	bottomRight, ok := mainMenu["bottom_right"].([]any)
	if !ok || len(bottomRight) == 0 {
		return false
	}
	first, ok := bottomRight[0].(map[string]any)
	if !ok {
		return false
	}
	first["text"] = packName + " " + version
	writeCompactJSON(path, obj)
	fmt.Printf("  %s -> %q\n", path, packName+" "+version)
	return true
}

func updateLoaderDeps(path, packName, version string) bool {
	obj, ok := loadJSONMap(path)
	if !ok {
		return false
	}
	overrides, ok := obj["overrides"].(map[string]any)
	if !ok {
		return false
	}
	minecraft, ok := overrides["minecraft"].(map[string]any)
	if !ok {
		return false
	}
	recommends, ok := minecraft["+recommends"].(map[string]any)
	if !ok {
		return false
	}
	recommends[packName] = ">" + version
	writeCompactJSON(path, obj)
	fmt.Printf("  %s -> %s: %q\n", path, packName, ">"+version)
	return true
}

func loadJSONMap(path string) (map[string]any, bool) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, false
	}
	var obj map[string]any
	if err := json.Unmarshal(data, &obj); err != nil {
		llWarn("invalid JSON in %s: %v; skipped", path, err)
		return nil, false
	}
	return obj, true
}

func writeCompactJSON(path string, v any) {
	var buf strings.Builder
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(v); err != nil {
		llFail(fmt.Sprintf("failed to marshal %s: %v", path, err))
	}
	if err := os.WriteFile(path, []byte(buf.String()), 0o644); err != nil {
		llFail(fmt.Sprintf("failed to write %s: %v", path, err))
	}
}

// â€” freeze / unfreeze â€”

var llFreezeCmd = &cobra.Command{
	Use:   "freeze <pack-subdir> [mod-slugs...]",
	Short: "Pin mods so updates skip them (no slugs: list what's frozen)",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		asJSON, _ := cmd.Flags().GetBool("json")
		subdir := llAbs(strings.TrimRight(args[0], "/"))
		slugs := args[1:]
		llChdir()
		packDir, subKey := splitPackSubdir(subdir)
		if len(slugs) == 0 {
			listFrozen(packDir, subKey, asJSON)
			return
		}
		applyFreeze(packDir, subKey, subdir, slugs, true)
	},
}

var llUnfreezeCmd = &cobra.Command{
	Use:   "unfreeze <pack-subdir> <mod-slugs...>",
	Short: "Unpin mods so updates can apply to them again",
	Args:  cobra.MinimumNArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		subdir := llAbs(strings.TrimRight(args[0], "/"))
		llChdir()
		packDir, subKey := splitPackSubdir(subdir)
		applyFreeze(packDir, subKey, subdir, args[1:], false)
	},
}

func splitPackSubdir(subdir string) (packDir, subKey string) {
	subKey = filepath.Base(subdir)
	packDir = filepath.Dir(subdir)
	if !strings.HasSuffix(subKey, "-mr") && !strings.HasSuffix(subKey, "-cf") {
		llFail(fmt.Sprintf("%q is not a pack subdir (expected a path ending in -mr or -cf)", subdir))
	}
	if _, err := os.Stat(filepath.Join(packDir, "manifest.json")); err != nil {
		llFail(fmt.Sprintf("no manifest.json in %s", packDir))
	}
	return packDir, subKey
}

func listFrozen(packDir, subKey string, asJSON bool) {
	frozen := manifest.ReadAutomation(packDir).Freeze[subKey]
	if asJSON {
		if frozen == nil {
			frozen = []string{}
		}
		sort.Strings(frozen)
		data, _ := json.MarshalIndent(frozen, "", "  ")
		fmt.Println(string(data))
		return
	}
	if len(frozen) == 0 {
		fmt.Printf("no frozen mods declared for %s/%s.\n", packDir, subKey)
		return
	}
	sort.Strings(frozen)
	fmt.Printf("%d frozen mod(s) in %s/%s:\n", len(frozen), packDir, subKey)
	for _, s := range frozen {
		fmt.Printf("  - %s\n", s)
	}
}

func applyFreeze(packDir, subKey, subdir string, slugs []string, freeze bool) {
	if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
		llFail("packwand binary not found; check PACKWIZ_BIN or PATH")
	}

	verb, gerund := "pin", "freezing"
	if !freeze {
		verb, gerund = "unpin", "unfreezing"
	}

	failures := 0
	var applied []string
	for _, slug := range slugs {
		if _, err := os.Stat(filepath.Join(subdir, "mods", slug+".pw.toml")); err != nil {
			llWarn("%s not found in %s (no mods/%s.pw.toml); skipped", slug, subdir, slug)
			continue
		}
		c := exec.Command(workspace.SelfBin(), verb, slug)
		c.Dir = subdir
		if out, err := c.CombinedOutput(); err != nil {
			fmt.Fprintf(os.Stderr, "  FAIL %s %s: %v\n%s", verb, slug, err, workspace.Indent(string(out), "    "))
			failures++
			continue
		}
		fmt.Printf("  %s %s: %s\n", gerund, slug, subdir)
		applied = append(applied, slug)
	}

	if failures > 0 {
		llFail(fmt.Sprintf("%d %s operation(s) failed; manifest NOT updated", failures, verb))
	}
	if len(applied) == 0 {
		fmt.Println("nothing changed.")
		return
	}

	current := map[string]bool{}
	for _, s := range manifest.ReadAutomation(packDir).Freeze[subKey] {
		current[s] = true
	}
	for _, s := range applied {
		if freeze {
			current[s] = true
		} else {
			delete(current, s)
		}
	}
	var list []string
	for s := range current {
		list = append(list, s)
	}
	sort.Strings(list)
	if err := manifest.SetAutomationFreeze(packDir, subKey, list); err != nil {
		llFail(err.Error())
	}

	state := "frozen (updates will skip them)"
	if !freeze {
		state = "unfrozen (updates apply again)"
	}
	fmt.Printf("%d mod(s) %s; recorded in %s/manifest.json (automation.freeze.%s)\n", len(applied), state, packDir, subKey)
}

func pinDrift(packDir string, freezeMap map[string][]string) []string {
	var drift []string
	for subKey, slugs := range freezeMap {
		for _, slug := range slugs {
			p := filepath.Join(packDir, subKey, "mods", slug+".pw.toml")
			data, err := os.ReadFile(p)
			if err != nil {
				continue
			}
			if !strings.Contains(string(data), "pin = true") && !strings.Contains(string(data), "pin=true") {
				drift = append(drift, p)
			}
		}
	}
	return drift
}

// â€” side â€”

var validSides = map[string]bool{"client": true, "server": true, "both": true, "either": true}

// contentFolders are scanned when looking for a mod file by slug.
var contentFolders = []string{"mods", "resourcepacks", "shaderpacks"}

// normalizeSide maps "either" to "both" for storage (they are semantically equivalent).
func normalizeSide(side string) string {
	if side == "either" {
		return "both"
	}
	return side
}

var llSideCmd = &cobra.Command{
	Use:   "side <pack-dir> <mod-slug> [client|server|both|either]",
	Short: "Check or fix a mod's side across all subdirs in a pack",
	Args:  cobra.RangeArgs(2, 3),
	Run: func(cmd *cobra.Command, args []string) {
		packDir := llAbs(args[0])
		slug := args[1]

		llChdir()

		if _, err := os.Stat(filepath.Join(packDir, "manifest.json")); err != nil {
			llFail(fmt.Sprintf("no manifest.json in %s", packDir))
		}
		if len(args) < 3 {
			showSides(packDir, slug)
			return
		}
		newSide := args[2]
		if !validSides[newSide] {
			llFail(fmt.Sprintf("invalid side %q (expected client, server, both, or either)", newSide))
		}
		setSides(packDir, slug, normalizeSide(newSide))
	},
}

type sideEntry struct{ side, folder, sub string }

func showSides(packDir, slug string) {
	var found []sideEntry
	for _, sub := range manifest.SubDirsOf(packDir) {
		for _, folder := range contentFolders {
			p := filepath.Join(sub, folder, slug+".pw.toml")
			data, err := os.ReadFile(p)
			if err != nil {
				continue
			}
			found = append(found, sideEntry{currentSide(string(data)), folder, sub})
		}
	}
	if len(found) == 0 {
		llFail(fmt.Sprintf("%s not found in any subdir of %s", slug, packDir))
	}

	sides := map[string]bool{}
	for _, e := range found {
		sides[e.side] = true
	}

	for _, e := range found {
		folderNote := ""
		if e.folder != "mods" {
			folderNote = " [" + e.folder + "]"
		}
		fmt.Printf("  %-8s  %s%s\n", e.side, e.sub, folderNote)
	}

	if len(sides) > 1 {
		var sideList []string
		for s := range sides {
			sideList = append(sideList, s)
		}
		llWarn("%s has inconsistent sides across subdirs: %s", slug, strings.Join(sideList, ", "))
	}
}

func setSides(packDir, slug, newSide string) {
	if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
		llFail("packwand binary not found; check PACKWIZ_BIN or PATH")
	}
	var touched []string
	for _, sub := range manifest.SubDirsOf(packDir) {
		for _, folder := range contentFolders {
			p := filepath.Join(sub, folder, slug+".pw.toml")
			data, err := os.ReadFile(p)
			if err != nil {
				continue
			}
			updated, old, changed := rewriteSide(string(data), newSide)
			if !changed {
				fmt.Printf("  ok (already %s): %s\n", newSide, sub)
				continue
			}
			if err := os.WriteFile(p, []byte(updated), 0o644); err != nil {
				llFail(fmt.Sprintf("failed to write %s: %v", p, err))
			}
			fmt.Printf("  %s -> %s: %s\n", old, newSide, sub)
			touched = append(touched, sub)
		}
	}
	if len(touched) == 0 {
		fmt.Printf("nothing to change for %s in %s.\n", slug, packDir)
		return
	}
	for _, sub := range touched {
		c := exec.Command(workspace.SelfBin(), "refresh")
		c.Dir = sub
		if out, err := c.CombinedOutput(); err != nil {
			llFail(fmt.Sprintf("packwand refresh failed in %s: %v\n%s", sub, err, workspace.Indent(string(out), "    ")))
		}
		fmt.Printf("  refreshed %s\n", sub)
	}
	fmt.Printf("%s is now %q in %d subdir(s).\n", slug, newSide, len(touched))
}

func currentSide(content string) string {
	for _, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[") {
			break
		}
		if k, v, ok := splitKV(line); ok && k == "side" {
			return v
		}
	}
	return "both"
}

func rewriteSide(content, newSide string) (updated, old string, changed bool) {
	old = currentSide(content)
	if old == newSide {
		return content, old, false
	}
	lines := strings.Split(content, "\n")
	inTop := true
	filenameIdx := -1
	for i, raw := range lines {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[") {
			inTop = false
		}
		if !inTop {
			break
		}
		if k, _, ok := splitKV(line); ok {
			if k == "side" {
				lines[i] = fmt.Sprintf("side = %q", newSide)
				return strings.Join(lines, "\n"), old, true
			}
			if k == "filename" {
				filenameIdx = i
			}
		}
	}
	insert := fmt.Sprintf("side = %q", newSide)
	if filenameIdx >= 0 {
		lines = append(lines[:filenameIdx+1], append([]string{insert}, lines[filenameIdx+1:]...)...)
	} else {
		lines = append([]string{insert}, lines...)
	}
	return strings.Join(lines, "\n"), old, true
}

// â€” packs â€”

type packRef struct {
	Category string
	Dir      string
	ID       string
	M        *manifest.Manifest
}

var llPacksCmd = &cobra.Command{
	Use:   "packs",
	Short: "Look up or edit any pack's manifest fields by id",
}

var llPacksListCmd = &cobra.Command{
	Use:   "list",
	Short: "List all registered packs",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		asJSON, _ := cmd.Flags().GetBool("json")
		packsList(asJSON)
	},
}

var llPacksGetCmd = &cobra.Command{
	Use:   "get <id> [field]",
	Short: "Print a pack's manifest (or a single field)",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		field := ""
		if len(args) > 1 {
			field = args[1]
		}
		packsGet(args[0], field)
	},
}

var llPacksSetCmd = &cobra.Command{
	Use:   "set <id> <field> <value>",
	Short: "Set a simple manifest field for a pack",
	Args:  cobra.ExactArgs(3),
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		packsSet(args[0], args[1], args[2])
	},
}

var llPacksIndexCmd = &cobra.Command{
	Use:   "index",
	Short: "Regenerate derived projects.json index files",
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		if _, err := writeProjectsIndex(); err != nil {
			llFail(fmt.Sprintf("index generation failed: %v", err))
		}
	},
}

func loadAllPacks() []packRef {
	var out []packRef
	for _, cat := range []string{"modpacks", "datapacks", "resourcepacks"} {
		entries, err := os.ReadDir(cat)
		if err != nil {
			continue
		}
		for _, e := range entries {
			if !e.IsDir() {
				continue
			}
			dir := filepath.Join(cat, e.Name())
			m, err := manifest.Read(filepath.Join(dir, "manifest.json"))
			if err != nil {
				llWarn("packs: %v; skipped", err)
				continue
			}
			id := m.ID
			if id == "" {
				id = e.Name()
			}
			out = append(out, packRef{Category: cat, Dir: dir, ID: id, M: m})
		}
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Category != out[j].Category {
			return out[i].Category < out[j].Category
		}
		return out[i].ID < out[j].ID
	})
	return out
}

func findPack(id string) packRef {
	packs := loadAllPacks()
	for _, p := range packs {
		if p.ID == id || p.Dir == id || filepath.Base(p.Dir) == id {
			return p
		}
	}
	var known []string
	for _, p := range packs {
		known = append(known, p.ID)
	}
	llFail(fmt.Sprintf("no pack %q (known: %s)", id, strings.Join(known, ", ")))
	return packRef{}
}

func packsList(asJSON bool) {
	packs := loadAllPacks()
	if len(packs) == 0 {
		llFail("no packs found â€” run packwand from the repo root")
	}
	if asJSON {
		type jsonEntry struct {
			ID       string            `json:"id"`
			Category string            `json:"category"`
			Dir      string            `json:"dir"`
			Manifest *manifest.Manifest `json:"manifest"`
		}
		out := make([]jsonEntry, len(packs))
		for i, p := range packs {
			out[i] = jsonEntry{ID: p.ID, Category: p.Category, Dir: p.Dir, Manifest: p.M}
		}
		data, _ := json.MarshalIndent(out, "", "  ")
		fmt.Println(string(data))
		return
	}
	idW, verW := 4, 7
	for _, p := range packs {
		if len(p.ID) > idW {
			idW = len(p.ID)
		}
		if len(p.M.Version) > verW {
			verW = len(p.M.Version)
		}
	}
	fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  platforms\n", idW, "id", "type", verW, "version", "loader", "role")
	for _, p := range packs {
		m := p.M
		loader := m.Loader
		if loader == "" {
			loader = "-"
		}
		role := m.Role.Label()
		var plats []string
		if m.ModrinthID != "" {
			plats = append(plats, "mr")
		}
		if m.CurseforgeID != "" {
			plats = append(plats, "cf")
		}
		platStr := strings.Join(plats, "+")
		if platStr == "" {
			platStr = "-"
		}
		typ := m.Type
		if len(m.Variants) > 0 {
			typ = fmt.Sprintf("%s(%dv)", typ, len(m.Variants))
		}
		fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  %s\n", idW, p.ID, typ, verW, m.Version, loader, role, platStr)
	}
	fmt.Printf("\n%d pack(s) registered\n", len(packs))
}

func packsGet(id, field string) {
	p := findPack(id)
	if field == "" {
		data, err := json.MarshalIndent(p.M, "", "  ")
		if err != nil {
			llFail(fmt.Sprintf("failed to render manifest: %v", err))
		}
		fmt.Println(string(data))
		return
	}
	data, _ := json.Marshal(p.M)
	var raw map[string]any
	json.Unmarshal(data, &raw) //nolint:errcheck
	val, ok := raw[field]
	if !ok {
		llFail(fmt.Sprintf("pack %q has no field %q", p.ID, field))
	}
	switch v := val.(type) {
	case string:
		fmt.Println(v)
	default:
		out, _ := json.MarshalIndent(v, "", "  ")
		fmt.Println(string(out))
	}
}

var settablePackFields = map[string]bool{
	"name": true, "version": true, "release_type": true, "description": true,
	"modrinth_id": true, "curseforge_id": true, "mc_version": true, "loader": true,
}

func packsSet(id, field, value string) {
	if !settablePackFields[field] {
		var allowed []string
		for f := range settablePackFields {
			allowed = append(allowed, f)
		}
		sort.Strings(allowed)
		llFail(fmt.Sprintf("field %q is not settable via packs set (allowed: %s)", field, strings.Join(allowed, ", ")))
	}
	p := findPack(id)
	m := p.M
	var old string
	switch field {
	case "name":
		old, m.Name = m.Name, value
	case "version":
		old, m.Version = m.Version, value
	case "release_type":
		old, m.ReleaseType = m.ReleaseType, value
	case "description":
		old, m.Description = m.Description, value
	case "modrinth_id":
		old, m.ModrinthID = m.ModrinthID, value
	case "curseforge_id":
		old, m.CurseforgeID = m.CurseforgeID, value
	case "mc_version":
		if m.MCVersion != nil {
			old = *m.MCVersion
		}
		m.MCVersion = &value
	case "loader":
		old, m.Loader = m.Loader, value
	}
	if err := manifest.Write(filepath.Join(p.Dir, "manifest.json"), m); err != nil {
		llFail(fmt.Sprintf("failed to write manifest: %v", err))
	}
	fmt.Printf("%s: %s: %q -> %q\n", p.ID, field, old, value)
	if field == "version" {
		fmt.Println("note: 'packwand bump' is the richer path for versions (supports --configs for in-pack version files)")
	}
}

// â€” automation â€”

var llAutomationCmd = &cobra.Command{
	Use:   "automation",
	Short: "Query effective automation settings for a pack",
}

var llAutomationGetCmd = &cobra.Command{
	Use:   "get <pack-dir>",
	Short: "Print the effective automation settings as JSON",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		packDir := llAbs(args[0])
		llChdir()
		a := manifest.ReadAutomation(packDir)
		data, err := json.MarshalIndent(a, "", "  ")
		if err != nil {
			llFail(fmt.Sprintf("failed to marshal automation: %v", err))
		}
		fmt.Println(string(data))
	},
}
