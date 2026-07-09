//IMMERSIVE WEATHERING ADDS THIS FEATURE NATIVELY -- THIS IS MAINLY FOR ARCHIVING PURPOSES

// var wood_type = [
//   "minecraft:acacia_log",
//   "minecraft:birch_log",
//   "minecraft:cherry_log",
//   "minecraft:dark_oak_log",
//   "minecraft:jungle_log",
//   "minecraft:mangrove_log",
//   "minecraft:oak_log",
//   "minecraft:spruce_log",
//   "minecraft:crimson_stem",
//   "minecraft:warped_stem",
//   "minecraft:acacia_wood",
//   "minecraft:birch_wood",
//   "minecraft:cherry_wood",
//   "minecraft:dark_oak_wood",
//   "minecraft:jungle_wood",
//   "minecraft:mangrove_wood",
//   "minecraft:oak_wood",
//   "minecraft:spruce_log",
//   "minecraft:crimson_hyphae",
//   "minecraft:warped_hyphae",
//   "aether:skyroot_log",
//   "aether:skyroot_wood",
//   "aether:golden_oak_log",
//   "aether:golden_oak_wood",
//   "deep_aether:roseroot_log",
//   "deep_aether:roseroot_wood",
//   "deep_aether:yagroot_log",
//   "deep_aether:yagroot_wood",
//   "deep_aether:cruderoot_log",
//   "deep_aether:conberry_log",
//   "deep_aether:conberry_wood",
//   "deep_aether:sunroot_log",
//   "deep_aether:sunroot_wood",
//   "betterend:mossy_glowshroom_log",
//   "betterend:mossy_glowshroom_bark",
//   "betterend:pythadendron_log",
//   "betterend:pythadendron_bark",
//   "betterend:end_lotus_log",
//   "betterend:end_lotus_bark",
//   "betterend:lacugrove_log",
//   "betterend:lacugrove_bark",
//   "betterend:dragon_tree_log",
//   "betterend:dragon_tree_bark",
//   "betterend:tenanea_log",
//   "betterend:tenanea_bark",
//   "betterend:helix_tree_log",
//   "betterend:helix_tree_bark",
//   "betterend:umbrella_tree_log",
//   "betterend:umbrella_tree_bark",
//   "betterend:jellyshroom_log",
//   "betterend:jellyshroom_bark",
//   "betterend:lucernia_log",
//   "betterend:lucernia_bark",
//   "quark:azalea_log",
//   "quark:azalea_wood",
//   "quark:ancient_log",
//   "quark:ancient_wood",
//   "quark:blossom_log",
//   "quark:blossom_wood",
//   "upgrade_aquatic:river_wood",
//   "upgrade_aquatic:river_log",
//   "upgrade_aquatic:driftwood_wood",
//   "upgrade_aquatic:driftwood"
// ];
// for (let i = 0; i < wood_type.length; i++) {
//   BlockEvents.rightClicked(wood_type[i], (event) => {
//     if (event.item.hasTag("minecraft:axes")) {
//       event.block.popItem("farmersdelight:tree_bark");
//       event.close();
//     } else if (event.item.is("bettercopper:copper_axe")) {
//       event.player.give("farmersdelight:tree_bark");
//       event.close();
//     } else {
//       event.close();
//     }
//   });
// }