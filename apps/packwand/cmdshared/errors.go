package cmdshared

import (
	"fmt"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/clistyle"
)

// IsTTY reports whether stdout is a terminal.
func IsTTY() bool {
	fi, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return fi.Mode()&os.ModeCharDevice != 0
}

// Fail prints msg to stderr (styled on terminals, CI annotation format
// otherwise), then exits 1.
func Fail(msg string) {
	if clistyle.Interactive() {
		fmt.Fprintln(os.Stderr, clistyle.ErrorText.Render("✗ error: ")+msg)
	} else if IsTTY() {
		fmt.Fprintln(os.Stderr, "error: "+msg)
	} else {
		fmt.Fprintln(os.Stderr, "::error::"+msg)
	}
	os.Exit(1)
}

// Failf formats a message and calls Fail.
func Failf(format string, a ...any) {
	Fail(fmt.Sprintf(format, a...))
}

// Warn prints a warning (styled on terminals, CI annotation format otherwise).
func Warn(format string, a ...any) {
	msg := fmt.Sprintf(format, a...)
	if clistyle.Interactive() {
		fmt.Fprintln(os.Stderr, clistyle.WarnText.Render("⚠ warning: ")+msg)
	} else if IsTTY() {
		fmt.Fprintf(os.Stderr, "warning: %s\n", msg)
	} else {
		fmt.Fprintf(os.Stderr, "::warning::%s\n", msg)
	}
}

// ErrFile prints a file-annotated error in TTY or CI annotation format.
func ErrFile(path, format string, a ...any) {
	msg := fmt.Sprintf(format, a...)
	if IsTTY() {
		fmt.Fprintf(os.Stderr, "  error: %s: %s\n", path, msg)
	} else {
		fmt.Fprintf(os.Stderr, "::error file=%s::%s\n", path, msg)
	}
}
