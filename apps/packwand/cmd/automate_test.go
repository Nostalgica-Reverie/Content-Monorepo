package cmd

import (
	"testing"
	"time"
)

func TestNextCalVerAt(t *testing.T) {
	june := time.Date(2026, time.June, 15, 0, 0, 0, 0, time.UTC)
	july := time.Date(2026, time.July, 1, 0, 0, 0, 0, time.UTC)

	cases := []struct {
		name    string
		current string
		now     time.Time
		want    string
	}{
		{"same cycle, bare -> patch 1", "26.06", june, "26.06.1"},
		{"same cycle, patch -> patch+1", "26.06.1", june, "26.06.2"},
		{"same cycle, double-digit patch", "26.06.9", june, "26.06.10"},
		{"new month -> bare cycle", "26.06.3", july, "26.07"},
		{"unrelated cycle -> bare cycle", "25.12", june, "26.06"},
		{"non-calver current -> bare cycle", "2.0.0-hotfix", june, "26.06"},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			got := nextCalVerAt(c.current, c.now)
			if got != c.want {
				t.Fatalf("nextCalVerAt(%q, %s) = %q, want %q", c.current, c.now, got, c.want)
			}
		})
	}
}
