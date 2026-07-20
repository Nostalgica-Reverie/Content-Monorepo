package core

import (
	"context"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"
)

func TestRetryAfterDelay(t *testing.T) {
	cases := []struct {
		name string
		v    string
		want time.Duration
	}{
		{"empty", "", 0},
		{"seconds", "2", 2 * time.Second},
		{"zero", "0", 0},
		{"negative", "-5", 0},
		{"garbage", "soon", 0},
	}
	for _, c := range cases {
		if got := retryAfterDelay(c.v); got != c.want {
			t.Errorf("%s: retryAfterDelay(%q) = %v, want %v", c.name, c.v, got, c.want)
		}
	}
	// HTTP-date form: a date ~2s in the future should yield roughly that delay.
	future := time.Now().Add(2 * time.Second).UTC().Format(http.TimeFormat)
	got := retryAfterDelay(future)
	if got <= 0 || got > 3*time.Second {
		t.Errorf("retryAfterDelay(http-date +2s) = %v, want ~2s", got)
	}
}

// TestRetryTransportHonorsRetryAfter verifies a 429 with Retry-After is
// retried after (at least) the advertised delay and then succeeds.
func TestRetryTransportHonorsRetryAfter(t *testing.T) {
	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if calls.Add(1) == 1 {
			w.Header().Set("Retry-After", "1")
			w.WriteHeader(http.StatusTooManyRequests)
			return
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	client := NewClient()
	start := time.Now()
	resp, err := client.Get(srv.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("status = %d, want 200", resp.StatusCode)
	}
	if got := calls.Load(); got != 2 {
		t.Fatalf("server saw %d calls, want 2", got)
	}
	if elapsed := time.Since(start); elapsed < time.Second {
		t.Fatalf("retried after %v, want >= Retry-After of 1s", elapsed)
	}
}

// TestTunedTransportPoolSizing verifies the shared transport keeps enough
// idle connections per host to cover the network fan-out width.
func TestTunedTransportPoolSizing(t *testing.T) {
	tr := tunedTransport()
	if want := NetworkConcurrent() * 2; tr.MaxIdleConnsPerHost != want {
		t.Errorf("MaxIdleConnsPerHost = %d, want %d", tr.MaxIdleConnsPerHost, want)
	}
	if tr.MaxIdleConns < tr.MaxIdleConnsPerHost {
		t.Errorf("MaxIdleConns (%d) < MaxIdleConnsPerHost (%d)", tr.MaxIdleConns, tr.MaxIdleConnsPerHost)
	}
	if !tr.ForceAttemptHTTP2 {
		t.Error("ForceAttemptHTTP2 = false, want true")
	}
}

// TestRoundTripperHonorsSwappedDefault verifies a swapped
// http.DefaultTransport (the httpmock pattern) takes precedence over the
// tuned pool, and the tuned pool is used otherwise.
func TestRoundTripperHonorsSwappedDefault(t *testing.T) {
	rt := &retryTransport{}
	if got := rt.roundTripper(); got != http.RoundTripper(tunedTransport()) {
		t.Errorf("with stock DefaultTransport, roundTripper() = %T, want the tuned pool", got)
	}
	mock := &retryTransport{} // any distinct RoundTripper works as a stand-in
	orig := http.DefaultTransport
	http.DefaultTransport = mock
	t.Cleanup(func() { http.DefaultTransport = orig })
	if got := rt.roundTripper(); got != http.RoundTripper(mock) {
		t.Errorf("with swapped DefaultTransport, roundTripper() = %T, want the swapped transport", got)
	}
}

// TestRateGatePacing verifies the per-host gate spaces callers by the
// configured interval and respects context cancellation.
func TestRateGatePacing(t *testing.T) {
	g := &rateGate{}
	const interval = 50 * time.Millisecond
	start := time.Now()
	for range 3 {
		if err := g.wait(context.Background(), interval); err != nil {
			t.Fatal(err)
		}
	}
	// First call is immediate; calls 2 and 3 wait one interval each.
	if elapsed := time.Since(start); elapsed < 2*interval {
		t.Errorf("3 waits took %v, want >= %v", elapsed, 2*interval)
	}

	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	if err := g.wait(ctx, time.Minute); err == nil {
		t.Error("wait with cancelled context returned nil, want error")
	}
}

// TestWaitHostRateLimitUnknownHost verifies hosts without a configured
// budget are not delayed.
func TestWaitHostRateLimitUnknownHost(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "https://example.invalid/x", nil)
	start := time.Now()
	if err := waitHostRateLimit(req); err != nil {
		t.Fatal(err)
	}
	if elapsed := time.Since(start); elapsed > 20*time.Millisecond {
		t.Errorf("unthrottled host delayed %v", elapsed)
	}
}

// TestRetryTransportBackoffOn5xx verifies the jittered-backoff path (no
// Retry-After header) still retries and succeeds.
func TestRetryTransportBackoffOn5xx(t *testing.T) {
	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if calls.Add(1) == 1 {
			w.WriteHeader(http.StatusInternalServerError)
			return
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	resp, err := NewClient().Get(srv.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("status = %d, want 200", resp.StatusCode)
	}
	if got := calls.Load(); got != 2 {
		t.Fatalf("server saw %d calls, want 2", got)
	}
}
