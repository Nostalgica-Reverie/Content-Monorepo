ServerEvents.recipes(event => {

    event.replaceInput(
        { input: 'aether:skyroot_stick' }, // Arg 1: the filter
        'aether:skyroot_stick',            // Arg 2: the item to replace
        'minecraft:stick'         // Arg 3: the item to replace it with
        // Note: tagged fluid ingredients do not work on Fabric, but tagged items do.
      )

      event.replaceInput(
        { input: 'caverns_and_chasms:spinel' }, // Arg 1: the filter
        'caverns_and_chasms:spinel',            // Arg 2: the item to replace
        'minecraft:lapis_lazuli'         // Arg 3: the item to replace it with
        // Note: tagged fluid ingredients do not work on Fabric, but tagged items do.
      )

    event.remove({ input: '#minecraft:goaway' })

})