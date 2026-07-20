package core

import (
	"context"
	"io"
	"math/rand/v2"
	"net/http"
	"strconv"
	"sync"
	"time"
)

// Shared HTTP client constructors. Every provider/API client in packwand must
// use one of these instead of a bare &http.Client{}: a client without a
// Timeout can block a NetworkConcurrent() worker slot forever on a single
// hung connection.

// apiTimeout bounds metadata/API calls (project lookups, version lists).
const apiTimeout = 30 * time.Second

// transferTimeout bounds large file transfers (mod jars, pack uploads) —
// generous enough that a real download over a slow link is never cut off,
// but a hung connection still fails eventually.
const transferTimeout = 10 * time.Minute

const (
	retryMaxAttempts    = 3
	retryInitialBackoff = 500 * time.Millisecond
	// retryAfterCap bounds how long a server-provided Retry-After header can
	// stall a worker; a hostile or misconfigured server must not be able to
	// park a NetworkConcurrent() slot for minutes.
	retryAfterCap = 60 * time.Second
)

// originalDefaultTransport is http.DefaultTransport as it was at process
// start, so the tuned pool below only replaces the stock transport — tools
// that swap http.DefaultTransport (httpmock in tests) still win.
var originalDefaultTransport = http.DefaultTransport

// tunedTransport is the shared connection pool for every packwand client.
// The stock http.DefaultTransport keeps only 2 idle connections per host
// (DefaultMaxIdleConnsPerHost), so NetworkConcurrent()-wide fan-out against a
// single API host (api.modrinth.com) re-dials and re-handshakes TLS for most
// requests. Sizing the idle pool to the fan-out width makes connections
// actually get reused.
var tunedTransport = sync.OnceValue(func() *http.Transport {
	t := originalDefaultTransport.(*http.Transport).Clone()
	t.MaxIdleConns = 64
	t.MaxIdleConnsPerHost = NetworkConcurrent() * 2
	t.ForceAttemptHTTP2 = true
	return t
})

// hostRateIntervals paces outbound requests per host: the minimum spacing
// between attempts. Providers with a documented request budget get an entry
// here so the client stays under the limit instead of discovering it through
// 429s. Modrinth allows 300 req/min; 220ms spacing (~272 req/min) leaves
// headroom for retries and other processes.
var hostRateIntervals = map[string]time.Duration{
	"api.modrinth.com":         220 * time.Millisecond,
	"staging-api.modrinth.com": 220 * time.Millisecond,
}

// rateGate spaces callers by handing each one the next free slot on a
// monotonic timeline. Contended goroutines queue in FIFO-ish slot order
// rather than thundering at once.
type rateGate struct {
	mu   sync.Mutex
	next time.Time
}

func (g *rateGate) wait(ctx context.Context, interval time.Duration) error {
	g.mu.Lock()
	at := g.next
	if now := time.Now(); at.Before(now) {
		at = now
	}
	g.next = at.Add(interval)
	g.mu.Unlock()
	d := time.Until(at)
	if d <= 0 {
		return nil
	}
	timer := time.NewTimer(d)
	defer timer.Stop()
	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-timer.C:
		return nil
	}
}

var hostGates sync.Map // hostname -> *rateGate

// waitHostRateLimit blocks until the request's host allows another attempt.
// Hosts without a configured budget pass through untouched.
func waitHostRateLimit(req *http.Request) error {
	host := req.URL.Hostname()
	interval, ok := hostRateIntervals[host]
	if !ok {
		return nil
	}
	g, _ := hostGates.LoadOrStore(host, &rateGate{})
	return g.(*rateGate).wait(req.Context(), interval)
}

// retryAfterDelay parses a Retry-After header value (either delta-seconds or
// an HTTP-date per RFC 9110 §10.2.3). Returns 0 if absent or unparseable.
func retryAfterDelay(v string) time.Duration {
	if v == "" {
		return 0
	}
	if secs, err := strconv.Atoi(v); err == nil {
		if secs <= 0 {
			return 0
		}
		return time.Duration(secs) * time.Second
	}
	if t, err := http.ParseTime(v); err == nil {
		return time.Until(t)
	}
	return 0
}

// NewClient returns the standard client for metadata/API requests: 30s
// timeout, transparent retry (3 attempts, doubling backoff) on network
// errors, HTTP 429, and HTTP 5xx.
func NewClient() *http.Client {
	return &http.Client{Timeout: apiTimeout, Transport: &retryTransport{}}
}

// NewDownloadClient returns the client for large file downloads: same retry
// policy as NewClient but with a transfer-scale timeout.
func NewDownloadClient() *http.Client {
	return &http.Client{Timeout: transferTimeout, Transport: &retryTransport{}}
}

// NewUploadClient returns a client with a transfer-scale timeout and no
// transparent retry, for callers that implement their own retry/backoff and
// error reporting (e.g. the publish upload path). This is deliberate — do
// not add retryTransport here: uploads are not safely idempotent, and a
// retried POST after an ambiguous failure can publish twice.
func NewUploadClient() *http.Client {
	return &http.Client{Timeout: transferTimeout}
}

// retryTransport retries transient failures (network errors, 429, 5xx) with
// doubling backoff. Requests whose body cannot be replayed (GetBody == nil)
// are never retried after the body has been consumed.
//
// The base transport is resolved per call, not captured at construction, so
// tools that swap http.DefaultTransport (httpmock in tests) keep working.
type retryTransport struct {
	base http.RoundTripper
}

func (t *retryTransport) roundTripper() http.RoundTripper {
	if t.base != nil {
		return t.base
	}
	// Honor a swapped http.DefaultTransport (httpmock); otherwise use the
	// tuned shared pool.
	if http.DefaultTransport != originalDefaultTransport {
		return http.DefaultTransport
	}
	return tunedTransport()
}

func (t *retryTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	backoff := retryInitialBackoff
	var resp *http.Response
	var err error
	for attempt := 1; ; attempt++ {
		// Every attempt (including retries) pays the per-host pacing cost, so
		// a retry storm can't blow the provider's request budget either.
		if lerr := waitHostRateLimit(req); lerr != nil {
			return nil, lerr
		}
		resp, err = t.roundTripper().RoundTrip(req)
		transient := err != nil || resp.StatusCode == http.StatusTooManyRequests || resp.StatusCode >= 500
		if !transient || attempt >= retryMaxAttempts {
			return resp, err
		}
		// A consumed request body can only be replayed via GetBody.
		if req.Body != nil {
			if req.GetBody == nil {
				return resp, err
			}
			body, bodyErr := req.GetBody()
			if bodyErr != nil {
				return resp, err
			}
			req.Body = body
		}
		// Jitter the backoff to [0.5x, 1.5x] so NetworkConcurrent() workers
		// that got rate-limited together don't all retry in lockstep. When the
		// server says how long to wait, believe it (capped).
		wait := backoff/2 + rand.N(backoff)
		if resp != nil {
			if ra := retryAfterDelay(resp.Header.Get("Retry-After")); ra > 0 {
				wait = min(ra, retryAfterCap)
			}
			_, _ = io.Copy(io.Discard, io.LimitReader(resp.Body, 4096))
			_ = resp.Body.Close()
		}
		select {
		case <-req.Context().Done():
			return nil, req.Context().Err()
		case <-time.After(wait):
		}
		backoff *= 2
	}
}
