package api

import (
	"path/filepath"
	"reflect"
	"testing"
)

// The Gleam GUI posts every action to POST /api/v1/actions with this wire
// shape (see gui/ui/apps/packwand_gui/api.gleam), so each Action variant the
// frontend can emit must resolve here — gui.go's own /api/actions mux is
// shadowed by Handler()'s /api/v1 gate and never reached.
func TestCompatibilityActionsForGUI(t *testing.T) {
	server, subdir := exampleServer(t)
	rel := filepath.ToSlash(filepath.Join("modpacks", "example", "1.21-mr"))

	tests := []struct {
		name    string
		input   actionInput
		wantDir string
		want    []string
	}{
		{"bump", actionInput{Action: "bump", Subdir: rel, Version: "1.2.3"}, subdir, []string{"bump", ".", "1.2.3"}},
		{"bump with configs", actionInput{Action: "bump", Subdir: rel, Version: "1.2.3", Configs: true}, subdir, []string{"bump", ".", "1.2.3", "--configs"}},
		{"freeze", actionInput{Action: "freeze-mod", Subdir: rel, Slug: "sodium"}, subdir, []string{"freeze", ".", "sodium"}},
		{"unfreeze", actionInput{Action: "unfreeze-mod", Subdir: rel, Slug: "sodium"}, subdir, []string{"unfreeze", ".", "sodium"}},
		{"set-side", actionInput{Action: "set-side", Subdir: rel, Slug: "sodium", Side: "client"}, subdir, []string{"side", ".", "sodium", "client"}},
		{"nix-gen", actionInput{Action: "nix-gen", Subdir: rel}, subdir, []string{"nix", "gen"}},
		{"docs-modlist", actionInput{Action: "docs-modlist", Subdir: rel}, subdir, []string{"modlist", "."}},
		{"docs-pages", actionInput{Action: "docs-pages"}, server.root, []string{"pages"}},
	}
	for _, test := range tests {
		dir, args, err := server.resolveCompatibilityAction(test.input)
		if err != nil {
			t.Errorf("%s: %v", test.name, err)
			continue
		}
		if dir != test.wantDir || !reflect.DeepEqual(args, test.want) {
			t.Errorf("%s: dir=%q args=%#v", test.name, dir, args)
		}
	}
}

func TestCompatibilityActionsRejectBadInput(t *testing.T) {
	server, _ := exampleServer(t)
	rel := filepath.ToSlash(filepath.Join("modpacks", "example", "1.21-mr"))

	for _, test := range []struct {
		name  string
		input actionInput
	}{
		{"bump without version", actionInput{Action: "bump", Subdir: rel}},
		{"freeze without slug", actionInput{Action: "freeze-mod", Subdir: rel}},
		{"unfreeze without slug", actionInput{Action: "unfreeze-mod", Subdir: rel}},
		{"set-side without slug", actionInput{Action: "set-side", Subdir: rel, Side: "client"}},
		{"set-side with invalid side", actionInput{Action: "set-side", Subdir: rel, Slug: "sodium", Side: "everywhere"}},
	} {
		if _, _, err := server.resolveCompatibilityAction(test.input); err == nil {
			t.Errorf("%s: expected an error", test.name)
		}
	}
}
