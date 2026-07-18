package core

import (
	"crypto/md5"
	"crypto/sha256"
	"crypto/sha512"
	"encoding/hex"
	"fmt"
	"math/rand"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

// requireHashutil skips the test unless a hashutil binary is on PATH.
// hashutil is optional CI/dev tooling built separately (`just build-hashutil`
// or `just test-hashutil`); it isn't built as part of the Go test run, so
// these tests skip gracefully rather than failing when it's absent -- the
// same convention Go tests use for other optional external tools.
func requireHashutil(t *testing.T) string {
	t.Helper()
	bin, err := exec.LookPath("hashutil")
	if err != nil {
		t.Skip("hashutil not found on PATH; run `just build-hashutil` and add tools/hashutil to PATH to run this test")
	}
	return bin
}

// hashutilBatch (and the sharded hashutilBatchParallel wrapper Index.Refresh
// uses) must reproduce the exact same golden vectors as the pure-Go
// GetHashImpl path -- see TestHashGoldenVectors in hash_test.go, which this
// mirrors, so the two hashing paths stay provably interchangeable.
func TestHashutilBatchMatchesGoldenVectors(t *testing.T) {
	bin := requireHashutil(t)
	dir := t.TempDir()

	vectors := []struct {
		name     string
		input    string
		expected map[string]string
	}{
		{"empty", "", map[string]string{
			"sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
			"sha512": "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
			"md5":    "d41d8cd98f00b204e9800998ecf8427e",
			"murmur2": "1540447798",
		}},
		{"packwiz", "packwiz", map[string]string{
			"sha256":  "2cc884ee64ae3d5312e0ad713857d0c97b4d6b5c9f657878b489af7ccbd9a2c1",
			"sha512":  "9e295fc75ab988af67b2b03bcb07a71198f23c36f0a62f61c363ea86a84724b5ba1987edb6b03622dca2f1c691efcc05b8dea1d6fa234a7f9254f583eb563fd9",
			"md5":     "c25cb1fdc83dc8ab3a5a8b4887a3dfe8",
			"murmur2": "2676380970",
		}},
		{"fox", "packwand: the quick brown fox jumps over the lazy dog", map[string]string{
			"sha256":  "86c9917988ef302683f6dbcbb28b0adb34ae39d37a75dfcec575b61982987a3a",
			"sha512":  "c0639fad2036a0a95999e04e3862755f7361e6a5944df6b5df6574efb14a3e85a06b22e1e3367a41bcc96325c5077936b2716ba0c89bdb0e6326696efb5fc215",
			"md5":     "872e9fd24efc757bdc004c0896a099d3",
			"murmur2": "574953714",
		}},
	}

	var paths []string
	for _, v := range vectors {
		p := filepath.Join(dir, v.name+".txt")
		if err := os.WriteFile(p, []byte(v.input), 0o644); err != nil {
			t.Fatal(err)
		}
		paths = append(paths, p)
	}

	for _, format := range []string{"sha256", "sha512", "md5", "murmur2"} {
		got, err := hashutilBatch(bin, paths, format)
		if err != nil {
			t.Fatalf("hashutilBatch(%s): %v", format, err)
		}
		for i, v := range vectors {
			if got[i] != v.expected[format] {
				t.Errorf("hashutilBatch %s(%s) = %q, want %q", format, v.name, got[i], v.expected[format])
			}
		}
	}
}

// hashutilBatchParallel shards the path list across N subprocesses; results
// must land back at the correct index regardless of shard count.
func TestHashutilBatchParallelPreservesOrder(t *testing.T) {
	bin := requireHashutil(t)
	dir := t.TempDir()

	var paths []string
	var want []string
	for i := 0; i < 17; i++ {
		p := filepath.Join(dir, filepathBase(i))
		content := filepathBase(i) + "-content"
		if err := os.WriteFile(p, []byte(content), 0o644); err != nil {
			t.Fatal(err)
		}
		paths = append(paths, p)

		h, err := GetHashImpl("sha256")
		if err != nil {
			t.Fatal(err)
		}
		h.Write([]byte(content))
		want = append(want, h.HashToString(h.Sum(nil)))
	}

	for _, shards := range []int{1, 3, 4, 17, 100} {
		got, err := hashutilBatchParallel(bin, paths, "sha256", shards)
		if err != nil {
			t.Fatalf("hashutilBatchParallel(shards=%d): %v", shards, err)
		}
		for i := range paths {
			if got[i] != want[i] {
				t.Errorf("shards=%d: result[%d] = %q, want %q", shards, i, got[i], want[i])
			}
		}
	}
}

// A per-file read error (file removed after the path list was built) must
// surface as a Go error, not be silently dropped -- matching
// computeFileHash's existing behavior for unreadable files.
func TestHashutilBatchSurfacesPerFileError(t *testing.T) {
	bin := requireHashutil(t)
	dir := t.TempDir()

	good := filepath.Join(dir, "good.txt")
	if err := os.WriteFile(good, []byte("ok"), 0o644); err != nil {
		t.Fatal(err)
	}
	missing := filepath.Join(dir, "does-not-exist.txt")

	_, err := hashutilBatch(bin, []string{good, missing}, "sha256")
	if err == nil {
		t.Fatal("expected an error for the missing file, got nil")
	}
}

// Differential test: no hardcoded oracle values at all -- every input's
// expected hash is computed by Go's own crypto/sha256, crypto/sha512,
// crypto/md5 (independent of hashutil.c's from-scratch implementations)
// and GetHashImpl("murmur2") (the production murmur2 path), then compared
// against hashutilBatch's output for the identical bytes. Covers every
// sha256/md5 block boundary (64-byte blocks) and sha512 block boundary
// (128-byte blocks) plus random lengths and random (not just deterministic
// ramp, complementing hashutil.c's own C-level boundary vectors) content,
// so a padding or block-accounting bug at any block boundary would show up
// here even if nobody thought to add a fixed vector for that exact length.
func TestHashutilBatchDifferentialAgainstStdlib(t *testing.T) {
	bin := requireHashutil(t)
	dir := t.TempDir()

	lengths := []int{
		0, 1, 2, 3,
		54, 55, 56, 57, 63, 64, 65, // sha256/md5 block boundary (64 bytes)
		111, 112, 113, 127, 128, 129, // sha512 block boundary (128 bytes)
		255, 256, 257,
		1000, 4096, 4097,
	}
	rng := rand.New(rand.NewSource(20260715))
	// A handful of extra random lengths between 0 and 5000, for coverage
	// beyond the hand-picked boundary set above.
	for i := 0; i < 20; i++ {
		lengths = append(lengths, rng.Intn(5000))
	}

	var paths []string
	type want struct {
		sha256, sha512, md5 string
	}
	wants := make([]want, len(lengths))

	for i, n := range lengths {
		data := make([]byte, n)
		if _, err := rng.Read(data); err != nil {
			t.Fatal(err)
		}
		p := filepath.Join(dir, fmt.Sprintf("f%d_len%d.bin", i, n))
		if err := os.WriteFile(p, data, 0o644); err != nil {
			t.Fatal(err)
		}
		paths = append(paths, p)

		s256 := sha256.Sum256(data)
		s512 := sha512.Sum512(data)
		m5 := md5.Sum(data) //nolint:gosec // content-addressing compat format, not security-sensitive here
		wants[i] = want{
			sha256: hex.EncodeToString(s256[:]),
			sha512: hex.EncodeToString(s512[:]),
			md5:    hex.EncodeToString(m5[:]),
		}
	}

	for _, format := range []string{"sha256", "sha512", "md5"} {
		got, err := hashutilBatch(bin, paths, format)
		if err != nil {
			t.Fatalf("hashutilBatch(%s): %v", format, err)
		}
		for i, n := range lengths {
			var wantHex string
			switch format {
			case "sha256":
				wantHex = wants[i].sha256
			case "sha512":
				wantHex = wants[i].sha512
			case "md5":
				wantHex = wants[i].md5
			}
			if got[i] != wantHex {
				t.Errorf("%s(len=%d): hashutil=%q go-stdlib=%q", format, n, got[i], wantHex)
			}
		}
	}

	// murmur2 has no Go stdlib equivalent; GetHashImpl("murmur2") (the
	// production path apps/packwand actually uses) is the oracle instead.
	murmurGot, err := hashutilBatch(bin, paths, "murmur2")
	if err != nil {
		t.Fatalf("hashutilBatch(murmur2): %v", err)
	}
	for i, p := range paths {
		data, err := os.ReadFile(p)
		if err != nil {
			t.Fatal(err)
		}
		h, err := GetHashImpl("murmur2")
		if err != nil {
			t.Fatal(err)
		}
		h.Write(data)
		want := h.HashToString(h.Sum(nil))
		if murmurGot[i] != want {
			t.Errorf("murmur2(len=%d): hashutil=%q GetHashImpl=%q", lengths[i], murmurGot[i], want)
		}
	}
}

func filepathBase(i int) string {
	return "file-" + string(rune('a'+i%26)) + string(rune('0'+i/26)) + ".txt"
}
