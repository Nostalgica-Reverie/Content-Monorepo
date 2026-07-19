package build

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/manifest"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
)

func queueModBuilds(sched *workspace.Scheduler, modID, sha, artifactsDir string) ([]queuedJob, error) {
	modDir := filepath.Join("mods", modID)
	m, err := manifest.Read(filepath.Join(modDir, "manifest.json"))
	if err != nil {
		return nil, err
	}
	if m.Type != "mod" {
		return nil, fmt.Errorf("%s manifest type is %q, expected mod", modID, m.Type)
	}
	if m.Version == "" {
		return nil, fmt.Errorf("missing 'version' in %s/manifest.json", modDir)
	}
	if len(m.Variants) == 0 {
		return nil, fmt.Errorf("mod %s has no Stonecutter variants", modID)
	}

	name := strings.ReplaceAll(m.Name, " ", "-")
	jobs := make([]queuedJob, 0, len(m.Variants))
	for _, variant := range m.Variants {
		variant := variant
		key := variant.ID
		if key == "" {
			key = variant.MCVersion
		}
		version := variant.Version
		if version == "" {
			version = m.Version
		}
		outputName := fmt.Sprintf("%s-%s-%s-%s.jar", name, version, key, sha)
		outputPath := filepath.Join(artifactsDir, outputName)
		done := sched.Submit(workspace.Task{
			Name:  outputName,
			Needs: []workspace.Resource{workspace.Resource("gradle:" + modDir)},
			Run: func() error {
				jar, err := buildModProject(modDir, variant.GradleProject)
				if err != nil {
					return err
				}
				if err := copyModArtifact(jar, outputPath); err != nil {
					return err
				}
				fmt.Printf("built %s\n", outputName)
				return nil
			},
		})
		jobs = append(jobs, queuedJob{label: outputName, done: done})
	}
	return jobs, nil
}

func pubBuildMod(modDir, artifactsDir string, r *pubResolved) {
	jar, err := buildModProject(modDir, r.gradleProject)
	if err != nil {
		cmd.Fail(err.Error())
	}
	outFile := filepath.Join(artifactsDir, pubArtifactName(*r, platModrinth))
	if err := copyModArtifact(jar, outFile); err != nil {
		cmd.Fail(err.Error())
	}
	fmt.Printf("built %s\n", outFile)
	r.builtMR = r.mrID != ""
	r.builtCF = r.cfID != ""
}

func buildModProject(modDir, project string) (string, error) {
	if !validGradleProjectName(project) {
		return "", fmt.Errorf("invalid or missing Gradle project %q", project)
	}

	task := ":" + project + ":build"
	var c *exec.Cmd
	if runtime.GOOS == "windows" {
		c = exec.Command("cmd", "/c", "gradlew.bat", "--no-daemon", task)
	} else {
		c = exec.Command("./gradlew", "--no-daemon", task)
	}
	c.Dir = modDir
	workspace.ConfigureSubprocess(c)
	if out, err := c.CombinedOutput(); err != nil {
		return "", fmt.Errorf("Gradle task %s failed in %s: %v\n%s", task, modDir, err, workspace.Indent(string(out), "    "))
	}

	return findBuiltModJar(filepath.Join(modDir, "versions", project, "build", "libs"))
}

func validGradleProjectName(project string) bool {
	if project == "" {
		return false
	}
	for _, r := range project {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || strings.ContainsRune("._-", r) {
			continue
		}
		return false
	}
	return true
}

func findBuiltModJar(libsDir string) (string, error) {
	entries, err := os.ReadDir(libsDir)
	if err != nil {
		return "", fmt.Errorf("read Gradle output %s: %w", libsDir, err)
	}
	var candidates []string
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(strings.ToLower(entry.Name()), ".jar") || isAuxiliaryModJar(entry.Name()) {
			continue
		}
		candidates = append(candidates, filepath.Join(libsDir, entry.Name()))
	}
	sort.Strings(candidates)
	if len(candidates) != 1 {
		return "", fmt.Errorf("expected exactly one distributable jar in %s, found %d: %v", libsDir, len(candidates), candidates)
	}
	return candidates[0], nil
}

func isAuxiliaryModJar(name string) bool {
	lower := strings.ToLower(name)
	for _, suffix := range []string{"-sources.jar", "-javadoc.jar", "-dev.jar", "-dev-shadow.jar"} {
		if strings.HasSuffix(lower, suffix) {
			return true
		}
	}
	return false
}

func copyModArtifact(src, dest string) error {
	in, err := os.Open(src)
	if err != nil {
		return fmt.Errorf("open built mod jar %s: %w", src, err)
	}
	defer in.Close()
	out, err := os.Create(dest)
	if err != nil {
		return fmt.Errorf("create mod artifact %s: %w", dest, err)
	}
	if _, err := io.Copy(out, in); err != nil {
		out.Close()
		return fmt.Errorf("copy mod artifact to %s: %w", dest, err)
	}
	if err := out.Close(); err != nil {
		return fmt.Errorf("close mod artifact %s: %w", dest, err)
	}
	return nil
}
