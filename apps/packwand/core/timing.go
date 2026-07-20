package core

import (
	"fmt"
	"os"
	"strings"
	"sync"
	"time"
)

// Stage timing instrumentation, enabled with PACKWAND_TIMINGS=1. Spans print
// to stderr on completion so they survive stdout capture (JSON output,
// subprocess piping). When disabled, StartSpan returns a shared no-op so the
// hot paths pay only a boolean check.

var timingsEnabled = sync.OnceValue(func() bool {
	switch strings.TrimSpace(os.Getenv("PACKWAND_TIMINGS")) {
	case "", "0", "false", "off":
		return false
	}
	return true
})

var noopSpan = func() {}

// TimingsEnabled reports whether PACKWAND_TIMINGS stage timing is active.
func TimingsEnabled() bool {
	return timingsEnabled()
}

// StartSpan begins a named timing span and returns the function that ends it.
// Typical use: defer StartSpan("mr-export: download+hash")().
func StartSpan(name string) func() {
	if !timingsEnabled() {
		return noopSpan
	}
	start := time.Now()
	return func() {
		fmt.Fprintf(os.Stderr, "[timing] %-32s %v\n", name, time.Since(start).Round(time.Millisecond))
	}
}
