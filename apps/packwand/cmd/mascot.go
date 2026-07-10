package cmd

import (
	"fmt"
	"os"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/clistyle"
)

func mascot() string {
	b, d, r := "", "", ""
	return "" +
		"                          " + d + "z" + r + "\n" +
		"      " + b + "▄▄▄▄▄▄▄▄▄▄▄▄" + r + "        " + d + "z z" + r + "\n" +
		"      " + b + "██ ──  ── ██" + r + "\n" +
		"  " + b + "▄█▄ ████████████ ▄█▄" + r + "\n" +
		"  " + b + "▀▀  ████████████  ▀▀" + r + "\n" +
		"      " + b + "▀█▀ ▀█▀▀█▀ ▀█▀" + r + "    " + d + "mimimi..." + r + "\n"
}

func printMascot() {
	if !clistyle.Interactive() {
		return
	}
	fmt.Fprint(os.Stderr, clistyle.SpinnerText.Render(mascot()))
}
