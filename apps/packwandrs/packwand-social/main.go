package main

import (
	"context"
	"fmt"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/cmd"
)

func main() {
	if err := cmd.Run(context.Background(), os.Args[1:]); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}
