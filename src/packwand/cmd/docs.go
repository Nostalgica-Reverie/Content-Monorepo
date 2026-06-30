package cmd

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"sync/atomic"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	llModlistCmd.Flags().StringP("subdir", "s", "", "Pack subdir to read mods from (e.g. nightfall-mr)")
	llModlistCmd.GroupID = GroupOther
	rootCmd.AddCommand(llModlistCmd)

	llPagesCmd.Flags().StringP("pack", "p", "", "Pack directory to regenerate (default: all)")
	llPagesCmd.GroupID = GroupOther
	rootCmd.AddCommand(llPagesCmd)

	llDiffCmd.GroupID = GroupOther
	rootCmd.AddCommand(llDiffCmd)
}

// â€” types shared by modlist and pages â€”

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

// â€” modlist â€”

var llModlistCmd = &cobra.Command{
	Use:   "modlist <subdir>",
	Short: "Write a crash-assistant modlist.json from a pack's mods/ directory",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		subdir := llAbs(args[0])
		llChdir()

		modsDir := filepath.Join(subdir, "mods")
		if info, err := os.Stat(modsDir); err != nil || !info.IsDir() {
			llFail(fmt.Sprintf("no mods/ directory at %s", modsDir))
		}

		entries, err := os.ReadDir(modsDir)
		if err != nil {
			llFail(fmt.Sprintf("failed to read %s: %v", modsDir, err))
		}

		modlist := make(map[string]modlistEntry)
		var parsed, withCF, withMR int

		for _, e := range entries {
			if e.IsDir() || !strings.HasSuffix(e.Name(), ".pw.toml") {
				continue
			}
			mod, err := parsePwToml(filepath.Join(modsDir, e.Name()))
			if err != nil {
				llWarn("skipping %s: %v", e.Name(), err)
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
			llFail(fmt.Sprintf("failed to create %s: %v", outDir, err))
		}
		outPath := filepath.Join(outDir, "modlist.json")
		data, err := json.MarshalIndent(modlist, "", "  ")
		if err != nil {
			llFail(fmt.Sprintf("failed to marshal modlist: %v", err))
		}
		data = append(data, '\n')
		if err := os.WriteFile(outPath, data, 0o644); err != nil {
			llFail(fmt.Sprintf("failed to write %s: %v", outPath, err))
		}

		fmt.Printf("wrote %s\n", outPath)
		fmt.Printf("  %d mod(s): %d with curseForgeHash, %d with modrinthHash(sha1)\n", parsed, withCF, withMR)
		if withMR < parsed {
			fmt.Printf("  note: %d mod(s) lack a usable modrinthHash (packwiz stores sha512, not sha1, or are MR-only). Names/versions are present.\n", parsed-withMR)
		}
	},
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

func parseInt64(s string) (int64, error) {
	var n int64
	_, err := fmt.Sscanf(s, "%d", &n)
	return n, err
}

func versionFromFilename(filename string) string {
	return strings.TrimSuffix(filename, ".jar")
}

// â€” pages â€”

var llPagesCmd = &cobra.Command{
	Use:     "pages [pack-dir]",
	Short:   "Regenerate modlist.md files for all packs (or a single pack) and the projects index",
	Aliases: []string{"docs"},
	Run: func(cmd *cobra.Command, args []string) {
		packArg, _ := cmd.Flags().GetString("pack")
		if packArg == "" && len(args) > 0 {
			packArg = llAbs(args[0])
		} else if packArg != "" {
			packArg = llAbs(packArg)
		}
		llChdir()
		runPages(packArg)
	},
}

// runPages regenerates modlist.md files and the projects index.
// packArg is an absolute path to a single pack directory, or "" to regenerate all.
// Called from wsSyncCmd after a successful sync.
func runPages(packArg string) {
	var subdirs []string
	if packArg != "" {
		subdirs = packModSubdirs(packArg)
		if len(subdirs) == 0 {
			llWarn("no mod subdirs found under %s", packArg)
			return
		}
	} else {
		root := workspace.ModpacksDir()
		packs, err := os.ReadDir(root)
		if err != nil {
			llWarn("failed to read %s: %v", root, err)
			return
		}
		for _, p := range packs {
			if p.IsDir() {
				subdirs = append(subdirs, packModSubdirs(filepath.Join(root, p.Name()))...)
			}
		}
		if len(subdirs) == 0 {
			fmt.Println("no mod subdirs found in any pack")
			return
		}
	}

	var written int64
	sched := workspace.NewScheduler(workspace.MaxConcurrent())
	dones := make([]<-chan error, len(subdirs))
	for i, sub := range subdirs {
		sub := sub
		dones[i] = sched.Submit(workspace.Task{
			Name:  sub,
			Needs: []workspace.Resource{workspace.Resource("pages:" + sub)},
			Run: func() error {
				n, err := writeModlistMD(sub)
				if err != nil {
					llWarn("%s: %v", sub, err)
					return nil
				}
				fmt.Printf("wrote %s/modlist.md (%d mods)\n", sub, n)
				atomic.AddInt64(&written, 1)
				return nil
			},
		})
	}
	sched.Close()
	for _, c := range dones {
		<-c
	}
	fmt.Printf("generated %d modlist.md file(s).\n", written)

	if packArg == "" {
		if _, err := writeProjectsIndex(); err != nil {
			llWarn("projects index not written: %v", err)
		}
	}
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
	fmt.Fprintf(b, "\n## %s\n\n", title)
	for _, l := range lines {
		b.WriteString(l)
		b.WriteByte('\n')
	}
}

func modPageURL(m *pwMod) string {
	if m.mrModID != "" {
		return "https://modrinth.com/mod/" + m.mrModID
	}
	if m.url != "" {
		return m.url
	}
	return ""
}

// â€” diff â€”

var llDiffCmd = &cobra.Command{
	Use:   "diff <old-ref> <new-ref> [path-prefix]",
	Short: "Show mod additions, removals, and updates between two git refs",
	Args:  cobra.RangeArgs(2, 3),
	Run: func(cmd *cobra.Command, args []string) {
		llChdir()
		oldRef, newRef := args[0], args[1]
		var pathPrefix string
		if len(args) > 2 {
			pathPrefix = args[2]
		}

		out, err := exec.Command("git", "diff", "--name-only", oldRef+".."+newRef).Output()
		if err != nil {
			llFail(fmt.Sprintf("git diff failed: %v", err))
		}

		var changed []string
		for _, l := range strings.Split(string(out), "\n") {
			l = strings.TrimSpace(l)
			if l == "" || !strings.HasSuffix(l, ".pw.toml") {
				continue
			}
			if pathPrefix != "" && !strings.HasPrefix(l, pathPrefix) {
				continue
			}
			changed = append(changed, l)
		}

		if len(changed) == 0 {
			fmt.Printf("no .pw.toml changes between %s and %s\n", oldRef, newRef)
			return
		}

		bySubdir := map[string][]string{}
		for _, p := range changed {
			sub := filepath.Dir(filepath.Dir(p))
			bySubdir[sub] = append(bySubdir[sub], p)
		}
		subdirs := make([]string, 0, len(bySubdir))
		for s := range bySubdir {
			subdirs = append(subdirs, s)
		}
		sort.Strings(subdirs)

		totalAdded, totalRemoved, totalUpdated := 0, 0, 0

		for _, sub := range subdirs {
			files := bySubdir[sub]
			sort.Strings(files)

			added, removed, updated := 0, 0, 0
			var lines []string

			for _, path := range files {
				oldContent := gitShowFile(oldRef, path)
				newContent := gitShowFile(newRef, path)
				slug := strings.TrimSuffix(filepath.Base(path), ".pw.toml")

				switch {
				case oldContent == "" && newContent != "":
					ver := pwFilename(newContent)
					lines = append(lines, fmt.Sprintf("  + %-38s %s", slug, ver))
					added++
				case oldContent != "" && newContent == "":
					ver := pwFilename(oldContent)
					lines = append(lines, fmt.Sprintf("  - %-38s %s", slug, ver))
					removed++
				default:
					oldFn := pwFilename(oldContent)
					newFn := pwFilename(newContent)
					if oldFn != newFn {
						lines = append(lines, fmt.Sprintf("  ~ %-38s %s -> %s", slug, oldFn, newFn))
					} else {
						lines = append(lines, fmt.Sprintf("  ~ %s", slug))
					}
					updated++
				}
			}

			totalAdded += added
			totalRemoved += removed
			totalUpdated += updated

			fmt.Printf("%s:\n", sub)
			for _, l := range lines {
				fmt.Println(l)
			}
			fmt.Printf("  +%d -%d ~%d\n\n", added, removed, updated)
		}

		fmt.Printf("%s..%s: +%d added  -%d removed  ~%d updated\n",
			oldRef, newRef, totalAdded, totalRemoved, totalUpdated)
	},
}

func gitShowFile(ref, path string) string {
	out, err := exec.Command("git", "show", ref+":"+path).Output()
	if err != nil {
		return ""
	}
	return string(out)
}

func pwFilename(content string) string {
	inSection := false
	for _, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[") {
			inSection = true
			continue
		}
		if inSection {
			continue
		}
		k, v, ok := splitKV(line)
		if ok && k == "filename" {
			return strings.Trim(v, `"`)
		}
	}
	return ""
}

