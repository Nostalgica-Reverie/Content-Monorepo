//CRAFTING TABLE
const itemsToRemove = [

'minecraft:sniffer_egg', 

'deep_aether:raw_clorite', 
'deep_aether:raw_clorite_stairs', 
'deep_aether:raw_clorite_slab', 
'deep_aether:raw_clorite_wall', 
'deep_aether:cobbled_aseterite', 
'deep_aether:cobbled_aseterite_stairs', 
'deep_aether:cobbled_aseterite_slab', 
'deep_aether:cobbled_aseterite_wall',

'ascended_quark:skyroot_stick_block',

'betterend:azure_jadestone',
'betterend:sandy_jadestone',
'betterend:virid_jadestone',

'betterend:sandy_jadestone_stairs', 
'betterend:sandy_jadestone_slab', 
'betterend:sandy_jadestone_wall', 
'betterend:sandy_jadestone_plate', 
'betterend:sandy_jadestone_button', 
'betterend:sandy_jadestone_bricks', 
'betterend:sandy_jadestone_bricks_stairs', 
'betterend:sandy_jadestone_bricks_slab', 
'betterend:sandy_jadestone_bricks_wall', 
'betterend:virid_jadestone', 
'betterend:virid_jadestone_stairs', 
'betterend:virid_jadestone_slab', 
'betterend:virid_jadestone_wall', 
'betterend:virid_jadestone_plate', 
'betterend:virid_jadestone_button', 
'betterend:virid_jadestone_bricks', 
'betterend:virid_jadestone_bricks_stairs', 
'betterend:virid_jadestone_bricks_slab', 
'betterend:virid_jadestone_bricks_wall', 
'betterend:virid_jadestone_polished', 
'betterend:virid_jadestone_pillar', 
'betterend:virid_jadestone_tiles', 
'betterend:azure_jadestone', 
'betterend:azure_jadestone_stairs', 
'betterend:azure_jadestone_slab', 
'betterend:azure_jadestone_wall', 
'betterend:azure_jadestone_plate', 
'betterend:azure_jadestone_button', 
'betterend:azure_jadestone_bricks', 
'betterend:azure_jadestone_bricks_stairs', 
'betterend:azure_jadestone_bricks_slab', 
'betterend:azure_jadestone_bricks_wall', 
'betterend:azure_jadestone_polished', 
'betterend:azure_jadestone_pillar', 
'betterend:azure_jadestone_tiles', 
'betterend:sandy_jadestone',
'betterend:sandy_jadestone_polished', 
'betterend:sandy_jadestone_pillar', 
'betterend:sandy_jadestone_tiles',

'deep_aether:stormforged_smithing_template',

  'enchantedcharms:copper_charm', 
  'enchantedcharms:obsidian_charm', 
  'enchantedcharms:amethyst_charm', 
  'enchantedcharms:emerald_charm',

  'hookshot:endershot', 
  'hookshot:hypershot',

    'quark:moss_paste',
    'aether:skyroot_bookshelf',

    'aether:skyroot_shovel',
    'aether:skyroot_pickaxe',
    'aether:skyroot_axe',
    'aether:skyroot_hoe',
    'aether:holystone_shovel',
    'aether:holystone_pickaxe',
    'aether:holystone_axe',
    'aether:holystone_hoe',
    'aether:skyroot_sword',
    'aether:holystone_sword',

    'quark:violet_corundum_cluster',
    'quark:white_corundum',
    'quark:waxed_white_corundum',
    'quark:white_corundum_pane',
    'quark:white_corundum_cluster',
    'quark:black_corundum',
    'quark:waxed_black_corundum',
    'quark:black_corundum_pane',
    'quark:black_corundum_cluster',
    'quark:red_corundum',
    'quark:waxed_red_corundum',
    'quark:red_corundum_pane',
    'quark:red_corundum_cluster',
    'quark:orange_corundum',
    'quark:waxed_orange_corundum',
    'quark:orange_corundum_pane',
    'quark:orange_corundum_cluster',
    'quark:yellow_corundum',
    'quark:waxed_yellow_corundum',
    'quark:yellow_corundum_pane',
    'quark:yellow_corundum_cluster',
    'quark:green_corundum',
    'quark:waxed_green_corundum',
    'quark:green_corundum_pane',
    'quark:green_corundum_cluster',
    'quark:blue_corundum',
    'quark:waxed_blue_corundum',
    'quark:blue_corundum_pane',
    'quark:blue_corundum_cluster',
    'quark:indigo_corundum',
    'quark:waxed_indigo_corundum',
    'quark:indigo_corundum_pane',
    'quark:indigo_corundum_cluster',
    'quark:violet_corundum',
    'quark:waxed_violet_corundum',
    'quark:violet_corundum_pane',

    'immersive_weathering:deep_aether/conberry_leaf_pile',
    'immersive_weathering:deep_aether/sunroot_leaf_pile',
    'immersive_weathering:upgrade_aquatic/river_leaf_pile',
    'immersive_weathering:quark/ancient_leaf_pile',
    'immersive_weathering:quark/blue_blossom_leaf_pile',
    'immersive_weathering:quark/lavender_blossom_leaf_pile',
    'immersive_weathering:quark/orange_blossom_leaf_pile',
    'immersive_weathering:quark/yellow_blossom_leaf_pile',
    'immersive_weathering:quark/red_blossom_leaf_pile',
    'immersive_weathering:oak_leaf_pile',
    'immersive_weathering:aether/golden_oak_leaf_pile',
    'immersive_weathering:aether/holiday_leaf_pile',
    'immersive_weathering:aether/decorated_holiday_leaf_pile',
    'immersive_weathering:aether/crystal_leaf_pile',
    'immersive_weathering:aether/crystal_fruit_leaf_pile',
    'immersive_weathering:deep_aether/flowering_roseroot_leaf_pile',
    'immersive_weathering:deep_aether/blue_roseroot_leaf_pile',
    'immersive_weathering:deep_aether/flowering_blue_roseroot_leaf_pile',
    'immersive_weathering:spruce_leaf_pile',
    'immersive_weathering:birch_leaf_pile',
    'immersive_weathering:jungle_leaf_pile',
    'immersive_weathering:acacia_leaf_pile',
    'immersive_weathering:cherry_leaf_pile',
    'immersive_weathering:dark_oak_leaf_pile',
    'immersive_weathering:mangrove_leaf_pile',
    'immersive_weathering:azalea_leaf_pile',
    'immersive_weathering:flowering_azalea_leaf_pile',
    'immersive_weathering:betterend/pythadendron_leaf_pile',
    'immersive_weathering:betterend/lacugrove_leaf_pile',
    'immersive_weathering:betterend/dragon_tree_leaf_pile',
    'immersive_weathering:betterend/tenanea_leaf_pile',
    'immersive_weathering:betterend/lucernia_leaf_pile',
    'immersive_weathering:aether/skyroot_leaf_pile',
    'immersive_weathering:deep_aether/roseroot_leaf_pile',
    'immersive_weathering:deep_aether/yagroot_leaf_pile',
    'immersive_weathering:deep_aether/cruderoot_leaf_pile',

    'aether:phoenix_gloves',
    'aether:valkyrie_gloves',
    'aether:neptune_gloves',
    'aether:gravitite_gloves',
    'deep_aether:skyjade_gloves',
    'aether:zanite_gloves',
    'aether:obsidian_gloves',
    'aether:book_of_lore',

    'caverns_and_chasms:bejeweled_apple',

    'naturalist:teddy_bear',
    'naturalist:shellstone',
    'naturalist:shellstone_stairs',
    'naturalist:shellstone_slab',
    'naturalist:shellstone_wall',
    'naturalist:shellstone_bricks',
    'naturalist:shellstone_brick_stairs',
    'naturalist:shellstone_brick_slab',
    'naturalist:shellstone_brick_wall',
    'naturalist:smooth_shellstone',
    'naturalist:smooth_shellstone_stairs',
    'naturalist:smooth_shellstone_slab',
    'naturalist:smooth_shellstone_wall',

    'caverns_and_chasms:calcite_stairs',
    'caverns_and_chasms:calcite_slab',
    'caverns_and_chasms:calcite_wall',

    'caverns_and_chasms:spinel_block',
    'caverns_and_chasms:spinel_bricks',
    'caverns_and_chasms:spinel_brick_stairs',
    'caverns_and_chasms:spinel_brick_slab',
    'caverns_and_chasms:spinel_brick_wall',
    'caverns_and_chasms:spinel_pillar',
    'caverns_and_chasms:spinel_lamp',
    'caverns_and_chasms:spinel_ore',
    'caverns_and_chasms:deepslate_spinel_ore',
    'caverns_and_chasms:spinel',
    'caverns_and_chasms:necromium_sword',
    'caverns_and_chasms:necromium_hoe',
    'caverns_and_chasms:necromium_axe',
    'caverns_and_chasms:necromium_pickaxe',
    'caverns_and_chasms:necromium_shovel',
    'caverns_and_chasms:necromium_ingot',
    'caverns_and_chasms:necromium_nugget',
    'caverns_and_chasms:necromium_horse_armor',
    'caverns_and_chasms:necromium_block',
    'caverns_and_chasms:depth_gauge',
    'caverns_and_chasms:copper_nugget',
    'caverns_and_chasms:living_flesh',
    'caverns_and_chasms:golden_bars',

    'caverns_and_chasms:cobblestone_bricks',
    'caverns_and_chasms:cobblestone_brick_stairs',
    'caverns_and_chasms:cobblestone_brick_slab',
    'caverns_and_chasms:cobblestone_brick_wall',
    'caverns_and_chasms:cobblestone_tiles',
    'caverns_and_chasms:cobblestone_tile_stairs',
    'caverns_and_chasms:cobblestone_tile_slab',
    'caverns_and_chasms:cobblestone_tile_wall',
    'caverns_and_chasms:mossy_cobblestone_tile_slab',
    'caverns_and_chasms:mossy_cobblestone_tile_wall',
    'caverns_and_chasms:mossy_cobblestone_tile_stairs',
    'caverns_and_chasms:mossy_cobblestone_tiles',
    'caverns_and_chasms:mossy_cobblestone_brick_wall',
    'caverns_and_chasms:mossy_cobblestone_brick_slab',
    'caverns_and_chasms:mossy_cobblestone_brick_stairs',
    'caverns_and_chasms:mossy_cobblestone_bricks',

    'upgrade_aquatic:kelpy_cobblestone_bricks',
    'upgrade_aquatic:kelpy_cobblestone_brick_stairs',
    'upgrade_aquatic:kelpy_cobblestone_brick_slab',
    'upgrade_aquatic:kelpy_cobblestone_brick_wall',
    'upgrade_aquatic:kelpy_cobblestone_tiles',
    'upgrade_aquatic:kelpy_cobblestone_tile_stairs',
    'upgrade_aquatic:kelpy_cobblestone_tile_slab',
    'upgrade_aquatic:kelpy_cobblestone_tile_wall',

    'caverns_and_chasms:polished_calcite',
    'caverns_and_chasms:polished_calcite_stairs',
    'caverns_and_chasms:polished_calcite_slab',

    'deep_aether:stormforged_helmet',
    'deep_aether:stormforged_chestplate',
    'deep_aether:stormforged_leggings',
    'deep_aether:stormforged_boots',
    'deep_aether:stormforged_gloves',
    'deep_aether:storm_sword',
    'deep_aether:storm_bow',
    'betterend:purple_polypore',
    'betterend:aurant_polypore',
    'betterend:jungle_fern',
    'betterend:bolux_mushroom_cooked',
    'betterend:chorus_mushroom_cooked',
    'betterend:chorus_mushroom_raw',
    'betterend:amber_root_raw',
    'betterend:flavolite_runed',
    'betterend:flavolite_runed_eternal',
    'betterend:salteago',
    'betterend:orango',
    'betterend:small_jellyshroom',
    'betterend:bubble_coral',
    'betterend:charnia_red',
    'betterend:charnia_purple',
    'betterend:charnia_orange',
    'betterend:charnia_light_blue',
    'betterend:charnia_cyan',
    'betterend:charnia_green',
    'betterend:magnula',
    'betterend:filalux_wings',
    'corpsecomplex:scroll',
    'betterend:ender_ore',
    'betterend:amber_ore',
    'betterend:amber_block',
    'betterend:ender_block',
    'betterend:raw_amber',
    'betterend:amber_gem',
    'farmersdelight:diamond_knife',
    'farmersdelight:golden_knife',
    'farmersdelight:netherite_knife',
    'deep_aether:zephyr_totem',
    'deep_aether:skyjade',
    'deep_aether:moa_totem',
    'deep_aether:aerwhale_totem',
    'minecraft:totem_of_undying',
    'deep_aether:bio_crystal',
    'deep_aether:antidote',
    'deep_aether:enchanted_antidote',
    'deep_aether:combiner',
    'deep_aether:frozen_goldenleaf_berries',
    'deep_aether:skyjade_lantern',
    'deep_aether:skyjade_nugget',
    'deep_aether:rotten_roseroot_log',
    'deep_aether:combiner',
    'deep_aether:cooked_quail',
    'deep_aether:raw_quail',
    'deep_aether:skyjade_chain',
    'aether:skyroot_stick',
    'deep_aether:afterburner',
    'betterend:iron_hammer',
    'incubation:fried_egg',
    'bettercopper:reversed_copper_heart',
    'betterend:golden_hammer',
    'betterend:diamond_hammer',
    'betterend:netherite_hammer',
    'deep_aether:stratus_ingot',
    'deep_aether:stratus_smithing_template',
    'deep_aether:stratus_block',
    'deep_aether:skyjade_block',
    'betterend:charcoal_block',
    'supplementaries:bunting',
    'supplementaries:cannon',
    'supplementaries:cannonball',
    'betterend:mossy_glowshroom_crafting_table',
    'betterend:pythadendron_crafting_table',
        'betterend:end_lotus_crafting_table',
        'betterend:lacugrove_crafting_table',
        'betterend:dragon_tree_crafting_table',
        'betterend:tenanea_crafting_table',
        'betterend:helix_tree_crafting_table',
        'betterend:umbrella_tree_crafting_table',
        'betterend:jellyshroom_crafting_table',
        'betterend:lucernia_crafting_table',
        'betterend:mossy_glowshroom_bookshelf',
        'betterend:pythadendron_bookshelf',
        'betterend:end_lotus_bookshelf',
        'betterend:lacugrove_bookshelf',
        'betterend:dragon_tree_bookshelf',
        'betterend:tenanea_bookshelf',
        'betterend:helix_tree_bookshelf',
        'betterend:umbrella_tree_bookshelf',
        'betterend:jellyshroom_bookshelf',
        'betterend:lucernia_bookshelf',
        'betterend:lucernia_wall',
        'betterend:umbrella_tree_wall',
        'betterend:jellyshroom_wall',
        'betterend:mossy_glowshroom_barrel',
        'betterend:pythadendron_barrel',
        'betterend:end_lotus_barrel',
        'betterend:lacugrove_barrel',
        'betterend:dragon_tree_barrel',
        'betterend:tenanea_barrel',
        'betterend:helix_tree_barrel',
        'betterend:umbrella_tree_barrel',
        'betterend:jellyshroom_barrel',
        'betterend:lucernia_barrel',
        'betterend:mossy_glowshroom_composter',
        'betterend:pythadendron_composter',
        'betterend:end_lotus_composter',
        'betterend:lacugrove_composter',
        'betterend:dragon_tree_composter',
        'betterend:tenanea_composter',
        'betterend:helix_tree_composter',
        'betterend:umbrella_tree_composter',
        'betterend:jellyshroom_composter',
        'betterend:lucernia_composter',
        'betterend:mossy_glowshroom_wall',
        'betterend:pythadendron_wall',
        'betterend:end_lotus_wall',
        'betterend:lacugrove_wall',
        'betterend:dragon_tree_wall',
        'betterend:tenanea_wall',
        'betterend:helix_tree_wall',
        'betterend:lucernia_taburet',
        'betterend:jellyshroom_taburet',
        'betterend:flavolite_furnace',
    'betterend:violecite_furnace',
    'betterend:sulphuric_rock_furnace',
    'betterend:virid_jadestone_furnace',
    'betterend:azure_jadestone_furnace',
    'betterend:sandy_jadestone_furnace',
    'betterend:umbralith_furnace',
    'betterend:end_stone_furnace',
    'betterend:end_stone_smelter',
    'betterend:jellyshroom_bar_stool',
    'betterend:lucernia_ladder',
    'betterend:mossy_glowshroom_ladder',
    'betterend:pythadendron_ladder',
    'betterend:end_lotus_ladder',
    'betterend:lacugrove_ladder',
    'betterend:dragon_tree_ladder',
    'betterend:tenanea_ladder',
    'betterend:helix_tree_ladder',
    'betterend:umbrella_tree_ladder',
    'betterend:jellyshroom_ladder',
    'betterend:mossy_glowshroom_taburet',
    'betterend:mossy_glowshroom_bar_stool',
    'betterend:pythadendron_taburet',
    'betterend:pythadendron_bar_stool',
    'betterend:end_lotus_taburet',
    'betterend:end_lotus_bar_stool',
    'betterend:lacugrove_taburet',
    'betterend:lacugrove_bar_stool',
    'betterend:dragon_tree_taburet',
    'betterend:dragon_tree_bar_stool',
    'betterend:lucernia_bar_stool',
    'betterend:tenanea_taburet',
    'betterend:tenanea_bar_stool',
    'betterend:helix_tree_taburet',
    'betterend:helix_tree_bar_stool',
    'betterend:umbrella_tree_taburet',
    'betterend:umbrella_tree_bar_stool',
    'betterend:mossy_glowshroom_chair',
    'betterend:pythadendron_chair',
    'betterend:end_lotus_chair',
    'betterend:lacugrove_chair',
    'betterend:dragon_tree_chair',
    'betterend:tenanea_chair',
    'betterend:helix_tree_chair',
    'betterend:umbrella_tree_chair',
    'betterend:jellyshroom_chair',
    'betterend:lucernia_chair',
    'minecraft:shield',
    'betterend:jellyshroom_chest',
    'betterend:lucernia_chest',
    'betterend:umbrella_tree_chest',
    'betterend:helix_tree_chest',
    'betterend:tenanea_chest',
    'betterend:dragon_tree_chest',
    'betterend:lacugrove_chest',
    'betterend:end_lotus_chest',
    'betterend:pythadendron_chest',
    'betterend:mossy_glowshroom_chest',
    'minecraft:enchanting_table',
    'minecraft:enchanted_book',
    'bettercopper:copper_heart',
    'betterend:thallasium_anvil',
    'betterend:terminite_anvil',
    'betterend:aeternium_anvil',
    'betterend:thallasium_chain',
    'betterend:terminite_chain',
    'betterend:crystalite_helmet',
    'betterend:crystalite_chestplate',
    'betterend:crystalite_leggings',
    'betterend:crystalite_boots',
    'betterend:elytra_crystalite',
    'betterend:aeternium_sword_blade',
    'betterend:aeternium_sword_handle',
    'betterend:thallasium_sword_blade',
    'betterend:thallasium_sword_handle',
    'betterend:terminite_sword_blade',
    'betterend:terminite_sword_handle',
    'minecraft:netherite_upgrade_smithing_template',
    'betterend:handle_attachment_smithing_template',
    'betterend:leather_handle_attachment_smithing_template',
    'betterend:tool_assembly_smithing_template',
    'betterend:plate_upgrade_smithing_template',
    'betterend:thallasium_upgrade_smithing_template',
    'betterend:terminite_upgrade_smithing_template',
    'betterend:aeternium_upgrade_smithing_template',
    'betterend:netherite_upgrade_smithing_template',
    'betterend:terminite_nugget',
    'betterend:thallasium_shovel_head',
    'betterend:thallasium_axe_head',
    'betterend:thallasium_pickaxe_head',
    'betterend:thallasium_hoe_head',
    'betterend:thallasium_forged_plate',
    'betterend:terminite_shovel_head',
    'betterend:terminite_pickaxe_head',
    'betterend:terminite_axe_head',
    'betterend:terminite_hoe_head',
    'betterend:terminite_hammer',
    'betterend:terminite_forged_plate',
    'betterend:aeternium_forged_plate',
    'betterend:aeternium_pickaxe_head',
    'betterend:aeternium_hammer',
    'betterend:aeternium_shovel_head',
    'betterend:aeternium_axe_head',
    'betterend:aeternium_hoe_head',
    'betterend:aeternium_hammer_head',
    'betterend:thallasium_hammer',
    'inmis:baby_backpack',
    'supplementaries:rope',
    'inmis:ender_pouch',
    'betterend:flavolite_pedestal',
    'betterend:violecite_pedestal',
    'betterend:sulphuric_rock_pedestal',
    'betterend:virid_jadestone_pedestal',
    'betterend:azure_jadestone_pedestal',
    'betterend:sandy_jadestone_pedestal',
    'betterend:umbralith_pedestal',
    'betterend:andesite_pedestal',
    'betterend:diorite_pedestal',
    'betterend:granite_pedestal',
    'betterend:quartz_pedestal',
    'betterend:purpur_pedestal',
    'betterend:smaragdant_crystal_pedestal',
    'betterend:eternal_pedestal',
    'betterend:infusion_pedestal',
    'betterend:leather_stripe',
    'betterend:mossy_glowshroom_wall',
    'betterend:pythadendron_wall',
    'betterend:end_lotus_wall',
    'betterend:lacugrove_wall',
    'betterend:dragon_tree_wall',
    'betterend:tenanea_wall',
    'betterend:helix_tree_wall',
    'betterend:umbrella_tree_wall',
    'betterend:jellyshroom_wall',
    'betterend:lucernia_wall',
    'deep_aether:roseroot_wall',
    'deep_aether:stripped_roseroot_wall',
    'deep_aether:yagroot_wall',
    'deep_aether:stripped_yagroot_wall',
    'deep_aether:cruderoot_wall',
    'deep_aether:stripped_cruderoot_wall',
    'deep_aether:conberry_wall',
    'deep_aether:stripped_conberry_wall',
    'deep_aether:sunroot_wall',
    'deep_aether:stripped_sunroot_wall',
    'libraryferret:gold_coins_jtl',
    'libraryferret:iron_coins_jtl',
    'libraryferret:diamond_coins_jtl',
    'libraryferret:netherite_coins_jtl',
    'libraryferret:emerald_coins_jtl',
    'upgrade_aquatic:bedroll',
    'upgrade_aquatic:white_bedroll',
    'upgrade_aquatic:light_gray_bedroll',
    'upgrade_aquatic:gray_bedroll',
    'upgrade_aquatic:black_bedroll',
    'upgrade_aquatic:brown_bedroll',
    'upgrade_aquatic:red_bedroll',
    'upgrade_aquatic:orange_bedroll',
    'upgrade_aquatic:yellow_bedroll',
    'upgrade_aquatic:lime_bedroll',
    'upgrade_aquatic:green_bedroll',
    'upgrade_aquatic:cyan_bedroll',
    'upgrade_aquatic:light_blue_bedroll',
    'upgrade_aquatic:blue_bedroll',
    'upgrade_aquatic:purple_bedroll',
    'upgrade_aquatic:magenta_bedroll',
    'upgrade_aquatic:pink_bedroll',
    'betterend:mossy_glowshroom_fur',
    'betterend:tenanea_outer_leaves',
    'betterend:lucernia_outer_leaves',
    'betterend:blue_vine_fur',
    'betterend:glowing_pillar_leaves',
    'betterend:amaranita_fur',
    'betterend:tail_moss',
    'betterend:cyan_moss',
    'betterend:twisted_moss',
    'betterend:bulb_moss',
    'betterend:ruscus',
    'betterend:flavolite_flower_pot',
    'betterend:violecite_flower_pot',
    'betterend:sulphuric_rock_flower_pot',
    'betterend:virid_jadestone_flower_pot',
    'betterend:azure_jadestone_flower_pot',
    'betterend:sandy_jadestone_flower_pot',
    'betterend:umbralith_flower_pot',
    'betterend:endstone_flower_pot',
    'betterend:flavolite_lantern',
    'betterend:violecite_lantern',
    'betterend:sulphuric_rock_lantern',
    'betterend:virid_jadestone_lantern',
    'betterend:azure_jadestone_lantern',
    'betterend:sandy_jadestone_lantern',
    'betterend:umbralith_lantern',
    'betterend:thallasium_bulb_lantern',
    'betterend:thallasium_bulb_lantern_white',
    'betterend:thallasium_bulb_lantern_gray',
    'betterend:thallasium_bulb_lantern_light_gray',
    'betterend:thallasium_bulb_lantern_black',
    'betterend:thallasium_bulb_lantern_blue',
    'betterend:thallasium_bulb_lantern_lime',
    'betterend:thallasium_bulb_lantern_magenta',
    'betterend:thallasium_bulb_lantern_red',
    'betterend:thallasium_bulb_lantern_orange',
    'betterend:thallasium_bulb_lantern_purple',
    'betterend:thallasium_bulb_lantern_cyan',
    'betterend:thallasium_bulb_lantern_brown',
    'betterend:thallasium_bulb_lantern_green',
    'betterend:thallasium_bulb_lantern_yellow',
    'betterend:thallasium_bulb_lantern_pink',
    'betterend:thallasium_bulb_lantern_light_blue',
    'betterend:terminite_bulb_lantern',
    'betterend:terminite_bulb_lantern_white',
    'betterend:terminite_bulb_lantern_gray',
    'betterend:terminite_bulb_lantern_light_gray',
    'betterend:terminite_bulb_lantern_black',
    'betterend:terminite_bulb_lantern_blue',
    'betterend:terminite_bulb_lantern_lime',
    'betterend:terminite_bulb_lantern_magenta',
    'betterend:terminite_bulb_lantern_red',
    'betterend:terminite_bulb_lantern_orange',
    'betterend:terminite_bulb_lantern_purple',
    'betterend:terminite_bulb_lantern_cyan',
    'betterend:terminite_bulb_lantern_brown',
    'betterend:terminite_bulb_lantern_green',
    'betterend:terminite_bulb_lantern_yellow',
    'betterend:terminite_bulb_lantern_pink',
    'betterend:terminite_bulb_lantern_light_blue',
    'betterend:iron_bulb_lantern',
    'betterend:iron_bulb_lantern_white',
    'betterend:iron_bulb_lantern_gray',
    'betterend:iron_bulb_lantern_light_gray',
    'betterend:iron_bulb_lantern_black',
    'betterend:iron_bulb_lantern_blue',
    'betterend:iron_bulb_lantern_lime',
    'betterend:iron_bulb_lantern_magenta',
    'betterend:iron_bulb_lantern_red',
    'betterend:iron_bulb_lantern_orange',
    'betterend:iron_bulb_lantern_purple',
    'betterend:iron_bulb_lantern_cyan',
    'betterend:iron_bulb_lantern_brown',
    'betterend:iron_bulb_lantern_green',
    'betterend:iron_bulb_lantern_yellow',
    'betterend:iron_bulb_lantern_pink',
    'betterend:iron_bulb_lantern_light_blue',
    'betterend:andesite_lantern',
    'betterend:diorite_lantern',
    'betterend:granite_lantern',
    'betterend:quartz_lantern',
    'betterend:purpur_lantern',
    'betterend:end_stone_lantern',
    'betterend:blackstone_lantern',
    'betterend:thallasium_chandelier',
    'betterend:terminite_chandelier',
    'betterend:iron_chandelier',
    'betterend:gold_chandelier',
    'deep_aether:mini_golden_grass',
    'deep_aether:short_golden_grass'
]

