package curseforge

import "testing"

func TestParseReleaseChannel(t *testing.T) {
	cases := []struct {
		in   string
		want fileType
		ok   bool
	}{
		{"release", fileTypeRelease, true},
		{"Beta", fileTypeBeta, true},
		{" alpha ", fileTypeAlpha, true},
		{"", 0, false},
		{"nightly", 0, false},
	}
	for _, c := range cases {
		got, ok := parseReleaseChannel(c.in)
		if got != c.want || ok != c.ok {
			t.Errorf("parseReleaseChannel(%q) = (%v, %v), want (%v, %v)", c.in, got, ok, c.want, c.ok)
		}
	}
}

func TestFindLatestFileHonorsReleaseChannelPreference(t *testing.T) {
	mcVersions := []string{"1.20.1"}
	// Beta has the higher file ID, so it wins when there is no channel
	// preference (ties/ordering favor the higher ID).
	info := modInfo{
		LatestFiles: []modFileInfo{
			{ID: 1, FileName: "mod-1.0.0-release.jar", GameVersions: []string{"1.20.1"}, FileType: fileTypeRelease},
			{ID: 2, FileName: "mod-1.1.0-beta.jar", GameVersions: []string{"1.20.1"}, FileType: fileTypeBeta},
		},
	}

	if fileID, _, name := findLatestFile(info, mcVersions, nil, 0); fileID != 2 || name != "mod-1.1.0-beta.jar" {
		t.Fatalf("no preference: got fileID=%d name=%q, want the newer beta file", fileID, name)
	}

	if fileID, _, name := findLatestFile(info, mcVersions, nil, fileTypeRelease); fileID != 1 || name != "mod-1.0.0-release.jar" {
		t.Fatalf("release-only: got fileID=%d name=%q, want the release file (beta excluded)", fileID, name)
	}
}

func TestFindLatestFileReturnsNothingWhenNoChannelMatches(t *testing.T) {
	mcVersions := []string{"1.20.1"}
	info := modInfo{
		LatestFiles: []modFileInfo{
			{ID: 5, FileName: "mod-1.2.0-alpha.jar", GameVersions: []string{"1.20.1"}, FileType: fileTypeAlpha},
		},
	}

	if fileID, _, _ := findLatestFile(info, mcVersions, nil, fileTypeRelease); fileID != 0 {
		t.Fatalf("expected no match for a release-only preference against an alpha-only file, got fileID=%d", fileID)
	}
}
