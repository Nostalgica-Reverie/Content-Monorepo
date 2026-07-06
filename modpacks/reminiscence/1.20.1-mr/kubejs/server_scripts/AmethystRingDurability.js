const CuriosApi = Java.loadClass('top.theillusivec4.curios.api.CuriosApi')
const ItemStack = Java.loadClass('net.minecraft.world.item.ItemStack')

BlockEvents.broken(event => {
  const player = event.player
  if (!player || player.level.isClientSide()) return

  if (player.isCreative()) return

  const optionalInventory = CuriosApi.getCuriosInventory(player)
  const inventory = optionalInventory.orElse(null)
  if (!inventory) {
    return
  }

  const ringHandlerOpt = inventory.getStacksHandler('ring')
  const ringHandler = ringHandlerOpt.orElse(null)
  if (!ringHandler) {
    return
  }

  const stacks = ringHandler.getStacks()
  const slots = stacks.getSlots()

  for (let i = 0; i < slots; i++) {
    const stack = stacks.getStackInSlot(i)
    if (!stack.isEmpty() && stack.id === 'aether:iron_ring') {
      const damagedStack = stack.copy()
      const damage = damagedStack.getDamageValue() + 1
      damagedStack.setDamageValue(damage)

      if (damage >= damagedStack.getMaxDamage()) {
        stacks.setStackInSlot(i, ItemStack.EMPTY)
      } else {
        stacks.setStackInSlot(i, damagedStack)
      }
    }
  }
})