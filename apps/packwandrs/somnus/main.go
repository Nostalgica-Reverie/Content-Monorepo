package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/deps"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/report"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/runner"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/schema"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/trigger"
)

func main() {
	if err := dispatch(context.Background(), os.Args[1:]); err != nil {
		fmt.Fprintln(os.Stderr, "somnus:", err)
		os.Exit(1)
	}
}

func dispatch(ctx context.Context, args []string) error {
	if len(args) == 0 {
		return fmt.Errorf("expected run, list, or status")
	}
	switch args[0] {
	case "run":
		return run(ctx, args[1:])
	case "list":
		return list(args[1:])
	case "status":
		return status(args[1:])
	default:
		return fmt.Errorf("unknown command %q", args[0])
	}
}

func run(ctx context.Context, args []string) error {
	flags := flag.NewFlagSet("run", flag.ContinueOnError)
	changed := flags.String("changed-paths", "", "comma-separated changed repository paths")
	root := flags.String("root", ".", "repository root")
	if err := flags.Parse(args); err != nil {
		return err
	}
	paths := splitPaths(*changed)
	files := flags.Args()
	explicit := len(files) > 0
	if !explicit {
		var err error
		files, err = discover(*root)
		if err != nil {
			return err
		}
	} else {
		for index, file := range files {
			if !filepath.IsAbs(file) {
				files[index] = filepath.Join(*root, file)
			}
		}
	}
	branch := trigger.CurrentBranch(*root)
	results := make([]report.WorkflowResult, 0, len(files))
	for _, file := range files {
		workflow, err := schema.Load(file)
		if err != nil {
			return err
		}
		if !explicit && !trigger.Matches(workflow.When, branch, paths) {
			continue
		}
		if missing := deps.Missing(workflow.Dependencies); len(missing) > 0 {
			return fmt.Errorf("%s requires missing dependencies: %s", file, strings.Join(missing, ", "))
		}
		result := runner.Run(ctx, file, workflow, *root)
		results = append(results, result)
		if err := report.Save(*root, report.NewStatus(results)); err != nil {
			return err
		}
		if !result.Passed {
			return fmt.Errorf("workflow %s failed", result.Name)
		}
	}
	return report.Save(*root, report.NewStatus(results))
}

func list(args []string) error {
	flags := flag.NewFlagSet("list", flag.ContinueOnError)
	changed := flags.String("changed-paths", "", "comma-separated changed repository paths")
	root := flags.String("root", ".", "repository root")
	jsonOutput := flags.Bool("json", false, "emit JSON")
	if err := flags.Parse(args); err != nil {
		return err
	}
	files, err := discover(*root)
	if err != nil {
		return err
	}
	branch := trigger.CurrentBranch(*root)
	type entry struct {
		Path    string `json:"path"`
		Name    string `json:"name"`
		Trigger bool   `json:"trigger"`
	}
	entries := make([]entry, 0, len(files))
	for _, file := range files {
		workflow, err := schema.Load(file)
		if err != nil {
			return err
		}
		entries = append(entries, entry{file, filepath.Base(file), trigger.Matches(workflow.When, branch, splitPaths(*changed))})
	}
	if *jsonOutput {
		return json.NewEncoder(os.Stdout).Encode(entries)
	}
	for _, entry := range entries {
		fmt.Printf("%s\ttrigger=%t\n", entry.Path, entry.Trigger)
	}
	return nil
}

func status(args []string) error {
	flags := flag.NewFlagSet("status", flag.ContinueOnError)
	root := flags.String("root", ".", "repository root")
	jsonOutput := flags.Bool("json", false, "emit JSON")
	if err := flags.Parse(args); err != nil {
		return err
	}
	value, err := report.Load(*root)
	if err != nil {
		return err
	}
	if *jsonOutput {
		return json.NewEncoder(os.Stdout).Encode(value)
	}
	for _, workflow := range value.Workflows {
		fmt.Printf("%s\tpassed=%t\tduration=%s\n", workflow.Name, workflow.Passed, workflow.Duration)
	}
	return nil
}

func discover(root string) ([]string, error) {
	return filepath.Glob(filepath.Join(root, ".tangled", "workflows", "*.yml"))
}

func splitPaths(value string) []string {
	var values []string
	for _, path := range strings.Split(value, ",") {
		if path = strings.TrimSpace(filepath.ToSlash(path)); path != "" {
			values = append(values, path)
		}
	}
	return values
}
