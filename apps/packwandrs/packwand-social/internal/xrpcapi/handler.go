package xrpcapi

import (
	"crypto/subtle"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
)

// NewHandler exposes the local authenticated identity and record API.
func NewHandler(backend Backend, token string, activity chan<- struct{}) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", func(writer http.ResponseWriter, _ *http.Request) {
		writeJSON(writer, http.StatusOK, map[string]any{"ok": true, "version": "26.2.0"})
	})
	mux.HandleFunc("GET /v1/session", func(writer http.ResponseWriter, _ *http.Request) {
		identity, ok := backend.CurrentIdentity()
		if !ok {
			writeError(writer, http.StatusUnauthorized, "not signed in")
			return
		}
		writeJSON(writer, http.StatusOK, identity)
	})
	mux.HandleFunc("GET /v1/identity/resolve", func(writer http.ResponseWriter, request *http.Request) {
		identifier := request.URL.Query().Get("identifier")
		if identifier == "" {
			writeError(writer, http.StatusBadRequest, "identifier is required")
			return
		}
		identity, err := backend.Resolve(request.Context(), identifier)
		if err != nil {
			writeError(writer, http.StatusBadGateway, err.Error())
			return
		}
		writeJSON(writer, http.StatusOK, identity)
	})
	mux.HandleFunc("POST /v1/record", func(writer http.ResponseWriter, request *http.Request) {
		var input struct {
			Collection string         `json:"collection"`
			RecordKey  string         `json:"rkey"`
			Record     map[string]any `json:"record"`
		}
		if err := decodeJSON(writer, request, &input); err != nil {
			writeError(writer, http.StatusBadRequest, err.Error())
			return
		}
		if input.Collection == "" || input.Record == nil {
			writeError(writer, http.StatusBadRequest, "collection and record are required")
			return
		}
		created, err := backend.CreateRecord(request.Context(), input.Collection, input.RecordKey, input.Record)
		if err != nil {
			writeError(writer, http.StatusBadGateway, err.Error())
			return
		}
		writeJSON(writer, http.StatusCreated, created)
	})
	mux.HandleFunc("GET /v1/record", func(writer http.ResponseWriter, request *http.Request) {
		collection := request.URL.Query().Get("collection")
		if collection == "" {
			writeError(writer, http.StatusBadRequest, "collection is required")
			return
		}
		limit := 50
		if raw := request.URL.Query().Get("limit"); raw != "" {
			parsed, err := strconv.Atoi(raw)
			if err != nil || parsed < 1 || parsed > 100 {
				writeError(writer, http.StatusBadRequest, "limit must be between 1 and 100")
				return
			}
			limit = parsed
		}
		page, err := backend.ListRecords(request.Context(), request.URL.Query().Get("repo"), collection, limit, request.URL.Query().Get("cursor"))
		if err != nil {
			writeError(writer, http.StatusBadGateway, err.Error())
			return
		}
		writeJSON(writer, http.StatusOK, page)
	})
	mux.HandleFunc("POST /v1/blob", func(writer http.ResponseWriter, request *http.Request) {
		mimeType := strings.TrimSpace(strings.Split(request.Header.Get("Content-Type"), ";")[0])
		if !strings.HasPrefix(mimeType, "image/") {
			writeError(writer, http.StatusUnsupportedMediaType, "an image Content-Type is required")
			return
		}
		request.Body = http.MaxBytesReader(writer, request.Body, 10<<20)
		data, err := io.ReadAll(request.Body)
		if err != nil {
			writeError(writer, http.StatusBadRequest, "image exceeds the 10 MiB upload limit")
			return
		}
		if len(data) == 0 {
			writeError(writer, http.StatusBadRequest, "image is empty")
			return
		}
		blob, err := backend.UploadBlob(request.Context(), mimeType, data)
		if err != nil {
			writeError(writer, http.StatusBadGateway, err.Error())
			return
		}
		writeJSON(writer, http.StatusCreated, blob)
	})
	mux.HandleFunc("GET /v1/friends", func(writer http.ResponseWriter, request *http.Request) {
		friends, err := backend.ListFriends(request.Context())
		if err != nil {
			writeError(writer, http.StatusBadGateway, err.Error())
			return
		}
		writeJSON(writer, http.StatusOK, friends)
	})
	mux.HandleFunc("GET /v1/invites", func(writer http.ResponseWriter, request *http.Request) {
		invites, err := backend.ListPendingInvites(request.Context())
		if err != nil {
			writeError(writer, http.StatusBadGateway, err.Error())
			return
		}
		writeJSON(writer, http.StatusOK, invites)
	})
	mux.HandleFunc("GET /v1/tangled/repos", func(writer http.ResponseWriter, request *http.Request) {
		repositories, err := backend.LinkedTangledRepos(request.Context())
		if err != nil {
			writeError(writer, http.StatusBadGateway, err.Error())
			return
		}
		writeJSON(writer, http.StatusOK, repositories)
	})

	return http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		select {
		case activity <- struct{}{}:
		default:
		}
		if request.URL.Path != "/health" && token != "" && !authorized(request, token) {
			writeError(writer, http.StatusUnauthorized, "unauthorized")
			return
		}
		writer.Header().Set("Cache-Control", "no-store")
		writer.Header().Set("X-Content-Type-Options", "nosniff")
		mux.ServeHTTP(writer, request)
	})
}

func authorized(request *http.Request, token string) bool {
	provided := request.Header.Get("Authorization")
	expected := "Bearer " + token
	return len(provided) == len(expected) && subtle.ConstantTimeCompare([]byte(provided), []byte(expected)) == 1
}

func decodeJSON(writer http.ResponseWriter, request *http.Request, output any) error {
	request.Body = http.MaxBytesReader(writer, request.Body, 1<<20)
	decoder := json.NewDecoder(request.Body)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(output); err != nil {
		return fmt.Errorf("decode request: %w", err)
	}
	return nil
}

func writeError(writer http.ResponseWriter, status int, message string) {
	writeJSON(writer, status, map[string]string{"error": message})
}

func writeJSON(writer http.ResponseWriter, status int, value any) {
	writer.Header().Set("Content-Type", "application/json; charset=utf-8")
	writer.WriteHeader(status)
	_ = json.NewEncoder(writer).Encode(value)
}
