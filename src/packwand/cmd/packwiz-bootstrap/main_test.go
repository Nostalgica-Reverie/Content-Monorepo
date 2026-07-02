package main

import (
	"reflect"
	"testing"
)

func TestParseJavaMajor(t *testing.T) {
	tests := []struct {
		output string
		want   int
	}{
		{`openjdk version "25.0.1" 2025-10-21 LTS`, 25},
		{`openjdk version "17.0.2" 2022-01-18`, 17},
		{`java version "1.8.0_392"`, 8},
		{`openjdk version "21-ea" 2023-09-19`, 21},
	}
	for _, test := range tests {
		got, err := parseJavaMajor(test.output)
		if err != nil {
			t.Errorf("parseJavaMajor(%q): %v", test.output, err)
			continue
		}
		if got != test.want {
			t.Errorf("parseJavaMajor(%q) = %d, want %d", test.output, got, test.want)
		}
	}
	if _, err := parseJavaMajor("no version here"); err == nil {
		t.Error("expected an error for unparsable output")
	}
}

func TestParseArgsSplitsBootstrapAndPassthrough(t *testing.T) {
	opts, err := parseArgs([]string{
		"--jar", "custom.jar",
		"--min-java", "17",
		"-g",
		"-s", "server",
		"https://example.com/pack.toml",
	})
	if err != nil {
		t.Fatal(err)
	}
	if opts.jar != "custom.jar" || opts.minJava != 17 {
		t.Fatalf("bootstrap options not parsed: %+v", opts)
	}
	want := []string{"-g", "-s", "server", "https://example.com/pack.toml"}
	if !reflect.DeepEqual(opts.passthrough, want) {
		t.Fatalf("passthrough = %v, want %v", opts.passthrough, want)
	}
}

func TestParseArgsRejectsMissingValue(t *testing.T) {
	if _, err := parseArgs([]string{"--jar"}); err == nil {
		t.Fatal("expected an error for --jar without a value")
	}
}
