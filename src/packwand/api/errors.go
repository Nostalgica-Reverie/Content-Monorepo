package api

import (
	"encoding/json"
	"net/http"
)

// Error is the stable error payload returned by every API endpoint.
type Error struct {
	Code    string `json:"code"`
	Message string `json:"message"`
	Field   string `json:"field,omitempty"`
}

type errorEnvelope struct {
	Error Error `json:"error"`
}

func writeError(w http.ResponseWriter, status int, code, message, field string) {
	writeJSONStatus(w, status, errorEnvelope{Error: Error{Code: code, Message: message, Field: field}})
}

func writeJSON(w http.ResponseWriter, value any) {
	writeJSONStatus(w, http.StatusOK, value)
}

func writeJSONStatus(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	_ = enc.Encode(value)
}
