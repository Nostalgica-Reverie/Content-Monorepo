package gui

import (
	"embed"
	"errors"
	"fmt"
	"io/fs"
	"net"
	"net/http"
	"os"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/api"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
	"github.com/skratchdot/open-golang/open"
	"github.com/spf13/cobra"
)

//go:embed static/*
var staticFiles embed.FS

type server struct {
	root string
}

var guiCmd = &cobra.Command{
	Use:   "gui",
	Short: "Run the local Packwand web GUI",
	Run: func(c *cobra.Command, args []string) {
		port, _ := c.Flags().GetInt("port")
		noOpen, _ := c.Flags().GetBool("no-open")
		portFile, _ := c.Flags().GetString("print-port-file")
		root := workspace.FindRepoRoot()
		if root == "" {
			cmd.Fail("could not locate repo root (no .git or modpacks/ found walking up from here)")
		}
		if err := os.Chdir(root); err != nil {
			cmd.Fail(fmt.Sprintf("failed to enter repo root %s: %v", root, err))
		}
		listener, err := net.Listen("tcp", listenAddr(port))
		if err != nil {
			cmd.Fail(fmt.Sprintf("failed to bind GUI server: %v", err))
		}
		addr := listener.Addr().String()
		srv := &server{root: root}
		httpSrv := &http.Server{Handler: srv.routes()}
		url := "http://" + addr + "/"
		if portFile != "" {
			if err := os.WriteFile(portFile, []byte(url+"\n"), 0o600); err != nil {
				cmd.Fail(fmt.Sprintf("failed to write port file: %v", err))
			}
		}
		fmt.Printf("packwand gui running at %s\n", url)
		if !noOpen {
			go func() {
				time.Sleep(200 * time.Millisecond)
				_ = open.Run(url)
			}()
		}
		if err := httpSrv.Serve(listener); err != nil && !errors.Is(err, http.ErrServerClosed) {
			cmd.Fail(fmt.Sprintf("gui server failed: %v", err))
		}
	},
}

func init() {
	guiCmd.Flags().IntP("port", "p", 0, "Port to bind; 0 chooses a free local port")
	guiCmd.Flags().Bool("no-open", false, "Do not open the browser automatically")
	guiCmd.Flags().String("print-port-file", "", "Write the selected server URL to this file")
	cmd.AddToGroup(guiCmd, cmd.GroupOther)
}

func listenAddr(port int) string {
	if port > 0 {
		return fmt.Sprintf("127.0.0.1:%d", port)
	}
	return "127.0.0.1:0"
}

func (s *server) routes() http.Handler {
	apiServer, err := api.New(s.root, api.Options{})
	if err != nil {
		panic(err)
	}
	static, err := fs.Sub(staticFiles, "static")
	if err != nil {
		panic(err)
	}
	mux := http.NewServeMux()
	mux.Handle("/", http.FileServer(http.FS(static)))
	return apiServer.Handler(mux)
}
