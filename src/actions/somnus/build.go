package main

import (
	"archive/zip"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
)

type platform struct {
	short string
	cli   string
	ext   string
}

var (
	modrinth   = platform{short: "mr", cli: "modrinth", ext: "mrpack"}
	curseforge = platform{short: "cf", cli: "curseforge", ext: "zip"}
)

func platformFromSuffix(s string) (platform, bool) {
	switch s {
	case "mr":
		return modrinth, true
	case "cf":
		return curseforge, true
	default:
		return platform{}, false
	}
}

type target struct {
	category string
	pack     string
}

type exportJob struct {
	plat   platform
	dir    string
	subKey string
}

// cmdBuild is the CI entry: `somnus build <sha>` builds what git changed;
// `somnus build --pack <name> <sha>` builds one named pack.
func cmdBuild(args []string) {
	var targetedPack, shortSHA string
	if len(args) >= 1 && args[0] == "--pack" {
		if len(args) < 3 {
			fail("usage: somnus build --pack <name> <short-sha>")
		}
		targetedPack = args[1]
		shortSHA = args[2]
	} else if len(args) >= 1 {
		shortSHA = args[0]
	} else {
		fail("usage: somnus build <short-sha>  |  somnus build --pack <name> <short-sha>")
	}
	runBuild(targetedPack, shortSHA)
}

func cmdExport(args []string) {
	if len(args) > 0 {
		runBuild(args[0], "local")
		return
	}
	runBuild("", "local")
}

func runBuild(targetedPack, shortSHA string) {
	repoRoot, err := os.Getwd()
	if err != nil {
		fail(fmt.Sprintf("failed to get current directory: %v", err))
	}
	artifactsDir := filepath.Join(repoRoot, "artifacts")
	if err := os.MkdirAll(artifactsDir, 0o755); err != nil {
		fail(fmt.Sprintf("failed to create %s: %v", artifactsDir, err))
	}

	var changed []target
	if targetedPack != "" {
		t, err := resolvePack(targetedPack)
		if err != nil {
			fail(err.Error())
		}
		changed = []target{t}
	} else {
		changed, err = detectChangedTargets()
		if err != nil {
			fail(fmt.Sprintf("failed to detect changed targets: %v", err))
		}
		if len(changed) == 0 {
			fmt.Println("no packs detected in git diff.")
			return
		}
	}

	for _, t := range changed {
		switch t.category {
		case "modpacks":
			if err := buildModpack(t.pack, shortSHA, artifactsDir); err != nil {
				fail(fmt.Sprintf("modpack '%s' failed: %v", t.pack, err))
			}
		case "datapacks", "resourcepacks":
			if err := buildZipPack(t.category, t.pack, shortSHA, artifactsDir); err != nil {
				fail(fmt.Sprintf("%s '%s' failed: %v", t.category, t.pack, err))
			}
		default:
			fmt.Printf("category '%s' does not require a build.\n", t.category)
		}
	}

	fmt.Println("all builds completed successfully.")
}

func resolvePack(name string) (target, error) {
	for _, category := range []string{"modpacks", "datapacks", "resourcepacks"} {
		if info, err := os.Stat(filepath.Join(category, name)); err == nil && info.IsDir() {
			return target{category: category, pack: name}, nil
		}
	}
	return target{}, fmt.Errorf("pack '%s' not found in modpacks/, datapacks/, or resourcepacks/", name)
}

func detectChangedTargets() ([]target, error) {
	fmt.Println("detecting changed files...")
	out, err := exec.Command("git", "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to invoke git: %w", err)
	}

	seen := make(map[target]struct{})
	var targets []target

	for _, line := range strings.Split(string(out), "\n") {
		if line == "" || strings.HasPrefix(line, "external/") || strings.HasPrefix(line, ".actions/") {
			continue
		}
		parts := strings.SplitN(line, "/", 3)
		if len(parts) < 2 {
			continue
		}
		t := target{category: parts[0], pack: parts[1]}
		if _, ok := seen[t]; !ok {
			seen[t] = struct{}{}
			targets = append(targets, t)
		}
	}
	return targets, nil
}

func subdirKeys(m *manifest) []string {
	if len(m.Variants) == 0 {
		return nil
	}
	keys := make([]string, 0, len(m.Variants))
	for _, v := range m.Variants {
		if v.ID != "" {
			keys = append(keys, v.ID)
		} else {
			keys = append(keys, v.MCVersion)
		}
	}
	return keys
}

