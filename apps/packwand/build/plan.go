package build

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
	"github.com/spf13/cobra"
)

func init() {
	publishPlanCmd.Flags().String("from", "", "Base git ref to compare manifests against (default: <to>^)")
	publishPlanCmd.Flags().String("to", "HEAD", "Target git ref whose manifests are candidates for publishing")
	publishPlanCmd.Flags().Bool("no-validate", false, "Skip running 'packwand validate' on included manifests")
	publishPlanCmd.Flags().String("pack", "", "Limit the plan to one repository-relative pack directory")
	publishCmd.AddCommand(publishPlanCmd)
}

var publishPlanCmd = &cobra.Command{
	Use:   "plan",
	Short: "Compute the publish matrix from git changes, with include/skip reasons (JSON on stdout)",
	Args:  cobra.NoArgs,
	Run: func(c *cobra.Command, args []string) {
		from, _ := c.Flags().GetString("from")
		to, _ := c.Flags().GetString("to")
		noValidate, _ := c.Flags().GetBool("no-validate")
		pack, _ := c.Flags().GetString("pack")
		cmd.Chdir()
		pubPlan(from, to, pack, !noValidate)
	},
}

type PlanEntry struct {
	Manifest     string  `json:"manifest"`
	Variant      *string `json:"variant"`
	Order        int     `json:"order"`
	Experimental bool    `json:"experimental"`
	Reason       string  `json:"reason"`
}

type PlanSkip struct {
	Manifest string `json:"manifest"`
	Reason   string `json:"reason"`
}

type PlanResult struct {
	From       string      `json:"from"`
	To         string      `json:"to"`
	Entries    []PlanEntry `json:"entries"`
	Skipped    []PlanSkip  `json:"skipped"`
	Invalid    []PlanSkip  `json:"invalid,omitempty"`
	HasEntries bool        `json:"has_entries"`
}

var (
	planManifestRe = regexp.MustCompile(`^(modpacks|datapacks)/[^/]+/manifest\.json$`)
	planPackDirRe  = regexp.MustCompile(`^(modpacks|datapacks)/[^/]+/`)
)

