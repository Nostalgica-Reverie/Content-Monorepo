package runner

import (
	"context"
	"os"
	"os/exec"
	"runtime"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/report"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/somnus/internal/schema"
)

func Run(ctx context.Context, path string, workflow schema.Workflow, root string) report.WorkflowResult {
	started := time.Now()
	result := report.WorkflowResult{Name: path, Engine: workflow.Engine, Image: workflow.Image, Passed: true}
	for _, step := range workflow.Steps {
		stepStarted := time.Now()
		command := shell(ctx, step.Command)
		command.Dir = root
		command.Stdin = os.Stdin
		command.Stdout = os.Stdout
		command.Stderr = os.Stderr
		err := command.Run()
		stepResult := report.StepResult{Name: step.Name, Passed: err == nil, Duration: time.Since(stepStarted)}
		if err != nil {
			stepResult.Error = err.Error()
			result.Passed = false
		}
		result.Steps = append(result.Steps, stepResult)
		if err != nil {
			break
		}
	}
	result.Duration = time.Since(started)
	return result
}

func shell(ctx context.Context, command string) *exec.Cmd {
	if runtime.GOOS == "windows" {
		return exec.CommandContext(ctx, "cmd", "/D", "/S", "/C", command)
	}
	return exec.CommandContext(ctx, "sh", "-c", command)
}
