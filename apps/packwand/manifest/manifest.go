package manifest

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type Manifest struct {
	Schema       string    `json:"$schema,omitempty"`
	ID           string    `json:"id"`
	Name         string    `json:"name"`
	Type         string    `json:"type"`
	Loader       string    `json:"loader,omitempty"`
	MCVersion    *string   `json:"mc_version,omitempty"`
	Variants     []Variant `json:"variants,omitempty"`
	Version      string    `json:"version,omitempty"`
	ReleaseType  string    `json:"release_type,omitempty"`
	Description  string    `json:"description,omitempty"`
	ModrinthID   string    `json:"modrinth_id,omitempty"`
	CurseforgeID string    `json:"curseforge_id,omitempty"`
	GitHubID     string    `json:"github_id,omitempty"`
	GiteaID      string    `json:"gitea_id,omitempty"`
	GitLabID     string    `json:"gitlab_id,omitempty"`
	Role         Role      `json:"role"`
	SharedAssets string    `json:"shared_assets,omitempty"`
	// Lifecycle declares the pack's maintenance state: active, maintenance, archived, eol.
	// archived and eol packs are excluded from workspace auto-update operations.
	Lifecycle  string      `json:"lifecycle,omitempty"`
	Automation *Automation `json:"automation,omitempty"`
}

// Entry identifies a manifest discovered beneath one of the repository's
// publishable content categories.
type Entry struct {
	Category string
	Dir      string
	ID       string
	Manifest *Manifest
}

// LoadAll discovers manifests in the repository's publishable content
// categories. Missing categories and directories without a valid manifest are
// ignored, matching the established CLI and HTTP API behaviour.
func LoadAll(root string) ([]Entry, error) {
	var out []Entry
	for _, category := range []string{"modpacks", "datapacks", "resourcepacks"} {
		entries, err := os.ReadDir(filepath.Join(root, category))
		if os.IsNotExist(err) {
			continue
		}
		if err != nil {
			return nil, fmt.Errorf("read %s: %w", category, err)
		}
		for _, entry := range entries {
			if !entry.IsDir() {
				continue
			}
			dir := filepath.Join(root, category, entry.Name())
			m, err := Read(filepath.Join(dir, "manifest.json"))
			if err != nil {
				continue
			}
			id := m.ID
			if id == "" {
				id = entry.Name()
			}
			out = append(out, Entry{Category: category, Dir: dir, ID: id, Manifest: m})
		}
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Category != out[j].Category {
			return out[i].Category < out[j].Category
		}
		return out[i].ID < out[j].ID
	})
	return out, nil
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

type Automation struct {
	AutoUpdate  *bool               `json:"auto_update,omitempty"`
	ServerPromo *bool               `json:"server_promo,omitempty"`
	SyncExclude []string            `json:"sync_exclude,omitempty"`
	Freeze      map[string][]string `json:"freeze,omitempty"`
	FullAuto    *FullAuto           `json:"full_auto,omitempty"`
}

// FullAuto configures the end-to-end unattended release pipeline
// (see 'packwand automation run'): update -> refresh -> validate -> tests ->
// docs -> bump. A commit landing on main from that pipeline is picked up by
// the existing publish.yml exactly like a human-triggered bump.
type FullAuto struct {
	Enabled bool `json:"enabled"`
	// Tests are shell commands run from the pack dir before bump; a nonzero
	// exit aborts the run and discards the update/refresh changes.
	Tests []string `json:"tests,omitempty"`
}

// Read parses a manifest.json file.
func Read(path string) (*Manifest, error) {
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

// Write serialises a Manifest to path.
func Write(path string, m *Manifest) error {
	data, err := json.MarshalIndent(m, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal %s: %w", path, err)
	}
	data = append(data, '\n')
	return os.WriteFile(path, data, 0o644)
}

type legacyOptOut struct {
	AutoUpdate  *bool    `json:"auto_update"`
	ServerPromo *bool    `json:"server_promo"`
	SyncExclude []string `json:"sync_exclude"`
	Freeze      []string `json:"freeze"`
}

// SubDirsOf returns all pack sub-directories (ending -mr or -cf) inside packDir.
func SubDirsOf(packDir string) []string {
	var out []string
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return nil
	}
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		if strings.HasSuffix(e.Name(), "-mr") || strings.HasSuffix(e.Name(), "-cf") {
			out = append(out, filepath.Join(packDir, e.Name()))
		}
	}
	return out
}