func pwVersion(content string) string {
	inUpdate := false
	for _, raw := range strings.Split(content, "\n") {
		line := strings.TrimSpace(raw)
		if strings.HasPrefix(line, "[update") {
			inUpdate = true
			continue
		}
		if strings.HasPrefix(line, "[") {
			inUpdate = false
			continue
		}
		if !inUpdate {
			continue
		}
		k, v, ok := splitKV(line)
		if ok && k == "version" {
			return strings.Trim(v, `"`)
		}
	}
	return ""
}

// â€” index types and writers (used by runPages and llPacksIndexCmd) â€”

type indexVariant struct {
	ID        string `json:"id,omitempty"`
	MCVersion string `json:"mc_version"`
	Loader    string `json:"loader,omitempty"`
	Version   string `json:"version,omitempty"`
}

type indexPlatforms struct {
	Modrinth   string `json:"modrinth,omitempty"`
	Curseforge string `json:"curseforge,omitempty"`
	GitHub     string `json:"github,omitempty"`
	Gitea      string `json:"gitea,omitempty"`
	GitLab     string `json:"gitlab,omitempty"`
}

type indexSubdir struct {
	Key      string `json:"key"`
	Path     string `json:"path"`
	Platform string `json:"platform,omitempty"`
	ModCount int    `json:"mod_count,omitempty"`
	HasIndex bool   `json:"has_index"`
	HasPack  bool   `json:"has_pack"`
}

