package xrpcapi

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
)

type fakeBackend struct {
	identity session.Identity
	created  map[string]any
	uploaded []byte
}

func (backend *fakeBackend) CurrentIdentity() (session.Identity, bool) {
	return backend.identity, backend.identity.DID != ""
}

func (backend *fakeBackend) Resolve(_ context.Context, _ string) (session.Identity, error) {
	return backend.identity, nil
}

func (backend *fakeBackend) CreateRecord(_ context.Context, _, _ string, record map[string]any) (StrongRef, error) {
	backend.created = record
	return StrongRef{URI: "at://did:plc:alice/net.nostalgica.packwand.pack/one", CID: "bafyrecord"}, nil
}

func (backend *fakeBackend) ListRecords(_ context.Context, _, _ string, _ int, _ string) (RecordPage, error) {
	return RecordPage{Records: []Record{{URI: "at://record", CID: "bafyrecord", Value: json.RawMessage(`{"name":"Pack"}`)}}}, nil
}

func (backend *fakeBackend) UploadBlob(_ context.Context, _ string, data []byte) (BlobRef, error) {
	backend.uploaded = data
	return BlobRef{Type: "blob", Ref: CIDLink{Link: "bafyblob"}, MimeType: "image/png", Size: int64(len(data))}, nil
}

func (backend *fakeBackend) ListFriends(_ context.Context) ([]Friend, error) {
	return []Friend{{DID: "did:plc:bob", Handle: "bob.example", Sources: []string{"mutual_follow"}}}, nil
}

func (backend *fakeBackend) ListPendingInvites(_ context.Context) ([]PendingInvite, error) {
	return []PendingInvite{{From: "did:plc:bob", Invite: "pw://invite"}}, nil
}

func (backend *fakeBackend) LinkedTangledRepos(_ context.Context) ([]TangledRepo, error) {
	return []TangledRepo{{URI: "at://did:plc:alice/sh.tangled.repo/one", CID: "bafyrepor"}}, nil
}

func TestHandlerRequiresBearerToken(t *testing.T) {
	backend := &fakeBackend{identity: session.Identity{DID: "did:plc:alice", Handle: "alice.example", PDS: "https://pds.example"}}
	handler := NewHandler(backend, "secret", make(chan struct{}, 1))
	request := httptest.NewRequest(http.MethodGet, "/v1/session", nil)
	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)
	if response.Code != http.StatusUnauthorized {
		t.Fatalf("status = %d", response.Code)
	}
}

func TestHandlerCreatesGenericRecord(t *testing.T) {
	backend := &fakeBackend{identity: session.Identity{DID: "did:plc:alice"}}
	handler := NewHandler(backend, "secret", make(chan struct{}, 1))
	request := httptest.NewRequest(http.MethodPost, "/v1/record", bytes.NewBufferString(`{"collection":"net.nostalgica.packwand.pack","record":{"name":"Pack"}}`))
	request.Header.Set("Authorization", "Bearer secret")
	request.Header.Set("Content-Type", "application/json")
	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)
	if response.Code != http.StatusCreated {
		t.Fatalf("status = %d, body = %s", response.Code, response.Body.String())
	}
	if backend.created["name"] != "Pack" {
		t.Fatalf("record = %#v", backend.created)
	}
}

func TestHandlerUploadsImageBlob(t *testing.T) {
	backend := &fakeBackend{identity: session.Identity{DID: "did:plc:alice"}}
	handler := NewHandler(backend, "secret", make(chan struct{}, 1))
	request := httptest.NewRequest(http.MethodPost, "/v1/blob", bytes.NewReader([]byte("png")))
	request.Header.Set("Authorization", "Bearer secret")
	request.Header.Set("Content-Type", "image/png")
	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)
	if response.Code != http.StatusCreated {
		t.Fatalf("status = %d, body = %s", response.Code, response.Body.String())
	}
	if string(backend.uploaded) != "png" {
		t.Fatalf("uploaded = %q", backend.uploaded)
	}
}