//WOODCUTTING
const itemsToRemoveWoodcutting = [
'ascended_quark:skyroot_stick_block',
'deep_aether:roseroot_chest_boat',
'deep_aether:yagroot_boat',
'deep_aether:yagroot_chest_boat',
'deep_aether:cruderoot_boat',
'deep_aether:cruderoot_chest_boat',
'deep_aether:conberry_boat',
'deep_aether:conberry_chest_boat',
'deep_aether:sunroot_boat',
'deep_aether:sunroot_chest_boat',
'aether:skyroot_chest_boat',
'aether:skyroot_boat',
'minecraft:spruce_boat',
'minecraft:spruce_chest_boat',
'minecraft:birch_boat',
'minecraft:birch_chest_boat',
'minecraft:jungle_boat',
'minecraft:jungle_chest_boat',
'minecraft:acacia_boat',
'minecraft:acacia_chest_boat',
'minecraft:dark_oak_boat',
'minecraft:dark_oak_chest_boat',
'minecraft:mangrove_boat',
'minecraft:mangrove_chest_boat',
'minecraft:cherry_boat',
'minecraft:cherry_chest_boat',
'upgrade_aquatic:driftwood_boat',
'upgrade_aquatic:driftwood_chest_boat',
'upgrade_aquatic:river_boat',
'upgrade_aquatic:river_chest_boat',
'quark:ancient_boat',
'quark:ancient_chest_boat',
'quark:azalea_boat',
'quark:azalea_chest_boat',
'quark:blossom_boat',
'quark:blossom_chest_boat',
'deep_aether:roseroot_boat',

'caverns_and_chasms:azalea_log',
'caverns_and_chasms:azalea_wood',
'caverns_and_chasms:stripped_azalea_log',
'caverns_and_chasms:stripped_azalea_wood',
'caverns_and_chasms:azalea_planks',
'caverns_and_chasms:azalea_stairs',
'caverns_and_chasms:azalea_slab',
'caverns_and_chasms:azalea_fence',
'caverns_and_chasms:azalea_fence_gate',
'caverns_and_chasms:azalea_door',
'caverns_and_chasms:azalea_trapdoor',
'caverns_and_chasms:azalea_pressure_plate',
'caverns_and_chasms:azalea_button',
'caverns_and_chasms:azalea_sign',
'caverns_and_chasms:azalea_hanging_sign',
'supplementaries:caverns_and_chasms/sign_post_azalea',
'caverns_and_chasms:azalea_boat',
'caverns_and_chasms:azalea_chest_boat',

'aether:skyroot_sword',
'aether:skyroot_shovel',
'aether:skyroot_pickaxe',
'aether:skyroot_axe',
'aether:skyroot_hoe'
]

