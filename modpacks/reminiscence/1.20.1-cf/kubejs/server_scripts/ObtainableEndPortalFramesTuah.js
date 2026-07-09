BlockEvents.broken(event => {

  if (event.block.id === 'minecraft:end_portal_frame') {

    event.server.scheduleInTicks(1, () => {

      event.level.runCommandSilent(`summon item ${event.block.x + 0.5} ${event.block.y + 0.7} ${event.block.z + 0.5} {Item:{id:"minecraft:end_portal_frame",Count:1b}}`)

    })
  }
})