package core

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// hashutilAlgos lists the hash formats tools/hashutil implements. Any other
// format (e.g. sha1, length-bytes) falls back to the pure-Go path in
// computeFileHash -- hashutil is a deliberately narrow, "shapeless" tool,
// not a full reimplementation of GetHashImpl. See c.md section 1.
var hashutilAlgos = map[string]bool{
	"sha256":  true,
	"sha512":  true,
	"md5":     true,
	"murmur2": true,
}

// hashutilBin returns the path to the hashutil binary from the
// HASHUTIL_BIN env var, or "" if unset. Deliberately does NOT fall back to
// a bare PATH lookup the way packeaterBin() does: benchmarking found
// hashutil slower than Go's own hashing for the actually-default format,
// DefaultHashFormat = "sha512" -- Go's crypto/sha512 is hardware-accelerated
// assembly, hashutil's sha512.h is a plain scalar C implementation with no
// SIMD, and on top of that the subprocess/IPC overhead isn't recouped.
// 2026-07 re-measurement after hashutil's block-at-a-time update rewrite
// (BenchmarkSha512Hashutil/BenchmarkMurmur2CFHashutil, 100x64KB batches):
// sha512 350 MB/s vs Go's 494 MB/s -- still loses, verdict unchanged.
// murmur2 is the one format where hashutil now wins (372 MB/s vs 300 MB/s
// for the best in-process Go loop), so HASHUTIL_BIN is worth setting for
// murmur2-heavy (CurseForge fingerprint) batches specifically. Silently
// activating hashutil for any developer who happens to have it built and on
// PATH would still regress default-format hashing, so it stays opt-in.
func hashutilBin() string {
	if b := os.Getenv("HASHUTIL_BIN"); b != "" {
		return b
	}
	return ""
}

// hashutilResult mirrors one NDJSON line from tools/hashutil/hashutil.c.
// sha256/sha512/md5 are lowercase hex, murmur2 is a decimal uint32 string --
// both already match core/hash.go's HashToString output exactly, so no
// conversion is needed once a field is read.
type hashutilResult struct {
	Path    string  `json:"path"`
	Sha256  *string `json:"sha256"`
	Sha512  *string `json:"sha512"`
	Md5     *string `json:"md5"`
	Murmur2 *string `json:"murmur2"`
	Error   *string `json:"error"`
}

func (r *hashutilResult) valueFor(format string) (string, error) {
	if r.Error != nil {
		return "", fmt.Errorf("%s", *r.Error)
	}
	var v *string
	switch format {
	case "sha256":
		v = r.Sha256
	case "sha512":
		v = r.Sha512
	case "md5":
		v = r.Md5
	case "murmur2":
		v = r.Murmur2
	}
	if v == nil {
		return "", fmt.Errorf("hashutil: missing %q field for %s", format, r.Path)
	}
	return *v, nil
}

// hashutilBatch runs bin once over paths, requesting only the given format,
// and returns one hash string per input path in the same order. hashutil
// processes stdin sequentially and emits one NDJSON line per path in order
// (see tools/hashutil/hashutil.c), so results are matched by position, not
// by re-parsing the echoed path. A per-file read error from hashutil (e.g.
// a file removed between the directory walk and this call) is surfaced as
// a Go error for that path, matching computeFileHash's existing behavior
// for unreadable files -- not silently skipped.
func hashutilBatch(bin string, paths []string, format string) ([]string, error) {
	if len(paths) == 0 {
		return nil, nil
	}

	cmd := exec.Command(bin, "--algos="+format)
	cmd.Stdin = strings.NewReader(strings.Join(paths, "\n") + "\n")
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		return nil, fmt.Errorf("hashutil: %w: %s", err, strings.TrimSpace(stderr.String()))
	}

	results := make([]string, len(paths))
	scanner := bufio.NewScanner(&stdout)
	scanner.Buffer(make([]byte, 0, 64*1024), 1024*1024)
	i := 0
	for scanner.Scan() {
		if i >= len(paths) {
			return nil, fmt.Errorf("hashutil: produced more output lines than the %d input paths", len(paths))
		}
		var r hashutilResult
		if err := json.Unmarshal(scanner.Bytes(), &r); err != nil {
			return nil, fmt.Errorf("hashutil: invalid output line: %w", err)
		}
		val, err := r.valueFor(format)
		if err != nil {
			return nil, fmt.Errorf("hashutil: %s: %w", paths[i], err)
		}
		results[i] = val
		i++
	}
	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("hashutil: reading output: %w", err)
	}
	if i != len(paths) {
		return nil, fmt.Errorf("hashutil: expected %d output lines, got %d", len(paths), i)
	}
	return results, nil
}

// hashutilBatchParallel shards paths across `shards` concurrent hashutil
// subprocesses (default: HashConcurrent(), matching the worker count the
// pure-Go path would otherwise use with ParallelFor) so the batch hasher
// doesn't collapse a multi-core hash pass onto a single core -- hashutil
// itself is deliberately single-threaded (c.md section 1.4); concurrency
// across files is the Go caller's job. Returns one hash string per input
// path in the same order as paths.
func hashutilBatchParallel(bin string, paths []string, format string, shards int) ([]string, error) {
	if len(paths) == 0 {
		return nil, nil
	}
	if shards < 1 {
		shards = 1
	}
	if shards > len(paths) {
		shards = len(paths)
	}

	shardOf := make([][]string, shards)
	shardIdx := make([][]int, shards)
	for i, p := range paths {
		s := i % shards
		shardOf[s] = append(shardOf[s], p)
		shardIdx[s] = append(shardIdx[s], i)
	}

	results := make([]string, len(paths))
	errs := make([]error, shards)
	ParallelFor(shardOf, shards, func(s int, shardPaths []string) {
		if len(shardPaths) == 0 {
			return
		}
		shardResults, err := hashutilBatch(bin, shardPaths, format)
		if err != nil {
			errs[s] = err
			return
		}
		for j, idx := range shardIdx[s] {
			results[idx] = shardResults[j]
		}
	})
	for _, err := range errs {
		if err != nil {
			return nil, err
		}
	}
	return results, nil
}
