package main

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/report"
)

func TestRunDiscoversWorkflowAndPersistsStatus(t *testing.T) {
	root := t.TempDir()
	workflowDirectory := filepath.Join(root, ".tangled", "workflows")
	if err := os.MkdirAll(workflowDirectory, 0o755); err != nil {
		t.Fatal(err)
	}
	workflow := []byte("when:\n  - event: [manual]\nengine: microvm\nimage: nixos\nsteps:\n  - name: pass\n    command: exit 0\n")
	if err := os.WriteFile(filepath.Join(workflowDirectory, "fixture.yml"), workflow, 0o600); err != nil {
		t.Fatal(err)
	}
	if err := run(context.Background(), []string{"--root", root}); err != nil {
		t.Fatal(err)
	}
	status, err := report.Load(root)
	if err != nil {
		t.Fatal(err)
	}
	if len(status.Workflows) != 1 || !status.Workflows[0].Passed {
		t.Fatalf("unexpected persisted status: %#v", status)
	}
}

func TestRunExecutesExplicitRootRelativeWorkflowRegardlessOfTrigger(t *testing.T) {
	root := t.TempDir()
	workflowDirectory := filepath.Join(root, ".tangled", "workflows")
	if err := os.MkdirAll(workflowDirectory, 0o755); err != nil {
		t.Fatal(err)
	}
	workflow := []byte("when:\n  - event: [push]\n    branch: [never-local]\nengine: microvm\nimage: nixos\nsteps:\n  - name: explicit\n    command: exit 0\n")
	if err := os.WriteFile(filepath.Join(workflowDirectory, "explicit.yml"), workflow, 0o600); err != nil {
		t.Fatal(err)
	}
	if err := run(context.Background(), []string{
		"--root",
		root,
		filepath.Join(".tangled", "workflows", "explicit.yml"),
	}); err != nil {
		t.Fatal(err)
	}
	status, err := report.Load(root)
	if err != nil {
		t.Fatal(err)
	}
	if len(status.Workflows) != 1 || !status.Workflows[0].Passed {
		t.Fatalf("explicit workflow did not run: %#v", status)
	}
}