//STONECUTTING
const itemsToRemoveStonecutting = [
'caverns_and_chasms:cobblestone_bricks',
'caverns_and_chasms:cobblestone_brick_stairs',
'caverns_and_chasms:cobblestone_brick_slab',
'caverns_and_chasms:cobblestone_brick_wall',
'caverns_and_chasms:cobblestone_tiles',
'caverns_and_chasms:cobblestone_tile_stairs',
'caverns_and_chasms:cobblestone_tile_slab',
'caverns_and_chasms:cobblestone_tile_wall',
'caverns_and_chasms:mossy_cobblestone_tile_slab',
'caverns_and_chasms:mossy_cobblestone_tile_wall',
'caverns_and_chasms:mossy_cobblestone_tile_stairs',
'caverns_and_chasms:mossy_cobblestone_tiles',
'caverns_and_chasms:mossy_cobblestone_brick_wall',
'caverns_and_chasms:mossy_cobblestone_brick_slab',
'caverns_and_chasms:mossy_cobblestone_brick_stairs',
'caverns_and_chasms:mossy_cobblestone_bricks',
'upgrade_aquatic:kelpy_cobblestone_bricks',
'upgrade_aquatic:kelpy_cobblestone_brick_stairs',
'upgrade_aquatic:kelpy_cobblestone_brick_slab',
'upgrade_aquatic:kelpy_cobblestone_brick_wall',
'upgrade_aquatic:kelpy_cobblestone_tiles',
'upgrade_aquatic:kelpy_cobblestone_tile_stairs',
'upgrade_aquatic:kelpy_cobblestone_tile_slab',
'upgrade_aquatic:kelpy_cobblestone_tile_wall',
'caverns_and_chasms:polished_calcite',
'caverns_and_chasms:polished_calcite_stairs',
'caverns_and_chasms:polished_calcite_slab',

'caverns_and_chasms:polished_tuff_slab',
'caverns_and_chasms:polished_tuff_stairs',
'caverns_and_chasms:polished_tuff',
'caverns_and_chasms:tuff_wall',
'caverns_and_chasms:tuff_slab',
'caverns_and_chasms:tuff_stairs',
'caverns_and_chasms:chiseled_dripstone_shingles',
'caverns_and_chasms:sugilite',
'caverns_and_chasms:sugilite_stairs',
'caverns_and_chasms:sugilite_slab',
'caverns_and_chasms:sugilite_wall',
'caverns_and_chasms:polished_sugilite',
'caverns_and_chasms:polished_sugilite_stairs',
'caverns_and_chasms:polished_sugilite_slab',
'caverns_and_chasms:cobbled_deepslate_tiles',
'caverns_and_chasms:cobbled_deepslate_tile_stairs',
'caverns_and_chasms:cobbled_deepslate_tile_slab',
'caverns_and_chasms:cobbled_deepslate_tile_wall',
'caverns_and_chasms:sanguine_block',
'caverns_and_chasms:sanguine_tiles',
'caverns_and_chasms:sanguine_tile_stairs',
'caverns_and_chasms:sanguine_tile_slab',
'caverns_and_chasms:sanguine_tile_wall',
'caverns_and_chasms:fortified_sanguine_tiles',
'caverns_and_chasms:fortified_sanguine_tile_stairs',
'caverns_and_chasms:fortified_sanguine_tile_slab',
'caverns_and_chasms:fortified_sanguine_tile_wall'
]

