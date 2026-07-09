package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/spf13/cobra"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/clistyle"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
)

func init() {
	llParityCmd.Flags().Bool("json", false, "Output the parity report as JSON")
	llParityCmd.Flags().Bool("strict", false, "Exit 1 when any variant pair has drifted")
	llParityCmd.GroupID = GroupInfo
	rootCmd.AddCommand(llParityCmd)
}

// The -mr and -cf subdirs of a variant are two build targets of the same
// pack, not independent forks — they should ship the same content unless a
// mod is genuinely unavailable on one platform. This command surfaces drift
// between the platform pairs of every variant.

var llParityCmd = &cobra.Command{
	Use:   "parity [pack-dir...]",
	Short: "Report content drift between a pack's Modrinth (-mr) and CurseForge (-cf) variant subdirs (no args: all modpacks)",
	Run: func(cmd *cobra.Command, args []string) {
		jsonOut, _ := cmd.Flags().GetBool("json")
		strict, _ := cmd.Flags().GetBool("strict")

		var packDirs []string
		if len(args) > 0 {
			for _, a := range args {
				packDirs = append(packDirs, llAbs(a))
			}
			llChdir()
		} else {
			llChdir()
			root := workspace.ModpacksDir()
			entries, err := os.ReadDir(root)
			if err != nil {
				llFail(fmt.Sprintf("failed to read %s: %v", root, err))
			}
			for _, e := range entries {
				if e.IsDir() {
					packDirs = append(packDirs, filepath.Join(root, e.Name()))
				}
			}
		}

		reports := []VariantParityReport{}
		for _, dir := range packDirs {
			reports = append(reports, packParityReports(dir)...)
		}
		drifted := 0
		for _, r := range reports {
			if r.Drifted() {
				drifted++
			}
		}

		if jsonOut {
			data, err := json.Marshal(reports)
			if err != nil {
				llFail(fmt.Sprintf("failed to render parity report: %v", err))
			}
			fmt.Println(string(data))
		} else {
			printParityReports(reports, drifted)
		}

		if outPath := os.Getenv("GITHUB_OUTPUT"); outPath != "" {
			if f, err := os.OpenFile(outPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644); err == nil {
				fmt.Fprintf(f, "drifted=%d\n", drifted)
				_ = f.Close()
			}
		}
		if strict && drifted > 0 {
			os.Exit(1)
		}
	},
}

// VariantParityReport is the drift result for one variant's -mr/-cf pair.
type VariantParityReport struct {
	Pack        string   `json:"pack"`
	Variant     string   `json:"variant"`
	OnlyMr      []string `json:"only_mr,omitempty"`
	OnlyCf      []string `json:"only_cf,omitempty"`
	FileDrift   []string `json:"file_drift,omitempty"` // slug: mr-filename != cf-filename
	MrCount     int      `json:"mr_count"`
	CfCount     int      `json:"cf_count"`
	MissingSide string   `json:"missing_side,omitempty"` // "mr"/"cf" when the pair is incomplete
}

func (r VariantParityReport) Drifted() bool {
	return len(r.OnlyMr) > 0 || len(r.OnlyCf) > 0 || len(r.FileDrift) > 0
}