type indexEntry struct {
	ID              string         `json:"id"`
	Name            string         `json:"name"`
	Type            string         `json:"type"`
	Category        string         `json:"category,omitempty"`
	Dir             string         `json:"dir,omitempty"`
	ManifestPath    string         `json:"manifest_path,omitempty"`
	Loader          string         `json:"loader,omitempty"`
	MCVersion       string         `json:"mc_version,omitempty"`
	Version         string         `json:"version,omitempty"`
	ReleaseType     string         `json:"release_type,omitempty"`
	Description     string         `json:"description,omitempty"`
	Lifecycle       string         `json:"lifecycle,omitempty"`
	Role            string         `json:"role,omitempty"`
	PerformanceBase string         `json:"performance_base,omitempty"`
	SharedAssets    string         `json:"shared_assets,omitempty"`
	AutoUpdate      bool           `json:"auto_update"`
	ModrinthID      string         `json:"modrinth_id,omitempty"`
	CurseforgeID    string         `json:"curseforge_id,omitempty"`
	GitHubID        string         `json:"github_id,omitempty"`
	GiteaID         string         `json:"gitea_id,omitempty"`
	GitLabID        string         `json:"gitlab_id,omitempty"`
	Platforms       indexPlatforms `json:"platforms"`
	DocsPath        string         `json:"docs_path,omitempty"`
	Variants        []indexVariant `json:"variants,omitempty"`
	Subdirs         []indexSubdir  `json:"subdirs,omitempty"`
}

type indexFile struct {
	Generated string       `json:"generated"`
	Projects  []indexEntry `json:"projects"`
}

func projectsIndexOutPath() string {
	if p := os.Getenv("PROJECTS_INDEX_OUT"); p != "" {
		return p
	}
	return filepath.Join("docs", "docs", "public", "projects.json")
}

func docsPathFor(typ, id string) string {
	switch typ {
	case "modpack":
		return "/modpacks/" + id + "/"
	case "datapack":
		return "/datapacks/" + id + "/"
	case "resourcepack":
		return "/resource-packs/" + id + "/"
	}
	return ""
}

func categoryForRoot(root string) string {
	return strings.TrimSuffix(root, "s")
}

func indexTypeRank(typ string) int {
	switch typ {
	case "modpack":
		return 0
	case "resourcepack":
		return 1
	case "datapack":
		return 2
	default:
		return 99
	}
}

func indexPlatformFromSubdir(key string) string {
	switch {
	case strings.HasSuffix(key, "-mr"):
		return "modrinth"
	case strings.HasSuffix(key, "-cf"):
		return "curseforge"
	default:
		return ""
	}
}

func indexSubdirs(packDir string) []indexSubdir {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return nil
	}
	out := make([]indexSubdir, 0, len(entries))
	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		sub := filepath.Join(packDir, entry.Name())
		key := filepath.Base(sub)
		modCount := 0
		if entries, err := os.ReadDir(filepath.Join(sub, "mods")); err == nil {
			for _, e := range entries {
				if !e.IsDir() && strings.HasSuffix(e.Name(), ".pw.toml") {
					modCount++
				}
			}
		}
		_, indexErr := os.Stat(filepath.Join(sub, "index.toml"))
		_, packErr := os.Stat(filepath.Join(sub, "pack.toml"))
		out = append(out, indexSubdir{
			Key:      key,
			Path:     filepath.ToSlash(sub),
			Platform: indexPlatformFromSubdir(key),
			ModCount: modCount,
			HasIndex: indexErr == nil,
			HasPack:  packErr == nil,
		})
	}
	return out
}

