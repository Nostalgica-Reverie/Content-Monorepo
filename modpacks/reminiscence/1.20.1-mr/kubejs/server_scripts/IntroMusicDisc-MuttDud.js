// Script for handling the Intro Disc


// Listen for a player getting an event
PlayerEvents.advancement(event => {
    console.info(`Advancement is ${event.advancement}`)

    // bug workaround
    event.server.scheduleInTicks(1, () => {})

    // Check if the advancement is the correct one
    if (event.advancement != "minecraft:end/kill_dragon") return
    console.info("Intro Disc event Triggered")
    console.info(`Player: ${event.player.name}`)
 
    event.server.scheduleInTicks(240, () => {
        event.player.playNotifySound("entity.ender_eye.death", "master", 1, 1),
        event.player.playNotifySound("entity.enderman.teleport", "master", 1, 1),
        event.player.give(Item.of('supplementaries:music_disc_heave_ho', {CustomModelData:8, HideFlags:32, display:{Lore:["\"\\u00a77C418 - Intro\""]}}))
    })
})