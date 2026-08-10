package cmd

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"flag"
	"fmt"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	identityresolver "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/identity"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/oauthflow"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/xrpcapi"
)

func serve(ctx context.Context, args []string) error {
	flags := flag.NewFlagSet("serve", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	bind := flags.String("bind", "127.0.0.1:0", "loopback bind address")
	tokenFile := flags.String("token-file", "", "bearer token file")
	generateToken := flags.Bool("generate-token", false, "generate a missing token file")
	portFile := flags.String("print-port-file", "", "file that receives the server URL")
	idleTimeout := flags.Duration("idle-timeout", 10*time.Minute, "stop after this period without requests")
	if err := flags.Parse(args); err != nil {
		return err
	}
	if flags.NArg() != 0 {
		return errorsUnexpectedArgs(flags.Args())
	}

	token, err := loadToken(*tokenFile, *generateToken)
	if err != nil {
		return err
	}
	listener, err := net.Listen("tcp", *bind)
	if err != nil {
		return err
	}
	defer listener.Close()
	if !listener.Addr().(*net.TCPAddr).IP.IsLoopback() {
		return errors.New("packwand-social may only bind to a loopback address")
	}

	store, err := session.OpenDefault()
	if err != nil {
		return err
	}
	service := xrpcapi.NewService(oauthflow.NewApp(store), store, identityresolver.New())
	activity := make(chan struct{}, 1)
	handler := xrpcapi.NewHandler(service, token, activity)
	server := &http.Server{
		Handler:           handler,
		ReadHeaderTimeout: 10 * time.Second,
	}
	url := "http://" + listener.Addr().String()
	if *portFile != "" {
		if err := writePrivateFile(*portFile, []byte(url+"\n")); err != nil {
			return err
		}
	}
	fmt.Printf("Packwand social API listening at %s\n", url)

	done := make(chan error, 1)
	go func() {
		done <- server.Serve(listener)
	}()
	if *idleTimeout > 0 {
		go stopWhenIdle(server, activity, *idleTimeout)
	}
	select {
	case <-ctx.Done():
		shutdownContext, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return server.Shutdown(shutdownContext)
	case err := <-done:
		if errors.Is(err, http.ErrServerClosed) {
			return nil
		}
		return err
	}
}

func loadToken(path string, generate bool) (string, error) {
	if path == "" {
		return "", nil
	}
	value, err := os.ReadFile(path)
	if err == nil && strings.TrimSpace(string(value)) != "" {
		return strings.TrimSpace(string(value)), nil
	}
	if !generate {
		if err != nil {
			return "", err
		}
		return "", errors.New("token file is empty; pass --generate-token")
	}
	buffer := make([]byte, 32)
	if _, err := rand.Read(buffer); err != nil {
		return "", fmt.Errorf("generate bearer token: %w", err)
	}
	token := hex.EncodeToString(buffer)
	if err := writePrivateFile(path, []byte(token+"\n")); err != nil {
		return "", err
	}
	return token, nil
}

func writePrivateFile(path string, value []byte) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return err
	}
	if err := os.WriteFile(path, value, 0o600); err != nil {
		return err
	}
	return os.Chmod(path, 0o600)
}

func stopWhenIdle(server *http.Server, activity <-chan struct{}, timeout time.Duration) {
	timer := time.NewTimer(timeout)
	defer timer.Stop()
	for {
		select {
		case <-activity:
			if !timer.Stop() {
				<-timer.C
			}
			timer.Reset(timeout)
		case <-timer.C:
			shutdownContext, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			_ = server.Shutdown(shutdownContext)
			cancel()
			return
		}
	}
}
