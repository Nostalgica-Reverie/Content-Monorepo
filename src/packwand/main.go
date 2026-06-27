package main

import (
	// Modules of packwiz
	"packwand/cmd"
	_ "packwand/curseforge"
	_ "packwand/github"
	_ "packwand/migrate"
	_ "packwand/modrinth"
	_ "packwand/settings"
	_ "packwand/url"
	_ "packwand/utils"
)

func main() {
	cmd.Execute()
}
