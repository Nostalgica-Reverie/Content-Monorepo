package main

import (
	"fmt"
	"strings"
)

var allVerbs = []string{
	"add", "automation", "build", "bump", "completion", "diff", "doctor",
	"export", "freeze", "help", "import", "init", "lint", "loader-update",
	"modlist", "packs", "pages", "port", "publish", "refresh", "side",
	"status", "sync", "test", "unfreeze", "update", "validate", "version",
	"packwiz",
}

func cmdCompletion(args []string) {
	if len(args) < 1 {
		failUsage(verbUsage["completion"])
	}
	switch args[0] {
	case "bash":
		fmt.Print(bashCompletion())
	case "fish":
		fmt.Print(fishCompletion())
	case "zsh":
		fmt.Print(zshCompletion())
	default:
		failUsage(fmt.Sprintf("unknown shell %q — supported: bash, fish, zsh", args[0]))
	}
}

func bashCompletion() string {
	verbs := strings.Join(allVerbs, " ")
	return `# somnus bash completion
# add to ~/.bashrc:  eval "$(somnus completion bash)"
_somnus_completion() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"
    local verbs="` + verbs + `"

    case "$prev" in
        help|h)
            COMPREPLY=($(compgen -W "$verbs" -- "$cur"))
            return ;;
    esac

    COMPREPLY=($(compgen -W "$verbs" -- "$cur"))
}
complete -F _somnus_completion somnus
`
}

func fishCompletion() string {
	verbs := strings.Join(allVerbs, " ")
	return `# somnus fish completion
# source this file or put it in ~/.config/fish/completions/somnus.fish
set -l somnus_verbs ` + verbs + `
complete -c somnus -f
complete -c somnus -n "__fish_use_subcommand" -a "$somnus_verbs"
`
}

func zshCompletion() string {
	var descLines []string
	for verb, u := range verbUsage {
		// grab the first non-usage line as the short description
		desc := ""
		for _, line := range strings.Split(u, "\n") {
			line = strings.TrimSpace(line)
			if line == "" || strings.HasPrefix(line, "usage:") || strings.HasPrefix(line, "e.g.") {
				continue
			}
			desc = strings.ReplaceAll(line, "'", `'\''`)
			break
		}
		descLines = append(descLines, fmt.Sprintf("    '%s:%s'", verb, desc))
	}
	entries := strings.Join(descLines, "\n")
	return `# somnus zsh completion
# add to ~/.zshrc:  eval "$(somnus completion zsh)"
_somnus() {
    local -a cmds
    cmds=(
` + entries + `
    )
    _describe 'somnus commands' cmds
}
compdef _somnus somnus
`
}
