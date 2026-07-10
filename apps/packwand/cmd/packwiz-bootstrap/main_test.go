package main

import (
	"fmt"
	"net/http"
	"net/http/httptest"
	"reflect"
	"strings"
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

func TestFetchChecksumParsing(t *testing.T) {
	bare := "a3f5" + strings.Repeat("0", 60)
	multi := bare + "  packwiz-installer.jar\n" +
		strings.Repeat("1", 64) + " *other.jar\n"

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/bare.sha256":
			fmt.Fprint(w, bare+"\n")
		case "/checksums.txt":
			fmt.Fprint(w, multi)
		default:
			http.NotFound(w, r)
		}
	}))
	defer srv.Close()

	if got := fetchChecksum(srv.URL+"/bare.sha256", "packwiz-installer.jar"); got != bare {
		t.Errorf("bare hash: got %q, want %q", got, bare)
	}
	if got := fetchChecksum(srv.URL+"/checksums.txt", "packwiz-installer.jar"); got != bare {
		t.Errorf("sha256sum lines: got %q, want %q", got, bare)
	}
	if got := fetchChecksum(srv.URL+"/checksums.txt", "other.jar"); got != strings.Repeat("1", 64) {
		t.Errorf("binary-mode marker: got %q", got)
	}
	if got := fetchChecksum(srv.URL+"/checksums.txt", "missing.jar"); got != "" {
		t.Errorf("missing entry should be empty, got %q", got)
	}
	if got := fetchChecksum(srv.URL+"/404", "x.jar"); got != "" {
		t.Errorf("404 should be empty, got %q", got)
	}
}

func TestExpectedSha256Precedence(t *testing.T) {
	bare := strings.Repeat("a", 64)
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/packwiz-installer.jar.sha256" {
			fmt.Fprint(w, bare)
			return
		}
		http.NotFound(w, r)
	}))
	defer srv.Close()

	// Explicit flag wins.
	h, src := expectedSha256(options{sha256: "ff", downloadURL: srv.URL + "/packwiz-installer.jar"})
	if h != "ff" || src != "--sha256" {
		t.Errorf("explicit flag: got %q from %q", h, src)
	}

	// Sibling .sha256 auto-probe.
	h, src = expectedSha256(options{downloadURL: srv.URL + "/packwiz-installer.jar"})
	if h != bare || src == "" {
		t.Errorf("sibling probe: got %q from %q", h, src)
	}

	// No source anywhere.
	h, _ = expectedSha256(options{downloadURL: srv.URL + "/elsewhere.jar"})
	if h != "" {
		t.Errorf("no source: got %q, want empty", h)
	}
}
