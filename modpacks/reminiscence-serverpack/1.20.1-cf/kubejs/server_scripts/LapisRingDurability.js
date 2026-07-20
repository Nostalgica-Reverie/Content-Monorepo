EntityEvents.spawned(event => {
  let entity = event.entity;
  let entityTypeName = entity.getType().toString();

  if (entityTypeName !== "minecraft:potion" && entityTypeName !== "minecraft:lingering_potion") return;

  let owner = entity.getOwner();
  if (!owner || !owner.isPlayer()) return;

  const player = owner;
  if (player.level.isClientSide()) return;

  const optionalInventory = CuriosApi.getCuriosInventory(player);
  const inventory = optionalInventory.orElse(null);
  if (!inventory) return;

  const ringHandlerOpt = inventory.getStacksHandler('ring');
  const ringHandler = ringHandlerOpt.orElse(null);
  if (!ringHandler) return;

  const stacks = ringHandler.getStacks();
  const slots = stacks.getSlots();

  for (let i = 0; i < slots; i++) {
    const stack = stacks.getStackInSlot(i);
    if (!stack.isEmpty() && stack.id === 'aether:golden_ring') {
      const damagedStack = stack.copy();
      const damage = damagedStack.getDamageValue() + 1;
      damagedStack.setDamageValue(damage);

      if (damage >= damagedStack.getMaxDamage()) {
        stacks.setStackInSlot(i, ItemStack.EMPTY);
      } else {
        stacks.setStackInSlot(i, damagedStack);
      }
    }
  }
});
