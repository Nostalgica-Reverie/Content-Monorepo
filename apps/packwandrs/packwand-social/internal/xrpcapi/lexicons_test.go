package xrpcapi

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/bluesky-social/indigo/atproto/syntax"
)

func TestPackwandLexiconFilesHaveValidIdentities(t *testing.T) {
	paths, err := filepath.Glob(filepath.Join("..", "..", "lexicons", "*.json"))
	if err != nil {
		t.Fatal(err)
	}
	packwandCollections := 0
	for collection := range writableCollections {
		if strings.HasPrefix(collection, "net.nostalgica.packwand.") {
			packwandCollections++
		}
	}
	if len(paths) != packwandCollections {
		t.Fatalf("found %d Lexicons for %d Packwand collections", len(paths), packwandCollections)
	}
	for _, path := range paths {
		data, readErr := os.ReadFile(path)
		if readErr != nil {
			t.Fatal(readErr)
		}
		var schema struct {
			Lexicon int                        `json:"lexicon"`
			ID      string                     `json:"id"`
			Defs    map[string]json.RawMessage `json:"defs"`
		}
		if decodeErr := json.Unmarshal(data, &schema); decodeErr != nil {
			t.Errorf("%s: %v", path, decodeErr)
			continue
		}
		if schema.Lexicon != 1 {
			t.Errorf("%s: lexicon = %d", path, schema.Lexicon)
		}
		if _, parseErr := syntax.ParseNSID(schema.ID); parseErr != nil {
			t.Errorf("%s: invalid NSID: %v", path, parseErr)
		}
		if _, writable := writableCollections[schema.ID]; !writable {
			t.Errorf("%s: schema ID %q is not in the OAuth allowlist", path, schema.ID)
		}
		if _, hasMain := schema.Defs["main"]; !hasMain {
			t.Errorf("%s: missing main definition", path)
		}
	}
}
