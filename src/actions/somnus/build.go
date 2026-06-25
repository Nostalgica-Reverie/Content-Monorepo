package main

import (
	"archive/zip"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
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

func cmdBuild(args []string) {
	var targetedPack, shortSHA string
	if len(args) >= 1 && args[0] == "--pack" {
		if len(args) < 3 {
			failUsage(verbUsage["build"])
		}
		targetedPack = args[1]
		shortSHA = args[2]
	} else if len(args) >= 1 {
		shortSHA = args[0]
	} else {
		failUsage(verbUsage["build"])
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
			failNotFound(err.Error())
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

	sched := NewScheduler(maxConcurrent())
	slots := cacheSlotCount()

	type pending struct {
		label string
		done  <-chan error
	}
	var jobs []pending

	for _, t := range changed {
		switch t.category {
		case "modpacks":
			ps, err := queueModpackExports(sched, slots, t.pack, shortSHA, artifactsDir)
			if err != nil {
				fail(fmt.Sprintf("modpack '%s' failed to enqueue: %v", t.pack, err))
			}
			for _, p := range ps {
				jobs = append(jobs, pending{label: p.label, done: p.done})
			}
		case "datapacks", "resourcepacks":
			p, err := queueZipPackBuild(sched, t.category, t.pack, shortSHA, artifactsDir)
			if err != nil {
				fail(fmt.Sprintf("%s '%s' failed to enqueue: %v", t.category, t.pack, err))
			}
			jobs = append(jobs, pending{label: p.label, done: p.done})
		default:
			fmt.Printf("category '%s' does not require a build.\n", t.category)
		}
	}

	var failed int
	for _, j := range jobs {
		if err := <-j.done; err != nil {
			fmt.Fprintf(os.Stderr, "build failed: %s: %v\n", j.label, err)
			failed++
		}
	}
	if failed > 0 {
		fail(fmt.Sprintf("%d build(s) failed", failed))
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

	for line := range strings.SplitSeq(string(out), "\n") {
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

func subdirKeys(m *Manifest) []string {
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

type queuedJob struct {
	label string
	done  <-chan error
}

func queueModpackExports(sched *Scheduler, slots int, packID, sha, artifactsDir string) ([]queuedJob, error) {
	fmt.Printf("queueing modpack exports: %s\n", packID)
	packDir := filepath.Join("modpacks", packID)

	m, err := ReadManifest(filepath.Join(packDir, "manifest.json"))
	if err != nil {
		return nil, err
	}
	if m.Version == "" {
		return nil, fmt.Errorf("missing 'version' in %s/manifest.json", packDir)
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
			return nil, fmt.Errorf("failed to read %s: %w", packDir, err)
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
		return nil, fmt.Errorf("no valid version dirs (expected '{key}-mr' or '{key}-cf') for %s", packID)
	}

	out := make([]queuedJob, 0, len(jobs))
	for _, j := range jobs {
		outputName := fmt.Sprintf("%s-%s-%s-%s-%s.%s",
			packID, j.subKey, j.plat.short, m.Version, sha, j.plat.ext)
		outputPath := filepath.Join(artifactsDir, outputName)
		done := sched.Submit(Task{
			Name: outputName,
			Needs: []Resource{
				Resource("export:" + j.dir),
				CacheSlot(j.dir, slots),
			},
			Run: func() error {
				cmd := exec.Command(packwizBin(), j.plat.cli, "export", "--output", outputPath)
				cmd.Dir = j.dir
				if out, err := cmd.CombinedOutput(); err != nil {
					return fmt.Errorf("packwiz export failed for %s: %v\n%s", j.dir, err, out)
				}
				fmt.Printf("exported %s\n", outputName)
				return nil
			},
		})
		out = append(out, queuedJob{label: outputName, done: done})
	}
	return out, nil
}

func queueZipPackBuild(sched *Scheduler, category, packID, sha, artifactsDir string) (queuedJob, error) {
	packDir := filepath.Join(category, packID)

	versionDir, version, err := findSingleVersionDir(packDir)
	if err != nil {
		return queuedJob{}, err
	}

	dest := filepath.Join(artifactsDir, fmt.Sprintf("%s-%s-%s.zip", packID, version, sha))
	label := filepath.Base(dest)

	done := sched.Submit(Task{
		Name:  label,
		Needs: []Resource{Resource("zip:" + packDir)},
		Run: func() error {
			if category == "resourcepacks" {
				if bin := packsquashBin(); bin != "" {
					fmt.Printf("squashing %s: %s (PackSquash)\n", category, packID)
					return squashContents(bin, packDir, versionDir, dest)
				}
				fmt.Printf("zipping %s: %s (packsquash not found; plain zip — install it or set PACKSQUASH_BIN for optimized builds)\n", category, packID)
			} else {
				fmt.Printf("zipping %s: %s\n", category, packID)
			}
			return zipContents(versionDir, dest)
		},
	})
	return queuedJob{label: label, done: done}, nil
}

func packsquashBin() string {
	if b := os.Getenv("PACKSQUASH_BIN"); b != "" {
		return b
	}
	if p, err := exec.LookPath("packsquash"); err == nil {
		return p
	}
	return ""
}

func squashContents(bin, packDir, src, dest string) error {
	opts := fmt.Sprintf("pack_directory = %q\noutput_file_path = %q\nzip_spec_conformance_level = \"high\"\n", src, dest)
	if extra, err := os.ReadFile(filepath.Join(packDir, "packsquash.toml")); err == nil {
		fmt.Printf("  applying pack-level packsquash.toml\n")
		opts += "\n" + string(extra)
	}
	optFile, err := os.CreateTemp("", "somnus-packsquash-*.toml")
	if err != nil {
		return fmt.Errorf("failed to create packsquash options file: %w", err)
	}
	defer os.Remove(optFile.Name())
	if _, err := optFile.WriteString(opts); err != nil {
		return fmt.Errorf("failed to write packsquash options: %w", err)
	}
	optFile.Close()

	cmd := exec.Command(bin, optFile.Name())
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("packsquash failed for %s: %v\n%s", src, err, indent(string(out), "    "))
	}
	fmt.Printf("squashed %s\n", filepath.Base(dest))
	return nil
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
