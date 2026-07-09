EntityEvents.spawned(event => {

    // No Vex swords
    if (event.entity.type.toString() == 'minecraft:vex') {
        event.entity.setItemSlot(0, 'minecraft:air')
    }

    // No Evoker armor
    if (event.entity.type.toString() == 'minecraft:evoker') {
        event.entity.setItemSlot(5, 'minecraft:air')
    }
    })