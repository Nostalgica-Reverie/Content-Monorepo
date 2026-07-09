package cmd

import (
	"strings"
	"testing"
)

func TestTablePlainOutputHasStableColumnsAndNoANSI(t *testing.T) {
	t.Setenv("PACKWAND_PLAIN", "1")
	output := Table([]string{"NAME", "SIDE"}, [][]string{{"Example", "both"}, {"A", "client"}})
	if strings.Contains(output, "\x1b[") {
		t.Fatalf("plain table contains ANSI: %q", output)
	}
	if !strings.Contains(output, "NAME") || !strings.Contains(output, "Example") {
		t.Fatalf("missing table values: %q", output)
	}
}
