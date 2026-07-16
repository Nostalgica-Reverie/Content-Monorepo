package registry

import (
	"fmt"
	"os"
	"regexp"
	"sort"
	"strings"
)

const (
	defaultCompleteLimit = 50
	maxCompleteLimit     = 200
	// maxInferFileSize bounds how much of a document InferFromFile reads;
	// pack JSON and scripts are far smaller in practice.
	maxInferFileSize = 4 << 20
)

// Complete returns the entries matching query, best matches first: ID-prefix
// matches, then path-part (after the namespace colon) prefix matches, then
// substring matches. An empty query returns entries in ID order. kinds
// filters by entry kind; a filter matches exactly or as a "kind/" group
// prefix, so "tag" matches "tag/function". A nil kinds matches everything.
func (r *Registry) Complete(query string, kinds []string, limit int) []Entry {
	if limit <= 0 {
		limit = defaultCompleteLimit
	}
	if limit > maxCompleteLimit {
		limit = maxCompleteLimit
	}
	needle := strings.ToLower(query)
	type match struct {
		entry Entry
		rank  int
	}
	var matches []match
	for _, entry := range r.Entries {
		if !kindMatches(entry.Kind, kinds) {
			continue
		}
		rank := matchRank(strings.ToLower(entry.ID), needle)
		if rank < 0 {
			continue
		}
		matches = append(matches, match{entry, rank})
	}
	sort.SliceStable(matches, func(i, j int) bool {
		if matches[i].rank != matches[j].rank {
			return matches[i].rank < matches[j].rank
		}
		return matches[i].entry.ID < matches[j].entry.ID
	})
	if len(matches) > limit {
		matches = matches[:limit]
	}
	out := make([]Entry, len(matches))
	for i, m := range matches {
		out[i] = m.entry
	}
	return out
}

func kindMatches(kind string, filters []string) bool {
	if len(filters) == 0 {
		return true
	}
	for _, filter := range filters {
		if kind == filter || strings.HasPrefix(kind, filter+"/") {
			return true
		}
	}
	return false
}

func matchRank(id, needle string) int {
	switch {
	case needle == "":
		return 0
	case strings.HasPrefix(id, needle):
		return 0
	case pathPartHasPrefix(id, needle):
		return 1
	case strings.Contains(id, needle):
		return 2
	}
	return -1
}

// pathPartHasPrefix reports whether the part after the namespace colon
// starts with needle, so "cobbled" completes "kubejs:andesite/cobbled_...".
func pathPartHasPrefix(id, needle string) bool {
	if i := strings.IndexByte(id, ':'); i >= 0 {
		return strings.HasPrefix(id[i+1:], needle)
	}
	return false
}

// tokenChars are the characters that can appear in a reference being typed:
// resource locations, file paths, tag refs, and dotted event names.
func isTokenChar(c byte) bool {
	switch {
	case c >= 'a' && c <= 'z', c >= 'A' && c <= 'Z', c >= '0' && c <= '9':
		return true
	case c == '_', c == ':', c == '.', c == '/', c == '#', c == '-':
		return true
	}
	return false
}

// keyRe captures the JSON key immediately preceding the value being typed.
var keyRe = regexp.MustCompile(`"([A-Za-z0-9_]+)"\s*:\s*"?$`)

// keyKinds maps a JSON key to the entry kinds its value can reference. This
// is a heuristic stand-in for full schema-position resolution (IDE.md §4.2);
// unknown keys return no filter so completion falls back to all kinds.
var keyKinds = map[string][]string{
	"parent":   {"model"},
	"model":    {"model"},
	"texture":  {"texture"},
	"textures": {"texture"},
	"particle": {"texture"},
	"function": {"function"},
	"values":   {"function", "tag/function"},
}

// InferFromFile extracts the token being typed at byte offset in the file at
// path and infers which entry kinds can appear there, so completion works
// without the editor sending explicit filters. A leading '#' (tag reference)
// is stripped from the returned query and narrows kinds to tags.
func InferFromFile(path string, offset int) (query string, kinds []string, err error) {
	info, err := os.Stat(path)
	if err != nil {
		return "", nil, err
	}
	if info.Size() > maxInferFileSize {
		return "", nil, fmt.Errorf("%s is too large for completion context (%d bytes)", path, info.Size())
	}
	data, err := os.ReadFile(path)
	if err != nil {
		return "", nil, err
	}
	if offset < 0 || offset > len(data) {
		return "", nil, fmt.Errorf("offset %d is out of range for %s (%d bytes)", offset, path, len(data))
	}
	start := offset
	for start > 0 && isTokenChar(data[start-1]) {
		start--
	}
	query = string(data[start:offset])

	if strings.HasPrefix(query, "#") {
		return strings.TrimPrefix(query, "#"), []string{"tag"}, nil
	}
	prefixStart := start - 256
	if prefixStart < 0 {
		prefixStart = 0
	}
	if m := keyRe.FindSubmatch(data[prefixStart:start]); m != nil {
		kinds = keyKinds[string(m[1])]
	}
	return query, kinds, nil
}
