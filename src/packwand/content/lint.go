package content

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/manifest"
	"github.com/spf13/cobra"
)

func init() {
	contentLintCmd.Flags().Bool("all", false, "Lint every datapack and resourcepack in the workspace")
	contentLintCmd.Flags().Bool("json", false, "Output lint reports as JSON")
	cmd.AddToGroup(contentLintCmd, cmd.GroupInfo)
}

var contentLintCmd = &cobra.Command{
	Use:   "content-lint [pack-dir...]",
	Short: "Lint pack content — namespaces, texture/model refs, pack.mcmeta, function tags, duplicate and case-colliding files",
	Run: func(c *cobra.Command, args []string) {
		all, _ := c.Flags().GetBool("all")
		asJSON, _ := c.Flags().GetBool("json")

		targets := make([]string, len(args))
		for i, a := range args {
			targets[i] = cmd.Abs(a)
		}
		cmd.Chdir()

		if all {
			for _, root := range []string{"datapacks", "resourcepacks"} {
				entries, err := os.ReadDir(root)
				if err != nil {
					continue
				}
				for _, e := range entries {
					if !e.IsDir() {
						continue
					}
					p := filepath.Join(root, e.Name())
					if _, err := os.Stat(filepath.Join(p, "manifest.json")); err == nil {
						targets = append(targets, p)
					}
				}
			}
		}
		if len(targets) == 0 {
			cmd.Fail("provide pack dir(s) or use --all")
		}

		var reports []LintResult
		errorsTotal := 0
		for _, t := range targets {
			rep := lintPack(t)
			reports = append(reports, rep)
			errorsTotal += rep.Errors
		}

		if asJSON {
			data, _ := json.MarshalIndent(reports, "", "  ")
			fmt.Println(string(data))
		} else {
			for _, rep := range reports {
				printLintReport(rep)
			}
			fmt.Printf("linted %d pack(s)\n", len(reports))
		}
		if errorsTotal > 0 {
			cmd.Fail(fmt.Sprintf("content lint found %d error(s)", errorsTotal))
		}
	},
}

type LintIssue struct {
	Level   string `json:"level"` // "error" or "warning"
	Path    string `json:"path,omitempty"`
	Message string `json:"message"`
}

type LintResult struct {
	Pack     string      `json:"pack"`
	Type     string      `json:"type"`
	Issues   []LintIssue `json:"issues"`
	Errors   int         `json:"errors"`
	Warnings int         `json:"warnings"`
	Files    int         `json:"files_scanned"`
}

func (r *LintResult) errorf(path, format string, a ...any) {
	r.Issues = append(r.Issues, LintIssue{Level: "error", Path: path, Message: fmt.Sprintf(format, a...)})
	r.Errors++
}

func (r *LintResult) warnf(path, format string, a ...any) {
	r.Issues = append(r.Issues, LintIssue{Level: "warning", Path: path, Message: fmt.Sprintf(format, a...)})
	r.Warnings++
}

func printLintReport(rep LintResult) {
	if cmd.Interactive() {
		lines := []string{fmt.Sprintf("%d files · %d errors · %d warnings", rep.Files, rep.Errors, rep.Warnings)}
		for _, issue := range rep.Issues {
			location := issue.Path
			if location != "" {
				location += ": "
			}
			lines = append(lines, fmt.Sprintf("%s %s%s", strings.ToUpper(issue.Level), location, issue.Message))
		}
		cmd.Boxed(rep.Pack+" ["+rep.Type+"]", lines)
		return
	}
	status := "OK"
	if rep.Errors > 0 {
		status = fmt.Sprintf("%d error(s), %d warning(s)", rep.Errors, rep.Warnings)
	} else if rep.Warnings > 0 {
		status = fmt.Sprintf("%d warning(s)", rep.Warnings)
	}
	fmt.Printf("%s (%s): %d file(s) — %s\n", rep.Pack, rep.Type, rep.Files, status)
	for _, is := range rep.Issues {
		marker := "warn "
		if is.Level == "error" {
			marker = "ERROR"
		}
		if is.Path != "" {
			fmt.Printf("  %s %s: %s\n", marker, is.Path, is.Message)
		} else {
			fmt.Printf("  %s %s\n", marker, is.Message)
		}
	}
}

// resourceSegmentRe is the character set Minecraft allows in resource
// locations; anything else (notably uppercase) breaks on case-sensitive
// filesystems or fails to load outright.
var resourceSegmentRe = regexp.MustCompile(`^[a-z0-9_.-]+$`)

