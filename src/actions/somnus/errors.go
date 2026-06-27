package main

import (
	"fmt"
	"os"
)

const (
	ExitOK       = 0
	ExitRuntime  = 1
	ExitUsage    = 2
	ExitEnv      = 3
	ExitNotFound = 4
)

func fail(msg string) {
	failWith(ExitRuntime, msg, "")
}

func failUsage(msg string) {
	failWith(ExitUsage, msg, "run 'somnus help' for the full verb list")
}

func failEnv(msg, hint string) {
	failWith(ExitEnv, msg, hint)
}

func failNotFound(msg string) {
	failWith(ExitNotFound, msg, "")
}

func failWith(code int, msg, hint string) {
	if isTTY() {
		fmt.Fprintf(os.Stderr, "error: %s\n", msg)
	} else {
		fmt.Fprintf(os.Stderr, "::error::%s\n", msg)
	}
	if hint != "" {
		fmt.Fprintf(os.Stderr, "  hint: %s\n", hint)
	}
	os.Exit(code)
}

// warnf prints a warning — plain on a TTY, ::warning:: annotation in CI.
func warnf(format string, args ...any) {
	msg := fmt.Sprintf(format, args...)
	if isTTY() {
		fmt.Fprintf(os.Stderr, "warning: %s\n", msg)
	} else {
		fmt.Fprintf(os.Stderr, "::warning::%s\n", msg)
	}
}

// errf prints a file-annotated error — inline in CI, plain on a TTY.
func errf(path, format string, args ...any) {
	msg := fmt.Sprintf(format, args...)
	if isTTY() {
		fmt.Fprintf(os.Stderr, "  error: %s: %s\n", path, msg)
	} else {
		fmt.Fprintf(os.Stderr, "::error file=%s::%s\n", path, msg)
	}
}
