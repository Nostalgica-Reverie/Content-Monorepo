package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync/atomic"
)

func cmdPages(args []string) {
	var packArg string
	if len(args) > 0 {
		packArg = args[0]
	}

	var subdirs []string
	if packArg != "" {
		subdirs = packModSubdirs(packArg)
		if len(subdirs) == 0 {
			fail(fmt.Sprintf("no mod subdirs found under %s", packArg))
		}
	} else {
		root := modpacksDir()
		packs, err := os.ReadDir(root)
		if err != nil {
			fail(fmt.Sprintf("failed to read %s: %v", root, err))
		}
		for _, p := range packs {
			if p.IsDir() {
				subdirs = append(subdirs, packModSubdirs(filepath.Join(root, p.Name()))...)
			}
		}
		if len(subdirs) == 0 {
			fail("no mod subdirs found in any pack")
		}
	}

	var written int64
	sched := NewScheduler(maxConcurrent())
	dones := make([]<-chan error, len(subdirs))
	for i, sub := range subdirs {
		dones[i] = sched.Submit(Task{
			Name:  sub,
			Needs: []Resource{Resource("pages:" + sub)},
			Run: func() error {
				n, err := writeModlistMD(sub)
				if err != nil {
					warnf("%s: %v", sub, err)
					return nil
				}
				fmt.Printf("wrote %s/modlist.md (%d mods)\n", sub, n)
				atomic.AddInt64(&written, 1)
				return nil
			},
		})
	}
	sched.Close()
	for _, c := range dones {
		<-c
	}
	fmt.Printf("generated %d modlist.md file(s).\n", written)

	if packArg == "" {
		if _, err := writeProjectsIndex(); err != nil {
			warnf("projects index not written: %v", err)
		}
	}
}

func packModSubdirs(packDir string) []string {
	var out []string
	entries, err := os.ReadDir(packDir)
	if err != nil {
		return nil
	}
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		if _, err := os.Stat(filepath.Join(packDir, e.Name(), "mods")); err == nil {
			out = append(out, filepath.Join(packDir, e.Name()))
		}
	}
	return out
}

func writeModlistMD(subdir string) (int, error) {
	modsDir := filepath.Join(subdir, "mods")
	entries, err := os.ReadDir(modsDir)
	if err != nil {
		return 0, err
	}

	var client, shared, server []string
	count := 0
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".pw.toml") {
			continue
		}
		mod, err := parsePwToml(filepath.Join(modsDir, e.Name()))
		if err != nil {
			continue
		}
		count++
		line := fmt.Sprintf("- [%s](%s)", mod.name, modPageURL(mod))
		switch mod.side {
		case "client":
			client = append(client, line)
		case "server":
			server = append(server, line)
		default:
			shared = append(shared, line)
		}
	}

	var b strings.Builder
	b.WriteString("# Modlist\n")
	writeSection(&b, "Client Mods", client)
	writeSection(&b, "Shared Mods", shared)
	writeSection(&b, "Server Mods", server)

	out := filepath.Join(subdir, "modlist.md")
	if err := os.WriteFile(out, []byte(b.String()), 0o644); err != nil {
		return 0, err
	}
	return count, nil
}

func writeSection(b *strings.Builder, title string, lines []string) {
	if len(lines) == 0 {
		return
	}
	sort.Strings(lines)
	fmt.Fprintf(b, "\n## %s\n\n", title)
	for _, l := range lines {
		b.WriteString(l)
		b.WriteByte('\n')
	}
}

func modPageURL(m *pwMod) string {
	if m.mrModID != "" {
		return "https://modrinth.com/mod/" + m.mrModID
	}
	if m.cfFileID != nil && m.url != "" {
		return m.url
	}
	if m.url != "" {
		return m.url
	}
	return ""
}
