package modrinth

import (
	"testing"

	modrinthApi "codeberg.org/jmansfield/go-modrinth/modrinth"
)

func TestParseSlugOrUrl(t *testing.T) {
	for _, tc := range []struct {
		name       string
		input      string
		wantSlug   string
		wantVer    string
		wantVerID  string
		wantFile   string
		parsedSlug bool
		wantErr    bool
	}{
		{name: "project URL", input: "https://modrinth.com/mod/sodium", wantSlug: "sodium"},
		{name: "project URL with version", input: "https://modrinth.com/mod/sodium/version/mc1.21-0.6.0", wantSlug: "sodium", wantVer: "mc1.21-0.6.0"},
		{name: "unknown category", input: "https://modrinth.com/notathing/sodium", wantErr: true},
		{name: "CDN URL", input: "https://cdn.modrinth.com/data/AANobbMI/versions/YAGZ1cCS/sodium-fabric-0.6.0.jar", wantSlug: "AANobbMI", wantVerID: "YAGZ1cCS", wantFile: "sodium-fabric-0.6.0.jar"},
		{name: "bare slug", input: "sodium", wantSlug: "sodium", parsedSlug: true},
		{name: "not matching", input: "!", wantSlug: ""},
	} {
		t.Run(tc.name, func(t *testing.T) {
			var slug, version, versionID, filename string
			parsedSlug, err := parseSlugOrUrl(tc.input, &slug, &version, &versionID, &filename)
			if tc.wantErr {
				if err == nil {
					t.Fatalf("expected an error, got slug=%q", slug)
				}
				return
			}
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if slug != tc.wantSlug || version != tc.wantVer || versionID != tc.wantVerID || filename != tc.wantFile {
				t.Errorf("got slug=%q version=%q versionID=%q filename=%q", slug, version, versionID, filename)
			}
			if parsedSlug != tc.parsedSlug {
				t.Errorf("parsedSlug = %v, want %v", parsedSlug, tc.parsedSlug)
			}
		})
	}
}

func TestGetBestHashPreference(t *testing.T) {
	file := &modrinthApi.File{Hashes: map[string]string{
		"sha1":   "aa",
		"sha256": "bb",
		"sha512": "cc",
	}}
	if format, hash := getBestHash(file); format != "sha512" || hash != "cc" {
		t.Errorf("got %s/%s, want sha512/cc", format, hash)
	}
	delete(file.Hashes, "sha512")
	if format, hash := getBestHash(file); format != "sha256" || hash != "bb" {
		t.Errorf("got %s/%s, want sha256/bb", format, hash)
	}
	delete(file.Hashes, "sha256")
	if format, hash := getBestHash(file); format != "sha1" || hash != "aa" {
		t.Errorf("got %s/%s, want sha1/aa", format, hash)
	}
	if format, hash := getBestHash(&modrinthApi.File{Hashes: map[string]string{}}); format != "" || hash != "" {
		t.Errorf("got %s/%s for empty hashes, want empty", format, hash)
	}
}

func TestGetSide(t *testing.T) {
	mk := func(client, server string) *modrinthApi.Project {
		return &modrinthApi.Project{ClientSide: &client, ServerSide: &server}
	}
	for _, tc := range []struct {
		client, server, want string
	}{
		{"required", "required", "both"},
		{"optional", "required", "both"},
		{"unsupported", "required", "server"},
		{"required", "unsupported", "client"},
		{"unsupported", "unsupported", ""},
	} {
		if got := getSide(mk(tc.client, tc.server)); got != tc.want {
			t.Errorf("getSide(client=%s, server=%s) = %q, want %q", tc.client, tc.server, got, tc.want)
		}
	}
}
