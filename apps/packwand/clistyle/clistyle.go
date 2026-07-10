// Package clistyle holds the shared terminal palette and interactivity
// detection for the packwand CLI, styled after modern agent CLIs
// (claude-code / codex): a Material-purple brand bar, dimmed secondary text,
// and bordered sections — all degrading to plain text off-terminal.
package clistyle

import (
	"os"

	"github.com/charmbracelet/lipgloss"
	"golang.org/x/term"
)

// Material purple with a deep-orange accent, matching the docs sites and GUI.
const (
	Brand      = lipgloss.Color("#8e24aa")
	BrandLight = lipgloss.Color("#ce93d8")
	Accent     = lipgloss.Color("#ff6e42")
	DimGrey    = lipgloss.Color("245")
	ErrorRed   = lipgloss.Color("#ff5c5c")
	OkGreen    = lipgloss.Color("#4ec964")
)

var (
	StatusBar   = lipgloss.NewStyle().Background(Brand).Foreground(lipgloss.Color("#ffffff")).Bold(true).Padding(0, 1)
	StatusMeta  = lipgloss.NewStyle().Foreground(BrandLight).Padding(0, 1)
	HeaderText  = lipgloss.NewStyle().Foreground(BrandLight).Bold(true)
	DimText     = lipgloss.NewStyle().Foreground(DimGrey)
	SuccessText = lipgloss.NewStyle().Foreground(OkGreen)
	WarnText    = lipgloss.NewStyle().Foreground(Accent)
	ErrorText   = lipgloss.NewStyle().Foreground(ErrorRed).Bold(true)
	Box         = lipgloss.NewStyle().Border(lipgloss.RoundedBorder()).BorderForeground(DimGrey).Padding(0, 1)
	BoxTitle    = lipgloss.NewStyle().Foreground(BrandLight).Bold(true)
	SpinnerText = lipgloss.NewStyle().Foreground(Brand)
)

// Interactive reports whether decorative terminal UI should be used: stderr
// must be a terminal and neither NO_COLOR nor PACKWAND_PLAIN may be set.
// Decorative output always goes to stderr so stdout stays machine-readable.
func Interactive() bool {
	if os.Getenv("NO_COLOR") != "" || os.Getenv("PACKWAND_PLAIN") != "" {
		return false
	}
	return term.IsTerminal(int(os.Stderr.Fd()))
}
