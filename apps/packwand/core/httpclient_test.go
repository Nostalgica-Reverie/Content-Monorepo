package core

import (
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
