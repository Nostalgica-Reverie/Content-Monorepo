package xrpcapi

import (
	"context"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
)

func TestPendingInvitesReadsFriendsPDSAndFiltersRecords(t *testing.T) {
	expires := time.Now().Add(time.Hour).UTC().Format(time.RFC3339)
	pds := httptest.NewServer(http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		if request.URL.Path != "/xrpc/com.atproto.repo.listRecords" {
			t.Fatalf("path = %s", request.URL.Path)
		}
		if request.URL.Query().Get("repo") != "did:plc:bob" || request.URL.Query().Get("collection") != inviteCollection {
			t.Fatalf("query = %s", request.URL.RawQuery)
		}
		writer.Header().Set("Content-Type", "application/json")
		_, _ = writer.Write([]byte(`{"records":[` +
			`{"uri":"at://did:plc:bob/net.nostalgica.packwand.session.invite/one","cid":"bafyone","value":{"to":"did:plc:alice","invite":"pw://valid","createdAt":"2026-08-08T12:00:00Z","expiresAt":"` + expires + `"}},` +
			`{"uri":"at://did:plc:bob/net.nostalgica.packwand.session.invite/two","cid":"bafytwo","value":{"to":"did:plc:other","invite":"pw://other","createdAt":"2026-08-08T12:00:00Z","expiresAt":"` + expires + `"}}]}`))
	}))
	defer pds.Close()

	store := signedInStore(t)
	service := &Service{
		store: store,
		http:  pds.Client(),
		friends: func(context.Context) ([]Friend, error) {
			return []Friend{{DID: "did:plc:bob", Handle: "bob.example", PDS: pds.URL}}, nil
		},
	}
	invites, err := service.ListPendingInvites(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if len(invites) != 1 || invites[0].Invite != "pw://valid" || invites[0].FromHandle != "bob.example" {
		t.Fatalf("invites = %#v", invites)
	}
}

func TestLinkedTangledReposUsesCurrentDID(t *testing.T) {
	bobbin := httptest.NewServer(http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		if request.URL.Query().Get("subject") != "did:plc:alice" {
			t.Fatalf("subject = %q", request.URL.Query().Get("subject"))
		}
		writer.Header().Set("Content-Type", "application/json")
		_, _ = writer.Write([]byte(`{"items":[{"uri":"at://did:plc:alice/sh.tangled.repo/one","cid":"bafyrepo","value":{"name":"pack"}}]}`))
	}))
	defer bobbin.Close()
	service := &Service{store: signedInStore(t), http: bobbin.Client(), bobbin: bobbin.URL}
	repositories, err := service.LinkedTangledRepos(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if len(repositories) != 1 || !strings.Contains(repositories[0].URI, "sh.tangled.repo") {
		t.Fatalf("repositories = %#v", repositories)
	}
}

func signedInStore(t *testing.T) *session.Store {
	t.Helper()
	store, err := session.Open(filepath.Join(t.TempDir(), "oauth.json"))
	if err != nil {
		t.Fatal(err)
	}
	if err := store.SetCurrent(session.Current{
		DID: "did:plc:alice", Handle: "alice.example", PDS: "https://pds.example", SessionID: "session",
	}); err != nil {
		t.Fatal(err)
	}
	return store
}
