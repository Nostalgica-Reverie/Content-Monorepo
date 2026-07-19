package core

import (
	"io"
	"net/http"
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
)

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
// error reporting (e.g. the publish upload path).
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
	return http.DefaultTransport
}

func (t *retryTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	backoff := retryInitialBackoff
	var resp *http.Response
	var err error
	for attempt := 1; ; attempt++ {
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
		if resp != nil {
			_, _ = io.Copy(io.Discard, io.LimitReader(resp.Body, 4096))
			_ = resp.Body.Close()
		}
		select {
		case <-req.Context().Done():
			return nil, req.Context().Err()
		case <-time.After(backoff):
		}
		backoff *= 2
	}
}
