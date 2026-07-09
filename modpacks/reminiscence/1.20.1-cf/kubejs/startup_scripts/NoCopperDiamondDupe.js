ForgeEvents.onEvent('net.minecraftforge.event.entity.player.PlayerEvent$BreakSpeed', event => {
  const { entity, state } = event

  //Only affect diamond ore and deepslate diamond ore
  const blockId = state.block.id
  if (!blockId.equals('minecraft:diamond_ore') && !blockId.equals('minecraft:deepslate_diamond_ore') && !blockId.equals('aether:gravitite_ore')) return

  const item = entity.mainHandItem

  if (item.id.equals('bettercopper:copper_pickaxe')) {

    event.setNewSpeed(1.0)

  }
})