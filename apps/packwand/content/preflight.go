package content

import (
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/registry"
	"github.com/BurntSushi/toml"
	"github.com/spf13/cobra"
)

func init() {
	preflightCmd.Flags().Bool("json", false, "Output the preflight report as JSON")
	cmd.AddToGroup(preflightCmd, cmd.GroupInfo)
}

// preflightCmd is the pre-launch/publish validation gate (IDE.md §4.4): one
// composite check the CLI, CI, and IDE all share. Errors block gated actions;
// warnings are surfaced but do not fail the run.
var preflightCmd = &cobra.Command{
	Use:   "preflight [dir]",
	Short: "Run the pre-launch validation gate — manifest, syntax, and registry reference checks",
	Args:  cobra.MaximumNArgs(1),
	Run: func(c *cobra.Command, args []string) {
		asJSON, _ := c.Flags().GetBool("json")
		dir := "."
		if len(args) == 1 {
			dir = args[0]
		}
		result := RunPreflight(cmd.Abs(dir))
		if asJSON {
			data, _ := json.MarshalIndent(result, "", "  ")
			fmt.Println(string(data))
		} else {
			printPreflight(result)
		}
		if !result.OK {
			cmd.Fail(fmt.Sprintf("preflight found %d error(s)", result.Errors))
		}
	},
}

type PreflightIssue struct {
	Level   string `json:"level"` // "error" or "warning"
	Path    string `json:"path,omitempty"`
	Message string `json:"message"`
}

type PreflightStep struct {
	Name     string           `json:"name"`
	Errors   int              `json:"errors"`
	Warnings int              `json:"warnings"`
	Issues   []PreflightIssue `json:"issues"`
}

type PreflightResult struct {
	Dir      string          `json:"dir"`
	Steps    []PreflightStep `json:"steps"`
	Errors   int             `json:"errors"`
	Warnings int             `json:"warnings"`
	OK       bool            `json:"ok"`
}

// RunPreflight runs the gate steps for a pack subdir (or standalone content
// pack directory): manifest validation, JSON/TOML syntax across the tree,
// and registry-backed reference checks.
func RunPreflight(dir string) PreflightResult {
	result := PreflightResult{Dir: filepath.ToSlash(dir)}
	result.addStep(preflightManifest(dir))
	result.addStep(preflightSyntax(dir))
	result.addStep(preflightReferences(dir))
	result.OK = result.Errors == 0
	return result
}

func (r *PreflightResult) addStep(step PreflightStep) {
	for _, issue := range step.Issues {
		if issue.Level == "error" {
			step.Errors++
		} else {
			step.Warnings++
		}
	}
	r.Errors += step.Errors
	r.Warnings += step.Warnings
	r.Steps = append(r.Steps, step)
}

// preflightManifest validates the pack manifest that governs dir: dir's own
// manifest.json, or the parent's when dir is a pack subdir.
func preflightManifest(dir string) PreflightStep {
	step := PreflightStep{Name: "manifest", Issues: []PreflightIssue{}}
	manifestPath := filepath.Join(dir, "manifest.json")
	if _, err := os.Stat(manifestPath); err != nil {
		manifestPath = filepath.Join(filepath.Dir(dir), "manifest.json")
	}
	if _, err := os.Stat(manifestPath); err != nil {
		step.Issues = append(step.Issues, PreflightIssue{Level: "error", Path: "manifest.json", Message: "no manifest.json found in the pack directory or its parent"})
		return step
	}
	m, err := manifest.Read(manifestPath)
	if err != nil {
		step.Issues = append(step.Issues, PreflightIssue{Level: "error", Path: "manifest.json", Message: err.Error()})
		return step
	}
	for field, value := range map[string]string{"id": m.ID, "name": m.Name, "type": m.Type} {
		if strings.TrimSpace(value) == "" {
			step.Issues = append(step.Issues, PreflightIssue{Level: "error", Path: "manifest.json", Message: "manifest is missing required field '" + field + "'"})
		}
	}
	if m.Version == "" {
		step.Issues = append(step.Issues, PreflightIssue{Level: "warning", Path: "manifest.json", Message: "manifest has no version"})
	}
	return step
}

// preflightSyntax parses every JSON and TOML document under dir.
const maxPreflightFileSize = 4 << 20

func preflightSyntax(dir string) PreflightStep {
	step := PreflightStep{Name: "syntax", Issues: []PreflightIssue{}}
	_ = filepath.WalkDir(dir, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return nil
		}
		if d.IsDir() {
			if name := d.Name(); name == ".git" || name == "node_modules" {
				return filepath.SkipDir
			}
			if d.Name() == "__MACOSX" {
				return filepath.SkipDir
			}
			return nil
		}
		name := d.Name()
		if name == ".DS_Store" || strings.HasPrefix(name, "._") {
			return nil
		}
		isJSON := strings.HasSuffix(name, ".json") || strings.HasSuffix(name, ".mcmeta")
		isTOML := strings.HasSuffix(name, ".toml")
		if !isJSON && !isTOML {
			return nil
		}
		if info, err := d.Info(); err != nil || info.Size() > maxPreflightFileSize {
			return nil
		}
		data, err := os.ReadFile(p)
		if err != nil {
			return nil
		}
		rel, _ := filepath.Rel(dir, p)
		rel = filepath.ToSlash(rel)
		if isJSON {
			var value any
			if err := json.Unmarshal(data, &value); err != nil {
				// Several mods intentionally use JSON-with-comments for config
				// files despite the .json suffix. Accept that narrow case only
				// when removing comments produces otherwise strict JSON; embedded
				// datapacks and resource packs remain strictly validated.
				if isCommentedConfigJSON(rel, name) {
					var commentedValue any
					if json.Unmarshal(stripJSONComments(data), &commentedValue) == nil {
						return nil
					}
				}
				step.Issues = append(step.Issues, PreflightIssue{Level: "error", Path: rel, Message: "invalid JSON: " + err.Error()})
			}
			return nil
		}
		var value any
		if err := toml.Unmarshal(data, &value); err != nil {
			step.Issues = append(step.Issues, PreflightIssue{Level: "error", Path: rel, Message: "invalid TOML: " + err.Error()})
		}
		return nil
	})
	return step
}

