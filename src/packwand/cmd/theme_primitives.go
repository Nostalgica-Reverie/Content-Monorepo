package cmd

import (
	"bytes"
	"fmt"
	"os"
	"strings"
	"sync"
	"text/tabwriter"

	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/clistyle"
	"github.com/charmbracelet/lipgloss"
)

// Failure and Warn apply the shared status colors without changing text in
// plain mode.
func Failure(s string) string {
	if Interactive() {
		return clistyle.ErrorText.Render(s)
	}
	return s
}
func WarningStyle(s string) string {
	if Interactive() {
		return clistyle.WarnText.Render(s)
	}
	return s
}

// Table renders a compact table. Its plain form intentionally uses only the
// standard library and contains no ANSI escapes.
func Table(headers []string, rows [][]string) string {
	if !Interactive() {
		return plainTable(headers, rows)
	}
	widths := make([]int, len(headers))
	for i, header := range headers {
		widths[i] = lipgloss.Width(header)
	}
	for _, row := range rows {
		for i, cell := range row {
			if i < len(widths) && lipgloss.Width(cell) > widths[i] {
				widths[i] = lipgloss.Width(cell)
			}
		}
	}
	render := func(row []string, header bool) string {
		cells := make([]string, len(widths))
		for i, width := range widths {
			value := ""
			if i < len(row) {
				value = row[i]
			}
			style := lipgloss.NewStyle().Width(width)
			if header {
				style = style.Inherit(clistyle.HeaderText)
			} else if i > 0 {
				style = style.Inherit(clistyle.DimText)
			}
			cells[i] = style.Render(value)
		}
		return strings.Join(cells, "  ")
	}
	lines := []string{render(headers, true)}
	for _, row := range rows {
		lines = append(lines, render(row, false))
	}
	return clistyle.Box.Render(strings.Join(lines, "\n"))
}

func plainTable(headers []string, rows [][]string) string {
	var buffer bytes.Buffer
	writer := tabwriter.NewWriter(&buffer, 0, 4, 2, ' ', 0)
	write := func(row []string) { _, _ = fmt.Fprintln(writer, strings.Join(row, "\t")) }
	write(headers)
	for _, row := range rows {
		write(row)
	}
	_ = writer.Flush()
	return strings.TrimSuffix(buffer.String(), "\n")
}

type ProgressReporter struct {
	label     string
	total     int
	current   int
	lastPlain int
	mu        sync.Mutex
}

// Progress creates a bounded progress reporter. Interactive updates are drawn
// on stderr; plain mode emits stable periodic N/total lines.
func Progress(label string, total int) *ProgressReporter {
	return &ProgressReporter{label: label, total: total}
}
func (p *ProgressReporter) Advance(n int, status string) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.current += n
	if p.current > p.total {
		p.current = p.total
	}
	if Interactive() {
		width := 24
		filled := 0
		if p.total > 0 {
			filled = p.current * width / p.total
		}
		bar := strings.Repeat("━", filled) + strings.Repeat("─", width-filled)
		fmt.Fprintf(os.Stderr, "\r%s %s %d/%d %s", clistyle.SpinnerText.Render(p.label), bar, p.current, p.total, status)
		return
	}
	if p.current == p.total || p.current-p.lastPlain >= max(1, p.total/10) {
		fmt.Fprintf(os.Stderr, "%s: %d/%d %s\n", p.label, p.current, p.total, status)
		p.lastPlain = p.current
	}
}
func (p *ProgressReporter) Done() {
	p.mu.Lock()
	defer p.mu.Unlock()
	if Interactive() {
		fmt.Fprintln(os.Stderr)
	}
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
