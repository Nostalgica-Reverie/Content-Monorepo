package api

import (
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
	"github.com/spf13/cobra"
)

var apiCommand = &cobra.Command{Use: "api", Short: "Run and inspect the Packwand HTTP API"}
var serveCommand = &cobra.Command{
	Use: "serve", Short: "Run the headless Packwand HTTP API",
	RunE: func(command *cobra.Command, _ []string) error {
		bind, _ := command.Flags().GetString("bind")
		tokenFile, _ := command.Flags().GetString("token-file")
		generate, _ := command.Flags().GetBool("generate-token")
		portFile, _ := command.Flags().GetString("print-port-file")
		if !loopbackBind(bind) && tokenFile == "" {
			return errors.New("non-loopback binds require --token-file")
		}
		token, err := loadToken(tokenFile, generate)
		if err != nil {
			return err
		}
		root := workspace.FindRepoRoot()
		if root == "" {
			return errors.New("could not locate repository root")
		}
		server, err := New(root, Options{Token: token})
		if err != nil {
			return err
		}
		listener, err := net.Listen("tcp", bind)
		if err != nil {
			return err
		}
		if portFile != "" {
			if err := writePortFile(portFile, listener.Addr().String()); err != nil {
				_ = listener.Close()
				return err
			}
		}
		fmt.Printf("packwand api running at http://%s%s\n", listener.Addr(), Prefix)
		httpServer := &http.Server{Handler: server.Handler(nil)}
		if err := httpServer.Serve(listener); err != nil && !errors.Is(err, http.ErrServerClosed) {
			return err
		}
		return nil
	},
}

func init() {
	serveCommand.Flags().String("bind", "127.0.0.1:0", "Address to bind")
	serveCommand.Flags().String("token-file", "", "File containing the bearer token")
	serveCommand.Flags().Bool("generate-token", false, "Generate the token file when it does not exist")
	serveCommand.Flags().String("print-port-file", "", "Write the selected server URL to this file")
	apiCommand.AddCommand(serveCommand)
	cmd.AddToGroup(apiCommand, cmd.GroupOther)
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
func writePortFile(path, address string) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	return os.WriteFile(path, []byte("http://"+address+"/\n"), 0o600)
}
