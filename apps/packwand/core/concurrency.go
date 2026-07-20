package core

import (
	"os"
	"runtime"
	"strconv"
	"strings"
	"sync"

	"github.com/spf13/viper"
)

const defaultConcurrencyCap = 8

// defaultNetworkConcurrency is deliberately higher than the CPU-bound cap:
// outbound API calls and downloads are latency-bound, not core-bound, so a
// worker mostly waits on the wire. Rate-limit pushback is handled by the
// retry transport's Retry-After support (httpclient.go).
const defaultNetworkConcurrency = 16

func concurrencyFromEnv(names ...string) int {
	for _, name := range names {
		if v := strings.TrimSpace(os.Getenv(name)); v != "" {
			if n, err := strconv.Atoi(v); err == nil && n > 0 {
				return n
			}
		}
	}
	return 0
}

func defaultConcurrency() int {
	return max(1, min(runtime.NumCPU(), defaultConcurrencyCap))
}

// jobsFlag returns the value of the --jobs/-j root flag (bound to viper key
// "jobs" in cmd/root.go), the discoverable way to tune every limit at once.
// Precedence for all limits: --jobs flag > PACKWAND_* env > default.
func jobsFlag() int {
	if n := viper.GetInt("jobs"); n > 0 {
		return n
	}
	return 0
}

// MaxConcurrent returns the default worker count for packwand operations.
// PACKWAND_CONCURRENCY is preferred; SOMNUS_CONCURRENCY remains supported for
// existing workspace automation.
func MaxConcurrent() int {
	if n := jobsFlag(); n > 0 {
		return n
	}
	if n := concurrencyFromEnv("PACKWAND_CONCURRENCY", "SOMNUS_CONCURRENCY"); n > 0 {
		return n
	}
	return defaultConcurrency()
}

// NetworkConcurrent returns the limit for outbound API and file download work.
func NetworkConcurrent() int {
	if n := jobsFlag(); n > 0 {
		return n
	}
	if n := concurrencyFromEnv("PACKWAND_NETWORK_CONCURRENCY"); n > 0 {
		return n
	}
	// An explicit global override still wins over the higher network default.
	if n := concurrencyFromEnv("PACKWAND_CONCURRENCY", "SOMNUS_CONCURRENCY"); n > 0 {
		return n
	}
	return defaultNetworkConcurrency
}

// HashConcurrent returns the limit for local file reads and hash computation.
func HashConcurrent() int {
	if n := jobsFlag(); n > 0 {
		return n
	}
	if n := concurrencyFromEnv("PACKWAND_HASH_CONCURRENCY"); n > 0 {
		return n
	}
	return MaxConcurrent()
}

// ParallelFor runs fn for every item up to limit concurrent goroutines.
// Results are written by fn; this helper only owns scheduling and waiting.
func ParallelFor[T any](items []T, limit int, fn func(int, T)) {
	if len(items) == 0 {
		return
	}
	if limit < 1 {
		limit = 1
	}
	sem := make(chan struct{}, limit)
	var wg sync.WaitGroup
	for i, item := range items {
		wg.Add(1)
		sem <- struct{}{}
		go func(i int, item T) {
			defer wg.Done()
			defer func() { <-sem }()
			fn(i, item)
		}(i, item)
	}
	wg.Wait()
}
