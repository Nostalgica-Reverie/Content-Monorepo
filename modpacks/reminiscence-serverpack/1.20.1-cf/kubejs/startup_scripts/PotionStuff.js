MoreJSEvents.registerPotionBrewing(event => {
    // ----- Remove ALL Potions -----
    const removedPotions = [
        'minecraft:mundane',
        'minecraft:thick',
        'minecraft:night_vision',
        'minecraft:long_night_vision',
        'minecraft:invisibility',
        'minecraft:long_invisibility',
        'minecraft:leaping',
        'minecraft:long_leaping',
        'minecraft:strong_leaping',
        'minecraft:fire_resistance',
        'minecraft:long_fire_resistance',
        'minecraft:swiftness',
        'minecraft:long_swiftness',
        'minecraft:strong_swiftness',
        'minecraft:slowness',
        'minecraft:long_slowness',
        'minecraft:strong_slowness',
        'minecraft:turtle_master',
        'minecraft:long_turtle_master',
        'minecraft:strong_turtle_master',
        'minecraft:water_breathing',
        'minecraft:long_water_breathing',
        'minecraft:healing',
        'minecraft:strong_healing',
        'minecraft:harming',
        'minecraft:strong_harming',
        'minecraft:poison',
        'minecraft:long_poison',
        'minecraft:strong_poison',
        'minecraft:regeneration',
        'minecraft:long_regeneration',
        'minecraft:strong_regeneration',
        'minecraft:strength',
        'minecraft:long_strength',
        'minecraft:strong_strength',
        'minecraft:weakness',
        'minecraft:long_weakness',
        'minecraft:luck',
        'minecraft:slow_falling',
        'minecraft:long_slow_falling',
        'betterend:end_veil',
        'betterend:long_end_veil',
        'deep_aether:remedy_potion',
        'naturalist:forest_dasher',
        'naturalist:long_forest_dasher',
        'naturalist:strong_forest_dasher',
        'upgrade_aquatic:insomnia',
        'upgrade_aquatic:insomnia_strong',
        'upgrade_aquatic:restfulness',
        'upgrade_aquatic:restfulness_strong',
        'upgrade_aquatic:repellence',
        'upgrade_aquatic:repellence_strong',
        'upgrade_aquatic:repellence_long',
        'upgrade_aquatic:vibing',
        'upgrade_aquatic:vibing_strong',
        'upgrade_aquatic:vibing_long',
        'caverns_and_chasms:rewind',
        'caverns_and_chasms:long_rewind',
        'buzzier_bees:long_luck',
        'buzzier_bees:strong_luck',
        'buzzier_bees:unluck',
        'buzzier_bees:long_unluck',
        'buzzier_bees:strong_unluck',
        'quark:resilience',
        'quark:long_resilience',
        'quark:strong_resilience'
    ]

    removedPotions.forEach(potionId => {
        event.removeByPotion(null, null, potionId)
    })

    // ----- Custom Containers -----
    event.removeContainer('caverns_and_chasms:tether_potion')
    event.addContainerRecipe('caverns_and_chasms:tether_potion', 'minecraft:lapis_lazuli', 'caverns_and_chasms:tether_potion') 

    event.removeContainer('minecraft:splash_potion')
    event.addContainerRecipe('minecraft:splash_potion','minecraft:gunpowder','minecraft:splash_potion')

    event.removeContainer('minecraft:lingering_potion')
    event.addContainerRecipe('minecraft:lingering_potion', 'deep_aether:cloudbloom_bouquet', 'minecraft:lingering_potion')



    // ----- Potion Data -----
    const potions = [
        //Positive
        { name: 'Swiftness', id: 1, color: 3407864, duration: 12000, amplifier: 1, ingredient: 'minecraft:sugar' },
        { name: 'Fire Immunity', id: 12, color: 16751685, duration: 12000, amplifier: 0, ingredient: 'minecraft:magma_cream' },
        { name: 'Strength', id: 5, color: 16718944, duration: 6000, amplifier: 0, ingredient: 'minecraft:amethyst_shard' },
        { name: 'Resistance', id: 11, color: 16724787, duration: 6000, amplifier: 0, ingredient: 'quark:crab_shell' },
        { name: 'Knockback Immunity', id: 48, color: 46079, duration: 12000, amplifier: 3, ingredient: 'naturalist:snail_shell' },
        { name: 'Haste', id: 3, color: 16760875, duration: 12000, amplifier: 0, ingredient: 'kubejs:antler' },
        { name: 'Invisibility', id: 14, color: 16773698, duration: 18000, amplifier: 0, ingredient: 'minecraft:quartz' },
        { name: 'Leaping', id: 8, color: 3669808, duration: 12000, amplifier: 3, ingredient: 'minecraft:rabbit_foot' },
        { name: 'Regeneration', id: 10, color: 16736751, duration: 6000, amplifier: 0, ingredient: 'minecraft:ghast_tear' },
        { name: 'Water Breathing', id: 13, color: 1519359, duration: 24000, amplifier: 0, ingredient: 'minecraft:nautilus_shell' },
        { name: 'Night Vision', id: 16, color: 12779775, duration: 18000, amplifier: 0, ingredient: 'minecraft:glow_ink_sac' },
        { name: 'Slow Falling', id: 28, color: 65424, duration: 12000, amplifier: 0, ingredient: 'minecraft:feather' },
        { name: 'Luck', id: 26, color: 10092288, duration: 36000, amplifier: 1, ingredient: 'minecraft:gold_ingot' },
        { name: 'Rewind', id: 46, color: 7274751, duration: 24000, amplifier: 0, ingredient: 'caverns_and_chasms:bejeweled_pearl' },

        //Negative
        { name: 'Slowness', id: 2, color: 10266329, duration: 600, amplifier: 1, ingredient: 'minecraft:rotten_flesh' },
        { name: 'Weakness', id: 18, color: 8204850, duration: 2400, amplifier: 0, ingredient: 'minecraft:spider_eye' },
        { name: 'Poison', id: 19, color: 3439924, duration: 600, amplifier: 1, ingredient: 'minecraft:fermented_spider_eye' },
        { name: 'Blindness', id: 15, color: 3483702, duration: 600, amplifier: 0, ingredient: 'minecraft:ink_sac' },
        { name: 'Harming', id: 7, color: 4327945, duration: 1, amplifier: 1, ingredient: 'upgrade_aquatic:thrasher_tooth' },
        { name: 'Bad Luck', id: 27, color: 6454579, duration: 12000, amplifier: 1, ingredient: 'minecraft:bone' }
    ]

    //Because some values are being turned into stuff that is NOT bytes or ints and I don't like that >:(
    const Byte = Java.loadClass('net.minecraft.nbt.ByteTag').valueOf
    const Int = Java.loadClass('net.minecraft.nbt.IntTag').valueOf

    function registerPotionSet(name, id, color, duration, amplifier, ingredient) 
    {
        //Making an nbt tag for every potion (For custom textures using CIT later on)
        const tagName = name.toLowerCase().replace(/[^a-z0-9_]/g, '_')

        //The custom effect for every potion
        const effects = [
            {
                Id: Byte(id),
                Amplifier: Byte(amplifier),
                Duration: Int(duration),
                ShowParticles: Byte(0)
            }
        ]

        //Base potion
        const baseNbt = {
            Potion: "minecraft:thick",
            CustomPotionColor: Int(color),
            CustomPotionEffects: effects,
            display: {
                Name: JSON.stringify({ text: `${name} Brew`, italic: false })
            }
        }
        baseNbt[tagName] = Byte(1)

        //Splash potion
        const splashNbt = {
            Potion: "minecraft:thick",
            CustomPotionColor: Int(color),
            CustomPotionEffects: effects,
            display: {
                Name: JSON.stringify({ text: `${name} Splash Brew`, italic: false })
            }
        }
        splashNbt[tagName] = Byte(1)

        //Lingering potion
        const lingeringNbt = {
            Potion: "minecraft:thick",
            CustomPotionColor: Int(color),
            CustomPotionEffects: effects,
            display: {
                Name: JSON.stringify({ text: `${name} Lingering Brew`, italic: false })
            }
        }
        lingeringNbt[tagName] = Byte(1)

        //Tether potion
        const tetherNbt = {
            Potion: "minecraft:thick",
            CustomPotionColor: Int(color),
            CustomPotionEffects: effects,
            display: {
                Name: JSON.stringify({ text: `${name} Tether Brew`, italic: false })
            }
        }
        tetherNbt[tagName] = Byte(1)

        //Creating values contianing potions with their new NBT
        const base = Item.of('minecraft:potion').withNBT(baseNbt).strongNBT()
        const splash = Item.of('minecraft:splash_potion').withNBT(splashNbt).strongNBT()
        const lingering = Item.of('minecraft:lingering_potion').withNBT(lingeringNbt).strongNBT()
        const tether = Item.of('caverns_and_chasms:tether_potion').withNBT(tetherNbt).strongNBT()

        const input = Item.of('minecraft:potion', '{Potion:"minecraft:awkward"}').strongNBT()

        //Oh yeah, it's all coming together :fire::fire::fire:
        event.addCustomBrewing(ingredient, input, base)
        event.addCustomBrewing('minecraft:lapis_lazuli', base, tether)
        event.addCustomBrewing('minecraft:gunpowder', base, splash)
        event.addCustomBrewing('deep_aether:cloudbloom_bouquet', splash, lingering)
    }

    potions.forEach(potion => {
        registerPotionSet(potion.name, potion.id, potion.color, potion.duration, potion.amplifier, potion.ingredient)
    })

}) 