package main

import (
	// Modules of packwand
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/build"
	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/cmd"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/content"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/curseforge"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/forgejo"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/github"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/gitlab"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/gui"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/migrate"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/modrinth"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/settings"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/url"
	_ "git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/utils"
)

func main() {
	cmd.Execute()
}