//-------------------------------------------------------------------------

//REMOVING ALL OF THOSE RECIPES

ServerEvents.recipes(event => {

    //CRAFTING TABLE
    for (const itemId of itemsToRemove) {

    event.remove({ output: itemId })

  }

    //WOODCUTTING
    for (const itemId3 of itemsToRemoveWoodcutting) {

    event.remove({ output: itemId3 })
    event.remove({

      output: itemId3,
      type: 'sawmill:woodcutting'

    })

  }

    //STONECUTTING
    for (const itemId2 of itemsToRemoveStonecutting) {

    event.remove({ output: itemId2 })
    event.remove({

      output: itemId2,
      type: 'minecraft:stonecutting'

    })

  }

})

//BYE BYE TAGS + UNOBTAINA BLE TAG
ServerEvents.tags('item', event => {

    //CRAFTING TABLE
    for (const itemId of itemsToRemove) {

        event.removeAllTagsFrom(itemId)

        event.get('minecraft:goaway')
        .add(itemId)
    }

    //WOODCUTTING
    for (const itemId3 of itemsToRemoveWoodcutting) {

        event.removeAllTagsFrom(itemId3)

        event.get('minecraft:goaway')
        .add(itemId3)
        event.get('sawmill:blacklist')
        .add(itemId3)
    }

    //STONECUTTING
    for (const itemId2 of itemsToRemoveStonecutting) {

        event.removeAllTagsFrom(itemId2)

        event.get('minecraft:goaway')
        .add(itemId2)
    }

})

