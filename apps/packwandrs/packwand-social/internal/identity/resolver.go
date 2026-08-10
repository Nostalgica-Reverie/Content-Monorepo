package identity

import (
	"context"
	"fmt"

	atidentity "github.com/bluesky-social/indigo/atproto/identity"
	"github.com/bluesky-social/indigo/atproto/syntax"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
)

// Resolver resolves handles and DIDs through Indigo's cached default directory.
type Resolver struct {
	directory atidentity.Directory
}

// New constructs the production identity resolver.
func New() *Resolver {
	return &Resolver{directory: atidentity.DefaultDirectory()}
}

// Resolve returns a verified identity for a handle or DID.
func (resolver *Resolver) Resolve(ctx context.Context, raw string) (session.Identity, error) {
	identifier, err := syntax.ParseAtIdentifier(raw)
	if err != nil {
		return session.Identity{}, fmt.Errorf("parse ATProto identifier: %w", err)
	}
	resolved, err := resolver.directory.Lookup(ctx, identifier)
	if err != nil {
		return session.Identity{}, fmt.Errorf("resolve ATProto identity: %w", err)
	}
	return fromIndigo(resolved)
}

// ResolveDID returns a verified identity for a parsed DID.
func (resolver *Resolver) ResolveDID(ctx context.Context, did syntax.DID) (session.Identity, error) {
	resolved, err := resolver.directory.LookupDID(ctx, did)
	if err != nil {
		return session.Identity{}, fmt.Errorf("resolve ATProto DID: %w", err)
	}
	return fromIndigo(resolved)
}

func fromIndigo(resolved *atidentity.Identity) (session.Identity, error) {
	if resolved.PDSEndpoint() == "" {
		return session.Identity{}, fmt.Errorf("identity %s does not declare a PDS", resolved.DID)
	}
	return session.Identity{
		DID:    resolved.DID.String(),
		Handle: resolved.Handle.String(),
		PDS:    resolved.PDSEndpoint(),
	}, nil
}
