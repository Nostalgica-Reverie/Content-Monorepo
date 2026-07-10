package core

import (
	"fmt"
	"io"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"strings"
	"time"

	"github.com/BurntSushi/toml"
	gitignore "github.com/sabhiram/go-gitignore"
	"github.com/spf13/viper"
	"github.com/vbauerster/mpb/v4"
	"github.com/vbauerster/mpb/v4/decor"
)

// Index is a representation of the index.toml file for referencing all the files in a pack.
type Index struct {
	HashFormat string
	Files      IndexFiles
	indexFile  string
	packRoot   string
}

// indexTomlRepresentation is the TOML representation of Index (Files must be converted)
type indexTomlRepresentation struct {
	HashFormat string                       `toml:"hash-format"`
	Files      indexFilesTomlRepresentation `toml:"files"`
}

// LoadIndex attempts to load the index file from a path
func LoadIndex(indexFile string) (Index, error) {
	// Decode as indexTomlRepresentation then convert to Index
	var rep indexTomlRepresentation
	if _, err := toml.DecodeFile(indexFile, &rep); err != nil {
		return Index{}, err
	}
	if len(rep.HashFormat) == 0 {
		rep.HashFormat = DefaultHashFormat
	}
	index := Index{
		HashFormat: rep.HashFormat,
		Files:      rep.Files.toMemoryRep(),
		indexFile:  indexFile,
		packRoot:   filepath.Dir(indexFile),
	}
	return index, nil
}

// RemoveFile removes a file from the index, given a file path
func (in *Index) RemoveFile(path string) error {
	relPath, err := in.RelIndexPath(path)
	if err != nil {
		return err
	}
	delete(in.Files, relPath)
	return nil
}

func (in *Index) updateFileHashGiven(path, format, hash string, markAsMetaFile bool) error {
	// Remove format if equal to index hash format
	if in.HashFormat == format {
		format = ""
	}

	// Find in index
	relPath, err := in.RelIndexPath(path)
	if err != nil {
		return err
	}
	in.Files.updateFileEntry(relPath, format, hash, markAsMetaFile)
	return nil
}

// computeFileHash returns the hash string and metafile flag for a path using the given format.
// Pure and goroutine-safe: reads from disk only, no shared state.
func computeFileHash(path string, format string) (hashString string, metaFile bool, err error) {
	if !viper.GetBool("no-internal-hashes") {
		f, err := os.Open(path)
		if err != nil {
			return "", false, err
		}
		h, err := GetHashImpl(format)
		if err != nil {
			_ = f.Close()
			return "", false, err
		}
		if _, err := io.Copy(h, f); err != nil {
			_ = f.Close()
			return "", false, err
		}
		if err := f.Close(); err != nil {
			return "", false, err
		}
		hashString = h.HashToString(h.Sum(nil))
	}
	return hashString, strings.HasSuffix(filepath.Base(path), MetaExtension), nil
}

// updateFile calculates the hash for a given path and updates it in the index.
func (in *Index) updateFile(path string) error {
	hashString, metaFile, err := computeFileHash(path, in.HashFormat)
	if err != nil {
		return err
	}
	return in.updateFileHashGiven(path, in.HashFormat, hashString, metaFile)
}

// ResolveIndexPath turns a path from the index into a file path on disk
func (in Index) ResolveIndexPath(p string) string {
	return filepath.Join(in.packRoot, filepath.FromSlash(p))
}

// RelIndexPath turns a file path on disk into a path from the index
func (in Index) RelIndexPath(p string) (string, error) {
	rel, err := filepath.Rel(in.packRoot, p)
	if err != nil {
		return "", err
	}
	return filepath.ToSlash(rel), nil
}

var ignoreDefaults = []string{
	// Defaults (can be overridden with a negating pattern preceded with !)

	// Exclude Git metadata
	".git/**",
	".gitattributes",
	".gitignore",

	// Exclude macOS metadata
	".DS_Store",

	// Exclude exported CurseForge zip files
	"/*.zip",

	// Exclude exported Modrinth packs
	"*.mrpack",

	// Exclude tool binaries placed in the pack folder
	"packwiz.exe",
	"packwiz",
	"packwand.exe",
	"packwand",
}

