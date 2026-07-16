package registry

import (
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
)

var kubeLogLocationRE = regexp.MustCompile(`(?m)(?:\bat\s+)?(?:.*?)(kubejs[\\/][^:\s)]+\.(?:js|ts)):(\d+)(?::(\d+))?`) //nolint:gochecknoglobals

// ParseKubeJSLog translates KubeJS/Rhino stack locations into Problems
// diagnostics. It accepts complete log payloads or incremental tailed chunks.
func ParseKubeJSLog(content string) []Diagnostic {
	var out []Diagnostic
	for _, match := range kubeLogLocationRE.FindAllStringSubmatch(content, -1) {
		line, col := parsePositive(match[2]), parsePositive(match[3])
		path := strings.ReplaceAll(match[1], "\\", "/")
		out = append(out, Diagnostic{Severity: "error", Line: line, Col: col, Code: "runtime", Message: "KubeJS runtime error in " + path})
	}
	return out
}

func parsePositive(value string) int {
	n := 0
	for _, r := range value {
		if r >= '0' && r <= '9' {
			n = n*10 + int(r-'0')
		}
	}
	if n < 1 {
		return 1
	}
	return n
}

// CheckKubeJSWithNode invokes the installed Node runtime for authoritative JS
// parsing. ProbeJS exports are discovered separately and returned as a count;
// Node's --check intentionally needs no npm dependency or network install.
func CheckKubeJSWithNode(dir string) []Diagnostic {
	reg, err := Build(dir, KubeJS)
	if err != nil {
		return []Diagnostic{{Severity: "error", Line: 1, Col: 1, Code: "kubejs", Message: err.Error()}}
	}
	var out []Diagnostic
	for _, entry := range reg.Entries {
		if !strings.HasPrefix(entry.Kind, "script/") || !strings.HasSuffix(entry.Path, ".js") {
			continue
		}
		full := filepath.Join(dir, filepath.FromSlash(entry.Origin), filepath.FromSlash(entry.Path))
		cmd := exec.Command("node", "--check", full)
		output, checkErr := cmd.CombinedOutput()
		if checkErr == nil {
			continue
		}
		message := strings.TrimSpace(string(output))
		if message == "" {
			message = checkErr.Error()
		}
		out = append(out, Diagnostic{Severity: "error", Line: 1, Col: 1, Code: "syntax", Message: message})
	}
	return out
}

// ReadKubeJSLogs gathers runtime diagnostics from the standard log location.
func ReadKubeJSLogs(dir string) []Diagnostic {
	root := filepath.Join(dir, "logs", "kubejs")
	entries, _ := os.ReadDir(root)
	var out []Diagnostic
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".log") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(root, entry.Name()))
		if err == nil {
			out = append(out, ParseKubeJSLog(string(data))...)
		}
	}
	return out
}
