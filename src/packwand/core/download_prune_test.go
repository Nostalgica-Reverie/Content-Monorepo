package core

import (
	"os"
	"path/filepath"
	"testing"
)

func writeCacheFile(t *testing.T, cachePath, hash string) {
	t.Helper()
	dir := filepath.Join(cachePath, hash[:2])
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, hash[2:]), []byte("fake-cached-file"), 0o644); err != nil {
		t.Fatal(err)
	}
}

func TestCacheIndexPruneRemovesUnreferencedEntries(t *testing.T) {
	cachePath := t.TempDir()
	keepHash := "aa11111111111111111111111111111111111111111111111111111111111111"
	dropHash := "bb22222222222222222222222222222222222222222222222222222222222222"
	writeCacheFile(t, cachePath, keepHash)
	writeCacheFile(t, cachePath, dropHash)

	index := &CacheIndex{
		Version:   cacheLatestVersion,
		Hashes:    map[string][]string{cacheHashFormat: {keepHash, dropHash}},
		cachePath: cachePath,
	}

	referenced := map[string]struct{}{keepHash: {}}
	result, err := index.Prune(referenced, false)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.RemovedEntries) != 1 || result.RemovedEntries[0].Hash != dropHash {
		t.Fatalf("removed entries = %#v", result.RemovedEntries)
	}
	if _, err := os.Stat(filepath.Join(cachePath, dropHash[:2], dropHash[2:])); !os.IsNotExist(err) {
		t.Fatalf("expected dropped cache file to be deleted, stat err = %v", err)
	}
	if _, err := os.Stat(filepath.Join(cachePath, keepHash[:2], keepHash[2:])); err != nil {
		t.Fatalf("expected kept cache file to survive: %v", err)
	}
	if len(index.Hashes[cacheHashFormat]) != 1 || index.Hashes[cacheHashFormat][0] != keepHash {
		t.Fatalf("index after prune = %#v", index.Hashes)
	}
}

func TestCacheIndexPruneDryRunLeavesFilesAlone(t *testing.T) {
	cachePath := t.TempDir()
	dropHash := "cc33333333333333333333333333333333333333333333333333333333333333"
	writeCacheFile(t, cachePath, dropHash)

	index := &CacheIndex{
		Version:   cacheLatestVersion,
		Hashes:    map[string][]string{cacheHashFormat: {dropHash}},
		cachePath: cachePath,
	}

	result, err := index.Prune(map[string]struct{}{}, true)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.RemovedEntries) != 1 {
		t.Fatalf("expected 1 entry reported for removal, got %#v", result.RemovedEntries)
	}
	if _, err := os.Stat(filepath.Join(cachePath, dropHash[:2], dropHash[2:])); err != nil {
		t.Fatalf("dry-run must not delete files: %v", err)
	}
	if len(index.Hashes[cacheHashFormat]) != 1 {
		t.Fatalf("dry-run must not mutate the index: %#v", index.Hashes)
	}
}
