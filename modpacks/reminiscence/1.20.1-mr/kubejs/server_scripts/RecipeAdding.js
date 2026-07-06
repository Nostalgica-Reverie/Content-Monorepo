//All recipes as of right now are done by datapacks. I will likely gradually convert them here
// SCRIPTS DONE BY MUTTDUD FOR THE LOST TRACKS DATAPACK/RESOURCE PACK BUNDLE
ServerEvents.recipes(event => {
  event.shaped('hookshot:longshot', [
      ' ba',
      'ccb',
      'dc '
    ],
    {
      c: 'betterend:aeternium_ingot',
      b: 'minecraft:chain',
      a: 'betterend:terminite_ingot',
      d: 'minecraft:stick'
    })
    .id('hookshot:longshot')

  event.shaped('minecraft:stonecutter', [
      'aaa',
      'aba',
      'aaa'
    ],
    {
      b: 'minecraft:iron_ingot',
      a: 'minecraft:stone',
    })
    .id('minecraft:stonecutter')

  event.shaped('supplementaries:safe', [
      'aba',
      'b b',
      'aba'
    ],
    {
      b: 'minecraft:netherite_ingot',
      a: 'caverns_and_chasms:silver_ingot',
    })
    .id('supplementaries:safe')

  event.shaped('deep_aether:aerwhale_saddle', [
      '###',
      'AaA'
    ], {
      '#': 'minecraft:leather',
      'A': 'minecraft:amethyst_shard',
      'a': 'kubejs:valkyrie_feather'
    }).id('kubejs:aerwhale_saddle')

  //BY MUTTDUD - FOR THE LOST TRACKS DATAPACK
	event.shapeless(
	Item.of('supplementaries:music_disc_heave_ho', {CustomModelData:5, HideFlags:32, display:{Lore:["\"\\u00a77C418 - Eleven\""]}}).strongNBT(),
		[
			'minecraft:music_disc_11', 			
			'2x minecraft:slime_ball'
		]
	)
	event.shapeless(
	Item.of('supplementaries:music_disc_heave_ho', {CustomModelData:14, HideFlags:32, display:{Lore:["\"\\u00a77Notch - Magnetic Circuit\""]}}).strongNBT(),
		[
			'minecraft:music_disc_cat', 			
			'minecraft:enchanted_golden_apple'
		]
	)

  //FIX BY EMMA
    function oreSmelt(ore, material) {
    event.smelting(material, ore)
    event.blasting(material, ore)
  }
  oreSmelt('kubejs:aquamarine_ore', 'kubejs:aquamarine')
  oreSmelt('kubejs:titanium_ore', 'kubejs:raw_titanium')
  oreSmelt('kubejs:opal_ore', 'betterend:terminite_ingot')
})