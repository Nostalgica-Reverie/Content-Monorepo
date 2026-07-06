JEIEvents.information((event) => {

    //EYES
    event.addItem("endrem:black_eye", [
      Text.of("Can be found in Buried Treasure or while fishing."),
    ]);
    event.addItem("endrem:cold_eye", [
        Text.of("Can be found in Igloos."),
      ]);
    event.addItem("endrem:corrupted_eye", [
        Text.of("Can be found in any instance of Suspicious Sand or Suspicious Gravel."),
      ]);
    event.addItem("endrem:lost_eye", [
        Text.of("Can be found in Mineshafts."),
      ]);
    event.addItem("endrem:nether_eye", [
        Text.of("Can be found in Nether Fortresses."),
      ]);
    event.addItem("endrem:old_eye", [
        Text.of("Can be found in Desert Pyramids."),
      ]);
    event.addItem("endrem:rogue_eye", [
        Text.of("Can be found in Jungle Temples."),
      ]);
    event.addItem("endrem:cursed_eye", [
        Text.of("Can be found in Bastion Remnants or while Bartering with Pigmen."),
      ]);
    event.addItem("endrem:evil_eye", [
        Text.of("Can be found in Silver Dungeons."),
      ]);
    event.addItem("endrem:guardian_eye", [
        Text.of("Can be obtained by killing an Elder Guardian."),
      ]);
    event.addItem("endrem:magical_eye", [
        Text.of("Can be obtained by killing a Warden."),
      ]);
    event.addItem("endrem:wither_eye", [
        Text.of("Can be obtained by killing the Wither."),
      ]);
    event.addItem("endrem:witch_eye", [
        Text.of("Can be found in Bronze Dungeons."),
      ]);
    event.addItem("endrem:cryptic_eye", [
        Text.of("Can be found in Gold Dungeons."),
      ]);

    //KNIVES

    //BEEKEEPING
    event.addItem("minecraft:campfire", [
        Text.of("When placed under a Beehive, Bees will no longer become angry after the collection of Honey or Honeycomb."),
      ]);
      event.addItem("minecraft:honey_bottle", [
        Text.of("Can be (directly) obtained by using a bottle on a Honey-filled Beehive."),
      ]);
      event.addItem("minecraft:honeycomb", [
        Text.of("Can be (directly) obtained by using shears on a Honey-filled Beehive."),
      ]);
      event.addItem("minecraft:bee_nest", [
        Text.of("Break using Golden tools to properly transport bees in your inventory."),
      ]);
      event.addItem("minecraft:beehive", [
        Text.of("Break using Golden tools to properly transport bees in your inventory."),
      ]);

    //ARCHAEOLOGY
    event.addItem("minecraft:brush", [
        Text.of("Continuously use on Suspicious Sand or Suspicious Gravel to reveal treasure."),
      ]);

    //REDSTONE
    event.addItem(['minecraft:redstone_block', 
      'minecraft:redstone_torch'], [
      Text.of("Constantly emits a redstone signal."),
    ]);

    event.addItem(['quark:ender_watcher', 
      'supplementaries:sconce_lever', 
      'minecraft:target', 
      'supplementaries:crank', 
      'minecraft:lever', 
      'minecraft:sculk_sensor', 
      'minecraft:calibrated_sculk_sensor', 
      'minecraft:tripwire_hook', 
      'minecraft:lectern', 
      'minecraft:daylight_detector', 
      'minecraft:lightning_rod', 
      'supplementaries:wind_vane', 
      'minecraft:chiseled_bookshelf', 
      'minecraft:jukebox', 
      'minecraft:observer', 
      'minecraft:note_block', 
      'supplementaries:speaker_block', 
      'minecraft:detector_rail', 
      'supplementaries:lock_block'], [
      Text.of("Only emits a redstone signal under certain circumstances."),
    ]);

    event.addItem(['supplementaries:turn_table', 
      'supplementaries:spring_launcher', 
      'minecraft:piston', 
      'minecraft:sticky_piston', 
      'minecraft:dispenser', 
      'minecraft:dropper', 
      'supplementaries:crystal_display', 
      'supplementaries:bellows', 
      'minecraft:note_block', 
      'minecraft:powered_rail', 
      'minecraft:activator_rail', 
      '#minecraft:doors', 
      'supplementaries:iron_gate', 
      'minecraft:tnt', 
      'supplementaries:redstone_illuminator', 
      'minecraft:redstone_lamp', 
      'minecraft:bell', 
      'minecraft:big_dripleaf', 
      'quark:crafter', 
      'quark:gravisand', ], [
      Text.of("Performs a specific action whenever given a redstone signal."),
    ]);

    event.addItem(['minecraft:repeater', 'minecraft:comparator', 'quark:redstone_randomizer', 'supplementaries:cog_block'], [
      Text.of("Extends the power of a redstone signal, either directly or under certain circumstances."),
    ]);



    //CRAFTING STATIONS/UTILITY BLOCKS
    event.addItem("minecraft:crafting_table", [
      Text.of("Allows for the crafting of items which require a 3x3 grid."),
    ]);
    event.addItem("minecraft:cartography_table", [
      Text.of("Allows for the streamlined modification of maps."),
    ]);
    event.addItem("minecraft:loom", [
      Text.of("Allows for the streamlined modification of banners."),
    ]);
    event.addItem("minecraft:smithing_table", [
      Text.of("Allows for the modification of equipment using Armor Trims."),
    ]);
    event.addItem("minecraft:lectern", [
      Text.of("Allows for the streamlined editing of Book and Quills. \n\nThis also includes several options which are not present from writing Book and Quills by hand."),
    ]);

    event.addItem("farmersdelight:cutting_board", [
      Text.of("Allows for the cutting of specific items, notably food-related. \n\nRequires a knife."),
    ]);
    event.addItem("farmersdelight:cooking_pot", [
      Text.of("Allows for the crafting of higher-quality food items."),
    ]);

    event.addItem("minecraft:stonecutter", [
      Text.of("Allows for the streamlined crafting of stone-related blocks and items."),
    ]);
    event.addItem("sawmill:sawmill", [
      Text.of("Allows for the streamlined crafting of wood-related blocks and items."),
    ]);

    event.addItem("minecraft:grindstone", [
      Text.of("Allows for the repairing of equipment by combining two damaged items."),
    ]);
    event.addItem("minecraft:anvil", [
      Text.of("Allows for the repairing of equipment, either by combining two damaged items or by using the item's main crafting material."),
    ]);
    event.addItem("aether:altar", [
      Text.of("Allows for the transfiguration of specific items. \n\nAlso allows for the repairing of equipment, requiring fuel."),
    ]);

    event.addItem("minecraft:furnace", [
      Text.of("Allows for item smelting of any kind. \n\nRequires fuel."),
    ]);
    event.addItem("minecraft:blast_furnace", [
      Text.of("A Furnace variant which specializes in smelting Ore-related items at a faster rate than a standard furnace. \n\nSome items may require this to be obtained."),
    ]);
    event.addItem("minecraft:smoker", [
      Text.of("A Furnace variant which specializes in smelting food-related items at a faster rate than a standard furnace."),
    ]);
    event.addItem("clayworks:kiln", [
      Text.of("A Furnace variant which specializes in smelting glass and ceramic-related items at a faster rate than a standard furnace. \n\nSome items may require this to be obtained."),
    ]);
    event.addItem("aether:freezer", [
      Text.of("Allows for the freezing of specific items. \n\nMany items may require this to be obtained."),
    ]);
    event.addItem("aether:incubator", [
      Text.of("Allows for the incubating of Moa Eggs."),
    ]);
    event.addItem("minecraft:cauldron", [
      Text.of("Allows for the storing of liquids. \n\nThe Cauldron also allows for the mixing of Brews."),
    ]);
    event.addItem("minecraft:brewing_stand", [
      Text.of("Allows for the creation of brews."),
    ]);

    event.addItem("minecraft:chest", [
      Text.of("Allows for the storing of items. \n\nCan merge with a Chest beside it to become larger."),
    ]);
    event.addItem("minecraft:ender_chest", [
      Text.of("Allows for the storing of items in an inventory which can only be accessed by the opener."),
    ]);
    event.addItem("supplementaries:safe", [
      Text.of("Allows for the storing of items in a container which holds its contents upon being broken. \n\nThis cannot burn in lava, and can be bound to a personal Key to prevent others from opening it."),
    ]);



    //ORE DISTRIBUTION
    event.addItem([
      'minecraft:iron_block', 
      'minecraft:raw_iron_block', 
      'minecraft:iron_ingot', 
      'minecraft:iron_nugget', 
      'minecraft:raw_iron', 
      'minecraft:iron_ore', 
      'minecraft:deepslate_iron_ore'
    ], [
      Text.of("Iron is an abundant resource which can be found at almost all Y-levels. \n\nIt is most common at Y-level 16."),
    ]);

    event.addItem([
      'minecraft:copper_block', 
      'minecraft:raw_copper_block', 
      'minecraft:copper_ingot', 
      'bettercopper:copper_nugget', 
      'minecraft:raw_copper', 
      'minecraft:copper_ore', 
      'minecraft:deepslate_copper_ore'
    ], [
      Text.of("Copper is an abundant resource which can be found at almost all Y-levels. \n\nIt is most common at Y-level 48."),
    ]);

    event.addItem([
      'caverns_and_chasms:silver_block', 
      'caverns_and_chasms:raw_silver_block', 
      'caverns_and_chasms:silver_ingot', 
      'caverns_and_chasms:silver_nugget', 
      'caverns_and_chasms:raw_silver', 
      'caverns_and_chasms:silver_ore', 
      'caverns_and_chasms:deepslate_silver_ore'
    ], [
      Text.of("Silver is an abundant resource which can be found at almost all Y-levels. \n\nIt is most common at Y-level -16, and spawns incredibly commonly in cold biomes."),
    ]);

    event.addItem([
      'minecraft:coal_block', 
      'minecraft:coal', 
      'minecraft:coal_ore', 
      'minecraft:deepslate_coal_ore'
    ], [
      Text.of("Coal is an abundant resource which can be found at most Y-levels above Y-level 0. \n\nIt is most common at Y-level 136, and does not generate below Y-level 0."),
    ]);

    event.addItem([
      'minecraft:gold_block', 
      'minecraft:raw_gold_block', 
      'minecraft:gold_ingot', 
      'minecraft:gold_nugget', 
      'minecraft:raw_gold', 
      'minecraft:gold_ore', 
      'minecraft:deepslate_gold_ore'
    ], [
      Text.of("Gold is an uncommon resource which can be found at many Y-levels below Y-level 32. \n\nIt is most common at Y-level -16, and spawns incredibly commonly in Badlands biomes."),
    ]);

    event.addItem([
      'minecraft:redstone_block', 
      'minecraft:redstone', 
      'minecraft:redstone_ore', 
      'minecraft:deepslate_redstone_ore'
    ], [
      Text.of("Redstone is an uncommon resource which can be found at most Y-levels below Y-level 15. \n\nIt is most common at Y-level -59."),
    ]);

    event.addItem([
      'minecraft:lapis_block', 
      'minecraft:lapis_lazuli', 
      'minecraft:lapis_ore', 
      'minecraft:deepslate_lapis_ore'
    ], [
      Text.of("Lapis Lazuli is an uncommon resource which can be found at most Y-levels below Y-level 64. \n\nIt is most common at Y-level -1."),
    ]);

    event.addItem([
      'minecraft:diamond_block', 
      'minecraft:diamond', 
      'minecraft:diamond_ore', 
      'minecraft:deepslate_diamond_ore'
    ], [
      Text.of("Diamond is a very rare resource which can be found at most Y-levels below Y-level 14. \n\nIt is most common at Y-level -59."),
    ]);

    event.addItem([
      'minecraft:netherite_block', 
      'minecraft:netherite_ingot', 
      'caverns_and_chasms:netherite_nugget', 
      'minecraft:netherite_scrap', 
      'minecraft:ancient_debris'
    ], [
      Text.of("Tungsten is a very rare resource which can be found at most Y-levels below Y-level 28 in the Nether. \n\nIt is most common at Y-level 15."),
    ]);

    event.addItem([
      'minecraft:quartz_block', 
      'minecraft:quartz', 
      'minecraft:nether_quartz_ore'
    ], [
      Text.of("Quartz is a very common resource which can be found anywhere between Y-level 10 and Y-level 117 in the Nether. \n\nIt is equally common at all Y-levels."),
    ]);


    event.addItem([
      'aether:zanite_block', 
      'aether:zanite', 
      'aether:zanite_ore'
    ], [
      Text.of("Tanzanite is an uncommon resource which can be found anywhere between Y-level 0 and Y-level 75 in the Aether. \n\nIt is equally common at all Y-levels."),
    ]);

    event.addItem([
      'aether:enchanted_gravitite',
      'aether:gravitite_ore'
    ], [
      Text.of("Gravitite is a rare resource which can be found anywhere between Y-level -58 and Y-level 74 in the Aether. \n\nIt is most common at Y-level 8."),
    ]);

    event.addItem([
      'minecraft:emerald_block',
      'minecraft:emerald', 
      'deep_aether:skyjade_ore',
      'minecraft:emerald_ore',
      'minecraft:deepslate_emerald_ore'
    ], [
      Text.of("Emerald is either an uncommon or rare resource depending on its location. \n\nIn the overworld, it can be can be found anywhere between Y-level -16 and Y-level 320, with the best level being Y-level 100. \n\nIn the Aether, it can be found between Y-level 0 and Y-level 60, with it being equally common on all Y-levels. \n\nDespite this, the easiest method of obtaining Emerald is through loot chests."),
    ]);


    //MOBS

    
    //FOOD
    event.addItem("#minecraft:lightweight", [
        Text.of("Lightweight foods are stackable to 8, but do not heal a lot. \n\nThese foods are more likely than not ingredients for larger meals."),
      ]);
      event.addItem("#minecraft:sweet", [
        Text.of("Sweet foods have no specific restrictions."),
      ]);
      event.addItem("#minecraft:hearty", [
        Text.of("Hearty foods stack to very small numbers (ranging from 1-4) but heal a good portion of health. \n\nThough these foods are viable on their own, they are also more likely than not ingredients for larger meals."),
      ]);
      event.addItem("#minecraft:fingerfood", [
        Text.of("Finger Foods are stackable to 6 and heal almost as much as Hearty foods."),
      ]);
      event.addItem("#minecraft:meal", [
        Text.of("Meals are stackable to 4 and heal double the health that Finger Foods do."),
      ]);
      event.addItem("#minecraft:feast", [
        Text.of("Feasts are required to be placed, able to be eaten only in portions with a bowl. \n\nThese portions stack to 2, but consistently heal the most out of any category."),
      ]);
      event.addItem("#minecraft:special", [
        Text.of("Special foods can give niche but beneficial effects."),
      ]);
      event.addItem("#minecraft:gross", [
        Text.of("Gross foods are an active detriment to be consumed."),
      ]);

      //GOAWAY
      event.addItem("#minecraft:goaway", [
        Text.of("This item is meant to be removed. \n\nI very highly recommend you throw this out and report it as a bug if you manage to come across this in a survival setting. \n\nBut hey, I can't control your playthrough."),
      ]);
    
  });