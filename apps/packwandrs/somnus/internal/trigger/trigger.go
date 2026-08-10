package trigger

import (
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/schema"
)

func CurrentBranch(root string) string {
	output, err := exec.Command("git", "-C", root, "branch", "--show-current").Output()
	if err != nil {
		return ""
	}
	return strings.TrimSpace(string(output))
}

func Matches(conditions []schema.Condition, branch string, changedPaths []string) bool {
	for _, condition := range conditions {
		if contains(condition.Event, "manual") {
			return true
		}
		// A local checkout has no upstream tag event. Tag-only release workflows
		// remain discoverable but must be selected explicitly to run locally.
		if len(condition.Tag) > 0 {
			continue
		}
		if len(condition.Branch) > 0 && !matchesAny(condition.Branch, branch) {
			continue
		}
		if len(condition.Paths) == 0 || pathsMatch(condition.Paths, changedPaths) {
			return true
		}
	}
	return false
}

func pathsMatch(patterns, paths []string) bool {
	for _, path := range paths {
		if matchesAny(patterns, filepath.ToSlash(path)) {
			return true
		}
	}
	return false
}

func matchesAny(patterns []string, value string) bool {
	for _, pattern := range patterns {
		if glob(pattern, value) {
			return true
		}
	}
	return false
}

func glob(pattern, value string) bool {
	pattern = filepath.ToSlash(pattern)
	var expression strings.Builder
	expression.WriteByte('^')
	for index := 0; index < len(pattern); {
		switch {
		case strings.HasPrefix(pattern[index:], "**/"):
			expression.WriteString("(?:.*/)?")
			index += 3
		case strings.HasPrefix(pattern[index:], "**"):
			expression.WriteString(".*")
			index += 2
		case pattern[index] == '*':
			expression.WriteString("[^/]*")
			index++
		case pattern[index] == '?':
			expression.WriteString("[^/]")
			index++
		default:
			expression.WriteString(regexp.QuoteMeta(pattern[index : index+1]))
			index++
		}
	}
	expression.WriteByte('$')
	return regexp.MustCompile(expression.String()).MatchString(filepath.ToSlash(value))
}

func contains(values []string, wanted string) bool {
	for _, value := range values {
		if value == wanted {
			return true
		}
	}
	return false
}