func lintPack(packDir string) LintResult {
	rep := LintResult{Pack: packDir, Issues: []LintIssue{}}

	m, err := manifest.Read(filepath.Join(packDir, "manifest.json"))
	if err != nil {
		rep.errorf("manifest.json", "unreadable manifest: %v", err)
		return rep
	}
	rep.Type = m.Type
	if m.Type != "datapack" && m.Type != "resourcepack" {
		rep.warnf("", "content lint supports datapacks and resourcepacks (got '%s'); skipped", m.Type)
		return rep
	}

	roots, err := findContentRoots(packDir)
	if err != nil {
		rep.errorf("", "%v", err)
		return rep
	}

	for _, contentRoot := range roots {
		start := len(rep.Issues)
		lintContentRoot(m.Type, contentRoot, &rep)
		if len(roots) > 1 {
			prefix := filepath.Base(contentRoot) + "/"
			for i := start; i < len(rep.Issues); i++ {
				if rep.Issues[i].Path != "" {
					rep.Issues[i].Path = prefix + rep.Issues[i].Path
				}
			}
		}
	}

	sort.SliceStable(rep.Issues, func(i, j int) bool { return rep.Issues[i].Path < rep.Issues[j].Path })
	return rep
}

func lintContentRoot(packType, contentRoot string, rep *LintResult) {
	files := collectContentFiles(contentRoot, rep)
	rep.Files += len(files)

	lintPackMcmeta(contentRoot, rep)
	lintPathCharsets(files, rep)
	lintCaseCollisions(files, rep)
	lintJSONParses(contentRoot, files, rep)
	lintDuplicateFiles(contentRoot, files, rep)
	lintNamespaces(files, rep)

	if packType == "datapack" && !hasTopDir(files, "data") {
		rep.errorf("", "datapack has no data/ directory under %s", contentRoot)
	}
	if packType == "resourcepack" && !hasTopDir(files, "assets") {
		rep.errorf("", "resourcepack has no assets/ directory under %s", contentRoot)
	}

	lintModelReferences(contentRoot, files, rep)
	lintFunctionTags(contentRoot, files, rep)
}

// findContentRoots locates the directories holding pack.mcmeta/data/assets:
// resourcepacks keep content at the pack root, datapacks nest it inside a
// version directory, and some packs ship several version directories.
func findContentRoots(packDir string) ([]string, error) {
	if _, err := os.Stat(filepath.Join(packDir, "pack.mcmeta")); err == nil {
		return []string{packDir}, nil
	}
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read %s: %v", packDir, err)
	}
	var candidates []string
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		sub := filepath.Join(packDir, e.Name())
		if _, err := os.Stat(filepath.Join(sub, "pack.mcmeta")); err == nil {
			candidates = append(candidates, sub)
			continue
		}
		if _, err := os.Stat(filepath.Join(sub, "data")); err == nil {
			candidates = append(candidates, sub)
		}
	}
	if len(candidates) == 0 {
		return nil, fmt.Errorf("no content root found in %s (no pack.mcmeta at root or in a version directory)", packDir)
	}
	return candidates, nil
}

// collectContentFiles returns all regular files under root as slash-separated
// paths relative to root.
func collectContentFiles(root string, rep *LintResult) []string {
	var files []string
	err := filepath.WalkDir(root, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			rep.errorf(p, "walk error: %v", err)
			return nil
		}
		if d.IsDir() {
			return nil
		}
		rel, err := filepath.Rel(root, p)
		if err != nil {
			return nil
		}
		files = append(files, filepath.ToSlash(rel))
		return nil
	})
	if err != nil {
		rep.errorf(root, "walk failed: %v", err)
	}
	sort.Strings(files)
	return files
}

func hasTopDir(files []string, dir string) bool {
	prefix := dir + "/"
	for _, f := range files {
		if strings.HasPrefix(f, prefix) {
			return true
		}
	}
	return false
}

