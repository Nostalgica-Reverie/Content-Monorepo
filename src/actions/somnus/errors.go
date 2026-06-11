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
	fmt.Fprintf(os.Stderr, "::error::%s\n", msg)
	if hint != "" {
		fmt.Fprintf(os.Stderr, "  hint: %s\n", hint)
	}
	os.Exit(code)
}
