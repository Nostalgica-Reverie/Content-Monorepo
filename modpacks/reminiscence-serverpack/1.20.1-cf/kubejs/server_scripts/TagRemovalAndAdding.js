// priority: 0

// Visit the wiki for more info - https://kubejs.com/

console.info('Hello, World! (Loaded startup scripts)')

ServerEvents.tags('item', event => {

  event.removeAllTagsFrom('#minecraft:goaway')

    event.get('c:ores')
      .add('kubejs:aquamarine_ore')
      .add('kubejs:opal_ore')
      .add('kubejs:titanium_ore')
      .add('deep_aether:skyjade_ore')

    event.get('forge:ores')      
      .add('kubejs:aquamarine_ore')
      .add('kubejs:opal_ore')
      .add('kubejs:titanium_ore')
      .add('deep_aether:skyjade_ore')

  //PROPER COPPER ARMOR & TOOL TAGS
    event.get('forge:tools/knives')
      .add('kubejs:copper_knife')
      .remove('farmersdelight:diamond_knife')
      .remove('farmersdelight:golden_knife')
      .remove('farmersdelight:netherite_knife')

      event.get('aether:slider_damaging_items')
      .add('bettercopper:copper_pickaxe')

      event.get('forge:tools')
      .add('bettercopper:copper_pickaxe')
      .add('bettercopper:copper_axe')
      .add('bettercopper:copper_shovel')
      .add('bettercopper:copper_sword')
      .add('bettercopper:copper_hoe')
      .remove('farmersdelight:diamond_knife')
      .remove('farmersdelight:golden_knife')
      .remove('farmersdelight:netherite_knife')
      
      event.get('minecraft:tools')
      .add('bettercopper:copper_pickaxe')
      .add('bettercopper:copper_axe')
      .add('bettercopper:copper_shovel')
      .add('bettercopper:copper_sword')
      .add('bettercopper:copper_hoe')

      event.get('forge:tools/pickaxes')
      .add('bettercopper:copper_pickaxe')
      event.get('forge:tools/axes')
      .add('bettercopper:copper_axe')
      event.get('forge:tools/shovels')
      .add('bettercopper:copper_shovel')
      event.get('forge:tools/hoes')
      .add('bettercopper:copper_hoe')

      event.get('c:pickaxes')
      .add('bettercopper:copper_pickaxe')
      event.get('c:axes')
      .add('bettercopper:copper_axe')
      event.get('c:shovels')
      .add('bettercopper:copper_shovel')
      event.get('c:hoes')
      .add('bettercopper:copper_hoe')

      event.get('minecraft:pickaxes')
      .add('bettercopper:copper_pickaxe')
      event.get('minecraft:axes')
      .add('bettercopper:copper_axe')
      event.get('minecraft:shovels')
      .add('bettercopper:copper_shovel')
      event.get('minecraft:hoes')
      .add('bettercopper:copper_hoe')

      event.get('farmersdelight:straw_harvesters')
      .add('kubejs:copper_knife')
      .remove('farmersdelight:diamond_knife')
      .remove('farmersdelight:golden_knife')
      .remove('farmersdelight:netherite_knife')

      event.get('farmersdelight:tools/knives')
      .add('kubejs:copper_knife')
      .remove('farmersdelight:diamond_knife')
      .remove('farmersdelight:golden_knife')
      .remove('farmersdelight:netherite_knife')

      event.get('c:swords')
      .add('bettercopper:copper_sword')

      event.get('minecraft:swords')
      .add('bettercopper:copper_sword')

      event.get('forge:nuggets/copper')
      .remove('caverns_and_chasms:copper_nugget')

      event.get('inmis:backpacks')
      .add('inmis:frayed_backpack')
      .add('inmis:plated_backpack')
      .add('inmis:gilded_backpack')
      .add('inmis:bejeweled_backpack')
      .add('inmis:blazing_backpack')
      .add('inmis:withered_backpack')
      .add('inmis:endless_backpack')

      event.get('quark:stone_tool_materials')
      .add('aether:holystone')
      .add('deep_aether:clorite')
      .add('deep_aether:aseterite')

      .add('kubejs:fired_cobblestone')

      .add('kubejs:cobbled_granite')
      .add('kubejs:cobbled_diorite')
      .add('kubejs:cobbled_andesite')

      .add('kubejs:fired_cobbled_granite')
      .add('kubejs:fired_cobbled_diorite')
      .add('kubejs:fired_cobbled_andesite')

      event.removeAll('caverns_and_chasms:experience_boost_items')

      event.get('forge:cobblestone')
      .add([
        'kubejs:fired_cobblestone', 
        'kubejs:mossy_fired_cobblestone', 
        'kubejs:mossy_cobbled_granite', 
        'kubejs:cobbled_granite', 
        'kubejs:fired_cobbled_granite', 
        'kubejs:mossy_fired_cobbled_granite', 
        'kubejs:mossy_cobbled_andesite', 
        'kubejs:cobbled_andesite', 
        'kubejs:fired_cobbled_andesite', 
        'kubejs:cobbled_diorite', 
        'kubejs:mossy_cobbled_diorite', 
        'kubejs:fired_cobbled_diorite', 
        'kubejs:mossy_fired_cobbled_diorite', 
        'kubejs:mossy_fired_cobbled_andesite', 
        'kubejs:fired_cobbled_deepslate', 
        'kubejs:mossy_fired_cobbled_deepslate', 
        'kubejs:mossy_cobbled_deepslate'
      ])



      event.get('minecraft:planks')
      .add('aether:skyroot_planks')
      .add('deep_aether:roseroot_planks')
      .add('deep_aether:yagroot_planks')
      .add('deep_aether:cruderoot_planks')
      .add('deep_aether:sunroot_planks')
      .add('deep_aether:conberry_planks')


      //DUNGEON TAGS
      event.get('aether:bronze_dungeon_loot')
      .remove('aether:cloud_staff')
      .remove('aether:lightning_knife')
      .add('aether:swet_cape')

      event.get('deep_aether:brass_dungeon_loot')
      .remove('deep_aether:aercloud_necklace')
      .remove('aether:swet_cape')
      .remove('deep_aether:wind_shield')
      .remove('deep_aether:cloud_cape')
      .remove('deep_aether:music_disc_cyclone')

      event.get('aether:silver_dungeon_loot')
      .remove('aether:regeneration_stone')
      .add('aether:lightning_knife')
      .add('aether:cloud_staff')
      .add('deep_aether:wind_shield')
      .add('deep_aether:music_disc_cyclone')

      event.get('aether:gold_dungeon_loot')
      .remove('aether:vampire_blade')
      .remove('aether:pig_slayer')
      .add('deep_aether:cloud_cape')

      event.get('deep_aether:flawless_items')
      .remove(['deep_aether:floaty_scarf', 'deep_aether:sun_core', 'deep_aether:medal_of_honor', 'deep_aether:slider_eye', 'deep_aether:aerwhale_saddle'])



      event.get('minecraft:andesite')
      .add('minecraft:andesite')
      .add('kubejs:cobbled_andesite')

      event.get('minecraft:diorite')
      .add('minecraft:diorite')
      .add('kubejs:cobbled_diorite')

      event.get('minecraft:granite')
      .add('minecraft:granite')
      .add('kubejs:cobbled_granite')





    //REMOVING TAGS
    event.get('forge:cooked_eggs')
      .remove('incubation:fried_egg')
      .remove('naturalist:cooked_egg')

    event.get('naturalist:eggs')
      .remove('naturalist:duck_egg')
      .remove('minecraft:egg')

    event.get('c:cooked_eggs')
      .remove('naturalist:cooked_egg')

    event.get('forge:eggs/cooked')
      .remove('naturalist:cooked_egg')

      event.get('forge:rods')
      .remove('aether:skyroot_stick')
      event.get('forge:rods/wooden')
      .remove('aether:skyroot_stick')
      event.get('deep_aether:all_sticks')
      .remove('aether:skyroot_stick')

      event.get('aether:skyroot_stick')
      .add('minecraft:stick')

      event.get('forge:ingots/iron')
      .remove('betterend:thallasium_ingot')

      event.get('forge:ingots/iron')
      .remove('betterend:thallasium_ingot')

      event.get('c:ingots')
      .remove('deep_aether:stratus_ingot')
      .add(['betterend:aeternium_ingot', 'betterend:thallasium_ingot', 'kubejs:meteorite_ingot'])

      event.get('forge:ingots')
      .remove('deep_aether:stratus_ingot')
      .add(['betterend:aeternium_ingot', 'betterend:thallasium_ingot', 'kubejs:meteorite_ingot'])

      event.get('supplementaries:ropes')
      .remove('supplementaries:rope') //fuck you supplementaries rope lmao

      event.get('forge:ropes')
      .remove('supplementaries:rope') //fuck you supplementaries rope lmao

      event.get('minecraft:beacon_payment_items')
      .remove('deep_aether:skyjade')
      .remove('deep_aether:stratus_ingot')
      .remove('aether:enchanted_gravitite')

      event.get('deep_aether:skyjade_repairing')
      .remove('deep_aether:skyjade')
      .add('minecraft:emerald')

      event.get('aether:treated_as_aether_item')
      .remove('deep_aether:skyjade')
      .remove('deep_aether:stratus_ingot')

      event.removeAll('immersive_weathering:bark')
      event.removeAll('immersive_weathering:scales')

      event.get('aether:processed/gravitite')
      .add('kubejs:meteorite_ingot')
      .remove('aether:enchanted_gravitite')

      event.get('aether:gravitite_repairing')
      .add('kubejs:meteorite_ingot')
      .remove('aether:enchanted_gravitite')

      event.get('forge:storage_blocks')
      .add('aether:enchanted_gravitite')



      //ADDING TAGS
      event.get('minecraft:wip')
      .add([
        'naturalist:duck', 
        'naturalist:cooked_duck', 
        'naturalist:bear_spawn_egg',
        'naturalist:firefly_spawn_egg',
        'naturalist:coral_snake_spawn_egg',
        'naturalist:rattlesnake_spawn_egg',
        'naturalist:bluejay_spawn_egg',
        'naturalist:cardinal_spawn_egg',
        'naturalist:canary_spawn_egg',
        'naturalist:robin_spawn_egg',
        'naturalist:finch_spawn_egg',
        'naturalist:caterpillar_spawn_egg',
        'naturalist:rhino_spawn_egg',
        'naturalist:lion_spawn_egg',
        'naturalist:elephant_spawn_egg',
        'naturalist:zebra_spawn_egg',
        'naturalist:hippo_spawn_egg',
        'naturalist:vulture_spawn_egg',
        'naturalist:dragonfly_spawn_egg',
        'naturalist:catfish_spawn_egg',
        'naturalist:bass_spawn_egg',
        'naturalist:lizard_spawn_egg',
        'naturalist:duck_spawn_egg'
      ])

      event.get('immersive_weathering:bark')
      .add('farmersdelight:tree_bark')

      event.get('immersive_weathering:scales')
      .add('farmersdelight:tree_bark')

      event.get('minecraft:piglin_loved')
      .add('aether:golden_ring')
      .add('aether:golden_pendant')

      event.get('minecraft:trimmable_armor')
      .add('betterend:thallasium_helmet', 'betterend:terminite_helmet', 'betterend:aeternium_helmet')
      .add('betterend:thallasium_chestplate', 'betterend:terminite_chestplate', 'betterend:aeternium_chestplate')
      .add('betterend:thallasium_leggings', 'betterend:terminite_leggings', 'betterend:aeternium_leggings')
      .add('betterend:thallasium_boots', 'betterend:terminite_boots', 'betterend:aeternium_boots')

      event.get('minecraft:trim_materials')
      .add('betterend:terminite_ingot')
      .add('betterend:thallasium_ingot')
      .add('kubejs:meteorite_ingot')
      .remove('aether:enchanted_gravitite')
      .remove('deep_aether:skyjade')
      .remove('deep_aether:stratus_ingot')
      .remove('caverns_and_chasms:necromium_ingot')
      .remove('caverns_and_chasms:living_flesh')
      .remove('caverns_and_chasms:spinel')

      event.get('minecraft:nether_planks')
      .add('minecraft:warped_planks')
      .add('minecraft:crimson_planks')

      event.get('minecraft:nether_wood')
      .add('minecraft:warped_stem')
      .add('minecraft:crimson_stem')
      .add('minecraft:stripped_warped_stem')
      .add('minecraft:stripped_crimson_stem')

      event.get('curios:back')
      .add(['inmis:frayed_backpack', 'inmis:plated_backpack', 'inmis:gilded_backpack', 'inmis:bejeweled_backpack', 'inmis:blazing_backpack', 'inmis:withered_backpack', 'inmis:endless_backpack'])

      event.get('curios:charm')
      .add(['aether:shield_of_repulsion', 'deep_aether:wind_shield'])

      event.get('curios:body')
      .remove(['aether:shield_of_repulsion', 'deep_aether:wind_shield'])

      event.get('minecraft:basic_fruit')
      .add(['minecraft:apple', 'kubejs:cherry', 'kubejs:cranberries', 'kubejs:plum', 'kubejs:durian', 'kubejs:pear', 'kubejs:mango', 'minecraft:melon_slice', 'farmersdelight:tomato'])

      event.get('forge:foods')
      .add(['kubejs:cherry', 
        'kubejs:cranberries', 
        'kubejs:plum', 
        'kubejs:durian', 
        'kubejs:pear', 
        'kubejs:mango', 
        'kubejs:chestnut', 
        'kubejs:frog_leg', 
        'kubejs:golden_frog_leg', 
        'kubejs:cooked_frog_leg'])

      event.get('aether:aether_logs')
      .add(['aether:skyroot_log', 
        'aether:skyroot_wood', 
        'aether:stripped_skyroot_log', 
        'aether:stripped_skyroot_wood', 
        'deep_aether:yagroot_log', 
        'deep_aether:yagroot_wood', 
        'deep_aether:stripped_yagroot_log', 
        'deep_aether:stripped_yagroot_wood', 
        'deep_aether:sunroot_log', 
        'deep_aether:sunroot_wood', 
        'deep_aether:stripped_sunroot_log', 
        'deep_aether:stripped_sunroot_wood', 
        'deep_aether:cruderoot_log', 
        'deep_aether:cruderoot_wood', 
        'deep_aether:stripped_cruderoot_log', 
        'deep_aether:stripped_cruderoot_wood', 
        'deep_aether:conberry_log', 
        'deep_aether:conberry_wood', 
        'deep_aether:stripped_conberry_log', 
        'deep_aether:stripped_conberry_wood', 
        'deep_aether:roseroot_log', 
        'deep_aether:roseroot_wood', 
        'deep_aether:stripped_roseroot_log', 
        'deep_aether:stripped_roseroot_wood'])


      //BETTEREND SPECIAL TAGS
      event.get('minecraft:opal_pickaxe')
      .add('betterend:terminite_pickaxe')

      event.get('minecraft:titanium_pickaxe')
      .add('betterend:aeternium_pickaxe')

      event.get('betterend:void_logs')
      .add(['betterend:pythadendron_log', 'betterend:end_lotus_log', 'betterend:lacugrove_log', 'betterend:dragon_tree_log', 'betterend:tenanea_log', 'betterend:helix_tree_log', 'betterend:umbrella_tree_log', 'betterend:jellyshroom_log', 'betterend:lucernia_log'])
      .add(['betterend:pythadendron_wood', 'betterend:end_lotus_wood', 'betterend:lacugrove_wood', 'betterend:dragon_tree_wood', 'betterend:tenanea_wood', 'betterend:helix_tree_wood', 'betterend:umbrella_tree_wood', 'betterend:jellyshroom_wood', 'betterend:lucernia_wood'])

      event.get('betterend:void_stone_types')
      .add(['quark:myalite', 'quark:dusky_myalite', 'betterend:flavolite', 'betterend:violecite', 'betterend:sulphuric_rock', 'betterend:umbralith', 'betterend:virid_jadestone', 'betterend:azure_jadestone', 'betterend:sandy_jadestone'])

      //---FOOD TYPE TAGS---
      //LIGHTWEIGHT
      event.get('minecraft:lightweight')
      .add([
        'minecraft:apple', 
        'minecraft:melon_slice', 
        'minecraft:sweet_berries', 
        'minecraft:glow_berries', 
        'betterend:sweet_berry_jelly', 
        'betterend:shadow_berry_jelly', 
        'betterend:blossom_berry_jelly', 
        'betterend:blossom_berry', 
        'betterend:chorus_mushroom_cooked', 
        'betterend:bolux_mushroom_cooked',
        'betterend:bolux_mushroom',
        'aether:blue_berry', 
        'aether:enchanted_berry', 
        'aether:white_apple', 
        'deep_aether:goldenleaf_berries', 
        'deep_aether:blue_squash_slice', 
        'deep_aether:green_squash_slice', 
        'deep_aether:purple_squash_slice',
        'upgrade_aquatic:mulberry', 
        'minecraft:chorus_fruit', 
        'quark:ancient_fruit', 
        'minecraft:carrot', 
        'minecraft:potato', 
        'minecraft:beetroot', 
        'upgrade_aquatic:boiled_pickerelweed', 
        'minecraft:dried_kelp', 
        'farmersdelight:pumpkin_slice', 
        'farmersdelight:raw_pasta', 
        'farmersdelight:fried_egg', 
        'farmersdelight:onion', 
        'farmersdelight:tomato', 
        'farmersdelight:cabbage', 
        'farmersdelight:cabbage_leaf', 
        'farmersdelight:wheat_dough', 
        'farmersdelight:tomato_sauce', 
        'farmersdelight:pie_crust',
        'betterend:chorus_mushroom_raw',
        'kubejs:cherry', 
        'kubejs:cranberries', 
        'kubejs:plum', 
        'kubejs:durian', 
        'kubejs:pear', 
        'kubejs:mango', 
        'kubejs:chestnut'
    ])

      //HEARTY
      event.get('minecraft:hearty')
      .add([
        'minecraft:bread', 
        'minecraft:beef', 
        'minecraft:cooked_beef', 
        'minecraft:porkchop', 
        'minecraft:mutton', 
        'minecraft:chicken', 
        'minecraft:rabbit', 
        'minecraft:cod', 
        'minecraft:salmon', 
        'minecraft:baked_potato', 
        'upgrade_aquatic:pike', 
        'upgrade_aquatic:perch', 
        'upgrade_aquatic:lionfish', 
        'farmersdelight:minced_beef', 
        'farmersdelight:beef_patty', 
        'farmersdelight:chicken_cuts', 
        'farmersdelight:cooked_chicken_cuts', 
        'farmersdelight:bacon', 
        'farmersdelight:cooked_bacon', 
        'farmersdelight:cod_slice', 
        'farmersdelight:cooked_cod_slice', 
        'farmersdelight:salmon_slice', 
        'farmersdelight:cooked_salmon_slice', 
        'farmersdelight:mutton_chops', 
        'farmersdelight:cooked_mutton_chops', 
        'farmersdelight:ham', 
        'farmersdelight:smoked_ham',
        'quark:crab_leg', 
        'deep_aether:raw_quail', 
        'deep_aether:raw_aerglow_fish', 
        'betterend:end_fish_cooked', 
        'betterend:end_fish_raw', 
        'minecraft:cooked_porkchop', 
        'minecraft:cooked_mutton', 
        'minecraft:cooked_chicken', 
        'minecraft:cooked_rabbit', 
        'minecraft:cooked_cod', 
        'minecraft:cooked_salmon', 
        'upgrade_aquatic:cooked_pike', 
        'upgrade_aquatic:cooked_perch', 
        'upgrade_aquatic:cooked_lionfish', 
        'quark:cooked_crab_leg', 
        'kubejs:cooked_frog_leg',
        'kubejs:frog_leg', 
        'deep_aether:cooked_quail', 
        'deep_aether:cooked_aerglow_fish',
        'naturalist:venison',
        'naturalist:cooked_venison',
        'naturalist:duck',
        'naturalist:cooked_duck'
    ])

      //FINGERFOOD
      event.get('minecraft:fingerfood')
      .add([
        'buzzier_bees:glazed_porkchop', 
        'buzzier_bees:honey_apple', 
        'buzzier_bees:honey_bread', 
        'incubation:scrambled_eggs', 
        'upgrade_aquatic:mulberry_bread', 
        'farmersdelight:barbecue_stick', 
        'farmersdelight:egg_sandwich', 
        'farmersdelight:chicken_sandwich', 
        'farmersdelight:hamburger', 
        'farmersdelight:bacon_sandwich', 
        'farmersdelight:mutton_wrap', 
        'farmersdelight:dumplings', 
        'farmersdelight:stuffed_potato', 
        'farmersdelight:cabbage_rolls', 
        'farmersdelight:salmon_roll', 
        'farmersdelight:cod_roll', 
        'farmersdelight:kelp_roll', 
        'farmersdelight:kelp_roll_slice', 
        'farmersdelight:cooked_rice',
        'minecraft:mushroom_stew',
        'minecraft:beetroot_soup'
    ])

      //MEAL
      event.get('minecraft:meal')
      .add([
        'farmersdelight:fruit_salad', 
        'farmersdelight:nether_salad', 
        'farmersdelight:bone_broth', 
        'farmersdelight:beef_stew', 
        'farmersdelight:chicken_soup', 
        'farmersdelight:vegetable_soup', 
        'farmersdelight:fish_stew', 
        'farmersdelight:fried_rice', 
        'farmersdelight:mixed_salad', 
        'farmersdelight:pumpkin_soup', 
        'farmersdelight:baked_cod_stew', 
        'farmersdelight:noodle_soup', 
        'farmersdelight:bacon_and_eggs', 
        'farmersdelight:pasta_with_meatballs', 
        'farmersdelight:pasta_with_mutton_chop', 
        'farmersdelight:mushroom_rice', 
        'farmersdelight:roasted_mutton_chops', 
        'farmersdelight:vegetable_noodles', 
        'farmersdelight:steak_and_potatoes', 
        'farmersdelight:ratatouille', 
        'farmersdelight:squid_ink_pasta', 
        'farmersdelight:grilled_salmon',
        'minecraft:rabbit_stew'
    ])

      //FEAST
      event.get('minecraft:feast')
      .add([
        'farmersdelight:roast_chicken_block', 
        'farmersdelight:roast_chicken', 
        'farmersdelight:stuffed_pumpkin_block', 
        'farmersdelight:stuffed_pumpkin', 
        'farmersdelight:honey_glazed_ham_block', 
        'farmersdelight:honey_glazed_ham', 
        'farmersdelight:shepherds_pie_block', 
        'farmersdelight:shepherds_pie', 
        'farmersdelight:rice_roll_medley_block'
    ])

      //SWEET
      event.get('minecraft:sweet')
      .add([
        'minecraft:cake', 
        'supplementaries:pancake', 
        'farmersdelight:sweet_berry_cheesecake', 
        'farmersdelight:cake_slice', 
        'farmersdelight:sweet_berry_cheesecake_slice', 
        'farmersdelight:chocolate_pie', 
        'upgrade_aquatic:mulberry_pie', 
        'minecraft:pumpkin_pie', 
        'farmersdelight:apple_pie_slice', 
        'farmersdelight:apple_pie', 
        'betterend:cave_pumpkin_pie', 
        'supplementaries:candy', 
        'aether:candy_cane', 
        'aether:gingerbread_man', 
        'farmersdelight:sweet_berry_cookie', 
        'farmersdelight:honey_cookie', 
        'minecraft:cookie', 
        'farmersdelight:melon_popsicle', 
        'farmersdelight:glow_berry_custard',
        'farmersdelight:chocolate_pie_slice'
    ])

      //SPECIAL
      event.get('minecraft:special')
      .add([
        'minecraft:golden_apple', 
        'minecraft:enchanted_golden_apple', 
        'minecraft:golden_carrot', 
        'kubejs:golden_frog_leg', 
        'aether:life_shard', 
        'aether:healing_stone', 
        'aether:golden_gummy_swet', 
        'minecraft:suspicious_stew', 
        'aether:blue_gummy_swet',
        'minecraft:glistering_melon_slice',
        'immersive_weathering:golden_moss_clump',
        'immersive_weathering:enchanted_golden_moss_clump'
    ])

    //GROSS
    event.get('minecraft:gross')
    .add([
      'minecraft:rotten_flesh', 
      'minecraft:pufferfish', 
      'minecraft:spider_eye', 
      'minecraft:poisonous_potato'
  ])

  })

  //EXPERIMENTAL, FOR LATER - OBTAINABLE END PORTAL FRAMES
  ServerEvents.tags('block', event => {

      event.get('bettercopper:incorrect_for_copper_tool')
      .add('aether:gravitite_ore')
      .add('minecraft:diamond_ore')
      .add('minecraft:deepslate_diamond_ore')

      event.get('moderner_beta:overworld_carver_replaceables')
      .add('kubejs:fired_bricks')
      .add('kubejs:fired_cobblestone')
      .add('kubejs:fired_fieldstone')

      event.get('c:ores')
      .add('kubejs:aquamarine_ore')
      .add('kubejs:opal_ore')
      .add('kubejs:titanium_ore')
      .add('deep_aether:skyjade_ore')

      event.get('forge:ores')      
      .add('kubejs:aquamarine_ore')
      .add('kubejs:opal_ore')
      .add('kubejs:titanium_ore')
      .add('deep_aether:skyjade_ore')

    event.get('forge:storage_blocks')
    .add('aether:enchanted_gravitite')

   event.get('minecraft:end_portal_frame')
      .add('minecraft:end_portal_frame')

   event.get('minecraft:mineable/pickaxe')
      .add('minecraft:end_portal_frame')


      event.get('farmersdelight:terrain')
      .add('kubejs:glowcelium')
      event.get('naturalist:dragonflies_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('naturalist:alligator_egg_layable_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:bamboo_plantable_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:big_dripleaf_placeable')
      .add('kubejs:glowcelium')
      event.get('minecraft:mangrove_logs_can_grow_through')
      .add('kubejs:glowcelium')
      event.get('minecraft:foxes_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('naturalist:ducks_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('quark:fallen_log_can_spawn_on')
      .add('kubejs:glowcelium')
      event.get('c:grass_soil')
      .add('kubejs:glowcelium')
      event.get('minecraft:frogs_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:lush_ground_replaceable')
      .add('kubejs:glowcelium')
      event.get('minecraft:goats_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:snow_layer_can_survive_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:moss_replaceable')
      .add('kubejs:glowcelium')
      event.get('minecraft:mud_replaceable')
      .add('kubejs:glowcelium')
      event.get('naturalist:tortoise_egg_layable_on')
      .add('kubejs:glowcelium')
      event.get('naturalist:fireflies_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('naturalist:cattail_placeable')
      .add('kubejs:glowcelium')
      event.get('minecraft:nether_carver_replaceables')
      .add('kubejs:glowcelium')
      event.get('minecraft:wolves_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:azalea_grows_on')
      .add('kubejs:glowcelium')
      event.get('naturalist:butterflies_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:sculk_replaceable')
      .add('kubejs:glowcelium')
      event.get('minecraft:dead_bush_may_place_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:enderman_holdable')
      .add('kubejs:glowcelium')
      event.get('minecraft:mangrove_roots_can_grow_through')
      .add('kubejs:glowcelium')
      event.get('aether:ruined_portal_ground_replaceable')
      .add('kubejs:glowcelium')
      event.get('minecraft:animals_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('minecraft:sniffer_diggable_block')
      .add('kubejs:glowcelium')
      event.get('minecraft:azalea_root_replaceable')
      .add('kubejs:glowcelium')
      event.get('minecraft:overworld_carver_replaceables')
      .add('kubejs:glowcelium')
      event.get('minecraft:rabbits_spawnable_on')
      .add('kubejs:glowcelium')
      event.get('forge:soil/grass')
      .add('kubejs:glowcelium')
      event.get('minecraft:dirt')
      .add('kubejs:glowcelium')
      event.get('minecraft:mineable/shovel')
      .add('kubejs:glowcelium')
      event.get('upgrade_aquatic:pickerelweed_placeable')
      .add('kubejs:glowcelium')
      event.get('minecraft:sculk_replaceable_world_gen')
      .add('kubejs:glowcelium')
      event.get('forge:soil/grass')
      .add('kubejs:glowcelium')
      event.get('quark:underground_biome_replaceable')
      .add('kubejs:glowcelium')

  })