package api

import (
	"bufio"
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

type webviewRequest struct {
	Provider string        `json:"provider,omitempty"`
	Files    []webviewFile `json:"files"`
}
type webviewFile struct {
	FileID string `json:"file_id"`
	URL    string `json:"url,omitempty"`
	Slug   string `json:"slug,omitempty"`
}
type webviewProvider struct {
	name       string
	projectURL *regexp.Regexp
	fileID     *regexp.Regexp
	slugToURL  func(string) string
}

var webviewProviders = map[string]webviewProvider{
	"curseforge": {"curseforge", regexp.MustCompile(`^https?://(?:(?:www|beta)\.)?curseforge\.com/[^/]+/[^/]+/[^/]+$`), regexp.MustCompile(`^[0-9]+$`), func(slug string) string { return "https://www.curseforge.com/minecraft/mc-mods/" + slug }},
	"modrinth":   {"modrinth", regexp.MustCompile(`^https?://(?:www\.)?modrinth\.com/[^/]+/[^/]+$`), regexp.MustCompile(`^[a-zA-Z0-9]+$`), func(slug string) string { return "https://modrinth.com/mod/" + slug }},
}

func (s *Server) handleWebviewOpen(w http.ResponseWriter, r *http.Request) {
	var request webviewRequest
	if err := decodeBody(r, &request); err != nil {
		writeError(w, 400, "invalid_argument", err.Error(), "body")
		return
	}
	if request.Provider == "" {
		request.Provider = "curseforge"
	}
	provider, ok := webviewProviders[request.Provider]
	if !ok {
		writeError(w, 400, "invalid_argument", "provider must be curseforge or modrinth", "provider")
		return
	}
	if len(request.Files) == 0 {
		writeError(w, 400, "invalid_argument", "at least one file is required", "files")
		return
	}
	for i := range request.Files {
		file := &request.Files[i]
		if file.URL == "" {
			if !idPattern.MatchString(file.Slug) {
				writeError(w, 400, "invalid_argument", "invalid project slug", "slug")
				return
			}
			file.URL = provider.slugToURL(file.Slug)
		}
		if !provider.projectURL.MatchString(file.URL) {
			writeError(w, 400, "invalid_argument", fmt.Sprintf("not a %s project URL", provider.name), "url")
			return
		}
		if !provider.fileID.MatchString(file.FileID) {
			writeError(w, 400, "invalid_argument", fmt.Sprintf("invalid %s file/version ID", provider.name), "file_id")
			return
		}
	}
	bin, err := s.findWebviewBin()
	if err != nil {
		writeError(w, http.StatusFailedDependency, "internal", err.Error(), "")
		return
	}
	action := Action{Name: "webview.open", Method: http.MethodPost, Path: Prefix + "/webview/open"}
	job := s.jobs.create(action, []string{provider.name, fmt.Sprintf("%d file(s)", len(request.Files))}, s.root)
	go s.runWebview(job, bin, provider.name, request.Files)
	writeJSONStatus(w, http.StatusAccepted, map[string]string{"job_id": job.ID})
}

func (s *Server) findWebviewBin() (string, error) {
	if bin := os.Getenv("MOD_BROWSER_WEBVIEW_BIN"); bin != "" {
		if _, err := os.Stat(bin); err == nil {
			return bin, nil
		}
		return "", fmt.Errorf("MOD_BROWSER_WEBVIEW_BIN does not exist: %s", bin)
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

func (s *Server) runWebview(job *Job, bin, provider string, files []webviewFile) {
	job.append(fmt.Sprintf("$ %s --provider %s (%d file(s))", filepath.Base(bin), provider, len(files)))
	command := exec.Command(bin, "--provider", provider)
	command.Dir = s.root
	stdin, err := command.StdinPipe()
	if err != nil {
		job.finish(-1, err)
		return
	}
	stdout, err := command.StdoutPipe()
	if err != nil {
		job.finish(-1, err)
		return
	}
	command.Stderr = io.Discard
	if err = command.Start(); err != nil {
		job.finish(-1, fmt.Errorf("failed to start webview: %w", err))
		return
	}
	go func() {
		defer stdin.Close()
		for _, file := range files {
			if _, err := fmt.Fprintf(stdin, "%s %s\n", file.FileID, file.URL); err != nil {
				return
			}
		}
		_, _ = io.WriteString(stdin, "DONE\n")
	}()
	captured := 0
	inError := false
	scanner := bufio.NewScanner(stdout)
	for scanner.Scan() {
		line := scanner.Text()
		switch {
		case inError:
			job.append("error: " + line)
		case line == "ERROR":
			inError = true
		case strings.HasPrefix(line, "mod_browser_webview "), strings.HasPrefix(line, "curseforge_webview "):
			job.append(line)
		default:
			indexText, downloadURL, ok := strings.Cut(line, " ")
			var index int
			if ok {
				_, scanErr := fmt.Sscanf(indexText, "%d", &index)
				ok = scanErr == nil && index >= 0 && index < len(files)
			}
			if !ok {
				job.append(line)
				continue
			}
			captured++
			job.append(fmt.Sprintf("DOWNLOAD %s %s", files[index].FileID, downloadURL))
		}
	}
	err = command.Wait()
	exitCode := 0
	if command.ProcessState != nil {
		exitCode = command.ProcessState.ExitCode()
	}
	if err == nil && inError {
		err = errors.New("webview reported an error")
	}
	job.append(fmt.Sprintf("captured %d of %d download URL(s)", captured, len(files)))
	job.finish(exitCode, err)
}