func isCommentedConfigJSON(rel, name string) bool {
	if !strings.HasSuffix(name, ".json") {
		return false
	}
	rel = filepath.ToSlash(rel)
	if !strings.HasPrefix(rel, "config/") && !strings.Contains(rel, "/config/") {
		return false
	}
	for _, nestedPack := range []string{"/datapacks/", "/resourcepacks/", "/global_packs/"} {
		if strings.Contains(rel, nestedPack) {
			return false
		}
	}
	return true
}

// stripJSONComments removes // and /* */ comments while preserving quoted
// strings and newlines. Its output is always passed through encoding/json, so
// this does not relax any other part of JSON syntax.
func stripJSONComments(data []byte) []byte {
	out := make([]byte, 0, len(data))
	inString := false
	escaped := false
	for i := 0; i < len(data); i++ {
		c := data[i]
		if inString {
			out = append(out, c)
			if escaped {
				escaped = false
			} else if c == '\\' {
				escaped = true
			} else if c == '"' {
				inString = false
			}
			continue
		}
		if c == '"' {
			inString = true
			out = append(out, c)
			continue
		}
		if c == '/' && i+1 < len(data) {
			switch data[i+1] {
			case '/':
				i += 2
				for ; i < len(data) && data[i] != '\n' && data[i] != '\r'; i++ {
				}
				if i < len(data) {
					out = append(out, data[i])
				}
				continue
			case '*':
				i += 2
				for ; i+1 < len(data) && !(data[i] == '*' && data[i+1] == '/'); i++ {
					if data[i] == '\n' || data[i] == '\r' {
						out = append(out, data[i])
					}
				}
				if i+1 < len(data) {
					i++
				}
				continue
			}
		}
		out = append(out, c)
	}
	return out
}

// preflightReferences runs the registry-backed document checks over every
// file-backed datapack, resource pack, config, and KubeJS entry. Syntax diagnostics are
// skipped here — the syntax step already reported them.
func preflightReferences(dir string) PreflightStep {
	step := PreflightStep{Name: "references", Issues: []PreflightIssue{}}
	// One session for the whole loop: registries are built at most once per
	// kind, instead of once per checked document.
	session := registry.NewDocCheckSession(dir)
	for _, kind := range []registry.Kind{registry.Datapack, registry.ResourcePack, registry.Config, registry.KubeJS} {
		reg, err := registry.Build(dir, kind)
		if err != nil {
			step.Issues = append(step.Issues, PreflightIssue{Level: "error", Message: err.Error()})
			continue
		}
		for _, entry := range reg.Entries {
			if kind == registry.Config && entry.Kind == "config_file" && entry.Owner == "" {
				step.Issues = append(step.Issues, PreflightIssue{Level: "warning", Path: entry.Origin + "/" + entry.Path, Message: "config is not associated with an installed mod"})
			}
			if entry.Path == "" || !checkablePreflightPath(entry.Path) {
				continue
			}
			rel := entry.Path
			if entry.Origin != "." && entry.Origin != "" {
				rel = entry.Origin + "/" + entry.Path
			}
			data, err := os.ReadFile(filepath.Join(dir, filepath.FromSlash(rel)))
			if err != nil {
				continue
			}
			for _, diag := range session.CheckDocument(rel, data) {
				if diag.Code == "syntax" {
					continue
				}
				step.Issues = append(step.Issues, PreflightIssue{Level: diag.Severity, Path: rel, Message: diag.Message})
			}
		}
	}
	for _, diag := range registry.CheckKubeJSWithNode(dir) {
		step.Issues = append(step.Issues, PreflightIssue{Level: diag.Severity, Path: "kubejs", Message: diag.Message})
	}
	return step
}

func checkablePreflightPath(path string) bool {
	for _, suffix := range []string{".json", ".mcmeta", ".toml", ".js", ".ts", ".mcfunction"} {
		if strings.HasSuffix(path, suffix) {
			return true
		}
	}
	return false
}

func printPreflight(result PreflightResult) {
	for _, step := range result.Steps {
		lines := []string{fmt.Sprintf("%d error(s) · %d warning(s)", step.Errors, step.Warnings)}
		for _, issue := range step.Issues {
			location := issue.Path
			if location != "" {
				location += ": "
			}
			lines = append(lines, fmt.Sprintf("%s %s%s", strings.ToUpper(issue.Level), location, issue.Message))
		}
		if cmd.Interactive() {
			cmd.Boxed("preflight/"+step.Name, lines)
			continue
		}
		fmt.Printf("preflight %s: %d error(s), %d warning(s)\n", step.Name, step.Errors, step.Warnings)
		for _, line := range lines[1:] {
			fmt.Println("  " + line)
		}
	}
	verdict := "PASS"
	if result.Errors > 0 {
		verdict = "FAIL"
	}
	fmt.Printf("preflight %s — %d error(s), %d warning(s)\n", verdict, result.Errors, result.Warnings)
}
