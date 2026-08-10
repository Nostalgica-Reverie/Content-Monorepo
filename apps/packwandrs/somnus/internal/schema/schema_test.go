package schema

import (
	"path/filepath"
	"testing"
)

func TestEveryRepositoryWorkflowParses(t *testing.T) {
	files, err := filepath.Glob(filepath.Join("..", "..", "..", "..", "..", ".tangled", "workflows", "*.yml"))
	if err != nil || len(files) == 0 {
		t.Fatalf("discover repository workflows: %v (%d files)", err, len(files))
	}
	for _, file := range files {
		if _, err := Load(file); err != nil {
			t.Errorf("%s: %v", file, err)
		}
	}
}
