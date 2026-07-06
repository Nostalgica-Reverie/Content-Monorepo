// priority: 0

// Visit the wiki for more info - https://kubejs.com/

console.info('Hello, World! (Loaded client scripts)')

//TOOLS & ARMOR
ItemEvents.tooltip(event => {

    // Shell
    event.add(['minecraft:turtle_helmet'], '\u00A79' + 'Water Breathing')

    // Agility Cape
    event.add(['aether:agility_cape'], '\u00A79' + 'Increases step height')

    //Silver
    event.add(['caverns_and_chasms:silver_helmet'], '\u00A79' + 'Immunity to Blindness, Darkness, & Nausea')
    event.add(['caverns_and_chasms:silver_chestplate'], '\u00A79' + 'Immunity to Poison & Mining Fatigue')
    event.add(['caverns_and_chasms:silver_leggings'], '\u00A79' + 'Immunity to Weakness & Wither')
    event.add(['caverns_and_chasms:silver_boots'], '\u00A79' + 'Immunity to Slowness')

    // Chainmail
    event.add(['minecraft:chainmail_helmet', 'minecraft:chainmail_chestplate', 'minecraft:chainmail_leggings', 'minecraft:chainmail_boots'], '\u00A79' + 'Defense Against Projectiles')

    // Iron
    event.add(['minecraft:iron_helmet', 'minecraft:iron_chestplate', 'minecraft:iron_leggings', 'minecraft:iron_boots', 'aether:iron_gloves', 'minecraft:iron_pickaxe', 'minecraft:iron_axe', 'minecraft:iron_shovel', 'minecraft:iron_hoe', 'minecraft:iron_sword'], '\u00A79' + 'Reforming Durability')
    
    // Diamond
    event.add(['minecraft:diamond_helmet', 'minecraft:diamond_chestplate', 'minecraft:diamond_leggings', 'minecraft:diamond_boots'], '\u00A79' + 'Defense Against Explosions')
    // Diamond Tools
    event.add(['minecraft:diamond_sword', 'minecraft:diamond_pickaxe', 'minecraft:diamond_shovel', 'minecraft:diamond_axe', 'minecraft:diamond_hoe'], '\u00A79' + 'Hasty Mining')
    
    // Tungsten
    event.add(['minecraft:netherite_helmet', 'minecraft:netherite_chestplate', 'minecraft:netherite_leggings', 'minecraft:netherite_boots'], '\u00A79' + 'Defense Against Fire & Increased Durability')
    // Tungsten Tools
    event.add(['minecraft:netherite_sword', 'minecraft:netherite_pickaxe', 'minecraft:netherite_axe', 'minecraft:netherite_shovel', 'minecraft:netherite_hoe'], '\u00A79' + 'Increased Durability')

    // Copper Tools
    event.add(['bettercopper:copper_sword', 'bettercopper:copper_pickaxe', 'bettercopper:copper_shovel', 'bettercopper:copper_axe', 'bettercopper:copper_hoe', 'kubejs:copper_knife'], '\u00A79' + 'Increased Fortune')
    // Gold Tools
    event.add(['minecraft:golden_sword', 'minecraft:golden_pickaxe', 'minecraft:golden_shovel', 'minecraft:golden_axe', 'minecraft:golden_hoe'], '\u00A79' + 'Silk Touch')

    // Bismuth Tools
    event.add(['betterend:thallasium_sword', 'betterend:thallasium_pickaxe', 'betterend:thallasium_shovel', 'betterend:thallasium_axe', 'betterend:thallasium_hoe'], '\u00A79' + 'Vein Mining')



    // Zanite
    event.add(['aether:zanite_gloves', 'aether:zanite_sword', 'aether:zanite_pickaxe', 'aether:zanite_shovel', 'aether:zanite_axe', 'aether:zanite_hoe', 'aether:zanite_helmet', 'aether:zanite_chestplate', 'aether:zanite_leggings','aether:zanite_boots'], '\u00A79' + 'Higher Stats With Lower Durability')
     // Zanite Ring
     event.add(['aether:zanite_ring', 'aether:zanite_pendant', 'deep_aether:skyjade_ring'], '\u00A79' + 'Boosts Mining Speed At The Cost Of Durability')
     // Ice Ring
     event.add(['aether:ice_ring', 'aether:ice_pendant'], '\u00A79' + 'Freezes Water & Lava At The Cost Of Durability')

     // Amethyst Ring
     event.add(['aether:iron_ring'], '\u00A79' + 'Reveals nearby ores when mining')
    // Amethyst Ring
     event.add(['aether:iron_pendant'], '\u00A79' + 'Doubled TNT ignition power')

     // Lapis Ring
     event.add(['aether:golden_ring'], '\u00A79' + 'Thrown brews cluster on impact')
    // Gold Necklace
     event.add(['aether:golden_pendant'], '\u00A79' + 'Increases duration of positive status effects')



    // Opal Tools
    event.add(['betterend:terminite_sword', 'betterend:terminite_pickaxe', 'betterend:terminite_shovel', 'betterend:terminite_axe', 'betterend:terminite_hoe'], '\u00A79' + '3x3 Area Mining')


    // Meteorite
    event.add(['aether:gravitite_helmet', 'aether:gravitite_chestplate', 'aether:gravitite_leggings','aether:gravitite_boots', 'aether:gravitite_gloves'], '\u00A79' + 'Jump Boost (With Full Set)')
    // Meteorite Tools
    event.add(['aether:gravitite_sword', 'aether:gravitite_pickaxe', 'aether:gravitite_shovel', 'aether:gravitite_axe', 'aether:gravitite_hoe'], '\u00A79' + 'Auto-Smelt')

    // Valkyrie
    event.add(['aether:valkyrie_helmet', 'aether:valkyrie_chestplate', 'aether:valkyrie_leggings','aether:valkyrie_boots', 'aether:valkyrie_gloves'], '\u00A79' + 'Short Flight & Fall Damage Reduction (With Full Set)')

    // Phoenix
    event.add(['aether:phoenix_helmet', 'aether:phoenix_chestplate', 'aether:phoenix_leggings','aether:phoenix_boots', 'aether:phoenix_gloves'], '\u00A79' + 'Fire & Lava Immunity (With Full Set)')

    // Neptune
    event.add(['aether:neptune_helmet', 'aether:neptune_chestplate', 'aether:neptune_leggings','aether:neptune_boots', 'aether:neptune_gloves'], '\u00A79' + 'Faster Water Traversal (With Full Set)')

    // Sentry
    event.add(['aether:sentry_boots'], '\u00A79' + 'Fall Damage Immunity')

    // Valkyrie Cape
    event.add(['aether:valkyrie_cape'], '\u00A79' + 'Slow Falling')
    // Swet Cape
    event.add(['aether:swet_cape'], '\u00A79' + 'Passifies Swets')
    // Cloud Cape
    event.add(['deep_aether:cloud_cape'], '\u00A79' + 'Double Jump')

    // Aercloud Necklace
    event.add(['deep_aether:aercloud_necklace'], '\u00A79' + 'Normal Traversal on Clouds')

    // Shield of Repulsion
    event.add(['aether:shield_of_repulsion'], '\u00A79' + 'Deflects Projectiles')
    // Wild Shield
    event.add(['deep_aether:wind_shield'], '\u00A79' + 'Blocks All Damage (With Cooldown)')

    // Medal of Honor
    event.add(['deep_aether:medal_of_honor'], '\u00A79' + 'Resistance Against Undead Mobs')
    // Slider Fragment
    event.add(['deep_aether:slider_eye'], '\u00A79' + 'Ground Slam Ability (Left Ctrl)')

    // Emerald
    event.add(['deep_aether:skyjade_gloves', 'deep_aether:skyjade_sword', 'deep_aether:skyjade_pickaxe', 'deep_aether:skyjade_shovel', 'deep_aether:skyjade_axe', 'deep_aether:skyjade_hoe', 'deep_aether:skyjade_helmet', 'deep_aether:skyjade_chestplate', 'deep_aether:skyjade_leggings','deep_aether:skyjade_boots'], '\u00A79' + 'Higher Stats With Higher Durability')
    
})

//FOOD
ItemEvents.tooltip(event => {

    // LIGHTWEIGHT
    event.add('#minecraft:lightweight', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Lightweight')

    // HEARTY FOODS
    event.add('#minecraft:hearty', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Hearty')

    // FINGER FOODS
    event.add('#minecraft:fingerfood', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Finger Food')

    // SWEETS
    event.add('#minecraft:sweet', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Sweet')

    // MEALS
    event.add('#minecraft:meal', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Meal')

    // FEASTS
    event.add('#minecraft:feast', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Feast')

    // SPECIAL
    event.add('#minecraft:special', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Special')

    // GROSS
    event.add('#minecraft:gross', '\u00A77' + 'Food Type:' + '\u00A7f' + ' Gross')

    //UNOBTAINABLE
    event.add([
        '#minecraft:goaway'
    ], '\u00A7c' + 'Unobtainable (Hopefully)')

    //WIP
    event.add([
        '#minecraft:wip'
    ], '\u00A7e' + 'Unimplemented (WIP)')
})