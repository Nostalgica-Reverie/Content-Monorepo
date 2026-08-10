package session

import (
	"context"
	"os"
	"path/filepath"
	"runtime"
	"testing"

	"github.com/bluesky-social/indigo/atproto/auth/oauth"
	"github.com/bluesky-social/indigo/atproto/syntax"
)

func TestStoreRoundTripsCurrentSession(t *testing.T) {
	path := filepath.Join(t.TempDir(), "state", "oauth.json")
	store, err := Open(path)
	if err != nil {
		t.Fatal(err)
	}
	did, err := syntax.ParseDID("did:plc:example")
	if err != nil {
		t.Fatal(err)
	}
	sessionData := oauth.ClientSessionData{AccountDID: did, SessionID: "session", HostURL: "https://pds.example"}
	if err := store.SaveSession(context.Background(), sessionData); err != nil {
		t.Fatal(err)
	}
	current := Current{DID: did.String(), Handle: "alice.example", PDS: sessionData.HostURL, SessionID: sessionData.SessionID}
	if err := store.SetCurrent(current); err != nil {
		t.Fatal(err)
	}

	reopened, err := Open(path)
	if err != nil {
		t.Fatal(err)
	}
	actual, ok := reopened.Current()
	if !ok || actual != current {
		t.Fatalf("current = %#v, %v", actual, ok)
	}
	loaded, err := reopened.GetSession(context.Background(), did, sessionData.SessionID)
	if err != nil {
		t.Fatal(err)
	}
	if loaded.HostURL != sessionData.HostURL {
		t.Fatalf("host URL = %q", loaded.HostURL)
	}
	if runtime.GOOS != "windows" {
		if info, err := os.Stat(path); err != nil {
			t.Fatal(err)
		} else if info.Mode().Perm()&0o077 != 0 {
			t.Fatalf("OAuth store permissions = %o", info.Mode().Perm())
		}
	}
}
