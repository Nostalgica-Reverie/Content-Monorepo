const $Rarity = Java.loadClass("net.minecraft.world.item.Rarity")
const $UnaryOperator = Java.loadClass("java.util.function.UnaryOperator")
const $UtilsJS = Java.loadClass("dev.latvian.mods.kubejs.util.UtilsJS")
const $Style = Java.loadClass("net.minecraft.network.chat.Style")
const withColorMethod = $Style.EMPTY.class.declaredMethods.filter((method) => method.name.includes("m_131148_"))[0]

function createRarity (/** @type {string} */ name, /** @type {number} */ colorCode) {
  let color = $UtilsJS.makeFunctionProxy("startup", $UnaryOperator, (style) => {
    return withColorMethod.invoke(style, Color.of(colorCode).createTextColorJS())
  })
  return $Rarity["create(java.lang.String,java.util.function.UnaryOperator)"](name, color)
}

// create your rarities here
createRarity("UNIQUE", 0x55ff55)
createRarity("ARTIFACT", 0xffaa01)

//EVERYTHING WRITTEN ABOVE WAS WRITTEN BY -- unknownrj -- ALL CREDIT GOES TO HIM

ItemEvents.modification(event => {


  // ---------- COMMON ----------

    event.modify([
      'aether:aerogel',
      'aether:aerogel_slab',
      'aether:aerogel_stairs',
      'aether:aerogel_wall'
    ], item => {
      item.rarity = 'COMMON'
    })








    //---------- UNCOMMON ----------
    event.modify('endrem:undead_soul', item => {
      item.rarity = 'UNCOMMON'
    })
    event.modify([
      'caverns_and_chasms:exile_armor_trim_smithing_template', 
      'minecraft:spire_armor_trim_smithing_template', 
      'minecraft:sentry_armor_trim_smithing_template', 
      'minecraft:vex_armor_trim_smithing_template', 
      'minecraft:wild_armor_trim_smithing_template', 
      'minecraft:coast_armor_trim_smithing_template', 
      'minecraft:dune_armor_trim_smithing_template',
      'minecraft:rib_armor_trim_smithing_template', 
      'minecraft:snout_armor_trim_smithing_template', 
      'minecraft:tide_armor_trim_smithing_template', 
      'minecraft:silence_armor_trim_smithing_template', 
      'minecraft:ward_armor_trim_smithing_template'], item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:echo_shard', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify([
        'minecraft:music_disc_13', 
        'minecraft:music_disc_cat', 
        'minecraft:music_disc_blocks', 
        'minecraft:music_disc_chirp', 
        'minecraft:music_disc_far', 
        'minecraft:music_disc_mall', 
        'minecraft:music_disc_mellohi', 
        'minecraft:music_disc_stal', 
        'minecraft:music_disc_strad', 
        'minecraft:music_disc_ward', 
        'minecraft:music_disc_11', 
        'minecraft:music_disc_wait', 
        'minecraft:music_disc_relic', 
        'quark:music_disc_endermosh', 
        'minecraft:music_disc_otherside', 
        'minecraft:music_disc_5', 
        'minecraft:music_disc_pigstep', 
        'upgrade_aquatic:music_disc_atlantis', 
        'quark:music_disc_chatter', 
        'quark:music_disc_crickets', 
        'quark:music_disc_clock', 
        'quark:music_disc_fire', 
        'quark:music_disc_wind', 
        'quark:music_disc_rain', 
        'quark:music_disc_ocean', 
        'quark:music_disc_drips', 
        'supplementaries:music_disc_heave_ho',
        'aether:music_disc_aether_tune', 
        'aether:music_disc_ascending_dawn', 
        'deep_aether:music_disc_a_morning_wish', 
        'deep_aether:music_disc_nabooru', 
        'betterend:music_disc_strange_and_alien', 
        'betterend:music_disc_grasping_at_stars', 
        'betterend:music_disc_endseeker', 
        'betterend:music_disc_eo_dracona', 
        'aether:music_disc_sliders_wrath'
      ], item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify([
        'aether:obsidian_gloves', 
        'aether:obsidian_boots', 
        'aether:obsidian_leggings', 
        'aether:obsidian_chestplate', 
        'aether:obsidian_helmet',
        'aether:pig_slayer'
      ], item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:trident', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:phantom_membrane', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('upgrade_aquatic:thrasher_tooth', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:suspicious_stew', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:potion', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:splash_potion', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:lingering_potion', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('caverns_and_chasms:tether_potion', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:sculk_catalyst', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:sculk_shrieker', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:sculk_sensor', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:calibrated_sculk_sensor', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:recovery_compass', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:ender_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('minecraft:goat_horn', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify([
        'farmersdelight:roast_chicken_block', 
        'farmersdelight:roast_chicken', 
        'farmersdelight:stuffed_pumpkin_block', 
        'farmersdelight:stuffed_pumpkin', 
        'farmersdelight:honey_glazed_ham_block', 
        'farmersdelight:honey_glazed_ham', 
        'farmersdelight:shepherds_pie_block', 
        'farmersdelight:shepherds_pie', 
        'farmersdelight:rice_roll_medley_block',
        'minecraft:golden_carrot',
        'kubejs:golden_frog_leg'
    ], item => {
        item.rarity = 'UNCOMMON'
      })

      //Eyes
      event.modify('endrem:black_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('endrem:cold_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('endrem:corrupted_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('endrem:lost_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('endrem:old_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('endrem:rogue_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('endrem:cold_eye', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('endrem:exotic_eye', item => {
        item.rarity = 'UNCOMMON'
      })

    //Voidscapes Stuff
    event.modify('betterend:thallasium_ore', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_block', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_ingot', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_nugget', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_raw', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_sword', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_pickaxe', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_axe', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_shovel', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_hoe', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_helmet', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_chestplate', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_leggings', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('betterend:thallasium_boots', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('quark:dragon_scale', item => {
        item.rarity = 'UNCOMMON'
      })
      event.modify('inmis:endless_backpack', item => {
        item.rarity = 'UNCOMMON'
      })

    //---------- RARE ----------

    event.modify('minecraft:nether_star', item => {
        item.rarity = 'RARE'
      })
      event.modify('supplementaries:lunch_basket', item => {
        item.rarity = 'RARE'
      })

    //Eyes
      event.modify('endrem:nether_eye', item => {
        item.rarity = 'RARE'
      })
      event.modify('endrem:cursed_eye', item => {
        item.rarity = 'RARE'
      })
      event.modify('endrem:undead_eye', item => {
        item.rarity = 'RARE'
      })

    //Voidscapes Stuff
    event.modify('kubejs:opal_ore', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_ingot', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_block', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_sword', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_pickaxe', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_axe', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_shovel', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_hoe', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_helmet', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_chestplate', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_leggings', item => {
        item.rarity = 'RARE'
      })
      event.modify('betterend:terminite_boots', item => {
        item.rarity = 'RARE'
      })


      event.modify('quark:ancient_fruit', item => {
        item.rarity = 'RARE'
      })

    //---------- EPIC ----------

    //Eyes
    event.modify('endrem:magical_eye', item => {
        item.rarity = 'EPIC'
      })
      event.modify('endrem:wither_eye', item => {
        item.rarity = 'EPIC'
      })
      event.modify('endrem:guardian_eye', item => {
        item.rarity = 'EPIC'
      })

//Voidscapes Stuff
event.modify('kubejs:titanium_ore', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_ingot', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_block', item => {
    item.rarity = 'EPIC'
  })
  event.modify('kubejs:raw_titanium', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_sword', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_pickaxe', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_axe', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_shovel', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_hoe', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_helmet', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_chestplate', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_leggings', item => {
    item.rarity = 'EPIC'
  })
  event.modify('betterend:aeternium_boots', item => {
    item.rarity = 'EPIC'
  })
  event.modify('minecraft:dragon_head', item => {
    item.rarity = 'EPIC'
  })

    //---------- UNIQUE ----------
    event.modify('endrem:witch_eye', item => {
        item.rarity = 'UNIQUE'
      })
      event.modify('endrem:evil_eye', item => {
        item.rarity = 'UNIQUE'
      })
      event.modify('endrem:cryptic_eye', item => {
        item.rarity = 'UNIQUE'
      })

      event.modify('kubejs:valkyrie_feather', item => {
        item.rarity = 'UNIQUE'
      })
      event.modify('kubejs:slider_rubble', item => {
        item.rarity = 'UNIQUE'
      })
      event.modify('kubejs:serpant_horn', item => {
        item.rarity = 'UNIQUE'
      })


    //---------- ARTIFACT ----------
      event.modify('hookshot:hookshot', item => {
        item.rarity = 'ARTIFACT'
      })
      event.modify('quark:forgotten_hat', item => {
        item.rarity = 'ARTIFACT'
      })
      event.modify('supplementaries:antique_ink', item => {
        item.rarity = 'ARTIFACT'
      })
      event.modify([
        'minecraft:angler_pottery_sherd', 
        'minecraft:archer_pottery_sherd', 
        'minecraft:arms_up_pottery_sherd', 
        'minecraft:blade_pottery_sherd', 
        'minecraft:brewer_pottery_sherd', 
        'minecraft:burn_pottery_sherd', 
        'minecraft:danger_pottery_sherd', 
        'minecraft:explorer_pottery_sherd', 
        'minecraft:friend_pottery_sherd', 
        'minecraft:heart_pottery_sherd', 
        'minecraft:heartbreak_pottery_sherd', 
        'minecraft:howl_pottery_sherd', 
        'minecraft:miner_pottery_sherd', 
        'minecraft:mourner_pottery_sherd', 
        'minecraft:plenty_pottery_sherd', 
        'upgrade_aquatic:predator_pottery_sherd', 
        'minecraft:prize_pottery_sherd', 
        'minecraft:sheaf_pottery_sherd', 
        'minecraft:shelter_pottery_sherd', 
        'minecraft:skull_pottery_sherd', 
        'minecraft:snort_pottery_sherd',

        'minecraft:wayfinder_armor_trim_smithing_template',
        'minecraft:raiser_armor_trim_smithing_template',
        'minecraft:shaper_armor_trim_smithing_template',
        'minecraft:host_armor_trim_smithing_template',
        'minecraft:spire_armor_trim_smithing_template',
        'minecraft:eye_armor_trim_smithing_template',
      ], item => {
        item.rarity = 'ARTIFACT'
      })


})