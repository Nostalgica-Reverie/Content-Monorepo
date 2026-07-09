// cursorapi serves Packwand's versioned HTTP API for a repository checkout.
package main

import (
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

	packwandapi "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/api"
)

func main() {
	bind := flag.String("bind", "127.0.0.1:8097", "address to bind")
	root := flag.String("root", ".", "repository root")
	tokenFile := flag.String("token-file", "", "file containing the bearer token")
	generateToken := flag.Bool("generate-token", false, "generate the token file when it does not exist")
	flag.Parse()

	if !loopbackBind(*bind) && *tokenFile == "" {
		fatal(errors.New("non-loopback binds require --token-file"))
	}
	token, err := loadToken(*tokenFile, *generateToken)
	if err != nil {
		fatal(err)
	}
	server, err := packwandapi.New(*root, packwandapi.Options{Token: token})
	if err != nil {
		fatal(err)
	}
	listener, err := net.Listen("tcp", *bind)
	if err != nil {
		fatal(err)
	}
	fmt.Printf("cursorapi running at http://%s%s\n", listener.Addr(), packwandapi.Prefix)
	if err := (&http.Server{Handler: server.Handler(nil)}).Serve(listener); err != nil && !errors.Is(err, http.ErrServerClosed) {
		fatal(err)
	}
}

func loopbackBind(bind string) bool {
	host, _, err := net.SplitHostPort(bind)
	if err != nil {
		return false
	}
	ip := net.ParseIP(host)
	return strings.EqualFold(host, "localhost") || (ip != nil && ip.IsLoopback())
}

func loadToken(path string, generate bool) (string, error) {
	if path == "" {
		return "", nil
	}
	data, err := os.ReadFile(path)
	if err == nil {
		token := strings.TrimSpace(string(data))
		if token == "" {
			return "", errors.New("token file is empty")
		}
		return token, nil
	}
	if !os.IsNotExist(err) || !generate {
		return "", fmt.Errorf("read token file: %w", err)
	}
	var random [32]byte
	if _, err = rand.Read(random[:]); err != nil {
		return "", err
	}
	token := hex.EncodeToString(random[:])
	if err = os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return "", err
	}
	if err = os.WriteFile(path, []byte(token+"\n"), 0o600); err != nil {
		return "", err
	}
	return token, nil
}

func fatal(err error) {
	fmt.Fprintln(os.Stderr, "cursorapi:", err)
	os.Exit(1)
}
