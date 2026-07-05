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

// — bump —

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
		runBump(packDir, newVer, doConfigs)
	},
}

// — transactional bump: plan / apply / rollback —

// bumpEdit is one planned file write, computed before anything touches disk.
type bumpEdit struct {
	path    string
	newData []byte
	label   string
}

// bumpTxn snapshots files before they are modified so a failed apply can
// restore every touched file, including index/pack files rewritten by refresh.
type bumpTxn struct {
	order     []string
	snapshots map[string][]byte // nil value = file did not exist before apply
}

func newBumpTxn() *bumpTxn { return &bumpTxn{snapshots: map[string][]byte{}} }

func (t *bumpTxn) snapshot(path string) {
	if _, ok := t.snapshots[path]; ok {
		return
	}
	data, err := os.ReadFile(path)
	if err != nil {
		data = nil
	}
	t.snapshots[path] = data
	t.order = append(t.order, path)
}

func (t *bumpTxn) rollback() {
	for i := len(t.order) - 1; i >= 0; i-- {
		path := t.order[i]
		data := t.snapshots[path]
		if data == nil {
			os.Remove(path)
			continue
		}
		if err := os.WriteFile(path, data, 0o644); err != nil {
			llWarn("rollback: failed to restore %s: %v", path, err)
		}
	}
}

func runBump(packDir, newVer string, doConfigs bool) {
	mfPath := filepath.Join(packDir, "manifest.json")
	m, err := manifest.Read(mfPath)
	if err != nil {
		llFail(fmt.Sprintf("failed to read %s: %v", mfPath, err))
	}
	old := m.Version
	m.Version = newVer

	packName := m.Name
	if packName == "" {
		packName = m.ID
	}
	if packName == "" {
		packName = filepath.Base(packDir)
	}

	// Plan phase: compute every edit up front; nothing is written yet.
	var edits []bumpEdit
	var refreshDirs []string
	if doConfigs {
		edits, refreshDirs = planPackConfigEdits(packDir, packName, newVer)
	}

	fmt.Printf("plan: %s version %s -> %s\n", mfPath, old, newVer)
	for _, e := range edits {
		fmt.Printf("plan: %s\n", e.label)
	}
	for _, dir := range refreshDirs {
		fmt.Printf("plan: refresh %s\n", dir)
	}

	if len(refreshDirs) > 0 {
		if _, err := exec.LookPath(workspace.SelfBin()); err != nil {
			llFail("packwand binary not found (needed for refresh); nothing was modified")
		}
	}

	// Apply phase: snapshot everything we will touch, then write. Any
	// failure — including a refresh failure — restores all snapshots.
	txn := newBumpTxn()
	txn.snapshot(mfPath)
	for _, e := range edits {
		txn.snapshot(e.path)
	}
	for _, dir := range refreshDirs {
		txn.snapshot(filepath.Join(dir, "index.toml"))
		txn.snapshot(filepath.Join(dir, "pack.toml"))
	}

	fail := func(msg string) {
		txn.rollback()
		llFail(msg + " — all changes rolled back")
	}

	if err := manifest.Write(mfPath, m); err != nil {
		fail(fmt.Sprintf("failed to write %s: %v", mfPath, err))
	}
	for _, e := range edits {
		if err := os.WriteFile(e.path, e.newData, 0o644); err != nil {
			fail(fmt.Sprintf("failed to write %s: %v", e.path, err))
		}
		fmt.Printf("  %s\n", e.label)
	}
	for _, dir := range refreshDirs {
		c := exec.Command(workspace.SelfBin(), "refresh")
		c.Dir = dir
		if out, err := c.CombinedOutput(); err != nil {
			fail(fmt.Sprintf("packwand refresh failed in %s: %v\n%s", dir, err, workspace.Indent(string(out), "    ")))
		}
		fmt.Printf("  refreshed %s\n", dir)
	}

	fmt.Printf("bumped %s: %s -> %s (%d config file(s), %d subdir refresh(es))\n", mfPath, old, newVer, len(edits), len(refreshDirs))
}

func planPackConfigEdits(packDir, packName, version string) ([]bumpEdit, []string) {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		llFail(fmt.Sprintf("failed to read %s: %v", packDir, err))
	}

	var edits []bumpEdit
	var refreshDirs []string
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
		if ed, ok := planMenuCredits(filepath.Join(cfgDir, "isxander-main-menu-credits.json"), packName, version); ok {
			edits = append(edits, ed)
			n++
		}
		if ed, ok := planLoaderDeps(filepath.Join(cfgDir, "fabric_loader_dependencies.json"), packName, version); ok {
			edits = append(edits, ed)
			n++
		}
		if n > 0 {
			refreshDirs = append(refreshDirs, filepath.Join(packDir, name))
		}
	}

	if len(edits) == 0 {
		fmt.Printf("no version-bearing configs found in any subdir of %s.\n", packDir)
	}
	return edits, refreshDirs
}

