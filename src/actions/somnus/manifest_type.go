package main

import (
	"encoding/json"
	"fmt"
	"os"
)

type Manifest struct {
	Schema       string      `json:"$schema,omitempty"`
	ID           string      `json:"id"`
	Name         string      `json:"name"`
	Type         string      `json:"type"`
	Loader       string      `json:"loader,omitempty"`
	MCVersion    *string     `json:"mc_version,omitempty"`
	Variants     []Variant   `json:"variants,omitempty"`
	Version      string      `json:"version,omitempty"`
	ReleaseType  string      `json:"release_type,omitempty"`
	Description  string      `json:"description,omitempty"`
	ModrinthID   string      `json:"modrinth_id,omitempty"`
	CurseforgeID string      `json:"curseforge_id,omitempty"`
	Role         Role        `json:"role"`
	SharedAssets string      `json:"shared_assets,omitempty"`
	Automation   *Automation `json:"automation,omitempty"`
}

type Variant struct {
	MCVersion string `json:"mc_version"`
	ID        string `json:"id,omitempty"`
	Name      string `json:"name,omitempty"`
	Loader    string `json:"loader,omitempty"`
	Version   string `json:"version,omitempty"`
}

type PerformanceBase struct {
	Pack     string    `json:"pack"`
	Mappings []Mapping `json:"mappings"`
}

type Mapping struct {
	Source string `json:"source"`
	Target string `json:"target"`
}

type Role struct {
	raw json.RawMessage
}

func (r *Role) UnmarshalJSON(b []byte) error {
	r.raw = append(r.raw[:0], b...)
	return nil
}

func (r Role) MarshalJSON() ([]byte, error) {
	if len(r.raw) == 0 {
		return []byte(`"none"`), nil
	}
	return r.raw, nil
}

func (r Role) IsZero() bool {
	return len(r.raw) == 0 || string(r.raw) == "null"
}

func (r Role) IsBase() bool {
	var s string
	return json.Unmarshal(r.raw, &s) == nil && s == "base"
}

func (r Role) ConsumerBase() *PerformanceBase {
	var obj struct {
		PB *PerformanceBase `json:"performance_base"`
	}
	if json.Unmarshal(r.raw, &obj) == nil && obj.PB != nil {
		return obj.PB
	}
	return nil
}

func (r Role) Label() string {
	var s string
	if json.Unmarshal(r.raw, &s) == nil && s != "" {
		return s
	}
	if r.ConsumerBase() != nil {
		return "consumer"
	}
	return "none"
}

func StringRole(s string) Role {
	b, _ := json.Marshal(s)
	return Role{raw: b}
}

func ConsumerRole(pb PerformanceBase) Role {
	type wrapper struct {
		PB PerformanceBase `json:"performance_base"`
	}
	b, _ := json.Marshal(wrapper{PB: pb})
	return Role{raw: b}
}

func ReadManifest(path string) (*Manifest, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to open %s: %w", path, err)
	}
	var m Manifest
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, fmt.Errorf("invalid JSON in %s: %w", path, err)
	}
	return &m, nil
}

func WriteManifest(path string, m *Manifest) error {
	data, err := json.MarshalIndent(m, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal %s: %w", path, err)
	}
	data = append(data, '\n')
	return os.WriteFile(path, data, 0o644)
}
