package registry

import (
	"bufio"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
)

// ProbeExport is one generated ProbeJS declaration file. ProbeJS writes these
// into kubejs/exported after a development launch; they are pack-local and
// therefore take precedence over the small builtin completion seed.
type ProbeExport struct {
	Path    string   `json:"path"`
	Symbols []string `json:"symbols"`
}

var probeSymbolRE = regexp.MustCompile(`\b(?:declare\s+(?:class|function|const|namespace)|interface|type)\s+([A-Za-z_$][A-Za-z0-9_$]*)`) //nolint:gochecknoglobals

// ProbeJSExports discovers generated declarations without assuming a specific
// ProbeJS layout, which changes between Minecraft/KubeJS generations.
func ProbeJSExports(dir string) ([]ProbeExport, error) {
	root := filepath.Join(dir, "kubejs", "exported")
	if !dirExists(root) {
		return []ProbeExport{}, nil
	}
	var out []ProbeExport
	err := filepath.WalkDir(root, func(full string, entry os.DirEntry, err error) error {
		if err != nil || entry.IsDir() || !(strings.HasSuffix(entry.Name(), ".d.ts") || strings.HasSuffix(entry.Name(), ".ts")) {
			return nil
		}
		file, err := os.Open(full)
		if err != nil {
			return nil
		}
		defer file.Close()
		symbols := []string{}
		scanner := bufio.NewScanner(file)
		for scanner.Scan() {
			for _, match := range probeSymbolRE.FindAllStringSubmatch(scanner.Text(), -1) {
				symbols = append(symbols, match[1])
			}
		}
		rel, _ := filepath.Rel(dir, full)
		out = append(out, ProbeExport{Path: filepath.ToSlash(rel), Symbols: symbols})
		return nil
	})
	sort.Slice(out, func(i, j int) bool { return out[i].Path < out[j].Path })
	return out, err
}
