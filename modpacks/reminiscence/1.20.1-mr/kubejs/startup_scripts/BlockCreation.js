//Create individual blocks
function createBlock(event, id, name, hardness, resistance, sound, mapColor, tool, requiresTool) {
  if (typeof requiresTool === 'undefined') requiresTool = true;

  var b = event.create(id)
    .displayName(name)
    .hardness(hardness)
    .resistance(resistance)
    .soundType(sound)
    .mapColor(mapColor)
    .requiresTool(requiresTool);

  if (tool) b.tagBlock('minecraft:mineable/' + tool);
  return b;
}

//Create individual slabs
function createSlab(event, id, name, hardness, resistance, sound, mapColor, tool, requiresTool) {
  if (typeof requiresTool === 'undefined') requiresTool = true;

  var b = event.create(id, 'slab')
    .displayName(name)
    .hardness(hardness)
    .resistance(resistance)
    .soundType(sound)
    .mapColor(mapColor)
    .requiresTool(requiresTool);

  if (tool) b.tagBlock('minecraft:mineable/' + tool);
  return b;
}

//Create entire families of blocks, including stairs, walls, and slabs
function createBlockFamily(event, baseId, baseName, hardness, resistance, sound, mapColor, tool, requiresTool) {
  if (typeof requiresTool === 'undefined') requiresTool = true;

  //Base block
  createBlock(event, baseId, baseName, hardness, resistance, sound, mapColor, tool, requiresTool);

  //Fix plural forms for variants
  function variantName(baseName, suffix) {
    if (baseName.endsWith("Bricks")) {
      return baseName.slice(0, -1) + " " + suffix; //"Bricks" -> "Brick"
    }
    if (baseName.endsWith("Tiles")) {
      return baseName.slice(0, -1) + " " + suffix; //"Tiles" -> "Tile"
    }
    return baseName + " " + suffix; //default
  }

  //Variants
  event.create(baseId + '_slab', 'slab')
    .displayName(variantName(baseName, 'Slab'))
    .hardness(hardness)
    .resistance(resistance)
    .soundType(sound)
    .mapColor(mapColor)
    .requiresTool(requiresTool)
    .tagBlock('minecraft:mineable/' + tool);

  event.create(baseId + '_stairs', 'stairs')
    .displayName(variantName(baseName, 'Stairs'))
    .hardness(hardness)
    .resistance(resistance)
    .soundType(sound)
    .mapColor(mapColor)
    .requiresTool(requiresTool)
    .tagBlock('minecraft:mineable/' + tool);

  event.create(baseId + '_wall', 'wall')
    .displayName(variantName(baseName, 'Wall'))
    .hardness(hardness)
    .resistance(resistance)
    .soundType(sound)
    .mapColor(mapColor)
    .requiresTool(requiresTool)
    .tagBlock('minecraft:mineable/' + tool);
}

