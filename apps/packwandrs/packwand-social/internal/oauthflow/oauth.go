package oauthflow

import (
	"context"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"runtime"
	"time"

	"github.com/bluesky-social/indigo/atproto/auth/oauth"
	"github.com/bluesky-social/indigo/atproto/syntax"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
)

const (
	defaultClientID    = "https://packwand.nostalgica.net/oauth/client-metadata.json"
	defaultCallbackURL = "http://127.0.0.1:38427/callback"
)

var scopes = []string{
	"atproto",
	"blob:image/*",
	"repo:net.nostalgica.packwand.contact",
	"repo:net.nostalgica.packwand.image",
	"repo:net.nostalgica.packwand.pack",
	"repo:net.nostalgica.packwand.profile",
	"repo:net.nostalgica.packwand.session.invite",
	"repo:net.nostalgica.packwand.snippet",
	"repo:sh.tangled.pipeline.status",
}

// NewApp builds the shared OAuth client around a persistent auth store.
func NewApp(store oauth.ClientAuthStore) *oauth.ClientApp {
	clientID := environmentOr("PACKWAND_SOCIAL_CLIENT_ID", defaultClientID)
	callbackURL := environmentOr("PACKWAND_SOCIAL_CALLBACK_URL", defaultCallbackURL)
	config := oauth.NewPublicConfig(clientID, callbackURL, scopes)
	config.UserAgent = "Packwand/26.2.0"
	return oauth.NewClientApp(&config, store)
}

// Login completes OAuth through the pinned loopback callback.
func Login(ctx context.Context, store oauth.ClientAuthStore, identifier string, messages io.Writer) (*oauth.ClientSessionData, error) {
	app := NewApp(store)
	listener, err := net.Listen("tcp", "127.0.0.1:38427")
	if err != nil {
		return nil, fmt.Errorf("listen for OAuth callback: %w", err)
	}
	defer listener.Close()

	redirect, err := app.StartAuthFlow(ctx, identifier)
	if err != nil {
		return nil, fmt.Errorf("start OAuth flow: %w", err)
	}
	result := make(chan callbackResult, 1)
	mux := http.NewServeMux()
	mux.HandleFunc("GET /callback", func(writer http.ResponseWriter, request *http.Request) {
		data, callbackErr := app.ProcessCallback(request.Context(), request.URL.Query())
		if callbackErr != nil {
			http.Error(writer, "Packwand could not complete sign-in. You may close this tab.", http.StatusBadRequest)
		} else {
			writer.Header().Set("Content-Type", "text/html; charset=utf-8")
			writer.Header().Set("X-Content-Type-Options", "nosniff")
			_, _ = io.WriteString(writer, "<!doctype html><title>Packwand sign-in complete</title><p>You can return to Packwand.</p>")
		}
		result <- callbackResult{data: data, err: callbackErr}
	})
	server := &http.Server{Handler: mux, ReadHeaderTimeout: 10 * time.Second}
	go func() {
		_ = server.Serve(listener)
	}()

	if err := openBrowser(redirect); err != nil {
		fmt.Fprintf(messages, "Open this URL to sign in:\n%s\n", redirect)
	}
	waitContext, cancel := context.WithTimeout(ctx, 10*time.Minute)
	defer cancel()
	select {
	case <-waitContext.Done():
		return nil, fmt.Errorf("wait for OAuth callback: %w", waitContext.Err())
	case outcome := <-result:
		shutdownContext, shutdownCancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer shutdownCancel()
		_ = server.Shutdown(shutdownContext)
		if outcome.err != nil {
			return nil, fmt.Errorf("complete OAuth flow: %w", outcome.err)
		}
		return outcome.data, nil
	}
}

// Logout revokes the current OAuth session where supported and deletes it locally.
func Logout(ctx context.Context, store oauth.ClientAuthStore, current session.Current) error {
	did, err := syntax.ParseDID(current.DID)
	if err != nil {
		return fmt.Errorf("parse persisted DID: %w", err)
	}
	if err := NewApp(store).Logout(ctx, did, current.SessionID); err != nil {
		return fmt.Errorf("logout ATProto session: %w", err)
	}
	return nil
}

type callbackResult struct {
	data *oauth.ClientSessionData
	err  error
}

func environmentOr(name, fallback string) string {
	if value := os.Getenv(name); value != "" {
		return value
	}
	return fallback
}

func openBrowser(url string) error {
	var command *exec.Cmd
	switch runtime.GOOS {
	case "darwin":
		command = exec.Command("open", url)
	case "windows":
		command = exec.Command("rundll32", "url.dll,FileProtocolHandler", url)
	default:
		command = exec.Command("xdg-open", url)
	}
	return command.Start()
}
