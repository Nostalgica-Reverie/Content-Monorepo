package main

import (
	// Modules of packwiz
	"git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/cmd"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/curseforge"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/forgejo"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/github"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/migrate"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/modrinth"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/settings"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/url"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/src/packwand/utils"
)

func main() {
	cmd.Execute()
}
