package cmd

import (
	"flag"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwandrs/packwand-social/internal/session"
)

func whoami(args []string) error {
	flags := flag.NewFlagSet("whoami", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	if err := flags.Parse(args); err != nil {
		return err
	}
	if flags.NArg() != 0 {
		return errorsUnexpectedArgs(flags.Args())
	}

	store, err := session.OpenDefault()
	if err != nil {
		return err
	}
	current, ok := store.Current()
	if !ok {
		return errNotSignedIn
	}
	return writeJSON(os.Stdout, current.Identity())
}
