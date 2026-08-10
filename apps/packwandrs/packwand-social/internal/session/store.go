package session

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sync"

	"github.com/bluesky-social/indigo/atproto/auth/oauth"
	"github.com/bluesky-social/indigo/atproto/syntax"
)

const stateDirectoryEnvironment = "PACKWAND_SOCIAL_STATE_DIR"

// Identity is the non-secret account information exposed to Rust and the UI.
type Identity struct {
	DID    string `json:"did"`
	Handle string `json:"handle"`
	PDS    string `json:"pds"`
}

// Current identifies the one OAuth session Packwand currently uses.
type Current struct {
	DID       string `json:"did"`
	Handle    string `json:"handle"`
	PDS       string `json:"pds"`
	SessionID string `json:"sessionId"`
}

// Identity returns the deliberately non-secret view of the current session.
func (current Current) Identity() Identity {
	return Identity{DID: current.DID, Handle: current.Handle, PDS: current.PDS}
}

type state struct {
	Current      *Current                           `json:"current,omitempty"`
	Sessions     map[string]oauth.ClientSessionData `json:"sessions"`
	AuthRequests map[string]oauth.AuthRequestData   `json:"authRequests"`
}

// Store persists Indigo's OAuth records in one owner-readable local file.
type Store struct {
	mu    sync.Mutex
	path  string
	state state
}

// OpenDefault opens the per-user Packwand OAuth store.
func OpenDefault() (*Store, error) {
	directory := os.Getenv(stateDirectoryEnvironment)
	if directory == "" {
		config, err := os.UserConfigDir()
		if err != nil {
			return nil, fmt.Errorf("find user config directory: %w", err)
		}
		directory = filepath.Join(config, "packwand", "social")
	}
	return Open(filepath.Join(directory, "oauth.json"))
}

// Open loads or initializes the OAuth store at path.
func Open(path string) (*Store, error) {
	store := &Store{
		path: path,
		state: state{
			Sessions:     make(map[string]oauth.ClientSessionData),
			AuthRequests: make(map[string]oauth.AuthRequestData),
		},
	}
	data, err := os.ReadFile(path)
	if errors.Is(err, fs.ErrNotExist) {
		return store, nil
	}
	if err != nil {
		return nil, fmt.Errorf("read OAuth store: %w", err)
	}
	data, err = unprotect(data)
	if err != nil {
		return nil, fmt.Errorf("decrypt OAuth store: %w", err)
	}
	if err := json.Unmarshal(data, &store.state); err != nil {
		return nil, fmt.Errorf("decode OAuth store: %w", err)
	}
	if store.state.Sessions == nil {
		store.state.Sessions = make(map[string]oauth.ClientSessionData)
	}
	if store.state.AuthRequests == nil {
		store.state.AuthRequests = make(map[string]oauth.AuthRequestData)
	}
	return store, nil
}

// Current returns the selected session without exposing its tokens.
func (store *Store) Current() (Current, bool) {
	store.mu.Lock()
	defer store.mu.Unlock()
	if store.state.Current == nil {
		return Current{}, false
	}
	return *store.state.Current, true
}

// SetCurrent selects a session after its identity has been resolved.
func (store *Store) SetCurrent(current Current) error {
	store.mu.Lock()
	defer store.mu.Unlock()
	store.state.Current = &current
	return store.persistLocked()
}

// ClearCurrent removes the selected session from local storage.
func (store *Store) ClearCurrent() error {
	store.mu.Lock()
	defer store.mu.Unlock()
	if store.state.Current != nil {
		delete(store.state.Sessions, sessionKey(store.state.Current.DID, store.state.Current.SessionID))
	}
	store.state.Current = nil
	return store.persistLocked()
}

// GetSession implements oauth.ClientAuthStore.
func (store *Store) GetSession(_ context.Context, did syntax.DID, sessionID string) (*oauth.ClientSessionData, error) {
	store.mu.Lock()
	defer store.mu.Unlock()
	value, ok := store.state.Sessions[sessionKey(did.String(), sessionID)]
	if !ok {
		return nil, fs.ErrNotExist
	}
	return &value, nil
}

// SaveSession implements oauth.ClientAuthStore.
func (store *Store) SaveSession(_ context.Context, value oauth.ClientSessionData) error {
	store.mu.Lock()
	defer store.mu.Unlock()
	store.state.Sessions[sessionKey(value.AccountDID.String(), value.SessionID)] = value
	return store.persistLocked()
}

// DeleteSession implements oauth.ClientAuthStore.
func (store *Store) DeleteSession(_ context.Context, did syntax.DID, sessionID string) error {
	store.mu.Lock()
	defer store.mu.Unlock()
	delete(store.state.Sessions, sessionKey(did.String(), sessionID))
	if store.state.Current != nil && store.state.Current.DID == did.String() && store.state.Current.SessionID == sessionID {
		store.state.Current = nil
	}
	return store.persistLocked()
}

// GetAuthRequestInfo implements oauth.ClientAuthStore.
func (store *Store) GetAuthRequestInfo(_ context.Context, requestState string) (*oauth.AuthRequestData, error) {
	store.mu.Lock()
	defer store.mu.Unlock()
	value, ok := store.state.AuthRequests[requestState]
	if !ok {
		return nil, fs.ErrNotExist
	}
	return &value, nil
}

// SaveAuthRequestInfo implements oauth.ClientAuthStore.
func (store *Store) SaveAuthRequestInfo(_ context.Context, value oauth.AuthRequestData) error {
	store.mu.Lock()
	defer store.mu.Unlock()
	if _, exists := store.state.AuthRequests[value.State]; exists {
		return fs.ErrExist
	}
	store.state.AuthRequests[value.State] = value
	return store.persistLocked()
}

// DeleteAuthRequestInfo implements oauth.ClientAuthStore.
func (store *Store) DeleteAuthRequestInfo(_ context.Context, requestState string) error {
	store.mu.Lock()
	defer store.mu.Unlock()
	delete(store.state.AuthRequests, requestState)
	return store.persistLocked()
}

func (store *Store) persistLocked() error {
	if err := os.MkdirAll(filepath.Dir(store.path), 0o700); err != nil {
		return fmt.Errorf("create OAuth store directory: %w", err)
	}
	data, err := json.MarshalIndent(store.state, "", "\t")
	if err != nil {
		return fmt.Errorf("encode OAuth store: %w", err)
	}
	data = append(data, '\n')
	data, err = protect(data)
	if err != nil {
		return fmt.Errorf("encrypt OAuth store: %w", err)
	}
	temporary := store.path + ".tmp"
	if err := os.WriteFile(temporary, data, 0o600); err != nil {
		return fmt.Errorf("write OAuth store: %w", err)
	}
	if err := os.Chmod(temporary, 0o600); err != nil {
		return fmt.Errorf("protect OAuth store: %w", err)
	}
	if err := os.Rename(temporary, store.path); err != nil {
		return fmt.Errorf("replace OAuth store: %w", err)
	}
	return nil
}

func sessionKey(did, sessionID string) string {
	return did + "\x00" + sessionID
}
