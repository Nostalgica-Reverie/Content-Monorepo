package runner

import (
	"context"
	"testing"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/schema"
)

func TestRunStopsAfterFirstFailure(t *testing.T) {
	workflow := schema.Workflow{Steps: []schema.Step{
		{Name: "fails", Command: "exit 7"},
		{Name: "must not run", Command: "exit 0"},
	}}
	result := Run(context.Background(), "fixture.yml", workflow, t.TempDir())
	if result.Passed || len(result.Steps) != 1 || result.Steps[0].Passed {
		t.Fatalf("unexpected result: %#v", result)
	}
}
