// Package characterization freezes observable behavior of the packwand
// binary against synthetic fixture packs (packwandrs.md phase 4). These
// tests run the real CLI the way a user does; a future Rust port of the
// refresh subsystem must reproduce these results byte for byte before it
// can become authoritative.
//
// Fixture packs are always generated in temp directories; the checked-in
// packs of this repository are never touched.
package characterization

import (
	"crypto/sha512"
	"encoding/hex"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/BurntSushi/toml"
)

var packwandBin string

func TestMain(m *testing.M) {
	tmp, err := os.MkdirTemp("", "packwand-characterization")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	defer os.RemoveAll(tmp)
	packwandBin = filepath.Join(tmp, "packwand")
	if runtime.GOOS == "windows" {
		packwandBin += ".exe"
	}
	build := exec.Command("go", "build", "-o", packwandBin, ".")
	build.Dir = ".." // module root
	if out, err := build.CombinedOutput(); err != nil {
		fmt.Fprintf(os.Stderr, "building packwand: %v\n%s", err, out)
		os.Exit(1)
	}
	os.Exit(m.Run())
}

// runRefresh runs `packwand refresh` in dir and returns stdout+stderr.
func runRefresh(t *testing.T, dir string, args ...string) string {
	t.Helper()
	return runPackwand(t, dir, append([]string{"refresh"}, args...)...)
}

func runPackwand(t *testing.T, dir string, args ...string) string {
	t.Helper()
	cmd := exec.Command(packwandBin, args...)
	cmd.Dir = dir
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("packwand %s failed: %v\n%s", strings.Join(args, " "), err, out)
	}
	return string(out)
}

func write(t *testing.T, dir, name, content string) {
	t.Helper()
	path := filepath.Join(dir, name)
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}
}

func read(t *testing.T, dir, name string) []byte {
	t.Helper()
	bytes, err := os.ReadFile(filepath.Join(dir, name))
	if err != nil {
		t.Fatal(err)
	}
	return bytes
}

func sha512hex(data []byte) string {
	sum := sha512.Sum512(data)
	return hex.EncodeToString(sum[:])
}

const settingsContent = "fixture settings\nline two\n"

const metafileContent = `name = "Example Mod"
filename = "example-mod.jar"
side = "both"

[download]
url = "https://example.invalid/example-mod.jar"
hash-format = "sha512"
hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
`

// writeFixturePack lays out a minimal but representative pack.
func writeFixturePack(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	write(t, dir, "pack.toml", `name = "Characterization Fixture"
author = "packwand tests"
version = "1.0.0"
pack-format = "packwand:26"

[index]
file = "index.toml"
hash-format = "sha512"
hash = ""

[versions]
minecraft = "1.21.1"
`)
	write(t, dir, "index.toml", "hash-format = \"sha512\"\n")
	write(t, dir, "config/settings.txt", settingsContent)
	write(t, dir, "mods/example.pw.toml", metafileContent)
	write(t, dir, "ignored.txt", "must never be indexed\n")
	write(t, dir, ".packwizignore", "ignored.txt\n")
	return dir
}

type indexFile struct {
	File     string `toml:"file"`
	Hash     string `toml:"hash"`
	Metafile bool   `toml:"metafile"`
}

type indexDoc struct {
	HashFormat string      `toml:"hash-format"`
	Files      []indexFile `toml:"files"`
}

func decodeIndex(t *testing.T, dir string) indexDoc {
	t.Helper()
	var doc indexDoc
	if _, err := toml.Decode(string(read(t, dir, "index.toml")), &doc); err != nil {
		t.Fatalf("decoding index.toml: %v", err)
	}
	return doc
}

func findFile(doc indexDoc, name string) *indexFile {
	for i := range doc.Files {
		if doc.Files[i].File == name {
			return &doc.Files[i]
		}
	}
	return nil
}

func TestRefreshIndexesExpectedFiles(t *testing.T) {
	dir := writeFixturePack(t)
	out := runRefresh(t, dir)
	if !strings.Contains(out, "Index refreshed:") {
		t.Errorf("summary line missing from output:\n%s", out)
	}

	doc := decodeIndex(t, dir)
	if doc.HashFormat != "sha512" {
		t.Errorf("index hash-format = %q, want sha512", doc.HashFormat)
	}
	settings := findFile(doc, "config/settings.txt")
	if settings == nil {
		t.Fatalf("config/settings.txt missing from index: %+v", doc.Files)
	}
	if settings.Metafile {
		t.Error("plain file must not be marked as metafile")
	}
	if expected := sha512hex([]byte(settingsContent)); settings.Hash != expected {
		t.Errorf("settings.txt hash = %s, want %s", settings.Hash, expected)
	}
	meta := findFile(doc, "mods/example.pw.toml")
	if meta == nil {
		t.Fatalf("mods/example.pw.toml missing from index: %+v", doc.Files)
	}
	if !meta.Metafile {
		t.Error("*.pw.toml file must be marked as metafile")
	}
	for _, excluded := range []string{"ignored.txt", "pack.toml", "index.toml"} {
		if findFile(doc, excluded) != nil {
			t.Errorf("%s must not be indexed", excluded)
		}
	}

	// Normal refreshes keep the generated index digest out of source metadata.
	var pack struct {
		Index struct {
			HashFormat string `toml:"hash-format"`
			Hash       string `toml:"hash"`
		} `toml:"index"`
	}
	if _, err := toml.Decode(string(read(t, dir, "pack.toml")), &pack); err != nil {
		t.Fatal(err)
	}
	if pack.Index.Hash != "" {
		t.Errorf("pack.toml source index hash = %s, want it omitted", pack.Index.Hash)
	}
}