func buildModpack(packID, sha string, artifactsDir string) error {
	fmt.Printf("building modpack: %s\n", packID)
	packDir := filepath.Join("modpacks", packID)

	m, err := readManifest(filepath.Join(packDir, "manifest.json"))
	if err != nil {
		return err
	}
	if m.Version == "" {
		return fmt.Errorf("missing 'version' in %s/manifest.json", packDir)
	}

	var jobs []exportJob
	keys := subdirKeys(m)

	if keys != nil {
		for _, key := range keys {
			for _, plat := range []platform{modrinth, curseforge} {
				dir := filepath.Join(packDir, key+"-"+plat.short)
				if info, err := os.Stat(dir); err == nil && info.IsDir() {
					jobs = append(jobs, exportJob{plat: plat, dir: dir, subKey: key})
				}
			}
		}
	} else {
		entries, err := os.ReadDir(packDir)
		if err != nil {
			return fmt.Errorf("failed to read %s: %w", packDir, err)
		}
		for _, e := range entries {
			if !e.IsDir() {
				continue
			}
			name := e.Name()
			idx := strings.LastIndex(name, "-")
			if idx < 0 {
				continue
			}
			key, suffix := name[:idx], name[idx+1:]
			plat, ok := platformFromSuffix(suffix)
			if !ok {
				continue
			}
			jobs = append(jobs, exportJob{plat: plat, dir: filepath.Join(packDir, name), subKey: key})
		}
	}

	if len(jobs) == 0 {
		return fmt.Errorf("no valid version dirs (expected '{key}-mr' or '{key}-cf') for %s", packID)
	}

	var wg sync.WaitGroup
	errCh := make(chan error, len(jobs))

	for _, j := range jobs {
		wg.Add(1)
		go func(j exportJob) {
			defer wg.Done()
			outputName := fmt.Sprintf("%s-%s-%s-%s-%s.%s",
				packID, j.subKey, j.plat.short, m.Version, sha, j.plat.ext)
			outputPath := filepath.Join(artifactsDir, outputName)

			cmd := exec.Command("packwiz", j.plat.cli, "export", "--output", outputPath)
			cmd.Dir = j.dir
			if out, err := cmd.CombinedOutput(); err != nil {
				errCh <- fmt.Errorf("packwiz export failed for %s: %v\n%s", j.dir, err, out)
				return
			}
			fmt.Printf("exported %s\n", outputName)
		}(j)
	}

	wg.Wait()
	close(errCh)

	var errs []error
	for e := range errCh {
		errs = append(errs, e)
	}
	if len(errs) > 0 {
		for _, e := range errs {
			fmt.Fprintf(os.Stderr, "error: %v\n", e)
		}
		return fmt.Errorf("%d export(s) failed for %s", len(errs), packID)
	}
	return nil
}

func buildZipPack(category, packID, sha, artifactsDir string) error {
	fmt.Printf("zipping %s: %s\n", category, packID)
	packDir := filepath.Join(category, packID)

	versionDir, version, err := findSingleVersionDir(packDir)
	if err != nil {
		return err
	}

	dest := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s-%s.zip", packID, version, sha))
	return zipContents(versionDir, dest)
}

func findSingleVersionDir(packDir string) (dir string, version string, err error) {
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return "", "", fmt.Errorf("failed to read %s: %w", packDir, err)
	}
	var versionDirs []string
	for _, e := range entries {
		if e.IsDir() {
			versionDirs = append(versionDirs, e.Name())
		}
	}
	switch len(versionDirs) {
	case 0:
		return "", "", fmt.Errorf("no version directory found in %s (expected %s/{version}/)", packDir, packDir)
	case 1:
		return filepath.Join(packDir, versionDirs[0]), versionDirs[0], nil
	default:
		return "", "", fmt.Errorf("expected exactly one version directory in %s, found %d: %v", packDir, len(versionDirs), versionDirs)
	}
}

func zipContents(src, dest string) error {
	f, err := os.Create(dest)
	if err != nil {
		return fmt.Errorf("failed to create %s: %w", dest, err)
	}
	defer f.Close()

	zw := zip.NewWriter(f)
	defer zw.Close()

	return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		if rel == "." {
			return nil
		}
		zipName := filepath.ToSlash(rel)
		if info.IsDir() {
			_, err := zw.Create(zipName + "/")
			return err
		}
		w, err := zw.Create(zipName)
		if err != nil {
			return err
		}
		in, err := os.Open(path)
		if err != nil {
			return fmt.Errorf("failed to open %s: %w", path, err)
		}
		defer in.Close()
		_, err = io.Copy(w, in)
		return err
	})
}
