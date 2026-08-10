package cmd

import (
	"bufio"
	"context"
	"flag"
	"fmt"
	"os"
	"strings"

	identityresolver "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/identity"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/oauthflow"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
)

func login(ctx context.Context, args []string) error {
	flags := flag.NewFlagSet("login", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	identifier := flags.String("identifier", "", "ATProto handle or DID")
	if err := flags.Parse(args); err != nil {
		return err
	}
	if flags.NArg() != 0 {
		return errorsUnexpectedArgs(flags.Args())
	}

	value := strings.TrimSpace(*identifier)
	if value == "" {
		fmt.Fprint(os.Stderr, "ATProto handle or DID: ")
		line, err := bufio.NewReader(os.Stdin).ReadString('\n')
		if err != nil {
			return fmt.Errorf("read identifier: %w", err)
		}
		value = strings.TrimSpace(line)
	}
	if value == "" {
		return errorsEmptyIdentifier()
	}

	store, err := session.OpenDefault()
	if err != nil {
		return err
	}
	data, err := oauthflow.Login(ctx, store, value, os.Stderr)
	if err != nil {
		return err
	}
	resolved, err := identityresolver.New().ResolveDID(ctx, data.AccountDID)
	if err != nil {
		return fmt.Errorf("resolve signed-in identity: %w", err)
	}
	if err := store.SetCurrent(session.Current{
		DID:       resolved.DID,
		Handle:    resolved.Handle,
		PDS:       resolved.PDS,
		SessionID: data.SessionID,
	}); err != nil {
		return err
	}
	return writeJSON(os.Stdout, resolved)
}

func errorsUnexpectedArgs(args []string) error {
	return fmt.Errorf("unexpected arguments: %s", strings.Join(args, " "))
}

func errorsEmptyIdentifier() error {
	return fmt.Errorf("ATProto handle or DID is required")
}
