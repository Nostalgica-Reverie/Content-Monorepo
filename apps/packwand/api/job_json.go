package api

import (
	"encoding/json"
	"time"
)

// MarshalJSON takes a consistent snapshot while a command may still be
// appending log lines or completing in another goroutine.
func (j *Job) MarshalJSON() ([]byte, error) {
	j.mu.Lock()
	defer j.mu.Unlock()
	type snapshot struct {
		ID       string    `json:"id"`
		Action   string    `json:"action"`
		Args     []string  `json:"args"`
		Dir      string    `json:"dir"`
		Status   string    `json:"status"`
		Started  time.Time `json:"started"`
		Finished time.Time `json:"finished,omitempty"`
		ExitCode int       `json:"exit_code,omitempty"`
		Error    string    `json:"error,omitempty"`
		Result   any       `json:"result,omitempty"`
		Lines    []string  `json:"lines,omitempty"`
	}
	return json.Marshal(snapshot{ID: j.ID, Action: j.Action, Args: append([]string(nil), j.Args...), Dir: j.Dir, Status: j.Status, Started: j.Started, Finished: j.Finished, ExitCode: j.ExitCode, Error: j.Error, Result: j.Result, Lines: append([]string(nil), j.Lines...)})
}
