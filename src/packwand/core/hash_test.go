package core

import (
	"strings"
	"testing"
)

// Characterization golden vectors for every supported hash format
// (packwandrs.md phase 4). These freeze the exact strings Packwand writes
// into index/metadata files and compares during downloads; a future Rust
// port must reproduce them byte for byte.
func TestHashGoldenVectors(t *testing.T) {
	vectors := []struct {
		input    string
		expected map[string]string
	}{
		{
			input: "",
			expected: map[string]string{
				"sha1":         "da39a3ee5e6b4b0d3255bfef95601890afd80709",
				"sha256":       "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
				"sha512":       "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
				"md5":          "d41d8cd98f00b204e9800998ecf8427e",
				"murmur2":      "1540447798",
				"length-bytes": "0",
			},
		},
		{
			input: "packwiz",
			expected: map[string]string{
				"sha1":         "0ea33eefe7606ab0a35bb7301e9d31f530a76a64",
				"sha256":       "2cc884ee64ae3d5312e0ad713857d0c97b4d6b5c9f657878b489af7ccbd9a2c1",
				"sha512":       "9e295fc75ab988af67b2b03bcb07a71198f23c36f0a62f61c363ea86a84724b5ba1987edb6b03622dca2f1c691efcc05b8dea1d6fa234a7f9254f583eb563fd9",
				"md5":          "c25cb1fdc83dc8ab3a5a8b4887a3dfe8",
				"murmur2":      "2676380970",
				"length-bytes": "7",
			},
		},
		{
			input: "packwand: the quick brown fox jumps over the lazy dog",
			expected: map[string]string{
				"sha1":         "ad0c069c8f6da3f7c09359f2047506a93b0ac66a",
				"sha256":       "86c9917988ef302683f6dbcbb28b0adb34ae39d37a75dfcec575b61982987a3a",
				"sha512":       "c0639fad2036a0a95999e04e3862755f7361e6a5944df6b5df6574efb14a3e85a06b22e1e3367a41bcc96325c5077936b2716ba0c89bdb0e6326696efb5fc215",
				"md5":          "872e9fd24efc757bdc004c0896a099d3",
				"murmur2":      "574953714",
				"length-bytes": "53",
			},
		},
	}
	for _, vector := range vectors {
		for format, expected := range vector.expected {
			h, err := GetHashImpl(format)
			if err != nil {
				t.Fatalf("GetHashImpl(%q): %v", format, err)
			}
			if _, err := h.Write([]byte(vector.input)); err != nil {
				t.Fatalf("write to %q hasher: %v", format, err)
			}
			got := h.HashToString(h.Sum(nil))
			if got != expected {
				t.Errorf("%s(%q) = %q, want %q", format, vector.input, got, expected)
			}
		}
	}
}

// The CurseForge murmur2 variant must ignore whitespace (tab, LF, CR,
// space) so that the same fingerprint is produced for re-encoded files.
func TestMurmur2StripsWhitespace(t *testing.T) {
	for _, input := range []string{"helloworld", "hello \tworld\r\n", " h e l l o w o r l d "} {
		h, err := GetHashImpl("murmur2")
		if err != nil {
			t.Fatal(err)
		}
		h.Write([]byte(input))
		if got := h.HashToString(h.Sum(nil)); got != "2824650221" {
			t.Errorf("murmur2(%q) = %s, want 2824650221", input, got)
		}
	}
}

// Hash format lookups are case-insensitive; unknown formats error rather
// than silently hashing with a default.
func TestGetHashImplLookup(t *testing.T) {
	if _, err := GetHashImpl("SHA512"); err != nil {
		t.Errorf("uppercase format should resolve: %v", err)
	}
	if _, err := GetHashImpl("blake3"); err == nil {
		t.Error("unknown format must return an error")
	}
}

// Incremental writes must equal one-shot writes: download verification
// streams files through the hashers chunk by chunk.
func TestHashersAreIncremental(t *testing.T) {
	const input = "packwand: the quick brown fox jumps over the lazy dog"
	for _, format := range []string{"sha1", "sha256", "sha512", "md5", "murmur2", "length-bytes"} {
		oneShot, err := GetHashImpl(format)
		if err != nil {
			t.Fatal(err)
		}
		oneShot.Write([]byte(input))

		chunked, err := GetHashImpl(format)
		if err != nil {
			t.Fatal(err)
		}
		for _, chunk := range strings.SplitAfter(input, " ") {
			chunked.Write([]byte(chunk))
		}
		if a, b := oneShot.HashToString(oneShot.Sum(nil)), chunked.HashToString(chunked.Sum(nil)); a != b {
			t.Errorf("%s: one-shot %s != chunked %s", format, a, b)
		}
	}
}
