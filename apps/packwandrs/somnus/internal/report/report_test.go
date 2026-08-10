package report

import (
	"reflect"
	"testing"
	"time"
)

func TestNewStatusIncludesTangledRecordShape(t *testing.T) {
	status := NewStatus([]WorkflowResult{{
		Name:     "rust.yml",
		Passed:   true,
		Duration: 1500 * time.Millisecond,
		Steps: []StepResult{{
			Name:     "test",
			Passed:   true,
			Duration: 250 * time.Millisecond,
		}},
	}})
	if len(status.Records) != 1 {
		t.Fatalf("expected one status record, got %d", len(status.Records))
	}
	record := status.Records[0]
	if record["$type"] != "sh.tangled.pipeline.status" || record["status"] != "success" {
		t.Fatalf("unexpected record: %#v", record)
	}
	if record["durationMs"] != int64(1500) {
		t.Fatalf("unexpected duration: %#v", record["durationMs"])
	}
	wantSteps := []map[string]any{{"name": "test", "status": "success", "durationMs": int64(250)}}
	if !reflect.DeepEqual(record["steps"], wantSteps) {
		t.Fatalf("unexpected steps: %#v", record["steps"])
	}
}

func TestSaveReplacesPreviousStatus(t *testing.T) {
	root := t.TempDir()
	first := NewStatus([]WorkflowResult{{Name: "first", Passed: true}})
	second := NewStatus([]WorkflowResult{{Name: "second", Passed: false}})
	if err := Save(root, first); err != nil {
		t.Fatal(err)
	}
	if err := Save(root, second); err != nil {
		t.Fatal(err)
	}
	got, err := Load(root)
	if err != nil {
		t.Fatal(err)
	}
	if len(got.Workflows) != 1 || got.Workflows[0].Name != "second" {
		t.Fatalf("unexpected replacement status: %#v", got)
	}
}