// packParityReports diffs every -mr/-cf pair of one pack directory. Variant
// keys come from the subdirs themselves so packs with legacy layouts (no
// variants list in the manifest) are still covered.
func packParityReports(packDir string) []VariantParityReport {
	pack := filepath.Base(packDir)
	if _, err := manifest.Read(filepath.Join(packDir, "manifest.json")); err != nil {
		return nil // not a manifest-managed pack
	}

	keys := map[string]struct{ mr, cf bool }{}
	for _, sub := range manifest.SubDirsOf(packDir) {
		name := filepath.Base(sub)
		switch {
		case strings.HasSuffix(name, "-mr"):
			k := keys[strings.TrimSuffix(name, "-mr")]
			k.mr = true
			keys[strings.TrimSuffix(name, "-mr")] = k
		case strings.HasSuffix(name, "-cf"):
			k := keys[strings.TrimSuffix(name, "-cf")]
			k.cf = true
			keys[strings.TrimSuffix(name, "-cf")] = k
		}
	}

	sortedKeys := make([]string, 0, len(keys))
	for k := range keys {
		sortedKeys = append(sortedKeys, k)
	}
	sort.Strings(sortedKeys)

	var reports []VariantParityReport
	for _, key := range sortedKeys {
		pair := keys[key]
		rep := VariantParityReport{Pack: pack, Variant: key}
		if !pair.mr || !pair.cf {
			// A single-platform variant is a deliberate choice, not drift —
			// note it and move on.
			if !pair.mr {
				rep.MissingSide = "mr"
			} else {
				rep.MissingSide = "cf"
			}
			reports = append(reports, rep)
			continue
		}

		mrMods := collectPwToml(filepath.Join(packDir, key+"-mr"))
		cfMods := collectPwToml(filepath.Join(packDir, key+"-cf"))
		rep.MrCount = len(mrMods)
		rep.CfCount = len(cfMods)

		compareFiles := func(slug, mrFile, cfFile string) {
			if mrFile != "" && cfFile != "" && mrFile != cfFile {
				rep.FileDrift = append(rep.FileDrift, fmt.Sprintf("%s: %s (mr) != %s (cf)", slug, mrFile, cfFile))
			}
		}

		// Pass 1: match by slug. Pass 2: platforms often use different slugs
		// for the same mod (ferrite-core vs ferritecore-fabric), so match the
		// leftovers by their declared display name before calling it drift.
		unmatchedMr := map[string]pwMeta{}
		for slug, mr := range mrMods {
			cf, ok := cfMods[slug]
			if !ok {
				unmatchedMr[slug] = mr
				continue
			}
			compareFiles(slug, mr.File, cf.File)
		}
		cfByName := map[string]string{} // normalized name -> slug
		cfByFile := map[string]string{} // jar filename -> slug
		for slug := range cfMods {
			if _, ok := mrMods[slug]; !ok {
				if n := normalizeModName(cfMods[slug].Name); n != "" {
					cfByName[n] = slug
				}
				if f := cfMods[slug].File; f != "" {
					cfByFile[f] = slug
				}
			}
		}
		matchedCf := map[string]bool{}
		for slug, mr := range unmatchedMr {
			if cfSlug, ok := cfByName[normalizeModName(mr.Name)]; ok && !matchedCf[cfSlug] {
				matchedCf[cfSlug] = true
				compareFiles(slug, mr.File, cfMods[cfSlug].File)
				continue
			}
			// Pass 3: identical jar filename is the strongest signal that two
			// differently-listed entries are the same mod.
			if cfSlug, ok := cfByFile[mr.File]; ok && mr.File != "" && !matchedCf[cfSlug] {
				matchedCf[cfSlug] = true
				continue
			}
			rep.OnlyMr = append(rep.OnlyMr, slug)
		}
		for slug := range cfMods {
			if _, ok := mrMods[slug]; !ok && !matchedCf[slug] {
				rep.OnlyCf = append(rep.OnlyCf, slug)
			}
		}
		sort.Strings(rep.OnlyMr)
		sort.Strings(rep.OnlyCf)
		sort.Strings(rep.FileDrift)
		reports = append(reports, rep)
	}
	return reports
}

type pwMeta struct {
	Name string
	File string
}

// collectPwToml maps slug (metadata filename without .pw.toml) to the
// declared display name and jar filename for every .pw.toml under subDir.
func collectPwToml(subDir string) map[string]pwMeta {
	mods := map[string]pwMeta{}
	_ = filepath.WalkDir(subDir, func(path string, d os.DirEntry, err error) error {
		if err != nil || d.IsDir() || !strings.HasSuffix(d.Name(), ".pw.toml") {
			return nil
		}
		var meta struct {
			Name     string `toml:"name"`
			FileName string `toml:"filename"`
		}
		if _, err := toml.DecodeFile(path, &meta); err != nil {
			llWarn("parity: skipping unparsable %s: %v", path, err)
			return nil
		}
		slug := strings.TrimSuffix(d.Name(), ".pw.toml")
		mods[slug] = pwMeta{Name: meta.Name, File: meta.FileName}
		return nil
	})
	return mods
}

// normalizeModName folds case and separators, and strips trailing loader
// decorations, so "FerriteCore (Fabric)" and "ferritecore-fabric"-style
// display names line up across platforms.
func normalizeModName(name string) string {
	var b strings.Builder
	for _, r := range strings.ToLower(name) {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') {
			b.WriteRune(r)
		}
	}
	n := b.String()
	for _, loader := range []string{"fabric", "forge", "neoforge", "quilt"} {
		n = strings.TrimSuffix(n, loader)
	}
	return n
}

func printParityReports(reports []VariantParityReport, drifted int) {
	pairs := 0
	for _, r := range reports {
		if r.MissingSide != "" {
			fmt.Printf("  %s %s/%s: single-platform variant (no -%s subdir)\n",
				clistyle.IconDot, r.Pack, r.Variant, r.MissingSide)
			continue
		}
		pairs++
		if !r.Drifted() {
			fmt.Printf("  %s %s/%s: in sync (%d mods)\n", clistyle.IconOK, r.Pack, r.Variant, r.MrCount)
			continue
		}
		fmt.Printf("  %s %s/%s: drifted (mr %d mods, cf %d mods)\n",
			clistyle.IconWarn, r.Pack, r.Variant, r.MrCount, r.CfCount)
		for _, s := range r.OnlyMr {
			fmt.Printf("      only on Modrinth:   %s\n", s)
		}
		for _, s := range r.OnlyCf {
			fmt.Printf("      only on CurseForge: %s\n", s)
		}
		for _, s := range r.FileDrift {
			fmt.Printf("      file drift: %s\n", s)
		}
	}
	if drifted == 0 {
		fmt.Printf(clistyle.IconOK+" all %d variant pair(s) in parity\n", pairs)
	} else {
		fmt.Printf(clistyle.IconWarn+" %d of %d variant pair(s) drifted\n", drifted, pairs)
	}
}