func lintPackMcmeta(contentRoot string, rep *LintResult) {
	p := filepath.Join(contentRoot, "pack.mcmeta")
	data, err := os.ReadFile(p)
	if err != nil {
		rep.errorf("pack.mcmeta", "missing at content root %s", contentRoot)
		return
	}
	var mcmeta struct {
		Pack *struct {
			PackFormat  *json.Number `json:"pack_format"`
			Description any          `json:"description"`
		} `json:"pack"`
	}
	dec := json.NewDecoder(strings.NewReader(string(data)))
	dec.UseNumber()
	if err := dec.Decode(&mcmeta); err != nil {
		rep.errorf("pack.mcmeta", "invalid JSON: %v", err)
		return
	}
	if mcmeta.Pack == nil {
		rep.errorf("pack.mcmeta", "missing 'pack' object")
		return
	}
	if mcmeta.Pack.PackFormat == nil {
		rep.errorf("pack.mcmeta", "missing 'pack.pack_format'")
	}
	if mcmeta.Pack.Description == nil {
		rep.warnf("pack.mcmeta", "missing 'pack.description'")
	}
}

// lintPathCharsets flags path segments under data/ and assets/ that violate
// the resource-location character set (uppercase letters, spaces, etc.).
func lintPathCharsets(files []string, rep *LintResult) {
	for _, f := range files {
		if !strings.HasPrefix(f, "data/") && !strings.HasPrefix(f, "assets/") {
			continue
		}
		for _, seg := range strings.Split(f, "/") {
			if !resourceSegmentRe.MatchString(seg) {
				rep.errorf(f, "path segment %q violates resource-location charset [a-z0-9_.-] (breaks on case-sensitive filesystems)", seg)
				break
			}
		}
	}
}

// lintCaseCollisions flags files whose paths collide when compared
// case-insensitively — such packs silently lose files on Windows/macOS.
func lintCaseCollisions(files []string, rep *LintResult) {
	byLower := map[string][]string{}
	for _, f := range files {
		l := strings.ToLower(f)
		byLower[l] = append(byLower[l], f)
	}
	for _, group := range byLower {
		if len(group) > 1 {
			rep.errorf(group[0], "case-insensitive path collision: %s", strings.Join(group, " <-> "))
		}
	}
}

func lintJSONParses(contentRoot string, files []string, rep *LintResult) {
	for _, f := range files {
		if !strings.HasSuffix(f, ".json") && !strings.HasSuffix(f, ".mcmeta") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(contentRoot, filepath.FromSlash(f)))
		if err != nil {
			rep.errorf(f, "unreadable: %v", err)
			continue
		}
		var v any
		if err := json.Unmarshal(data, &v); err != nil {
			rep.errorf(f, "invalid JSON: %v", err)
		}
	}
}

// lintDuplicateFiles warns about byte-identical files (size-grouped first,
// then hashed) — usually a copy-paste leftover that bloats the zip.
func lintDuplicateFiles(contentRoot string, files []string, rep *LintResult) {
	bySize := map[int64][]string{}
	for _, f := range files {
		info, err := os.Stat(filepath.Join(contentRoot, filepath.FromSlash(f)))
		if err != nil || info.Size() == 0 {
			continue
		}
		bySize[info.Size()] = append(bySize[info.Size()], f)
	}
	for _, group := range bySize {
		if len(group) < 2 {
			continue
		}
		byHash := map[string][]string{}
		for _, f := range group {
			data, err := os.ReadFile(filepath.Join(contentRoot, filepath.FromSlash(f)))
			if err != nil {
				continue
			}
			sum := sha256.Sum256(data)
			h := hex.EncodeToString(sum[:])
			byHash[h] = append(byHash[h], f)
		}
		for _, dupes := range byHash {
			if len(dupes) > 1 {
				sort.Strings(dupes)
				rep.warnf(dupes[0], "duplicate content: %s", strings.Join(dupes, " == "))
			}
		}
	}
}

func lintNamespaces(files []string, rep *LintResult) {
	seen := map[string]bool{}
	for _, f := range files {
		parts := strings.Split(f, "/")
		if len(parts) < 2 || (parts[0] != "data" && parts[0] != "assets") {
			continue
		}
		ns := parts[0] + "/" + parts[1]
		if seen[ns] {
			continue
		}
		seen[ns] = true
		if !resourceSegmentRe.MatchString(parts[1]) {
			rep.errorf(ns, "invalid namespace %q (must match [a-z0-9_.-])", parts[1])
		}
	}
}

// packNamespaces returns the namespaces present under the given top dir
// ("data" or "assets").
func packNamespaces(files []string, top string) map[string]bool {
	ns := map[string]bool{}
	for _, f := range files {
		parts := strings.Split(f, "/")
		if len(parts) >= 2 && parts[0] == top {
			ns[parts[1]] = true
		}
	}
	return ns
}

