// packwiz-bootstrap is a native replacement for packwiz-installer-bootstrap:
// it verifies a Java runtime, ensures packwiz-installer.jar is present
// (optionally downloading and checksumming it), launches the installer with
// the remaining arguments, and passes its exit code through.
//
// It deliberately depends only on the standard library so the binary stays
// small and starts fast as a launcher pre-launch command.
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
)

const usage = `usage: packwiz-bootstrap [options] <pack.toml URL> [installer options...]

Bootstrap options:
  --java <path>          Path to the java executable (default: $JAVA_HOME/bin/java, then PATH)
  --min-java <version>   Minimum Java major version to accept (default: 8)
  --jar <path>           Location of packwiz-installer.jar (default: next to this executable)
  --download-url <url>   URL to download packwiz-installer.jar from when missing
  --sha256 <hash>        Expected SHA-256 of a downloaded jar
  -h, --help             Show this message

All other arguments are passed through to packwiz-installer (e.g. -g, -s server, the pack URL).`

type options struct {
	java        string
	minJava     int
	jar         string
	downloadURL string
	sha256      string
	passthrough []string
}

func fail(format string, args ...any) {
	fmt.Fprintf(os.Stderr, "packwiz-bootstrap: "+format+"\n", args...)
	os.Exit(1)
}

func main() {
	opts, err := parseArgs(os.Args[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, "packwiz-bootstrap: "+err.Error())
		fmt.Fprintln(os.Stderr, usage)
		os.Exit(2)
	}
	if len(opts.passthrough) == 0 {
		fmt.Fprintln(os.Stderr, usage)
		os.Exit(2)
	}

	java, version, err := findJava(opts.java)
	if err != nil {
		fail("%v", err)
	}
	if version < opts.minJava {
		fail("Java %d found at %s, but at least Java %d is required", version, java, opts.minJava)
	}
	fmt.Printf("using Java %d (%s)\n", version, java)

	jar, err := ensureJar(opts)
	if err != nil {
		fail("%v", err)
	}

	// The jar's Main-Class (RequiresBootstrap) is a guard that always refuses;
	// like the Java bootstrap, invoke the real entry point via the classpath.
	args := append([]string{"-cp", jar, "link.infra.packwiz.installer.Main"}, opts.passthrough...)
	c := exec.Command(java, args...)
	c.Stdin = os.Stdin
	c.Stdout = os.Stdout
	c.Stderr = os.Stderr
	if err := c.Run(); err != nil {
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			os.Exit(exitErr.ExitCode())
		}
		fail("failed to run packwiz-installer: %v", err)
	}
}

func parseArgs(args []string) (options, error) {
	opts := options{minJava: 8}
	needValue := func(i int, name string) (string, error) {
		if i+1 >= len(args) {
			return "", fmt.Errorf("%s requires a value", name)
		}
		return args[i+1], nil
	}
	for i := 0; i < len(args); i++ {
		var err error
		switch args[i] {
		case "--java":
			opts.java, err = needValue(i, "--java")
			i++
		case "--min-java":
			var v string
			v, err = needValue(i, "--min-java")
			if err == nil {
				opts.minJava, err = strconv.Atoi(v)
			}
			i++
		case "--jar":
			opts.jar, err = needValue(i, "--jar")
			i++
		case "--download-url":
			opts.downloadURL, err = needValue(i, "--download-url")
			i++
		case "--sha256":
			opts.sha256, err = needValue(i, "--sha256")
			i++
		case "-h", "--help":
			fmt.Println(usage)
			os.Exit(0)
		default:
			opts.passthrough = append(opts.passthrough, args[i])
		}
		if err != nil {
			return options{}, err
		}
	}
	return opts, nil
}

var javaVersionPattern = regexp.MustCompile(`version "([^"]+)"`)

// findJava locates a java executable and returns its path and major version.
func findJava(explicit string) (string, int, error) {
	candidates := []string{}
	if explicit != "" {
		candidates = append(candidates, explicit)
	} else {
		if home := os.Getenv("JAVA_HOME"); home != "" {
			candidates = append(candidates, filepath.Join(home, "bin", "java"))
		}
		candidates = append(candidates, "java")
	}
	var lastErr error
	for _, candidate := range candidates {
		out, err := exec.Command(candidate, "-version").CombinedOutput()
		if err != nil {
			lastErr = fmt.Errorf("could not run %s: %w", candidate, err)
			continue
		}
		version, err := parseJavaMajor(string(out))
		if err != nil {
			lastErr = fmt.Errorf("%s: %w", candidate, err)
			continue
		}
		path := candidate
		if resolved, err := exec.LookPath(candidate); err == nil {
			path = resolved
		}
		return path, version, nil
	}
	if lastErr == nil {
		lastErr = fmt.Errorf("no java executable found; install a JRE/JDK or pass --java")
	}
	return "", 0, lastErr
}

// parseJavaMajor extracts the major version from `java -version` output,
// handling both legacy ("1.8.0_392") and modern ("17.0.2", "25.0.1") schemes.
func parseJavaMajor(output string) (int, error) {
	m := javaVersionPattern.FindStringSubmatch(output)
	if m == nil {
		return 0, fmt.Errorf("could not parse java -version output")
	}
	parts := strings.Split(m[1], ".")
	idx := 0
	if parts[0] == "1" && len(parts) > 1 {
		idx = 1
	}
	major, err := strconv.Atoi(strings.SplitN(parts[idx], "-", 2)[0])
	if err != nil {
		return 0, fmt.Errorf("could not parse java version %q", m[1])
	}
	return major, nil
}

// ensureJar returns the path to packwiz-installer.jar, downloading it when
// missing and a download URL was provided.
func ensureJar(opts options) (string, error) {
	jar := opts.jar
	if jar == "" {
		exe, err := os.Executable()
		if err != nil {
			return "", fmt.Errorf("could not locate own executable: %w", err)
		}
		jar = filepath.Join(filepath.Dir(exe), "packwiz-installer.jar")
	}
	if _, err := os.Stat(jar); err == nil {
		return jar, nil
	}
	if opts.downloadURL == "" {
		return "", fmt.Errorf("packwiz-installer.jar not found at %s (pass --jar or --download-url)", jar)
	}

	fmt.Printf("downloading packwiz-installer.jar from %s\n", opts.downloadURL)
	resp, err := http.Get(opts.downloadURL)
	if err != nil {
		return "", fmt.Errorf("download failed: %w", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("download failed: HTTP %d from %s", resp.StatusCode, opts.downloadURL)
	}

	tmp, err := os.CreateTemp(filepath.Dir(jar), "packwiz-installer-*.jar.tmp")
	if err != nil {
		return "", err
	}
	defer func() { _ = os.Remove(tmp.Name()) }()

	hasher := sha256.New()
	if _, err := io.Copy(io.MultiWriter(tmp, hasher), resp.Body); err != nil {
		_ = tmp.Close()
		return "", fmt.Errorf("download failed: %w", err)
	}
	if err := tmp.Close(); err != nil {
		return "", err
	}

	if opts.sha256 != "" {
		got := hex.EncodeToString(hasher.Sum(nil))
		if !strings.EqualFold(got, opts.sha256) {
			return "", fmt.Errorf("sha256 mismatch for downloaded jar\n  expected: %s\n  got:      %s", opts.sha256, got)
		}
	}
	if err := os.Rename(tmp.Name(), jar); err != nil {
		return "", err
	}
	fmt.Println("downloaded packwiz-installer.jar")
	return jar, nil
}
