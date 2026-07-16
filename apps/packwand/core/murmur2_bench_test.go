package core

import (
	"crypto/rand"
	"crypto/sha512"
	"encoding/hex"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"testing"

	murmur "github.com/aviddiviner/go-murmur"
)

func isWhitespaceCF(b byte) bool {
	return b == 9 || b == 10 || b == 13 || b == 32
}

// naiveMurmur2CF mirrors apps/packwand/curseforge/murmur2's Write(): a
// byte-by-byte append loop that re-grows the backing slice incrementally
// (Go's append() does amortized-O(1) doubling growth, but still pays a
// capacity check and possible copy on every growth step, plus a function
// call and bounds check per byte).
func naiveMurmur2CF(data []byte) uint32 {
	buf := make([]byte, 0) //nolint:prealloc // deliberately mirrors the un-presized production code
	for _, b := range data {
		if !isWhitespaceCF(b) {
			buf = append(buf, b)
		}
	}
	return murmur.MurmurHash2(buf, 1)
}

// correctedMurmur2CF avoids naiveMurmur2CF's per-byte append() growth
// overhead by counting non-whitespace bytes first, allocating the exact
// backing array once, then copying via direct indexing. This is the
// "genuinely fixable inefficiency" c.md section 1.2 describes -- same
// algorithm, byte-for-byte identical output (verified by
// TestCorrectedMurmur2CFMatchesNaive below), just less allocator/bounds-
// check overhead per byte. True streaming isn't possible here regardless
// (see c.md section 1.2): the seed must incorporate the final
// stripped-byte count before mixing starts.
func correctedMurmur2CF(data []byte) uint32 {
	keep := 0
	for _, b := range data {
		if !isWhitespaceCF(b) {
			keep++
		}
	}
	buf := make([]byte, keep)
	i := 0
	for _, b := range data {
		if !isWhitespaceCF(b) {
			buf[i] = b
			i++
		}
	}
	return murmur.MurmurHash2(buf, 1)
}

// makeBenchData returns pseudo-random content with whitespace bytes
// sprinkled in periodically, so the strip loop has real work to do --
// pure random binary almost never contains 9/10/13/32 by chance, which
// would make the benchmark measure a strip loop that never strips
// anything, not representative of real mod-jar/text content.
func makeBenchData(n int) []byte {
	data := make([]byte, n)
	if _, err := rand.Read(data); err != nil {
		panic(err)
	}
	for i := 0; i < len(data); i += 37 {
		data[i] = 32
	}
	return data
}

func fmtSize(n int) string {
	switch {
	case n >= 1<<20:
		return fmt.Sprintf("%dMB", n/(1<<20))
	case n >= 1<<10:
		return fmt.Sprintf("%dKB", n/(1<<10))
	default:
		return fmt.Sprintf("%dB", n)
	}
}

// Correctness gate for the benchmark itself: if corrected and naive ever
// disagreed, the benchmark comparison below would be meaningless (or
// actively misleading). Checked across sizes crossing several
// growth-doubling boundaries of the naive implementation's append().
func TestCorrectedMurmur2CFMatchesNaive(t *testing.T) {
	for _, n := range []int{0, 1, 36, 37, 38, 100, 1000, 10000, 100000} {
		data := makeBenchData(n)
		naive := naiveMurmur2CF(data)
		corrected := correctedMurmur2CF(data)
		if naive != corrected {
			t.Errorf("len=%d: naive=%d corrected=%d", n, naive, corrected)
		}
		// Also cross-check against the actual production implementation
		// (apps/packwand/curseforge/murmur2), not just the two benchmark
		// variants agreeing with each other.
		h, err := GetHashImpl("murmur2")
		if err != nil {
			t.Fatal(err)
		}
		h.Write(data)
		want := h.HashToString(h.Sum(nil))
		got := fmt.Sprintf("%d", naive)
		if got != want {
			t.Errorf("len=%d: naive=%s production=%s", n, got, want)
		}
	}
}

// BenchmarkMurmur2CF compares the naive (production-mirroring) and
// corrected in-process Go implementations across sizes representative of
// mod jars, from small config-file-sized content up to a large jar.
func BenchmarkMurmur2CF(b *testing.B) {
	sizes := []int{1 << 10, 1 << 16, 1 << 20, 20 << 20} // 1KB, 64KB, 1MB, 20MB
	for _, n := range sizes {
		data := makeBenchData(n)

		b.Run(fmtSize(n)+"/naive", func(b *testing.B) {
			b.SetBytes(int64(n))
			for i := 0; i < b.N; i++ {
				naiveMurmur2CF(data)
			}
		})

		b.Run(fmtSize(n)+"/corrected", func(b *testing.B) {
			b.SetBytes(int64(n))
			for i := 0; i < b.N; i++ {
				correctedMurmur2CF(data)
			}
		})
	}
}

