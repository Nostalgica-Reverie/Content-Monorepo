package core

import (
	"os"
	"runtime"
	"strconv"
	"strings"
	"sync"
)

const defaultConcurrencyCap = 8

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

// MaxConcurrent returns the default worker count for packwand operations.
// PACKWAND_CONCURRENCY is preferred; SOMNUS_CONCURRENCY remains supported for
// existing workspace automation.
func MaxConcurrent() int {
	if n := concurrencyFromEnv("PACKWAND_CONCURRENCY", "SOMNUS_CONCURRENCY"); n > 0 {
		return n
	}
	return defaultConcurrency()
}

// NetworkConcurrent returns the limit for outbound API and file download work.
func NetworkConcurrent() int {
	if n := concurrencyFromEnv("PACKWAND_NETWORK_CONCURRENCY"); n > 0 {
		return n
	}
	return MaxConcurrent()
}

// HashConcurrent returns the limit for local file reads and hash computation.
func HashConcurrent() int {
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
