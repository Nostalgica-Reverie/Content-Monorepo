package trigger

import (
	"testing"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/schema"
)

func TestPathAndBranchConditions(t *testing.T) {
	conditions := []schema.Condition{{Event: []string{"push"}, Branch: []string{"main"}, Paths: []string{"apps/packwandrs/**"}}}
	if !Matches(conditions, "main", []string{"apps/packwandrs/somnus/main.go"}) {
		t.Fatal("expected matching nested path")
	}
	if Matches(conditions, "feature", []string{"apps/packwandrs/somnus/main.go"}) {
		t.Fatal("unexpected branch match")
	}
}

func TestManualConditionAlwaysMatches(t *testing.T) {
	if !Matches([]schema.Condition{{Event: []string{"manual"}}}, "feature", nil) {
		t.Fatal("manual condition must always match locally")
	}
}

func TestTagOnlyWorkflowDoesNotTriggerWithoutTagEvent(t *testing.T) {
	conditions := []schema.Condition{{Event: []string{"push"}, Tag: []string{"gui-v*"}}}
	if Matches(conditions, "main", nil) {
		t.Fatal("tag-only workflow unexpectedly matched a local branch")
	}
}