func writeCategoryIndexes(entries []indexEntry) {
	byCat := map[string][]indexEntry{}
	for _, e := range entries {
		cat := e.Type + "s"
		byCat[cat] = append(byCat[cat], e)
	}
	for cat, list := range byCat {
		if _, err := os.Stat(cat); err != nil {
			continue
		}
		out := filepath.Join(cat, "Project.json")
		llWriteJSON(out, map[string]any{
			"_generated": "Used by Packwand do not touch pls thx",
			"generated":  time.Now().UTC().Format(time.RFC3339),
			"projects":   list,
		})
		fmt.Printf("wrote %s (%d project(s))\n", out, len(list))
	}
}

func writeProjectsIndex() (int, error) {
	var entries []indexEntry
	seen := map[string]bool{}

	for _, root := range []string{"modpacks", "datapacks", "resourcepacks"} {
		packs, err := os.ReadDir(root)
		if err != nil {
			continue
		}
		for _, p := range packs {
			if !p.IsDir() {
				continue
			}
			path := filepath.Join(root, p.Name(), "manifest.json")
			m, err := manifest.Read(path)
			if err != nil {
				if errors.Is(err, os.ErrNotExist) {
					continue
				}
				llWarn("index: %v", err)
				continue
			}
			if m.ID == "" || m.Name == "" || seen[m.ID] {
				continue
			}
			seen[m.ID] = true

			auto := manifest.ReadAutomation(filepath.Join(root, p.Name()))
			autoUpdate := auto.AutoUpdate == nil || *auto.AutoUpdate
			lifecycle := m.Lifecycle
			if lifecycle == "" {
				lifecycle = "active"
			}
			role := m.Role.Label()
			performanceBase := ""
			if pb := m.Role.ConsumerBase(); pb != nil {
				performanceBase = pb.Pack
			}
			e := indexEntry{
				ID:              m.ID,
				Name:            m.Name,
				Type:            m.Type,
				Category:        categoryForRoot(root),
				Dir:             filepath.ToSlash(filepath.Join(root, p.Name())),
				ManifestPath:    filepath.ToSlash(path),
				Version:         m.Version,
				ReleaseType:     m.ReleaseType,
				Description:     m.Description,
				Lifecycle:       lifecycle,
				Role:            role,
				PerformanceBase: performanceBase,
				SharedAssets:    m.SharedAssets,
				AutoUpdate:      autoUpdate,
				ModrinthID:      m.ModrinthID,
				CurseforgeID:    m.CurseforgeID,
				GitHubID:        m.GitHubID,
				GiteaID:         m.GiteaID,
				GitLabID:        m.GitLabID,
				Platforms: indexPlatforms{
					Modrinth:   m.ModrinthID,
					Curseforge: m.CurseforgeID,
					GitHub:     m.GitHubID,
					Gitea:      m.GiteaID,
					GitLab:     m.GitLabID,
				},
				DocsPath: docsPathFor(m.Type, m.ID),
				Subdirs:  indexSubdirs(filepath.Join(root, p.Name())),
			}
			if len(m.Variants) > 0 {
				for _, v := range m.Variants {
					loader := v.Loader
					if loader == "" {
						loader = m.Loader
					}
					e.Variants = append(e.Variants, indexVariant{
						ID:        v.ID,
						MCVersion: v.MCVersion,
						Loader:    loader,
						Version:   v.Version,
					})
				}
			} else {
				if m.MCVersion != nil {
					e.MCVersion = *m.MCVersion
				}
				e.Loader = m.Loader
			}
			entries = append(entries, e)
		}
	}

	sort.Slice(entries, func(i, j int) bool {
		if indexTypeRank(entries[i].Type) != indexTypeRank(entries[j].Type) {
			return indexTypeRank(entries[i].Type) < indexTypeRank(entries[j].Type)
		}
		if entries[i].ID != entries[j].ID {
			return entries[i].ID < entries[j].ID
		}
		return entries[i].Name < entries[j].Name
	})

	out := projectsIndexOutPath()
	if err := os.MkdirAll(filepath.Dir(out), 0o755); err != nil {
		return 0, fmt.Errorf("creating %s: %w", filepath.Dir(out), err)
	}
	llWriteJSON(out, indexFile{
		Generated: time.Now().UTC().Format(time.RFC3339),
		Projects:  entries,
	})
	fmt.Printf("wrote %s (%d project(s))\n", out, len(entries))
	writeCategoryIndexes(entries)
	return len(entries), nil
}
