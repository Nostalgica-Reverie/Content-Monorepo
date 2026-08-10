package schema

import (
	"bytes"
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

type Workflow struct {
	When         []Condition `yaml:"when"`
	Engine       string      `yaml:"engine"`
	Image        string      `yaml:"image"`
	Clone        Clone       `yaml:"clone"`
	Dependencies []string    `yaml:"dependencies"`
	Steps        []Step      `yaml:"steps"`
}

type Condition struct {
	Event  []string `yaml:"event"`
	Branch []string `yaml:"branch"`
	Tag    []string `yaml:"tag"`
	Paths  []string `yaml:"paths"`
}

type Clone struct {
	Depth int `yaml:"depth"`
}

type Step struct {
	Name    string `yaml:"name"`
	Command string `yaml:"command"`
}

func Load(path string) (Workflow, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return Workflow{}, err
	}
	var workflow Workflow
	decoder := yaml.NewDecoder(bytes.NewReader(data))
	decoder.KnownFields(true)
	if err := decoder.Decode(&workflow); err != nil {
		return Workflow{}, fmt.Errorf("parse %s: %w", path, err)
	}
	if workflow.Engine == "" || workflow.Image == "" || len(workflow.Steps) == 0 {
		return Workflow{}, fmt.Errorf("%s is missing engine, image, or steps", path)
	}
	for index, step := range workflow.Steps {
		if step.Name == "" || step.Command == "" {
			return Workflow{}, fmt.Errorf("%s step %d is missing name or command", path, index+1)
		}
	}
	return workflow, nil
}
