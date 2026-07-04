package migrate

import (
	"testing"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/core"
)

func TestUpdatePackToVersionSupportsMultipleLoaders(t *testing.T) {
	pack := core.Pack{Versions: map[string]string{"forge": "old-forge", "neoforge": "old-neoforge"}}
	forge := core.ModLoaderComponent{Name: "forge", FriendlyName: "Forge"}
	neoforge := core.ModLoaderComponent{Name: "neoforge", FriendlyName: "NeoForge"}

	if !updatePackToVersion("new-forge", pack, forge) || !updatePackToVersion("new-neoforge", pack, neoforge) {
		t.Fatal("expected both loader versions to change")
	}
	if pack.Versions["forge"] != "new-forge" || pack.Versions["neoforge"] != "new-neoforge" {
		t.Fatalf("versions = %#v", pack.Versions)
	}
}