func planMenuCredits(path, packName, version string) (bumpEdit, bool) {
	obj, ok := loadJSONMap(path)
	if !ok {
		return bumpEdit{}, false
	}
	mainMenu, ok := obj["main_menu"].(map[string]any)
	if !ok {
		return bumpEdit{}, false
	}
	bottomRight, ok := mainMenu["bottom_right"].([]any)
	if !ok || len(bottomRight) == 0 {
		return bumpEdit{}, false
	}
	first, ok := bottomRight[0].(map[string]any)
	if !ok {
		return bumpEdit{}, false
	}
	first["text"] = packName + " " + version
	return bumpEdit{
		path:    path,
		newData: marshalCompactJSON(path, obj),
		label:   fmt.Sprintf("%s -> %q", path, packName+" "+version),
	}, true
}

func planLoaderDeps(path, packName, version string) (bumpEdit, bool) {
	obj, ok := loadJSONMap(path)
	if !ok {
		return bumpEdit{}, false
	}
	overrides, ok := obj["overrides"].(map[string]any)
	if !ok {
		return bumpEdit{}, false
	}
	minecraft, ok := overrides["minecraft"].(map[string]any)
	if !ok {
		return bumpEdit{}, false
	}
	recommends, ok := minecraft["+recommends"].(map[string]any)
	if !ok {
		return bumpEdit{}, false
	}
	recommends[packName] = ">" + version
	return bumpEdit{
		path:    path,
		newData: marshalCompactJSON(path, obj),
		label:   fmt.Sprintf("%s -> %s: %q", path, packName, ">"+version),
	}, true
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

func marshalCompactJSON(path string, v any) []byte {
	var buf strings.Builder
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(v); err != nil {
		llFail(fmt.Sprintf("failed to marshal %s: %v", path, err))
	}
	return []byte(buf.String())
}

// — freeze / unfreeze —

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
		llFail("packwand binary not found; check PACKWAND_BIN or PATH")
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

// — side —

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
		llFail("packwand binary not found; check PACKWAND_BIN or PATH")
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

// — packs —

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
		if _, err := writeProjectsIndex(false); err != nil {
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
		llFail("no packs found — run packwand from the repo root")
	}
	if asJSON {
		type jsonEntry struct {
			ID       string             `json:"id"`
			Category string             `json:"category"`
			Dir      string             `json:"dir"`
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
	if Interactive() {
		rows := make([][]string, 0, len(packs))
		for _, pack := range packs {
			m := pack.M
			lifecycle := m.Lifecycle
			if lifecycle == "" {
				lifecycle = "active"
			}
			platforms := []string{}
			if m.ModrinthID != "" {
				platforms = append(platforms, "mr")
			}
			if m.CurseforgeID != "" {
				platforms = append(platforms, "cf")
			}
			if m.GitHubID != "" {
				platforms = append(platforms, "gh")
			}
			if m.GiteaID != "" {
				platforms = append(platforms, "gitea")
			}
			if m.GitLabID != "" {
				platforms = append(platforms, "gl")
			}
			rows = append(rows, []string{pack.ID, m.Type, m.Version, m.Loader, m.Role.Label(), lifecycle, strings.Join(platforms, "+")})
		}
		fmt.Fprintln(os.Stderr, Table([]string{"PACK", "TYPE", "VERSION", "LOADER", "ROLE", "LIFECYCLE", "PLATFORMS"}, rows))
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
	fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  %-12s  platforms\n", idW, "id", "type", verW, "version", "loader", "role", "lifecycle")
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
		if m.GitHubID != "" {
			plats = append(plats, "gh")
		}
		if m.GiteaID != "" {
			plats = append(plats, "gitea")
		}
		if m.GitLabID != "" {
			plats = append(plats, "gl")
		}
		platStr := strings.Join(plats, "+")
		if platStr == "" {
			platStr = "-"
		}
		typ := m.Type
		if len(m.Variants) > 0 {
			typ = fmt.Sprintf("%s(%dv)", typ, len(m.Variants))
		}
		lc := m.Lifecycle
		if lc == "" {
			lc = "active"
		}
		fmt.Printf("%-*s  %-13s  %-*s  %-8s  %-10s  %-12s  %s\n", idW, p.ID, typ, verW, m.Version, loader, role, lc, platStr)
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
	"modrinth_id": true, "curseforge_id": true, "github_id": true, "gitea_id": true, "gitlab_id": true,
	"mc_version": true, "loader": true, "lifecycle": true,
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
	case "github_id":
		old, m.GitHubID = m.GitHubID, value
	case "gitea_id":
		old, m.GiteaID = m.GiteaID, value
	case "gitlab_id":
		old, m.GitLabID = m.GitLabID, value
	case "mc_version":
		if m.MCVersion != nil {
			old = *m.MCVersion
		}
		m.MCVersion = &value
	case "loader":
		old, m.Loader = m.Loader, value
	case "lifecycle":
		validLifecycles := map[string]bool{"active": true, "maintenance": true, "archived": true, "eol": true}
		if value != "" && !validLifecycles[value] {
			llFail(fmt.Sprintf("invalid lifecycle %q (valid: active, maintenance, archived, eol)", value))
		}
		old, m.Lifecycle = m.Lifecycle, value
	}
	if err := manifest.Write(filepath.Join(p.Dir, "manifest.json"), m); err != nil {
		llFail(fmt.Sprintf("failed to write manifest: %v", err))
	}
	fmt.Printf("%s: %s: %q -> %q\n", p.ID, field, old, value)
	if field == "version" {
		fmt.Println("note: 'packwand bump' is the richer path for versions (supports --configs for in-pack version files)")
	}
}

// — automation —

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