// BenchmarkMurmur2CFHashutil compares hashutil against both Go variants,
// in two shapes: one file per subprocess call (an honest worst case,
// dominated by process-spawn overhead) and a 100-file batch in a single
// process (the shape Index.Refresh() actually uses via
// hashutilBatchParallel -- see core/index.go). Skips if hashutil isn't
// built and on PATH (`just build-hashutil`), matching hashutil_test.go's
// convention for optional-external-tool tests.
func BenchmarkMurmur2CFHashutil(b *testing.B) {
	bin, err := exec.LookPath("hashutil")
	if err != nil {
		b.Skip("hashutil not found on PATH; run `just build-hashutil` and add tools/hashutil to PATH to run this benchmark")
	}

	sizes := []int{1 << 10, 1 << 16, 1 << 20, 20 << 20}
	for _, n := range sizes {
		data := makeBenchData(n)
		dir := b.TempDir()
		path := filepath.Join(dir, "bench.bin")
		if writeErr := os.WriteFile(path, data, 0o644); writeErr != nil {
			b.Fatal(writeErr)
		}

		b.Run(fmtSize(n)+"/hashutil-single-file-per-call", func(b *testing.B) {
			b.SetBytes(int64(n))
			for i := 0; i < b.N; i++ {
				if _, batchErr := hashutilBatch(bin, []string{path}, "murmur2"); batchErr != nil {
					b.Fatal(batchErr)
				}
			}
		})
	}

	const batchN = 100
	const batchFileSize = 1 << 16 // 64KB each, roughly mod-jar-sized
	dir := b.TempDir()
	paths := make([]string, batchN)
	for i := range paths {
		d := makeBenchData(batchFileSize)
		p := filepath.Join(dir, fmt.Sprintf("f%d.bin", i))
		if writeErr := os.WriteFile(p, d, 0o644); writeErr != nil {
			b.Fatal(writeErr)
		}
		paths[i] = p
	}
	totalBytes := int64(batchN * batchFileSize)

	b.Run(fmt.Sprintf("%dx%s/hashutil-batch-one-process", batchN, fmtSize(batchFileSize)), func(b *testing.B) {
		b.SetBytes(totalBytes)
		for i := 0; i < b.N; i++ {
			if _, batchErr := hashutilBatch(bin, paths, "murmur2"); batchErr != nil {
				b.Fatal(batchErr)
			}
		}
	})

	// Both Go loop variants re-read each file from disk on every b.N
	// iteration too (not hashing pre-loaded in-memory data) -- matching
	// what Index.Refresh()'s pure-Go path (computeFileHash) actually does
	// in production, so this isolates the hashing-path difference rather
	// than comparing "hashutil does real I/O" against "Go skips it."
	b.Run(fmt.Sprintf("%dx%s/go-naive-loop", batchN, fmtSize(batchFileSize)), func(b *testing.B) {
		b.SetBytes(totalBytes)
		for i := 0; i < b.N; i++ {
			for _, p := range paths {
				d, readErr := os.ReadFile(p)
				if readErr != nil {
					b.Fatal(readErr)
				}
				naiveMurmur2CF(d)
			}
		}
	})

	b.Run(fmt.Sprintf("%dx%s/go-corrected-loop", batchN, fmtSize(batchFileSize)), func(b *testing.B) {
		b.SetBytes(totalBytes)
		for i := 0; i < b.N; i++ {
			for _, p := range paths {
				d, readErr := os.ReadFile(p)
				if readErr != nil {
					b.Fatal(readErr)
				}
				correctedMurmur2CF(d)
			}
		}
	})
}

// BenchmarkSha512Hashutil is the benchmark that actually matters for the
// real Index.Refresh() integration: DefaultHashFormat is "sha512" (see
// core/hash.go), not murmur2 -- murmur2 above is benchmarked because
// c.md section 1.2's inefficiency claim was specifically about
// Murmur2CF.Write(), but sha512 is what most packs actually hash. Go's
// crypto/sha512 uses hardware-accelerated assembly; hashutil's sha512.h
// is a plain portable scalar C implementation with no SIMD/hardware
// acceleration (see tools/hashutil/sha512.h), so this comparison is
// expected to be considerably less favorable to hashutil than murmur2's,
// and should be read as the primary result, not the murmur2 numbers.
func BenchmarkSha512Hashutil(b *testing.B) {
	bin, err := exec.LookPath("hashutil")
	if err != nil {
		b.Skip("hashutil not found on PATH; run `just build-hashutil` and add tools/hashutil to PATH to run this benchmark")
	}

	const batchN = 100
	const batchFileSize = 1 << 16 // 64KB each, roughly mod-jar-sized
	dir := b.TempDir()
	paths := make([]string, batchN)
	for i := range paths {
		d := makeBenchData(batchFileSize)
		p := filepath.Join(dir, fmt.Sprintf("f%d.bin", i))
		if writeErr := os.WriteFile(p, d, 0o644); writeErr != nil {
			b.Fatal(writeErr)
		}
		paths[i] = p
	}
	totalBytes := int64(batchN * batchFileSize)

	b.Run(fmt.Sprintf("%dx%s/hashutil-batch-one-process", batchN, fmtSize(batchFileSize)), func(b *testing.B) {
		b.SetBytes(totalBytes)
		for i := 0; i < b.N; i++ {
			if _, batchErr := hashutilBatch(bin, paths, "sha512"); batchErr != nil {
				b.Fatal(batchErr)
			}
		}
	})

	b.Run(fmt.Sprintf("%dx%s/go-crypto-sha512-loop", batchN, fmtSize(batchFileSize)), func(b *testing.B) {
		b.SetBytes(totalBytes)
		for i := 0; i < b.N; i++ {
			for _, p := range paths {
				d, readErr := os.ReadFile(p)
				if readErr != nil {
					b.Fatal(readErr)
				}
				sum := sha512.Sum512(d)
				_ = hex.EncodeToString(sum[:])
			}
		}
	})
}
