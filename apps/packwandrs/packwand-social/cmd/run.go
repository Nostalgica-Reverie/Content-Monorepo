package cmd

import (
	"context"
	"errors"
	"fmt"
	"io"
)

var errNotSignedIn = errors.New("not signed in")

// Run dispatches the small one-shot commands and the long-lived local API.
func Run(ctx context.Context, args []string) error {
	if len(args) == 0 {
		return usageError()
	}

	switch args[0] {
	case "login":
		return login(ctx, args[1:])
	case "logout":
		return logout(ctx, args[1:])
	case "serve":
		return serve(ctx, args[1:])
	case "whoami":
		return whoami(args[1:])
	default:
		return fmt.Errorf("unknown command %q", args[0])
	}
}

func usageError() error {
	return errors.New("usage: packwand-social <login|logout|whoami|serve>")
}

func writeJSON(writer io.Writer, value any) error {
	return jsonEncoder(writer).Encode(value)
}