// splitResourceLocation splits "ns:path" (default namespace "minecraft").
func splitResourceLocation(ref string) (ns, p string) {
	if i := strings.IndexByte(ref, ':'); i >= 0 {
		return ref[:i], ref[i+1:]
	}
	return "minecraft", ref
}

func fileSet(files []string) map[string]bool {
	s := make(map[string]bool, len(files))
	for _, f := range files {
		s[f] = true
	}
	return s
}

// lintModelReferences checks that texture and parent-model references in
// model JSON files resolve within the pack. References into namespaces the
// pack does not ship (vanilla or other packs) are skipped.
func lintModelReferences(contentRoot string, files []string, rep *LintResult) {
	assetNS := packNamespaces(files, "assets")
	if len(assetNS) == 0 {
		return
	}
	have := fileSet(files)

	checkRef := func(src, ref, kind string) {
		if ref == "" || strings.HasPrefix(ref, "#") {
			return
		}
		ns, p := splitResourceLocation(ref)
		if ns == "minecraft" || !assetNS[ns] {
			return
		}
		var target string
		switch kind {
		case "texture":
			target = path.Join("assets", ns, "textures", p+".png")
		case "model":
			target = path.Join("assets", ns, "models", p+".json")
		}
		if !have[target] {
			rep.errorf(src, "missing %s reference %q (expected %s)", kind, ref, target)
		}
	}

	for _, f := range files {
		parts := strings.Split(f, "/")
		if len(parts) < 4 || parts[0] != "assets" || parts[2] != "models" || !strings.HasSuffix(f, ".json") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(contentRoot, filepath.FromSlash(f)))
		if err != nil {
			continue
		}
		var model struct {
			Parent   string            `json:"parent"`
			Textures map[string]string `json:"textures"`
		}
		if json.Unmarshal(data, &model) != nil {
			continue // parse errors are reported by lintJSONParses
		}
		checkRef(f, model.Parent, "model")
		for _, tex := range model.Textures {
			checkRef(f, tex, "texture")
		}
	}
}

// lintFunctionTags checks that datapack function tags reference functions
// (or nested tags) that exist within the pack. Both the modern singular
// ("function") and legacy plural ("functions") layouts are accepted.
func lintFunctionTags(contentRoot string, files []string, rep *LintResult) {
	dataNS := packNamespaces(files, "data")
	if len(dataNS) == 0 {
		return
	}
	have := fileSet(files)

	exists := func(candidates ...string) bool {
		for _, c := range candidates {
			if have[c] {
				return true
			}
		}
		return false
	}

	for _, f := range files {
		parts := strings.Split(f, "/")
		if len(parts) < 5 || parts[0] != "data" || parts[2] != "tags" || !strings.HasSuffix(f, ".json") {
			continue
		}
		if parts[3] != "function" && parts[3] != "functions" {
			continue
		}
		data, err := os.ReadFile(filepath.Join(contentRoot, filepath.FromSlash(f)))
		if err != nil {
			continue
		}
		var tag struct {
			Values []any `json:"values"`
		}
		if json.Unmarshal(data, &tag) != nil {
			continue // parse errors are reported by lintJSONParses
		}
		if tag.Values == nil {
			rep.errorf(f, "function tag has no 'values' array")
			continue
		}
		for _, v := range tag.Values {
			var ref string
			var required = true
			switch val := v.(type) {
			case string:
				ref = val
			case map[string]any:
				ref, _ = val["id"].(string)
				if req, ok := val["required"].(bool); ok {
					required = req
				}
			default:
				rep.errorf(f, "function tag value has unexpected type %T", v)
				continue
			}
			if ref == "" {
				rep.errorf(f, "function tag value missing id")
				continue
			}

			isTagRef := strings.HasPrefix(ref, "#")
			ns, p := splitResourceLocation(strings.TrimPrefix(ref, "#"))
			if !dataNS[ns] {
				continue // external/vanilla namespace; cannot verify
			}
			var ok bool
			if isTagRef {
				ok = exists(
					path.Join("data", ns, "tags", "function", p+".json"),
					path.Join("data", ns, "tags", "functions", p+".json"),
				)
			} else {
				ok = exists(
					path.Join("data", ns, "function", p+".mcfunction"),
					path.Join("data", ns, "functions", p+".mcfunction"),
				)
			}
			if !ok && required {
				kind := "function"
				if isTagRef {
					kind = "function tag"
				}
				rep.errorf(f, "missing %s reference %q", kind, ref)
			}
		}
	}
}