func pubPlan(from, to, pack string, validate bool) {
	if from == "" {
		from = to + "^"
	}
	res := PlanResult{From: from, To: to, Entries: []PlanEntry{}, Skipped: []PlanSkip{}}

	changed := planChangedFiles(from, to)
	if pack != "" {
		prefix := strings.TrimSuffix(filepath.ToSlash(filepath.Clean(pack)), "/") + "/"
		filtered := changed[:0]
		for _, file := range changed {
			if strings.HasPrefix(file, prefix) {
				filtered = append(filtered, file)
			}
		}
		changed = filtered
	}

	// Production candidates: manifest.json files whose 'version' changed.
	type inclusion struct {
		manifest, reason string
		experimental     bool
	}
	var included []inclusion
	seen := map[string]bool{}

	for _, f := range changed {
		if !planManifestRe.MatchString(f) {
			continue
		}
		newVer := planVersionAt(to, f)
		if newVer == "" {
			res.Skipped = append(res.Skipped, PlanSkip{Manifest: f, Reason: fmt.Sprintf("no version at %s (manifest removed or empty)", to)})
			continue
		}
		oldVer := planVersionAt(from, f)
		if newVer == oldVer {
			res.Skipped = append(res.Skipped, PlanSkip{Manifest: f, Reason: fmt.Sprintf("version unchanged ('%s')", newVer)})
			continue
		}
		if !seen[f] {
			seen[f] = true
			included = append(included, inclusion{manifest: f, reason: fmt.Sprintf("version '%s' -> '%s'", oldVer, newVer)})
		}
	}

	// Experimental candidates: any change inside a pack dir that carries a
	// manifest-experimental.json (experimental builds republish on every change).
	packDirs := map[string]bool{}
	for _, f := range changed {
		if m := planPackDirRe.FindString(f); m != "" {
			packDirs[strings.TrimSuffix(m, "/")] = true
		}
	}
	var dirs []string
	for d := range packDirs {
		dirs = append(dirs, d)
	}
	sort.Strings(dirs)
	for _, d := range dirs {
		exp := path.Join(d, "manifest-experimental.json")
		if info, err := os.Stat(exp); err == nil && !info.IsDir() {
			if !seen[exp] {
				seen[exp] = true
				included = append(included, inclusion{manifest: exp, reason: "pack files changed; experimental manifest present", experimental: true})
			}
		}
	}

	// Validate and expand each included manifest into matrix entries.
	for _, inc := range included {
		if validate {
			if out, err := exec.Command(workspace.SelfBin(), "validate", inc.manifest).CombinedOutput(); err != nil {
				res.Invalid = append(res.Invalid, PlanSkip{Manifest: inc.manifest, Reason: planLastLine(string(out))})
				continue
			}
		}
		m, err := manifest.Read(inc.manifest)
		if err != nil {
			res.Invalid = append(res.Invalid, PlanSkip{Manifest: inc.manifest, Reason: fmt.Sprintf("unreadable: %v", err)})
			continue
		}
		if len(m.Variants) > 0 {
			for idx, v := range m.Variants {
				key := v.ID
				if key == "" {
					key = v.MCVersion
				}
				if key == "" {
					res.Invalid = append(res.Invalid, PlanSkip{Manifest: inc.manifest, Reason: "variant missing both 'id' and 'mc_version'"})
					continue
				}
				variant := key
				res.Entries = append(res.Entries, PlanEntry{Manifest: inc.manifest, Variant: &variant, Order: idx, Experimental: inc.experimental, Reason: inc.reason})
			}
		} else {
			res.Entries = append(res.Entries, PlanEntry{Manifest: inc.manifest, Variant: nil, Order: 0, Experimental: inc.experimental, Reason: inc.reason})
		}
	}

	res.HasEntries = len(res.Entries) > 0

	for _, e := range res.Entries {
		v := ""
		if e.Variant != nil {
			v = " [" + *e.Variant + "]"
		}
		fmt.Fprintf(os.Stderr, "plan: include %s%s — %s\n", e.Manifest, v, e.Reason)
	}
	for _, s := range res.Skipped {
		fmt.Fprintf(os.Stderr, "plan: skip %s — %s\n", s.Manifest, s.Reason)
	}
	for _, s := range res.Invalid {
		fmt.Fprintf(os.Stderr, "plan: INVALID %s — %s\n", s.Manifest, s.Reason)
	}

	data, err := json.Marshal(res)
	if err != nil {
		cmd.Fail(fmt.Sprintf("failed to render plan: %v", err))
	}
	fmt.Println(string(data))

	if outPath := os.Getenv("GITHUB_OUTPUT"); outPath != "" {
		f, err := os.OpenFile(outPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
		if err != nil {
			cmd.Fail(fmt.Sprintf("failed to open GITHUB_OUTPUT: %v", err))
		}
		defer f.Close()
		entries, _ := json.Marshal(res.Entries)
		fmt.Fprintf(f, "entries=%s\n", string(entries))
		fmt.Fprintf(f, "has_entries=%t\n", res.HasEntries)
	}

	if len(res.Invalid) > 0 {
		cmd.Fail(fmt.Sprintf("plan: %d manifest(s) failed validation", len(res.Invalid)))
	}
}

// planChangedFiles lists files that differ between the two refs.
func planChangedFiles(from, to string) []string {
	out, err := exec.Command("git", "diff", "--name-only", "--no-renames", from, to).Output()
	if err != nil {
		cmd.Fail(fmt.Sprintf("git diff %s %s failed: %v", from, to, err))
	}
	var files []string
	for _, l := range strings.Split(string(out), "\n") {
		if l = strings.TrimSpace(l); l != "" {
			files = append(files, l)
		}
	}
	return files
}

// planVersionAt reads the manifest 'version' field as it exists at the given
// ref. Returns "" if the file does not exist there or has no version.
func planVersionAt(ref, file string) string {
	out, err := exec.Command("git", "show", ref+":"+file).Output()
	if err != nil {
		return ""
	}
	var m struct {
		Version string `json:"version"`
	}
	if json.Unmarshal(out, &m) != nil {
		return ""
	}
	return m.Version
}

func planLastLine(s string) string {
	lines := strings.Split(strings.TrimSpace(s), "\n")
	if len(lines) == 0 {
		return "validation failed"
	}
	return strings.TrimSpace(lines[len(lines)-1])
}
