// --- Helpers (Rhino / ES5 compatible) ---

function setTier(event, ids, level, speed) {
  ids.forEach(function(id) {
    event.modify(id, function(item) {
      item.tier = tier => {
        tier.level = level
        if (typeof speed !== "undefined" && speed !== null) {
          tier.speed = speed
        }
      }
    })
  })
}


function setFood(event, id, nutrition, meat, fast, effects) {
  if (typeof nutrition === "undefined") nutrition = null
  if (typeof meat === "undefined") meat = false
  if (typeof fast === "undefined") fast = false
  if (typeof effects === "undefined") effects = []
  event.modify(id, function(item) {
    item.foodProperties = function(food) {
      if (nutrition !== null) food.hunger(nutrition)
      if (meat) food.meat()
      if (fast) food.fastToEat()
      effects.forEach(function(eff) {
        // eff = [effectId, duration, amplifier, chance]
        food.effect(eff[0], eff[1], eff[2], eff[3])
      })
    }
  })
}

function setStack(event, ids, count) {
  ids.forEach(function(id) {
    event.modify(id, function(item) {
      item.maxStackSize = count
    })
  })
}

function setFireResistant(event, ids, resistant) {
  ids.forEach(function(id) {
    event.modify(id, function(item) {
      item.fireResistant = resistant
    })
  })
}

function setDurability(event, ids, value) {
  ids.forEach(function(id) {
    event.modify(id, function(item) {
      item.maxDamage = value
    })
  })
}



ItemEvents.modification(event => {
  // GOLD tools upgrade
  setTier(event, [
    'minecraft:golden_pickaxe',
    'minecraft:golden_axe',
    'minecraft:golden_shovel',
    'minecraft:golden_sword',
    'minecraft:golden_hoe'
  ], 2, 16.0)

  // OPAL (terminite) tools
  setTier(event, [
    'betterend:terminite_pickaxe',
    'betterend:terminite_axe',
    'betterend:terminite_shovel',
    'betterend:terminite_sword',
    'betterend:terminite_hoe'
  ], 5)

  // TITANIUM (aeternium) tools
  setTier(event, [
    'betterend:aeternium_pickaxe',
    'betterend:aeternium_axe',
    'betterend:aeternium_shovel',
    'betterend:aeternium_sword',
    'betterend:aeternium_hoe'
  ], 6)

  // FOOD CHANGES
  setFood(event, 'minecraft:rotten_flesh', null, false, false, [
    ['minecraft:poison', 600, 0, 1.0],
    ['minecraft:nausea', 150, 0, 1.0]
  ])
  setFood(event, 'minecraft:poisonous_potato', null, false, false, [
    ['minecraft:poison', 300, 0, 1.0],
    ['minecraft:nausea', 150, 0, 1.0]
  ])
  setFood(event, 'minecraft:pufferfish', null, false, false, [
    ['minecraft:poison', 1200, 0, 1.0],
    ['minecraft:nausea', 300, 0, 1.0]
  ])

  setFood(event, 'kubejs:frog_leg', 1, true)
  setFood(event, 'kubejs:cooked_frog_leg', 6, true)
  setFood(event, 'kubejs:golden_frog_leg', 12, true, true, [
    ['minecraft:absorption', 1200, 1, 1.0],
    ['minecraft:jump_boost', 600, 1, 1.0]
  ])
  setFood(event, 'minecraft:golden_carrot', null, false, false, [
    ['minecraft:absorption', 1200, 1, 1.0],
    ['minecraft:night_vision', 600, 1, 1.0]
  ])
  setFood(event, 'minecraft:glistering_melon_slice', null, false, false, [
    ['minecraft:regeneration', 300, 0, 1.0]
  ])

  ;['kubejs:cherry','kubejs:cranberries','kubejs:plum','kubejs:durian','kubejs:pear','kubejs:chestnut','kubejs:mango']
    .forEach(fruit => setFood(event, fruit, 1))

  // Remove food props
  setFood(event, 'immersive_weathering:moss_clump')
  setFood(event, 'immersive_weathering:icicle')

  // STACK SIZES
  setStack(event, ['incubation:scrambled_eggs','minecraft:mushroom_stew','minecraft:beetroot_soup','minecraft:rabbit_stew'], 6)
  setStack(event, ['kubejs:cherry','kubejs:cranberries','kubejs:plum','kubejs:chestnut','kubejs:pear','kubejs:mango','kubejs:durian'], 8)
  setStack(event, ['minecraft:potion','minecraft:splash_potion','minecraft:lingering_potion'], 6)

  // Fire resistance
  setFireResistant(event, ['inmis:withered_backpack','inmis:endless_backpack'], true)
  setFireResistant(event, ['inmis:blazing_backpack'], false)

  // DURABILITY
  setDurability(event, [
    'minecraft:golden_sword','minecraft:golden_axe','minecraft:golden_pickaxe','minecraft:golden_shovel','minecraft:golden_hoe', 'aether:golden_gloves'
  ], 170)

  setDurability(event, [
    'deep_aether:skyjade_sword','deep_aether:skyjade_axe','deep_aether:skyjade_pickaxe','deep_aether:skyjade_shovel','deep_aether:skyjade_hoe'
  ], 215)

  setDurability(event, [
    'betterend:thallasium_sword','betterend:thallasium_axe','betterend:thallasium_pickaxe','betterend:thallasium_shovel','betterend:thallasium_hoe'
  ], 10)

  setDurability(event, ['farmersdelight:iron_knife'], 400)

  setDurability(event, [
    'betterend:thallasium_helmet','betterend:thallasium_chestplate','betterend:thallasium_leggings','betterend:thallasium_boots'
  ], 20)

  setDurability(event, ['minecraft:golden_helmet'], 115)
  setDurability(event, ['minecraft:golden_chestplate'], 168)
  setDurability(event, ['minecraft:golden_leggings'], 158)
  setDurability(event, ['minecraft:golden_boots'], 136)

  setDurability(event, ['deep_aether:skyjade_helmet'], 185)
  setDurability(event, ['deep_aether:skyjade_chestplate'], 275)
  setDurability(event, ['deep_aether:skyjade_leggings'], 260)
  setDurability(event, ['deep_aether:skyjade_boots'], 200)

  setDurability(event, ['aether:iron_ring','aether:golden_ring','aether:iron_pendant','aether:golden_pendant','aether:zanite_ring','aether:zanite_pendant','deep_aether:skyjade_ring'], 250)
  setDurability(event, ['deep_aether:skyjade_ring'], 64)

})