func readGitignore(path string) (*gitignore.GitIgnore, bool) {
	data, err := os.ReadFile(path)
	if err != nil {
		// TODO: check for read errors (and present them)
		return gitignore.CompileIgnoreLines(ignoreDefaults...), false
	}

	s := strings.Split(string(data), "\n")
	var lines []string
	lines = append(lines, ignoreDefaults...)
	lines = append(lines, s...)
	return gitignore.CompileIgnoreLines(lines...), true
}

// RefreshStats summarises what changed during a Refresh call.
type RefreshStats struct {
	Added   int
	Updated int
	Removed int
	// HashUpgraded is true when the index hash-format was promoted to DefaultHashFormat.
	HashUpgraded bool
}

// Refresh updates the hashes of all the files in the index, and adds new files to the index.
// It automatically upgrades the index hash-format to DefaultHashFormat when it detects an older format.
func (in *Index) Refresh() (RefreshStats, error) {
	var stats RefreshStats

	// Upgrade legacy hash format transparently.
	if in.HashFormat != DefaultHashFormat {
		in.HashFormat = DefaultHashFormat
		stats.HashUpgraded = true
	}

	// Is case-sensitivity a problem?
	pathPF, _ := filepath.Abs(viper.GetString("pack-file"))
	pathIndex, _ := filepath.Abs(in.indexFile)

	pathIgnore, _ := filepath.Abs(filepath.Join(in.packRoot, ".packwizignore"))
	ignore, ignoreExists := readGitignore(pathIgnore)

	var fileList []string
	err := filepath.WalkDir(in.packRoot, func(path string, info os.DirEntry, err error) error {
		if err != nil {
			return err
		}

		// Never ignore pack root itself (gitignore doesn't allow ignoring the root)
		if path == in.packRoot {
			return nil
		}

		if info.IsDir() {
			// Don't traverse ignored directories (consistent with Git handling of ignored dirs)
			if ignore.MatchesPath(path) {
				return fs.SkipDir
			}
			return nil
		}
		absPath, _ := filepath.Abs(path)
		if absPath == pathPF || absPath == pathIndex {
			return nil
		}
		if ignoreExists {
			if absPath == pathIgnore {
				return nil
			}
		}
		if ignore.MatchesPath(path) {
			return nil
		}

		fileList = append(fileList, path)
		return nil
	})
	if err != nil {
		return stats, err
	}

	// Capture which paths are already indexed so we can distinguish adds from updates.
	knownPaths := make(map[string]bool, len(in.Files))
	for p := range in.Files {
		knownPaths[p] = true
	}

	// Build a cache of existing hashes for mtime-based skip: if a file's mtime
	// predates the index file's mtime, the stored hash is still valid.
	type cachedEntry struct {
		hash string
		meta bool
	}
	existing := make(map[string]cachedEntry, len(in.Files))
	for relPath, holder := range in.Files {
		switch h := holder.(type) {
		case *indexFile:
			existing[relPath] = cachedEntry{h.Hash, h.MetaFile}
		case *indexFileMultipleAlias:
			for _, f := range *h {
				existing[relPath] = cachedEntry{f.Hash, f.MetaFile}
				break
			}
		}
	}
	var indexMtime time.Time
	if fi, err := os.Stat(in.indexFile); err == nil {
		indexMtime = fi.ModTime()
	}

	progressContainer := mpb.New()
	progress := progressContainer.AddBar(int64(len(fileList)),
		mpb.PrependDecorators(
			decor.Name("Refreshing index..."),
			decor.Percentage(decor.WCSyncSpace),
		),
		mpb.AppendDecorators(
			decor.OnComplete(
				decor.EwmaETA(decor.ET_STYLE_GO, 60), "done",
			),
		),
	)

	type hashResult struct {
		path string
		hash string
		meta bool
		err  error
	}
	hashResults := make([]hashResult, len(fileList))
	ParallelFor(fileList, HashConcurrent(), func(i int, v string) {
		// Mtime short-circuit: if the file hasn't changed since the last
		// index write, reuse the stored hash instead of reading the file.
		relPath, _ := in.RelIndexPath(v)
		if !indexMtime.IsZero() {
			if fi, err := os.Stat(v); err == nil && !fi.ModTime().After(indexMtime) {
				if ce, ok := existing[relPath]; ok {
					hashResults[i] = hashResult{v, ce.hash, ce.meta, nil}
					progress.Increment(0)
					return
				}
			}
		}

		start := time.Now()
		hash, meta, err := computeFileHash(v, in.HashFormat)
		hashResults[i] = hashResult{v, hash, meta, err}
		progress.Increment(time.Since(start))
	})
	progress.SetTotal(int64(len(fileList)), true)
	progressContainer.Wait()

	for _, r := range hashResults {
		if r.err != nil {
			return stats, r.err
		}
		relPath, err := in.RelIndexPath(r.path)
		if err != nil {
			return stats, err
		}
		if knownPaths[relPath] {
			stats.Updated++
		} else {
			stats.Added++
		}
		if err := in.updateFileHashGiven(r.path, in.HashFormat, r.hash, r.meta); err != nil {
			return stats, err
		}
	}

	// Remove entries for files that no longer exist on disk.
	for p, file := range in.Files {
		if !file.markedFound() {
			delete(in.Files, p)
			stats.Removed++
		}
	}

	return stats, nil
}