// ReadAutomation returns the effective automation config for a pack directory,
// merging manifest.json automation with legacy opt-out.json if present.
func ReadAutomation(packDir string) Automation {
	var a Automation
	if m, err := Read(filepath.Join(packDir, "manifest.json")); err == nil && m.Automation != nil {
		a = *m.Automation
	}
	data, err := os.ReadFile(filepath.Join(packDir, "opt-out.json"))
	if err != nil {
		return a
	}
	var legacy legacyOptOut
	if err := json.Unmarshal(data, &legacy); err != nil {
		return a
	}
	if a.AutoUpdate == nil {
		a.AutoUpdate = legacy.AutoUpdate
	}
	if a.ServerPromo == nil {
		a.ServerPromo = legacy.ServerPromo
	}
	a.SyncExclude = append(a.SyncExclude, legacy.SyncExclude...)
	if len(legacy.Freeze) > 0 {
		if a.Freeze == nil {
			a.Freeze = map[string][]string{}
		}
		for _, sub := range SubDirsOf(packDir) {
			key := filepath.Base(sub)
			a.Freeze[key] = append(a.Freeze[key], legacy.Freeze...)
		}
	}
	return a
}

// HasLegacyOptOut reports whether packDir contains a legacy opt-out.json.
func HasLegacyOptOut(packDir string) bool {
	_, err := os.Stat(filepath.Join(packDir, "opt-out.json"))
	return err == nil
}

// LifecycleState returns the lifecycle value for the pack at packDir,
// or "active" if unset or the manifest cannot be read.
func LifecycleState(packDir string) string {
	m, err := Read(filepath.Join(packDir, "manifest.json"))
	if err != nil || m.Lifecycle == "" {
		return "active"
	}
	return m.Lifecycle
}

// OptedOutOfAutoUpdate reports whether auto-update is disabled for packDir.
// Packs with lifecycle "archived" or "eol" are always skipped.
func OptedOutOfAutoUpdate(packDir string) (skip bool, legacy bool) {
	lc := LifecycleState(packDir)
	if lc == "archived" || lc == "eol" {
		return true, false
	}
	a := ReadAutomation(packDir)
	if a.AutoUpdate != nil && !*a.AutoUpdate {
		return true, HasLegacyOptOut(packDir)
	}
	return false, false
}

// FullAutoEnabled reports whether packDir has opted into the full automation
// pipeline. Packs with lifecycle "archived" or "eol" are always excluded,
// regardless of the manifest's full_auto.enabled value.
func FullAutoEnabled(packDir string) bool {
	if lc := LifecycleState(packDir); lc == "archived" || lc == "eol" {
		return false
	}
	a := ReadAutomation(packDir)
	return a.FullAuto != nil && a.FullAuto.Enabled
}

// SetAutomationFreeze updates the freeze list for a single sub-directory key
// inside packDir/manifest.json.
func SetAutomationFreeze(packDir, subKey string, slugs []string) error {
	mfPath := filepath.Join(packDir, "manifest.json")
	m, err := Read(mfPath)
	if err != nil {
		return fmt.Errorf("failed to read %s: %w", mfPath, err)
	}
	if m.Automation == nil {
		m.Automation = &Automation{}
	}
	if len(slugs) == 0 {
		delete(m.Automation.Freeze, subKey)
	} else {
		if m.Automation.Freeze == nil {
			m.Automation.Freeze = map[string][]string{}
		}
		m.Automation.Freeze[subKey] = slugs
	}
	if len(m.Automation.Freeze) == 0 {
		m.Automation.Freeze = nil
	}
	if m.Automation.AutoUpdate == nil && m.Automation.ServerPromo == nil &&
		len(m.Automation.SyncExclude) == 0 && m.Automation.Freeze == nil {
		m.Automation = nil
	}
	return Write(mfPath, m)
}
