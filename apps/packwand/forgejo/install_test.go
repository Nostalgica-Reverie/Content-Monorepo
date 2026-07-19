package forgejo

import (
	"testing"

	"github.com/dlclark/regexp2"
)

func TestCodebergRegex(t *testing.T) {
	if m := CodebergRegex.FindStringSubmatch("https://codeberg.org/owner/repo"); len(m) != 2 || m[1] != "owner/repo" {
		t.Errorf("CodebergRegex = %v", m)
	}
	if m := CodebergRegex.FindStringSubmatch("https://github.com/owner/repo"); m != nil {
		t.Errorf("CodebergRegex should not match github.com, got %v", m)
	}
}

func TestGenericForgejoRegex(t *testing.T) {
	m := GenericForgejoRegex.FindStringSubmatch("https://git.nostalgica.net/Reverie-Projects/monorepo")
	if len(m) != 3 || m[1] != "git.nostalgica.net" || m[2] != "Reverie-Projects/monorepo" {
		t.Errorf("GenericForgejoRegex = %v", m)
	}
}

// defaultRegex is the asset filter used when --regex is not passed; it must
// select the main jar and reject api/dev/sources classifiers.
func TestDefaultAssetRegex(t *testing.T) {
	expr := regexp2.MustCompile(defaultRegex, 0)
	for asset, want := range map[string]bool{
		"mod-1.0.0.jar":               true,
		"mod-1.0.0-api.jar":           false,
		"mod-1.0.0-dev.jar":           false,
		"mod-1.0.0-dev-preshadow.jar": false,
		"mod-1.0.0-sources.jar":       false,
		"mod-1.0.0.zip":               false,
	} {
		got, err := expr.MatchString(asset)
		if err != nil {
			t.Fatalf("MatchString(%q): %v", asset, err)
		}
		if got != want {
			t.Errorf("defaultRegex match %q = %v, want %v", asset, got, want)
		}
	}
}
