package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"packwand/workspace"
)

// llStartCwd is captured at startup before any chdir by workspace commands.
var llStartCwd string

func init() {
	llStartCwd, _ = os.Getwd()
}

// llAbs resolves p relative to llStartCwd (the directory the user ran packwand from).
func llAbs(p string) string {
	if p == "" || filepath.IsAbs(p) {
		return p
	}
	return filepath.Join(llStartCwd, p)
}

// llFail prints msg to stderr in TTY or CI annotation format, then exits 1.
func llFail(msg string) {
	if llIsTTY() {
		fmt.Fprintln(os.Stderr, "error: "+msg)
	} else {
		fmt.Fprintln(os.Stderr, "::error::"+msg)
	}
	os.Exit(1)
}

// llWarn prints a warning in TTY or CI annotation format.
func llWarn(format string, a ...any) {
	msg := fmt.Sprintf(format, a...)
	if llIsTTY() {
		fmt.Fprintf(os.Stderr, "warning: %s\n", msg)
	} else {
		fmt.Fprintf(os.Stderr, "::warning::%s\n", msg)
	}
}

// llErrFile prints a file-annotated error in TTY or CI annotation format.
func llErrFile(path, format string, a ...any) {
	msg := fmt.Sprintf(format, a...)
	if llIsTTY() {
		fmt.Fprintf(os.Stderr, "  error: %s: %s\n", path, msg)
	} else {
		fmt.Fprintf(os.Stderr, "::error file=%s::%s\n", path, msg)
	}
}

// llIsTTY reports whether stdout is a terminal.
func llIsTTY() bool {
	fi, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return fi.Mode()&os.ModeCharDevice != 0
}

// llChdir changes directory to the repo root, calling llFail if it cannot be found.
func llChdir() {
	root := workspace.FindRepoRoot()
	if root == "" {
		llFail("could not locate repo root (no .git or modpacks/ found walking up from here)")
	}
	if err := os.Chdir(root); err != nil {
		llFail(fmt.Sprintf("failed to enter repo root %s: %v", root, err))
	}
}

// llWriteJSON writes v as indented JSON to path, calling llFail on error.
func llWriteJSON(path string, v any) {
	if err := workspace.WriteJSON(path, v); err != nil {
		llFail(err.Error())
	}
}

// llPlatformSuffix returns "mr", "cf", or "" from a subdir name ending in -mr/-cf.
func llPlatformSuffix(s string) string {
	if strings.HasSuffix(s, "-mr") {
		return "mr"
	}
	if strings.HasSuffix(s, "-cf") {
		return "cf"
	}
	return ""
}

// splitKV splits a TOML "key = value" line into key and unquoted value.
// Returns ok=false if no '=' is present.
func splitKV(line string) (key, val string, ok bool) {
	idx := strings.Index(line, "=")
	if idx < 0 {
		return "", "", false
	}
	key = strings.TrimSpace(line[:idx])
	val = strings.TrimSpace(line[idx+1:])
	val = strings.Trim(val, `"`)
	return key, val, true
}
