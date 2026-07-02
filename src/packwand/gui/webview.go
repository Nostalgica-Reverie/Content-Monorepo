package gui

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
)

// The mod_browser_webview binary (lib/mod-browser-webview) speaks a line
// protocol: it reads "<fileID> <projectURL>" lines then "DONE" on stdin,
// shows the provider's pages in a native webview window, and prints
// "<index> <downloadURL>" on stdout for every file the user downloads.
// The provider (curseforge or modrinth) is selected with a --provider flag.
// This bridge exposes that flow over the GUI's job/SSE machinery so the
// frontend can watch navigation and download events live.

type webviewRequest struct {
	// Provider is "curseforge" (default) or "modrinth".
	Provider string        `json:"provider,omitempty"`
	Files    []webviewFile `json:"files"`
}

type webviewFile struct {
	// FileID is the CurseForge file ID (numeric) or Modrinth version ID.
	FileID string `json:"file_id"`
	// URL is the project page. When empty, it is derived from Slug.
	URL  string `json:"url,omitempty"`
	Slug string `json:"slug,omitempty"`
}

// webviewProvider mirrors the per-provider validation in
// lib/mod-browser-webview/src/main.rs.
type webviewProvider struct {
	name       string
	projectURL *regexp.Regexp
	fileID     *regexp.Regexp
	slugToURL  func(slug string) string
}

var webviewProviders = map[string]webviewProvider{
	"curseforge": {
		name:       "curseforge",
		projectURL: regexp.MustCompile(`^https?://(?:(?:www|beta)\.)?curseforge\.com/[^/]+/[^/]+/[^/]+$`),
		fileID:     regexp.MustCompile(`^[0-9]+$`),
		slugToURL:  func(slug string) string { return "https://www.curseforge.com/minecraft/mc-mods/" + slug },
	},
	"modrinth": {
		name:       "modrinth",
		projectURL: regexp.MustCompile(`^https?://(?:www\.)?modrinth\.com/[^/]+/[^/]+$`),
		fileID:     regexp.MustCompile(`^[a-zA-Z0-9]+$`),
		slugToURL:  func(slug string) string { return "https://modrinth.com/mod/" + slug },
	},
}

func (s *server) handleWebviewOpen(w http.ResponseWriter, r *http.Request) {
	var req webviewRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid JSON", http.StatusBadRequest)
		return
	}
	if req.Provider == "" {
		req.Provider = "curseforge"
	}
	provider, ok := webviewProviders[req.Provider]
	if !ok {
		http.Error(w, fmt.Sprintf("unknown provider %q (expected curseforge or modrinth)", req.Provider), http.StatusBadRequest)
		return
	}
	if len(req.Files) == 0 {
		http.Error(w, "at least one file is required", http.StatusBadRequest)
		return
	}
	for i := range req.Files {
		f := &req.Files[i]
		if f.URL == "" {
			slug, err := cleanSlug(f.Slug)
			if err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			f.URL = provider.slugToURL(slug)
		}
		if !provider.projectURL.MatchString(f.URL) {
			http.Error(w, fmt.Sprintf("not a %s project URL: %s", provider.name, f.URL), http.StatusBadRequest)
			return
		}
		if !provider.fileID.MatchString(f.FileID) {
			http.Error(w, fmt.Sprintf("invalid %s file/version ID: %q", provider.name, f.FileID), http.StatusBadRequest)
			return
		}
	}

	bin, err := s.findWebviewBin()
	if err != nil {
		http.Error(w, err.Error(), http.StatusFailedDependency)
		return
	}

	j := s.jobs.create("mod-browser-webview", []string{provider.name, fmt.Sprintf("%d file(s)", len(req.Files))}, s.root)
	go s.runWebview(j, bin, provider.name, req.Files)
	writeJSON(w, actionResponse{JobID: j.ID})
}

// findWebviewBin locates the mod_browser_webview executable: an explicit
// MOD_BROWSER_WEBVIEW_BIN, the in-repo cargo output, then PATH.
func (s *server) findWebviewBin() (string, error) {
	if bin := os.Getenv("MOD_BROWSER_WEBVIEW_BIN"); bin != "" {
		if _, err := os.Stat(bin); err == nil {
			return bin, nil
		}
		return "", fmt.Errorf("MOD_BROWSER_WEBVIEW_BIN is set but does not exist: %s", bin)
	}
	name := "mod_browser_webview"
	if runtime.GOOS == "windows" {
		name += ".exe"
	}
	for _, profile := range []string{"release", "debug"} {
		candidate := filepath.Join(s.root, "lib", "mod-browser-webview", "target", profile, name)
		if _, err := os.Stat(candidate); err == nil {
			return candidate, nil
		}
	}
	if bin, err := exec.LookPath("mod_browser_webview"); err == nil {
		return bin, nil
	}
	return "", errors.New("mod_browser_webview binary not found; run 'task build-webview' or set MOD_BROWSER_WEBVIEW_BIN")
}

func (s *server) runWebview(j *job, bin, provider string, files []webviewFile) {
	j.append(fmt.Sprintf("$ %s --provider %s (%d file(s))", filepath.Base(bin), provider, len(files)))

	c := exec.Command(bin, "--provider", provider)
	c.Dir = s.root
	stdin, err := c.StdinPipe()
	if err != nil {
		j.finish(-1, err)
		return
	}
	stdout, err := c.StdoutPipe()
	if err != nil {
		j.finish(-1, err)
		return
	}
	c.Stderr = io.Discard
	if err := c.Start(); err != nil {
		j.finish(-1, fmt.Errorf("failed to start webview: %w", err))
		return
	}

	// Feed the request lines, then DONE to open the window.
	go func() {
		defer func() { _ = stdin.Close() }()
		for _, f := range files {
			if _, err := fmt.Fprintf(stdin, "%s %s\n", f.FileID, f.URL); err != nil {
				return
			}
		}
		_, _ = io.WriteString(stdin, "DONE\n")
	}()

	// Parse stdout: a version banner, then "<index> <downloadURL>" per
	// downloaded file; "ERROR" precedes failure details.
	captured := 0
	inError := false
	scanner := bufio.NewScanner(stdout)
	for scanner.Scan() {
		line := scanner.Text()
		switch {
		case inError:
			j.append("error: " + line)
		case line == "ERROR":
			inError = true
		case strings.HasPrefix(line, "mod_browser_webview "), strings.HasPrefix(line, "curseforge_webview "):
			j.append(line)
		default:
			idxStr, url, ok := strings.Cut(line, " ")
			var idx int
			if ok {
				_, err := fmt.Sscanf(idxStr, "%d", &idx)
				ok = err == nil && idx >= 0 && idx < len(files)
			}
			if !ok {
				j.append(line)
				continue
			}
			captured++
			j.append(fmt.Sprintf("DOWNLOAD %s %s", files[idx].FileID, url))
		}
	}

	err = c.Wait()
	exitCode := 0
	if c.ProcessState != nil {
		exitCode = c.ProcessState.ExitCode()
	}
	if err == nil && inError {
		err = errors.New("webview reported an error")
	}
	j.append(fmt.Sprintf("captured %d of %d download URL(s)", captured, len(files)))
	j.finish(exitCode, err)
}