//Oh my god bruh I just want this one recipe gone
ServerEvents.recipes(event => {

  //WHY THE HELL IS THIS RECIPE NOT REMOVING WITH DATAPACKS
  event.remove({ output: 'minecraft:nether_wart_block' })
  event.remove({ output: 'minecraft:stonecutter' })
  event.remove({ output: 'supplementaries:safe' })

  event.remove({ id: 'fletchingrecipe:fletching/arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/awkward_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/empty_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/fire_resistance_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/harming_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/healing_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/invisibility_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/leaping_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/luck_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/mundane_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/night_vision_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/poison_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/regeneration_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/slow_falling_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/slowness_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strength_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/swiftness_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/thick_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/turtle_master_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/water_breathing_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/water_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/weakness_tipped_arrow' })

  event.remove({ id: 'fletchingrecipe:fletching/long_fire_resistance_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_harming_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_healing_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_invisibility_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_leaping_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_luck_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_mundane_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_night_vision_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_poison_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_regeneration_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_slow_falling_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_slowness_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_strength_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_swiftness_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_thick_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_turtle_master_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_water_breathing_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_water_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/long_weakness_tipped_arrow' })

  event.remove({ id: 'fletchingrecipe:fletching/strong_harming_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_healing_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_invisibility_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_leaping_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_luck_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_mundane_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_night_vision_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_poison_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_regeneration_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_slow_falling_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_slowness_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_strength_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_swiftness_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_thick_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_turtle_master_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_water_breathing_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_water_tipped_arrow' })
  event.remove({ id: 'fletchingrecipe:fletching/strong_weakness_tipped_arrow' })

})