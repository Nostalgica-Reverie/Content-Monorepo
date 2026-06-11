package main

import (
	"fmt"
	"os"
)

const (
	ansiBlue  = "\033[38;5;39m"
	ansiDim   = "\033[38;5;245m"
	ansiReset = "\033[0m"
)

func mascot() string {
	b := ansiBlue
	d := ansiDim
	r := ansiReset
	return "" +
		"                          " + d + "z" + r + "\n" +
		"      " + b + "▄▄▄▄▄▄▄▄▄▄▄▄" + r + "        " + d + "z z" + r + "\n" +
		"      " + b + "██ ──  ── ██" + r + "\n" +
		"  " + b + "▄█▄ ████████████ ▄█▄" + r + "\n" +
		"  " + b + "▀▀  ████████████  ▀▀" + r + "\n" +
		"      " + b + "▀█▀ ▀█▀▀█▀ ▀█▀" + r + "    " + d + "mimimi..." + r + "\n"
}

func printMascot() {
	if !isTTY() {
		return
	}
	fmt.Fprint(os.Stdout, mascot())
}
