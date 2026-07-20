package core

import "testing"

func TestExportHashes(t *testing.T) {
	dl := ModDownload{
		HashFormat:  "sha512",
		Hash:        "aa11",
		ExtraHashes: map[string]string{"sha1": "bb22"},
		Size:        1234,
	}

	hashes, ok := dl.ExportHashes([]string{"sha1", "sha512", "length-bytes"})
	if !ok {
		t.Fatal("ExportHashes returned ok=false with all values present")
	}
	want := map[string]string{"sha1": "bb22", "sha512": "aa11", "length-bytes": "1234"}
	for k, v := range want {
		if hashes[k] != v {
			t.Errorf("hashes[%q] = %q, want %q", k, hashes[k], v)
		}
	}

	// Missing extra hash → not exportable from metadata.
	noSha1 := ModDownload{HashFormat: "sha512", Hash: "aa11", Size: 1}
	if _, ok := noSha1.ExportHashes([]string{"sha1", "sha512", "length-bytes"}); ok {
		t.Error("ok=true without sha1 available")
	}
	// Unknown size → not exportable from metadata.
	dl.Size = 0
	if _, ok := dl.ExportHashes([]string{"sha1", "sha512", "length-bytes"}); ok {
		t.Error("ok=true without size available")
	}
}
