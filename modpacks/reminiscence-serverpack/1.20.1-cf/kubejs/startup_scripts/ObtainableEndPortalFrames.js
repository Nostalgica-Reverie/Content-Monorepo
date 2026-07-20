BlockEvents.modification(event => {

  event.modify('minecraft:end_portal_frame', block => {

    block.requiresTool = true
    block.destroySpeed = 10.0
    
  })

})

ForgeEvents.onEvent('net.minecraftforge.event.entity.player.PlayerEvent$BreakSpeed', event => {

  const { entity, state } = event

  //Only trigger on End Portal Frames
    if (!state.block.id.equals('minecraft:end_portal_frame')) return

    const item = entity.mainHandItem

  //Check if player is holding the correct pickaxe
        if (!item.hasTag('betterend:can_break_end_portal_frames')) {
        event.setNewSpeed(0)
    }

})