// Write saves the index file
func (in Index) Write() error {
	// Convert to indexTomlRepresentation
	rep := indexTomlRepresentation{
		HashFormat: in.HashFormat,
		Files:      in.Files.toTomlRep(),
	}

	// TODO: calculate and provide hash while writing?
	f, err := os.Create(in.indexFile)
	if err != nil {
		return err
	}

	enc := toml.NewEncoder(f)
	// Disable indentation
	enc.Indent = ""
	err = enc.Encode(rep)
	if err != nil {
		_ = f.Close()
		return err
	}
	return f.Close()
}

// RefreshFileWithHash updates a file in the index, given a file hash and whether it should be marked as metafile or not
func (in *Index) RefreshFileWithHash(path, format, hash string, markAsMetaFile bool) error {
	if viper.GetBool("no-internal-hashes") {
		hash = ""
	}
	return in.updateFileHashGiven(path, format, hash, markAsMetaFile)
}

// FindMod finds a mod in the index and returns its path and whether it has been found
func (in Index) FindMod(modName string) (string, bool) {
	for p, v := range in.Files {
		if v.IsMetaFile() {
			_, fileName := path.Split(p)
			fileTrimmed := strings.TrimSuffix(strings.TrimSuffix(fileName, MetaExtension), MetaExtensionOld)
			if fileTrimmed == modName {
				return in.ResolveIndexPath(p), true
			}
		}
	}
	return "", false
}

// getAllMods finds paths to every metadata file (Mod) in the index
func (in Index) getAllMods() []string {
	var list []string
	for p, v := range in.Files {
		if v.IsMetaFile() {
			list = append(list, in.ResolveIndexPath(p))
		}
	}
	return list
}

// LoadAllMods reads all metadata files into Mod structs
func (in Index) LoadAllMods() ([]*Mod, error) {
	modPaths := in.getAllMods()
	results := make([]*Mod, len(modPaths))
	errs := make([]error, len(modPaths))
	ParallelFor(modPaths, HashConcurrent(), func(i int, v string) {
		modData, err := LoadMod(v)
		if err != nil {
			errs[i] = fmt.Errorf("failed to read metadata file %s: %w", v, err)
			return
		}
		results[i] = &modData
	})
	mods := make([]*Mod, len(modPaths))
	for i, err := range errs {
		if err != nil {
			return nil, err
		}
		mods[i] = results[i]
	}
	return mods, nil
}
