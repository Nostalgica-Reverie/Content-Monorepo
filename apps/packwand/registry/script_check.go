package registry

import (
	"path"
	"regexp"
	"strings"
)

// checkFunctionDocument verifies that local mcfunction invocations resolve.
func checkFunctionDocument(cache *registryCache, rel string, content []byte) []Diagnostic {
	checker := &docChecker{registryCache: cache, content: content, diags: []Diagnostic{}}
	reg := checker.datapackRegistry()
	known := namespaces(reg)
	for _, match := range functionRefRE.FindAllStringSubmatch(string(content), -1) {
		id := match[1]
		ns, _ := splitRef(id)
		if known[ns] && !has(reg, id, "function") {
			checker.errorf("reference", id, "missing function reference %q", id)
		}
	}
	return checker.diags
}

var functionRefRE = regexp.MustCompile(`(?m)^\s*(?:execute\s+.*?\s+run\s+)?function\s+([a-z0-9_.-]+:[a-z0-9_./-]+)`) //nolint:gochecknoglobals

// checkKubeJSScript provides the phase-3 baseline without requiring Node:
// delimiter syntax and curated-event folder discipline. Full type checking can
// layer on top when ProbeJS dumps are available.
func checkKubeJSScript(cache *registryCache, rel string, content []byte) []Diagnostic {
	checker := &docChecker{registryCache: cache, content: content, diags: []Diagnostic{}}
	source := string(content)
	if !balancedJS(source) {
		checker.errorf("syntax", "", "unbalanced JavaScript delimiters")
	}
	folder := scriptFolder(rel)
	if folder == "" {
		checker.warnf("structure", rel, "KubeJS script is outside startup_scripts, server_scripts, or client_scripts")
		return checker.diags
	}
	reg, err := checker.kubejsRegistry()
	if err != nil {
		return checker.diags
	}
	for _, entry := range reg.Entries {
		if !strings.HasPrefix(entry.Kind, "event/") || !strings.Contains(source, entry.ID) {
			continue
		}
		expected := strings.TrimPrefix(entry.Kind, "event/")
		if expected != folder {
			checker.errorf("structure", entry.ID, "%s belongs in kubejs/%s_scripts, not %s_scripts", entry.ID, expected, folder)
		}
	}
	return checker.diags
}

func scriptFolder(rel string) string {
	parts := strings.Split(path.Clean(rel), "/")
	for i, part := range parts {
		if part == "kubejs" && i+1 < len(parts) {
			return strings.TrimSuffix(parts[i+1], "_scripts")
		}
	}
	return ""
}

func balancedJS(source string) bool {
	stack := make([]rune, 0, 16)
	quote, escaped := rune(0), false
	for _, r := range source {
		if quote != 0 {
			if escaped {
				escaped = false
				continue
			}
			if r == '\\' {
				escaped = true
				continue
			}
			if r == quote {
				quote = 0
			}
			continue
		}
		if r == '\'' || r == '"' || r == '`' {
			quote = r
			continue
		}
		switch r {
		case '(', '[', '{':
			stack = append(stack, r)
		case ')', ']', '}':
			if len(stack) == 0 || !matchesJS(stack[len(stack)-1], r) {
				return false
			}
			stack = stack[:len(stack)-1]
		}
	}
	return quote == 0 && len(stack) == 0
}

func matchesJS(open, close rune) bool {
	return open == '(' && close == ')' || open == '[' && close == ']' || open == '{' && close == '}'
}
