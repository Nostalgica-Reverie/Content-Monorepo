package cmd

import (
	"fmt"
	"os"
	"strings"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/clistyle"
)

// claude-code-style presentation helpers for commands: a branded status bar,
// bordered sections, and dimmed secondary text. Everything degrades to plain
// text when not on a terminal (CI, pipes) or when NO_COLOR / PACKWAND_PLAIN
// is set. Decorative output is written to stderr so stdout stays clean for
// machine-readable output (list --json, docs modlist, etc.).

// Interactive reports whether decorative terminal UI should be used.
func Interactive() bool { return clistyle.Interactive() }

// StatusBar prints the branded top bar to stderr. No-op when non-interactive.
func StatusBar(context string) {
	if !Interactive() {
		return
	}
	bar := clistyle.StatusBar.Render("packwand " + packwandVersion)
	if context != "" {
		bar += clistyle.StatusMeta.Render(context)
	}
	fmt.Fprintln(os.Stderr, bar)
}

// Header prints a styled section header, or a plain one when non-interactive.
func Header(title string) {
	if Interactive() {
		fmt.Fprintln(os.Stderr, clistyle.HeaderText.Render("― "+title))
		return
	}
	fmt.Fprintln(os.Stderr, "-- "+title)
}

// Dim returns s styled as secondary information.
func Dim(s string) string {
	if Interactive() {
		return clistyle.DimText.Render(s)
	}
	return s
}

// Success returns s styled as a success message.
func Success(s string) string {
	if Interactive() {
		return clistyle.SuccessText.Render(s)
	}
	return s
}

// Boxed prints a claude-code-style bordered section: a bold title and body
// lines. Falls back to indented plain text off-terminal.
func Boxed(title string, lines []string) {
	if !Interactive() {
		fmt.Fprintln(os.Stderr, title+":")
		for _, l := range lines {
			fmt.Fprintln(os.Stderr, "  "+l)
		}
		return
	}
	body := clistyle.BoxTitle.Render(title)
	if len(lines) > 0 {
		body += "\n" + strings.Join(lines, "\n")
	}
	fmt.Fprintln(os.Stderr, clistyle.Box.Render(body))
}