//Putting the helpers to use
StartupEvents.registry("block", event => {
  
  //Simple single blocks
  createBlock(event, 'planked_stone', 'Planked Stone', 2, 5.0, 'stone', 'stone', 'pickaxe', false)
  createBlock(event, 'glowcelium', 'Glowcelium', 0.5, 0.5, 'gravel', 'gravel', 'shovel', false).lightLevel(0.4)
  createBlock(event, 'nether_wart_block', 'Nether Wart Block', 1, 1, 'stone', 'stone', 'hoe', false)

  //Block families
  createBlockFamily(event, 'fieldstone', 'Fieldstone', 3, 24.0, 'stone', 'stone', 'pickaxe')
  createBlockFamily(event, 'fired_fieldstone', 'Fired Fieldstone', 3, 24.0, 'stone', 'stone', 'pickaxe')
  createBlockFamily(event, 'fieldstone_bricks', 'Fieldstone Bricks', 3, 24.0, 'stone', 'stone', 'pickaxe')

  createBlockFamily(event, 'fired_cobblestone', 'Fired Cobblestone', 3, 12.0, 'stone', 'stone', 'pickaxe')
  createBlockFamily(event, 'mossy_fired_cobblestone', 'Mossy Fired Cobblestone', 3, 12.0, 'stone', 'stone', 'pickaxe')
  createBlockFamily(event, 'fired_bricks', 'Fired Bricks', 3, 12.0, 'stone', 'stone', 'pickaxe')
  createBlockFamily(event, 'enchanted_bricks', 'Enchanted Bricks', 3, 20.0, 'stone', 'stone', 'pickaxe')
  
  createBlockFamily(event, 'fired_cobbled_deepslate', 'Fired Cobbled Deepslate', 3.5, 6.0, 'deepslate', 'deepslate', 'pickaxe')
  createBlockFamily(event, 'mossy_fired_cobbled_deepslate', 'Mossy Fired Cobbled Deepslate', 3.5, 6.0, 'deepslate', 'deepslate', 'pickaxe')
  createBlockFamily(event, 'mossy_cobbled_deepslate', 'Mossy Cobbled Deepslate', 3.5, 6.0, 'deepslate', 'deepslate', 'pickaxe')
  createBlockFamily(event, 'mossy_cobbled_deepslate_bricks', 'Mossy Cobbled Deepslate Bricks', 3.5, 6.0, 'deepslate', 'deepslate', 'pickaxe')

  createBlockFamily(event, 'clorite_bricks', 'Azurite Bricks', 3, 12.0, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'chiseled_clorite', 'Chiseled Azurite', 3, 12.0, 'stone', 'stone', 'pickaxe')
  //createBlock(event, 'aseterite_pillar', 'Aseterite Pillar', 3, 12.0, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'chiseled_aseterite', 'Chiseled Aseterite', 3, 12.0, 'stone', 'stone', 'pickaxe')

  //Dawg I couldn't be fucked trying to assimilate this with the helper functions
  //CREDIT GOES TO HALEX
    event.create("aseterite_pillar")
      .hardness(3)
      .resistance(12.0)
      .soundType('stone')
      .mapColor('stone')
      .property(BlockProperties.AXIS)
      .placementState(event => event.set(BlockProperties.AXIS, event.clickedFace.axis))
      .model('kubejs:block/aseterite_pillar')
      .displayName("Aseterite Pillar")
      .requiresTool(true)
      .tagBlock("mineable/pickaxe")
      .blockstateJson = {
          "variants": {
              "axis=x": {
                  "model": "kubejs:block/aseterite_pillar",
                  "x": 90,
                  "y": 90
              },
              "axis=y": {
                  "model": "kubejs:block/aseterite_pillar"
              },
              "axis=z": {
                  "model": "kubejs:block/aseterite_pillar",
                  "x": 90
              }
          }
      }

  //Cobble families
  let cobbles = [
    ['cobbled_granite', 'Cobbled Granite'],
    ['cobbled_diorite', 'Cobbled Diorite'],
    ['cobbled_andesite', 'Cobbled Andesite'],

    ['mossy_cobbled_granite', 'Mossy Cobbled Granite'],
    ['mossy_cobbled_diorite', 'Mossy Cobbled Diorite'],
    ['mossy_cobbled_andesite', 'Mossy Cobbled Andesite'],

    ['mossy_fired_cobbled_granite', 'Mossy Fired Cobbled Granite'],
    ['mossy_fired_cobbled_diorite', 'Mossy Fired Cobbled Diorite'],
    ['mossy_fired_cobbled_andesite', 'Mossy Fired Cobbled Andesite'],

    ['cobbled_granite_bricks', 'Cobbled Granite Bricks'],
    ['cobbled_diorite_bricks', 'Cobbled Diorite Bricks'],
    ['cobbled_andesite_bricks', 'Cobbled Andesite Bricks'],

    ['mossy_cobbled_granite_bricks', 'Mossy Cobbled Granite Bricks'],
    ['mossy_cobbled_diorite_bricks', 'Mossy Cobbled Diorite Bricks'],
    ['mossy_cobbled_andesite_bricks', 'Mossy Cobbled Andesite Bricks'],

    ['fired_cobbled_granite', 'Fired Cobbled Granite'],
    ['fired_cobbled_diorite', 'Fired Cobbled Diorite'],
    ['fired_cobbled_andesite', 'Fired Cobbled Andesite'],
  ]
  cobbles.forEach(([id, name]) => {
    createBlockFamily(event, id, name, 2, 6.0, 'stone', 'stone', 'pickaxe')
  })

  //Ores
  createBlock(event, 'opal_ore', 'Opal Ore', 30, 1200, 'stone', 'stone', 'pickaxe')
    .tagBlock('minecraft:needs_diamond_tool').tagBlock('c:ores')
  createBlock(event, 'titanium_ore', 'Titanium Ore', 30, 1200, 'stone', 'stone', 'pickaxe')
    .tagBlock('minecraft:needs_diamond_tool').tagBlock('c:ores')
  createBlock(event, 'aquamarine_ore', 'Aquamarine Ore', 3, 3, 'stone', 'stone', 'shovel')
    .tagBlock('minecraft:needs_diamond_tool').tagBlock('c:ores')

  //Smooth stone variants
  createBlock(event, 'smooth_granite', 'Smooth Granite', 1.5, 6, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'smooth_diorite', 'Smooth Diorite', 1.5, 6, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'smooth_andesite', 'Smooth Andesite', 1.5, 6, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'smooth_deepslate', 'Smooth Deepslate', 3, 6, 'stone', 'stone', 'pickaxe')

  createSlab(event, 'smooth_granite_slab', 'Smooth Granite Slab', 1.5, 6, 'stone', 'stone', 'pickaxe')
  createSlab(event, 'smooth_diorite_slab', 'Smooth Diorite Slab', 1.5, 6, 'stone', 'stone', 'pickaxe')
  createSlab(event, 'smooth_andesite_slab', 'Smooth Andesite Slab', 1.5, 6, 'stone', 'stone', 'pickaxe')
  createSlab(event, 'smooth_deepslate_slab', 'Smooth Deepslate Slab', 3, 6, 'stone', 'stone', 'pickaxe')

  //Sturdy stone variants
  createBlock(event, 'sturdy_granite', 'Sturdy Granite', 4, 10, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'sturdy_diorite', 'Sturdy Diorite', 4, 10, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'sturdy_andesite', 'Sturdy Andesite', 4, 10, 'stone', 'stone', 'pickaxe')
  createBlock(event, 'sturdy_deepslate', 'Sturdy Deepslate', 6, 10, 'stone', 'stone', 'pickaxe')

  //Crates (loop)
  let fruits = ['cherry','chestnut','cranberry','durian','mango','pear','plum','dragonfruit']
  fruits.forEach(fruit => {
    createBlock(event, `${fruit}_crate`, `${fruit.charAt(0).toUpperCase() + fruit.slice(1)} Crate`, 2.5, 2.5, 'wood', 'wood', 'axe', false)
  })
})