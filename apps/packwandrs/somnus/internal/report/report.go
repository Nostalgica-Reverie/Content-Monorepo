package report

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"runtime"
	"time"
)

type Status struct {
	Workflows []WorkflowResult `json:"workflows"`
	Records   []map[string]any `json:"records,omitempty"`
}

type WorkflowResult struct {
	Name     string        `json:"workflow"`
	Engine   string        `json:"engine"`
	Image    string        `json:"image"`
	Passed   bool          `json:"passed"`
	Duration time.Duration `json:"duration"`
	Steps    []StepResult  `json:"steps"`
}

type StepResult struct {
	Name     string        `json:"name"`
	Passed   bool          `json:"passed"`
	Duration time.Duration `json:"duration"`
	Error    string        `json:"error,omitempty"`
}

func Save(root string, status Status) error {
	path := statusPath(root)
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	data, err := json.MarshalIndent(status, "", "  ")
	if err != nil {
		return err
	}
	data = append(data, '\n')
	temporary, err := os.CreateTemp(filepath.Dir(path), ".status-*.tmp")
	if err != nil {
		return err
	}
	temporaryPath := temporary.Name()
	defer os.Remove(temporaryPath)
	if err := temporary.Chmod(0o600); err != nil {
		temporary.Close()
		return err
	}
	if _, err := temporary.Write(data); err != nil {
		temporary.Close()
		return err
	}
	if err := temporary.Close(); err != nil {
		return err
	}
	if runtime.GOOS == "windows" {
		if err := os.Remove(path); err != nil && !errors.Is(err, os.ErrNotExist) {
			return err
		}
	}
	return os.Rename(temporaryPath, path)
}

func NewStatus(results []WorkflowResult) Status {
	records := make([]map[string]any, 0, len(results))
	for _, result := range results {
		records = append(records, PipelineStatusRecord(result))
	}
	return Status{Workflows: results, Records: records}
}

func Load(root string) (Status, error) {
	data, err := os.ReadFile(statusPath(root))
	if errors.Is(err, os.ErrNotExist) {
		return Status{}, nil
	}
	if err != nil {
		return Status{}, err
	}
	var status Status
	err = json.Unmarshal(data, &status)
	return status, err
}

func statusPath(root string) string {
	return filepath.Join(root, ".somnus", "status.json")
}

func PipelineStatusRecord(result WorkflowResult) map[string]any {
	steps := make([]map[string]any, 0, len(result.Steps))
	for _, step := range result.Steps {
		value := map[string]any{
			"name":       step.Name,
			"status":     map[bool]string{true: "success", false: "failure"}[step.Passed],
			"durationMs": step.Duration.Milliseconds(),
		}
		if step.Error != "" {
			value["error"] = step.Error
		}
		steps = append(steps, value)
	}
	return map[string]any{
		"$type":      "sh.tangled.pipeline.status",
		"workflow":   result.Name,
		"status":     map[bool]string{true: "success", false: "failure"}[result.Passed],
		"durationMs": result.Duration.Milliseconds(),
		"createdAt":  time.Now().UTC().Format(time.RFC3339),
		"steps":      steps,
	}
}
