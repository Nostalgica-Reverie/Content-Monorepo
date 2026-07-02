package cmdshared

import (
	"fmt"
	"os"

	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/clistyle"
)

// WithSpinner runs work while showing an animated spinner with a live status
// line (Bubble Tea), claude-code style. The work function receives an update
// callback for progress messages. In non-interactive environments (CI, pipes,
// NO_COLOR/PACKWAND_PLAIN) it falls back to plain text lines automatically.
func WithSpinner(label string, work func(update func(string)) error) error {
	if !clistyle.Interactive() {
		fmt.Fprintf(os.Stderr, "%s...\n", label)
		err := work(func(msg string) {
			fmt.Fprintf(os.Stderr, "  %s\n", msg)
		})
		if err == nil {
			fmt.Fprintf(os.Stderr, "%s: done\n", label)
		}
		return err
	}

	model := spinnerModel{
		label:   label,
		spinner: spinner.New(spinner.WithSpinner(spinner.Dot), spinner.WithStyle(clistyle.SpinnerText)),
	}
	program := tea.NewProgram(model, tea.WithOutput(os.Stderr))

	done := make(chan error, 1)
	go func() {
		err := work(func(msg string) {
			program.Send(statusMsg(msg))
		})
		done <- err
		program.Send(finishedMsg{err: err})
	}()

	if _, err := program.Run(); err != nil {
		// Terminal UI failure: keep the work's result authoritative.
		return <-done
	}
	return <-done
}

type statusMsg string

type finishedMsg struct{ err error }

type spinnerModel struct {
	label   string
	status  string
	err     error
	done    bool
	spinner spinner.Model
}

func (m spinnerModel) Init() tea.Cmd {
	return m.spinner.Tick
}

func (m spinnerModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case statusMsg:
		m.status = string(msg)
		return m, nil
	case finishedMsg:
		m.done = true
		m.err = msg.err
		return m, tea.Quit
	case tea.KeyMsg:
		if msg.String() == "ctrl+c" {
			return m, tea.Quit
		}
		return m, nil
	default:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd
	}
}

func (m spinnerModel) View() string {
	if m.done {
		if m.err != nil {
			return clistyle.ErrorText.Render("✗ ") + m.label + "\n"
		}
		return clistyle.SuccessText.Render("✓ ") + m.label + "\n"
	}
	line := m.spinner.View() + m.label
	if m.status != "" {
		line += clistyle.DimText.Render("  " + m.status)
	}
	return line + "\n"
}
