PlayerEvents.tick(event => {
  const player = event.player
  if (!player || player.isFake()) return

  const armorInv = player.inventory

  const boots = armorInv.getArmor(0)
  const legs = armorInv.getArmor(1)
  const chest = armorInv.getArmor(2)
  const helmet = armorInv.getArmor(3)

  if (helmet?.id === 'caverns_and_chasms:silver_helmet') {
    player.removeEffect('minecraft:blindness')
    player.removeEffect('minecraft:darkness')
    player.removeEffect('minecraft:nausea')
  }

  if (chest?.id === 'caverns_and_chasms:silver_chestplate') {
    player.removeEffect('minecraft:mining_fatigue')
    player.removeEffect('minecraft:poison')
    player.removeEffect('aether:inebriation')
  }

  if (legs?.id === 'caverns_and_chasms:silver_leggings') {
    player.removeEffect('minecraft:wither')
    player.removeEffect('minecraft:weakness')
  }

  if (boots?.id === 'caverns_and_chasms:silver_boots') {
    player.removeEffect('minecraft:slowness')
    player.removeEffect('minecraft:levitation')
  }
})