func TestBuildRefreshWritesDistributionHash(t *testing.T) {
	dir := writeFixturePack(t)
	runRefresh(t, dir, "--build")

	var pack struct {
		Index struct {
			Hash string `toml:"hash"`
		} `toml:"index"`
	}
	if _, err := toml.Decode(string(read(t, dir, "pack.toml")), &pack); err != nil {
		t.Fatal(err)
	}
	if expected := sha512hex(read(t, dir, "index.toml")); pack.Index.Hash != expected {
		t.Errorf("distribution index hash = %s, want %s", pack.Index.Hash, expected)
	}
}

func TestRefreshRegeneratesMissingIndex(t *testing.T) {
	dir := writeFixturePack(t)
	if err := os.Remove(filepath.Join(dir, "index.toml")); err != nil {
		t.Fatal(err)
	}
	runRefresh(t, dir)
	if findFile(decodeIndex(t, dir), "mods/example.pw.toml") == nil {
		t.Fatal("refresh did not regenerate the missing index")
	}
}

func TestIndexConsumerRegeneratesMissingIndexInMemory(t *testing.T) {
	dir := writeFixturePack(t)
	if err := os.Remove(filepath.Join(dir, "index.toml")); err != nil {
		t.Fatal(err)
	}
	out := runPackwand(t, dir, "list")
	if !strings.Contains(out, "Example Mod") {
		t.Fatalf("list did not consume a regenerated index:\n%s", out)
	}
	if _, err := os.Stat(filepath.Join(dir, "index.toml")); !os.IsNotExist(err) {
		t.Fatalf("read-only index consumer unexpectedly persisted index.toml: %v", err)
	}
}

// A second refresh over an unchanged tree must be byte-identical: the
// emitters are deterministic, and this is the property differential
// Go/Rust parity tests will lean on.
//
// Characterized quirk: the "updated" counter counts every re-indexed
// existing file, not only files whose hash actually changed, so a no-op
// refresh over this two-file fixture reports "~2 updated". A Rust port
// must either reproduce this or the owner must sign off on changing it.
func TestRefreshIsDeterministicAndIdempotent(t *testing.T) {
	dir := writeFixturePack(t)
	runRefresh(t, dir)
	firstIndex := read(t, dir, "index.toml")
	firstPack := read(t, dir, "pack.toml")

	out := runRefresh(t, dir)
	if !strings.Contains(out, "+0 added  ~2 updated  -0 removed") {
		t.Errorf("unexpected refresh summary:\n%s", out)
	}
	if string(read(t, dir, "index.toml")) != string(firstIndex) {
		t.Error("index.toml changed on a no-op refresh")
	}
	if string(read(t, dir, "pack.toml")) != string(firstPack) {
		t.Error("pack.toml changed on a no-op refresh")
	}
}

func TestRefreshTracksModificationsAndRemovals(t *testing.T) {
	dir := writeFixturePack(t)
	runRefresh(t, dir)

	// Modify: the entry's hash follows the new content. (The "updated"
	// counter reports all existing entries, so it is not asserted here;
	// see TestRefreshIsDeterministicAndIdempotent.)
	updated := settingsContent + "changed\n"
	write(t, dir, "config/settings.txt", updated)
	out := runRefresh(t, dir)
	if !strings.Contains(out, "+0 added") || !strings.Contains(out, "-0 removed") {
		t.Errorf("modification must not add or remove entries:\n%s", out)
	}
	settings := findFile(decodeIndex(t, dir), "config/settings.txt")
	if settings == nil {
		t.Fatal("settings.txt disappeared from index")
	}
	if expected := sha512hex([]byte(updated)); settings.Hash != expected {
		t.Errorf("updated hash = %s, want %s", settings.Hash, expected)
	}

	// Remove: the entry disappears.
	if err := os.Remove(filepath.Join(dir, "config", "settings.txt")); err != nil {
		t.Fatal(err)
	}
	out = runRefresh(t, dir)
	if !strings.Contains(out, "-1 removed") {
		t.Errorf("expected one removal, got:\n%s", out)
	}
	if findFile(decodeIndex(t, dir), "config/settings.txt") != nil {
		t.Error("removed file still present in index")
	}
}
