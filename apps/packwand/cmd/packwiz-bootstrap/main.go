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
  --checksums-url <url>  URL of a SHA-256 file to verify a downloaded jar against
                         (a bare hash, or "hash  filename" lines à la sha256sum)
  --no-verify            Acknowledge an unverifiable download (silences the warning)
  -h, --help             Show this message

Downloads are verified whenever a hash is available: --sha256 wins, then
--checksums-url, then a "<download-url>.sha256" sibling file is probed
automatically. With no hash source at all, a prominent warning is printed
unless --no-verify is given.

All other arguments are passed through to packwiz-installer (e.g. -g, -s server, the pack URL).`

type options struct {
	java         string
	minJava      int
	jar          string
	downloadURL  string
	sha256       string
	checksumsURL string
	noVerify     bool
	passthrough  []string
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
		case "--checksums-url":
			opts.checksumsURL, err = needValue(i, "--checksums-url")
			i++
		case "--no-verify":
			opts.noVerify = true
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

	// Verification is the default path: the downloaded jar is executed code,
	// so treat the download as a trust boundary and verify whenever any hash
	// source exists (explicit flag, checksums file, or auto-probed sibling).
	expected, source := expectedSha256(opts)
	if expected != "" {
		got := hex.EncodeToString(hasher.Sum(nil))
		if !strings.EqualFold(got, expected) {
			return "", fmt.Errorf("sha256 mismatch for downloaded jar (expected hash from %s)\n  expected: %s\n  got:      %s", source, expected, got)
		}
		fmt.Printf("verified downloaded jar (sha256 from %s)\n", source)
	} else if !opts.noVerify {
		fmt.Fprintln(os.Stderr, "packwiz-bootstrap: WARNING: downloaded jar could not be verified — no --sha256,")
		fmt.Fprintln(os.Stderr, "packwiz-bootstrap: WARNING: no --checksums-url, and no .sha256 file next to the download URL.")
		fmt.Fprintln(os.Stderr, "packwiz-bootstrap: WARNING: pass --no-verify to acknowledge running unverified code.")
	}
	if err := os.Rename(tmp.Name(), jar); err != nil {
		return "", err
	}
	fmt.Println("downloaded packwiz-installer.jar")
	return jar, nil
}

// expectedSha256 resolves the expected hash for a downloaded jar, preferring
// --sha256, then --checksums-url, then a "<download-url>.sha256" sibling.
// Returns the hash (or "") and a human-readable source label.
func expectedSha256(opts options) (hash, source string) {
	if opts.sha256 != "" {
		return opts.sha256, "--sha256"
	}
	jarName := filepath.Base(strings.SplitN(opts.downloadURL, "?", 2)[0])
	if opts.checksumsURL != "" {
		if h := fetchChecksum(opts.checksumsURL, jarName); h != "" {
			return h, "--checksums-url"
		}
		fmt.Fprintf(os.Stderr, "packwiz-bootstrap: WARNING: no usable entry for %s in %s\n", jarName, opts.checksumsURL)
		return "", ""
	}
	sibling := strings.SplitN(opts.downloadURL, "?", 2)[0] + ".sha256"
	if h := fetchChecksum(sibling, jarName); h != "" {
		return h, sibling
	}
	return "", ""
}

var sha256Pattern = regexp.MustCompile(`^[0-9a-fA-F]{64}$`)

// fetchChecksum downloads a checksum file and extracts the SHA-256 for
// jarName. Accepts a bare hash, or sha256sum-style "hash  filename" lines
// (also matching "*filename" binary-mode markers). Returns "" on any failure
// — the caller decides whether unverified is acceptable.
func fetchChecksum(url, jarName string) string {
	resp, err := http.Get(url)
	if err != nil {
		return ""
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		return ""
	}
	body, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
	if err != nil {
		return ""
	}
	lines := strings.Split(strings.TrimSpace(string(body)), "\n")
	if len(lines) == 1 && sha256Pattern.MatchString(strings.TrimSpace(lines[0])) {
		return strings.TrimSpace(lines[0])
	}
	for _, line := range lines {
		fields := strings.Fields(line)
		if len(fields) < 2 || !sha256Pattern.MatchString(fields[0]) {
			continue
		}
		name := strings.TrimPrefix(fields[len(fields)-1], "*")
		if filepath.Base(name) == jarName {
			return fields[0]
		}
	}
	return ""
}
