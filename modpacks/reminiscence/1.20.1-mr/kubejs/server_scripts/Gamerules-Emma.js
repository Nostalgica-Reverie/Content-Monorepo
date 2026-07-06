ServerEvents.loaded(event => {
    event.server.gameRules.set("doTraderSpawning", false)
    event.server.gameRules.set("doInsomnia", false)
    event.server.gameRules.set("doPatrolSpawning", false)

    event.server.runCommandSilent("scoreboard players set @s trueEnding_settings.dragonhealth 200");
})