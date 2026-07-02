package gui

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// buildFakeWebview compiles a stub that speaks the curseforge_webview
// stdin/stdout protocol: banner, then one download line per requested file.
func buildFakeWebview(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	src := filepath.Join(dir, "main.go")
	program := `package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func main() {
	fmt.Println("curseforge_webview 0.0.0-test")
	var count int
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		line := scanner.Text()
		if line == "DONE" {
			break
		}
		if strings.Contains(line, " ") {
			count++
		}
	}
	for i := 0; i < count; i++ {
		fmt.Printf("%d https://edge.forgecdn.net/files/100%d/file.jar\n", i, i)
	}
}
`
	if err := os.WriteFile(src, []byte(program), 0o644); err != nil {
		t.Fatal(err)
	}
	bin := filepath.Join(dir, "fake-webview.exe")
	cmd := exec.Command("go", "build", "-o", bin, src)
	cmd.Dir = dir
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("building fake webview: %v\n%s", err, out)
	}
	return bin
}

func TestWebviewBridgeCapturesDownloads(t *testing.T) {
	t.Setenv("CURSEFORGE_WEBVIEW_BIN", buildFakeWebview(t))
	s := &server{root: t.TempDir(), jobs: &jobStore{jobs: map[string]*job{}}}

	body, _ := json.Marshal(webviewRequest{Files: []webviewFile{
		{FileID: 3643025, Slug: "jei"},
		{FileID: 123456, URL: "https://www.curseforge.com/minecraft/mc-mods/sodium"},
	}})
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodPost, "/api/webview/open", bytes.NewReader(body))
	s.handleWebviewOpen(recorder, request)
	if recorder.Code != http.StatusOK {
		t.Fatalf("status = %d: %s", recorder.Code, recorder.Body.String())
	}
	var resp actionResponse
	if err := json.Unmarshal(recorder.Body.Bytes(), &resp); err != nil {
		t.Fatal(err)
	}

	deadline := time.Now().Add(10 * time.Second)
	for {
		j := s.jobs.get(resp.JobID)
		if j == nil {
			t.Fatal("job not found")
		}
		snap := j.snapshot()
		if snap.Status != "running" {
			if snap.Status != "completed" {
				t.Fatalf("job %s: %s", snap.Status, snap.Error)
			}
			joined := strings.Join(snap.Lines, "\n")
			for _, want := range []string{
				"DOWNLOAD 3643025 https://edge.forgecdn.net/files/1000/file.jar",
				"DOWNLOAD 123456 https://edge.forgecdn.net/files/1001/file.jar",
				"captured 2 of 2",
			} {
				if !strings.Contains(joined, want) {
					t.Fatalf("missing %q in job lines:\n%s", want, joined)
				}
			}
			return
		}
		if time.Now().After(deadline) {
			t.Fatal("timed out waiting for webview job")
		}
		time.Sleep(20 * time.Millisecond)
	}
}

func TestWebviewOpenRejectsBadURL(t *testing.T) {
	s := &server{root: t.TempDir(), jobs: &jobStore{jobs: map[string]*job{}}}
	body, _ := json.Marshal(webviewRequest{Files: []webviewFile{
		{FileID: 1, URL: "https://example.com/not-curseforge"},
	}})
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodPost, "/api/webview/open", bytes.NewReader(body))
	s.handleWebviewOpen(recorder, request)
	if recorder.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", recorder.Code)
	}
}
