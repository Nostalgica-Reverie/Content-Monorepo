package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmdshared"
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/workspace"
)

// llStartCwd is captured at startup before any chdir by workspace commands.
var llStartCwd string

func init() {
	llStartCwd, _ = os.Getwd()
}

// Exported variants for use by sub-packages (build/, content/, etc.)

func Fail(msg string)                         { llFail(msg) }
func Warn(format string, a ...any)            { llWarn(format, a...) }
func ErrFile(path, format string, a ...any)   { llErrFile(path, format, a...) }
func IsTTY() bool                             { return llIsTTY() }
func Chdir()                                  { llChdir() }
func Abs(p string) string                     { return llAbs(p) }
func WriteJSON(path string, v any)            { llWriteJSON(path, v) }
func PlatformSuffix(s string) string          { return llPlatformSuffix(s) }

// llAbs resolves p relative to llStartCwd (the directory the user ran packwand from).
func llAbs(p string) string {
	if p == "" || filepath.IsAbs(p) {
		return p
	}
	return filepath.Join(llStartCwd, p)
}

// llFail prints msg to stderr in TTY or CI annotation format, then exits 1.
func llFail(msg string) { cmdshared.Fail(msg) }

// llWarn prints a warning in TTY or CI annotation format.
func llWarn(format string, a ...any) { cmdshared.Warn(format, a...) }

// llErrFile prints a file-annotated error in TTY or CI annotation format.
func llErrFile(path, format string, a ...any) { cmdshared.ErrFile(path, format, a...) }

// llIsTTY reports whether stdout is a terminal.
func llIsTTY() bool { return cmdshared.IsTTY() }

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
