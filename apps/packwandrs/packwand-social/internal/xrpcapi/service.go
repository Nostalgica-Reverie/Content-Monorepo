package xrpcapi

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/bluesky-social/indigo/atproto/atclient"
	"github.com/bluesky-social/indigo/atproto/auth/oauth"
	"github.com/bluesky-social/indigo/atproto/syntax"

	identityresolver "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/identity"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
)

var writableCollections = map[string]struct{}{
	"net.nostalgica.packwand.contact":        {},
	"net.nostalgica.packwand.image":          {},
	"net.nostalgica.packwand.pack":           {},
	"net.nostalgica.packwand.profile":        {},
	"net.nostalgica.packwand.session.invite": {},
	"net.nostalgica.packwand.snippet":        {},
	"sh.tangled.pipeline.status":             {},
}

// StrongRef is the stable reference returned by record creation.
type StrongRef struct {
	URI string `json:"uri"`
	CID string `json:"cid"`
}

// Record contains a repository record and its stable reference.
type Record struct {
	URI   string          `json:"uri"`
	CID   string          `json:"cid"`
	Value json.RawMessage `json:"value"`
}

// RecordPage is one page returned by com.atproto.repo.listRecords.
type RecordPage struct {
	Records []Record `json:"records"`
	Cursor  string   `json:"cursor,omitempty"`
}

// CIDLink is the JSON representation of an IPLD link in an ATProto blob.
type CIDLink struct {
	Link string `json:"$link"`
}

// BlobRef is the reusable blob descriptor returned by uploadBlob.
type BlobRef struct {
	Type     string  `json:"$type"`
	Ref      CIDLink `json:"ref"`
	MimeType string  `json:"mimeType"`
	Size     int64   `json:"size"`
}

// Friend is a mutual Bluesky follow or an explicit Packwand contact.
type Friend struct {
	DID         string   `json:"did"`
	Handle      string   `json:"handle,omitempty"`
	DisplayName string   `json:"displayName,omitempty"`
	Avatar      string   `json:"avatar,omitempty"`
	PDS         string   `json:"pds,omitempty"`
	Sources     []string `json:"sources"`
}

// PendingInvite is an unexpired collaboration invitation addressed to the current user.
type PendingInvite struct {
	From       string `json:"from"`
	FromHandle string `json:"fromHandle,omitempty"`
	Invite     string `json:"invite"`
	CreatedAt  string `json:"createdAt"`
	ExpiresAt  string `json:"expiresAt"`
	URI        string `json:"uri"`
	CID        string `json:"cid"`
}

// TangledRepo is a repository record returned by Bobbin for the current DID.
type TangledRepo struct {
	URI   string          `json:"uri"`
	CID   string          `json:"cid"`
	Value json.RawMessage `json:"value"`
}

// Backend is the HTTP surface's testable ATProto boundary.
type Backend interface {
	CurrentIdentity() (session.Identity, bool)
	Resolve(context.Context, string) (session.Identity, error)
	CreateRecord(context.Context, string, string, map[string]any) (StrongRef, error)
	ListRecords(context.Context, string, string, int, string) (RecordPage, error)
	UploadBlob(context.Context, string, []byte) (BlobRef, error)
	ListFriends(context.Context) ([]Friend, error)
	ListPendingInvites(context.Context) ([]PendingInvite, error)
	LinkedTangledRepos(context.Context) ([]TangledRepo, error)
}

// Service implements Backend with Indigo's OAuth API client.
type Service struct {
	app      *oauth.ClientApp
	store    *session.Store
	resolver *identityresolver.Resolver
	http     *http.Client
	appView  string
	bobbin   string
	friends  func(context.Context) ([]Friend, error)
}

// NewService constructs the production XRPC service.
func NewService(app *oauth.ClientApp, store *session.Store, resolver *identityresolver.Resolver) *Service {
	return &Service{
		app:      app,
		store:    store,
		resolver: resolver,
		http:     &http.Client{Timeout: 15 * time.Second},
		appView:  environmentOr("PACKWAND_SOCIAL_APPVIEW_URL", "https://public.api.bsky.app"),
		bobbin:   environmentOr("PACKWAND_SOCIAL_BOBBIN_URL", "https://api.tangled.org"),
	}
}

// CurrentIdentity returns only non-secret session information.
func (service *Service) CurrentIdentity() (session.Identity, bool) {
	current, ok := service.store.Current()
	return current.Identity(), ok
}

// Resolve resolves an ATProto identity through the shared directory.
func (service *Service) Resolve(ctx context.Context, identifier string) (session.Identity, error) {
	return service.resolver.Resolve(ctx, identifier)
}

// CreateRecord writes a generic Lexicon record to the signed-in repository.
func (service *Service) CreateRecord(ctx context.Context, collection, recordKey string, record map[string]any) (StrongRef, error) {
	client, current, err := service.client(ctx)
	if err != nil {
		return StrongRef{}, err
	}
	if _, allowed := writableCollections[collection]; !allowed {
		return StrongRef{}, fmt.Errorf("collection %s is outside Packwand's OAuth grant", collection)
	}
	nsid, err := syntax.ParseNSID(collection)
	if err != nil {
		return StrongRef{}, fmt.Errorf("parse collection NSID: %w", err)
	}
	if _, exists := record["$type"]; !exists {
		record["$type"] = collection
	}
	body := map[string]any{"repo": current.DID, "collection": collection, "record": record}
	if recordKey != "" {
		body["rkey"] = recordKey
	}
	var output StrongRef
	if err := client.Post(ctx, nsidForMethod("com.atproto.repo.createRecord"), body, &output); err != nil {
		return StrongRef{}, fmt.Errorf("create %s record: %w", nsid, err)
	}
	return output, nil
}

// ListRecords reads records from the current PDS, defaulting to the signed-in repository.
func (service *Service) ListRecords(ctx context.Context, repo, collection string, limit int, cursor string) (RecordPage, error) {
	client, current, err := service.client(ctx)
	if err != nil {
		return RecordPage{}, err
	}
	if repo == "" {
		repo = current.DID
	}
	if _, err := syntax.ParseNSID(collection); err != nil {
		return RecordPage{}, fmt.Errorf("parse collection NSID: %w", err)
	}
	params := map[string]any{"repo": repo, "collection": collection, "limit": limit}
	if cursor != "" {
		params["cursor"] = cursor
	}
	var output RecordPage
	if err := client.Get(ctx, nsidForMethod("com.atproto.repo.listRecords"), params, &output); err != nil {
		return RecordPage{}, fmt.Errorf("list %s records: %w", collection, err)
	}
	return output, nil
}

func (service *Service) client(ctx context.Context) (*atclient.APIClient, session.Current, error) {
	current, ok := service.store.Current()
	if !ok {
		return nil, session.Current{}, fmt.Errorf("not signed in")
	}
	did, err := syntax.ParseDID(current.DID)
	if err != nil {
		return nil, session.Current{}, fmt.Errorf("parse persisted DID: %w", err)
	}
	resumed, err := service.app.ResumeSession(ctx, did, current.SessionID)
	if err != nil {
		return nil, session.Current{}, fmt.Errorf("resume OAuth session: %w", err)
	}
	return resumed.APIClient(), current, nil
}

func environmentOr(name, fallback string) string {
	if value := strings.TrimSpace(os.Getenv(name)); value != "" {
		return strings.TrimRight(value, "/")
	}
	return fallback
}

func nsidForMethod(value string) syntax.NSID {
	nsid, err := syntax.ParseNSID(value)
	if err != nil {
		panic("invalid built-in NSID: " + value)
	}
	return nsid
}
