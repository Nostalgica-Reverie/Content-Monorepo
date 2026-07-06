import crafttweaker.api.entity.attribute.AttributeOperation;
import crafttweaker.api.item.IItemStack;
import crafttweaker.api.tag.MCTag;
import crafttweaker.api.tag.manager.ITagManager;

//AETHER
<item:aether:valkyrie_lance>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_sword_buff", 6.5, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:aether:valkyrie_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_axe_nerf", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:aether:zanite_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_sword_buff2", 7.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:aether:zanite_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_axe_nerf2", 6.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:aether:gravitite_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_sword_buff3", 8.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:aether:gravitite_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_axe_nerf3", 7.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:aether:skyroot_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_sword_buff4", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:aether:skyroot_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_axe_nerf4", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:aether:holystone_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_sword_buff5", 6.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:aether:holystone_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "aether_axe_nerf5", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);


//BETTEREND
<item:betterend:thallasium_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "betterend_sword_buff", 15, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:betterend:thallasium_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "betterend_axe_nerf", 5.5, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:betterend:terminite_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "betterend_sword_buff2", 9.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:betterend:terminite_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "betterend_axe_nerf2", 8.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:betterend:aeternium_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "betterend_sword_buff3", 12, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:betterend:aeternium_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "betterend_axe_nerf3", 11, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);



<item:betterend:thallasium_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:head>]);
<item:betterend:thallasium_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:chest>]);
<item:betterend:thallasium_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:legs>]);
<item:betterend:thallasium_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:feet>]);

<item:betterend:thallasium_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "0a67f499-a8fa-4924-b7fc-0698e4fa5ba7", "bismuth buff31", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:betterend:thallasium_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "3b34a921-e1a6-4ac3-bc9f-2aa26f9e32a7", "bismuth buff41", 10.0, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:betterend:thallasium_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "86e8dad6-2c8c-465a-8d3e-b9b5e15a218d", "bismuth buff51", 8.0, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:betterend:thallasium_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "23ce10da-5975-45e9-91aa-9321beb05d47", "bismuth buff61", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:betterend:thallasium_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "56dcfc81-20a7-4754-8ad4-8fc7971199e9", "bismuth buff7", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:betterend:thallasium_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "b3bd6d3c-dd68-499b-be97-a3847ba75e82", "bismuth buff8", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:betterend:thallasium_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "cbe6d3e0-6b57-427f-9b60-c6b8663ea07b", "bismuth buff9", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:betterend:thallasium_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "33f0d05d-53b8-4327-a0f8-968ffb7ce2b2", "bismuth buff10", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);



<item:betterend:terminite_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:head>]);
<item:betterend:terminite_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:chest>]);
<item:betterend:terminite_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:legs>]);
<item:betterend:terminite_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:feet>]);
<item:betterend:terminite_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:head>]);
<item:betterend:terminite_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:chest>]);
<item:betterend:terminite_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:legs>]);
<item:betterend:terminite_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:feet>]);

<item:betterend:terminite_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "006fe424-c839-4669-be9e-29afe68d5205", "bismuth buff3", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:betterend:terminite_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "2061dfb8-eafd-4759-9816-39da22469f08", "bismuth buff4", 9.0, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:betterend:terminite_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "d7b00ce7-b240-4a7e-bee0-fdf778e5f072", "bismuth buff5", 7.0, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:betterend:terminite_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "ed13ac91-1d8c-4a7b-ab26-169cd38514d3", "bismuth buff6", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:betterend:terminite_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "043cae4b-c5ce-4bc8-a062-f948085092d6", "bismuth buff7", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:betterend:terminite_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "84f586a4-d2a8-4122-86be-d3539192ce41", "bismuth buff8", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:betterend:terminite_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "ef42226c-dda7-48b1-a071-4df04c2498bb", "bismuth buff9", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:betterend:terminite_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "e8edb83f-5ade-4f48-b3bf-d291f13b04c8", "bismuth buff10", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);



<item:betterend:aeternium_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:head>]);
<item:betterend:aeternium_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:chest>]);
<item:betterend:aeternium_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:legs>]);
<item:betterend:aeternium_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:feet>]);
<item:betterend:aeternium_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:head>]);
<item:betterend:aeternium_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:chest>]);
<item:betterend:aeternium_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:legs>]);
<item:betterend:aeternium_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:feet>]);
<item:betterend:aeternium_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:head>]);
<item:betterend:aeternium_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:chest>]);
<item:betterend:aeternium_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:legs>]);
<item:betterend:aeternium_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:feet>]);

<item:betterend:aeternium_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "c82dfac0-4baa-45b5-8654-d18935c0e3bc", "bismuth buff30", 5.5, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:betterend:aeternium_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "d968a33a-0957-4d55-9f88-8becf858d10a", "bismuth buff40", 10.5, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:betterend:aeternium_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "91936fe9-0ad6-4d37-b891-74125645a74c", "bismuth buff50", 8.5, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:betterend:aeternium_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "538f75d6-9c25-4043-a7af-4b213c8e8c93", "bismuth buff60", 5.5, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:betterend:aeternium_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "2381dab1-1fca-42e3-b131-eab1d8bb83b2", "bismuth buff7", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:betterend:aeternium_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "597bc252-2957-42ab-968d-05c6f193ddf9", "bismuth buff8", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:betterend:aeternium_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "a14228e6-d9a6-4c4d-8a0b-c7eb1a964cd2", "bismuth buff9", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:betterend:aeternium_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "dd84c19b-2cb0-417e-ba89-452cf4e419ab", "bismuth buff10", 5.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

//GOLD BUFF
<item:minecraft:golden_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "gold buff", 7.5, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:minecraft:golden_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "gold buff2", 6.5, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:minecraft:golden_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:head>]);
<item:minecraft:golden_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:chest>]);
<item:minecraft:golden_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:legs>]);
<item:minecraft:golden_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:feet>]);

<item:minecraft:golden_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "cf3f55d3-645c-4f38-a497-9c13a33db5cf", "gold buff3", 3.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:minecraft:golden_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "cc3f55d3-645c-4f38-a497-9c13a33db5cf", "gold buff4", 8.0, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:minecraft:golden_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "c43f55d3-645c-4f38-a497-9c13a33db5cf", "gold buff5", 6.0, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:minecraft:golden_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "ce3f55d3-645c-4f38-a497-9c13a33db5cf", "gold buff6", 3.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:minecraft:golden_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.luck>, "3cb67a1b-45e9-414a-8b75-abd19f72ebf8", "gold buff7", 0.25, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:minecraft:golden_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.luck>, "5cb3f3f7-ad31-4f3a-b29f-c7faf3cf1c79", "gold buff8", 0.25, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:minecraft:golden_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.luck>, "57b7afd4-60bc-4a1c-997a-67dd29938466", "gold buff9", 0.25, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:minecraft:golden_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.luck>, "7eda9177-b81a-4338-a22a-5481bc88d7ef", "gold buff10", 0.25, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

//NETHERITE BUFF
<item:minecraft:netherite_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:head>]);
<item:minecraft:netherite_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:chest>]);
<item:minecraft:netherite_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:legs>]);
<item:minecraft:netherite_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:feet>]);

<item:minecraft:netherite_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:head>]);
<item:minecraft:netherite_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:chest>]);
<item:minecraft:netherite_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:legs>]);
<item:minecraft:netherite_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.knockback_resistance>, [<constant:equipmentslot:feet>]);

<item:minecraft:netherite_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "cf3f55d3-645c-4f38-a497-9c13a33db5cf", "netherite buff", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:minecraft:netherite_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "cc3f55d3-645c-4f38-a497-9c13a33db5cf", "netherite buff2", 9.0, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:minecraft:netherite_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "c43f55d3-645c-4f38-a497-9c13a33db5cf", "netherite buff3", 7.0, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:minecraft:netherite_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "ce3f55d3-645c-4f38-a497-9c13a33db5cf", "netherite buff4", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

//OBSIDIAN ARMOR BUFF
<item:aether:obsidian_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.knockback_resistance>, "09fbef04-e612-4653-94e1-eaae5f7b5809", "obsidian buff", 0.3, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:aether:obsidian_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.knockback_resistance>, "2bc72f66-4a77-44fb-8c4f-c2eedd42cfef", "obsidian buff2", 0.3, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:aether:obsidian_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.knockback_resistance>, "1f018787-4785-4be6-86dd-b8bcb0cc910b", "obsidian buff3", 0.3, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:aether:obsidian_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.knockback_resistance>, "399d9698-8dbf-4865-a78a-5a0d53d3d9c9", "obsidian buff4", 0.3, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

//LEATHER BUFF
<item:minecraft:leather_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.movement_speed>, "cf3f55d3-645c-4f38-a497-9c13a33db5cf", "leather buff", 0.005, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:minecraft:leather_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.movement_speed>, "cc3f55d3-645c-4f38-a497-9c13a33db5cf", "leather buff2", 0.005, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:minecraft:leather_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.movement_speed>, "c43f55d3-645c-4f38-a497-9c13a33db5cf", "netherite buff3", 0.005, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:minecraft:leather_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.movement_speed>, "ce3f55d3-645c-4f38-a497-9c13a33db5cf", "leather buff4", 0.005, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

//FARMERS DELIGHT NERF
<item:farmersdelight:skillet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.attack_damage>, [<constant:equipmentslot:mainhand>]);
<item:farmersdelight:skillet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "skilletnerf", 4.0, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

//COPPER NERF
<item:bettercopper:copper_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:head>]);
<item:bettercopper:copper_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "18b389ff-07ab-4e93-8475-e928f3162284", "copper nerf", 1.0, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);

<item:bettercopper:copper_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:feet>]);
<item:bettercopper:copper_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "8aa6d483-d8c6-47b9-a299-fc2742f1867a", "copper nerf 2", 1.0, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:bettercopper:copper_sword>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "copper nerf 3", 6.5, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);
<item:bettercopper:copper_axe>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.attack_damage>, "CB3F55D3-645C-4F38-A497-9C13A33DB5CF", "copper nerf 4", 5.5, AttributeOperation.ADDITION, [<constant:equipmentslot:mainhand>]);

<item:bettercopper:copper_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:mining_speed>, "6ca0655f-18a0-4942-9883-99550ec79609", "copper mining 1", 0.5, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:bettercopper:copper_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:mining_speed>, "5058f64c-4446-4a6a-af90-f68257ac56ed", "copper mining 2", 0.5, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:bettercopper:copper_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:mining_speed>, "89ca7889-104b-462d-9ed4-b8556de360d6", "copper mining 3", 0.5, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:bettercopper:copper_boots>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:mining_speed>, "3ed0587a-b2d1-49be-a337-8507e34ab710", "copper mining 4", 0.5, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

//BONE ARMOR
<item:caverns_and_chasms:necromium_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:head>]);
<item:caverns_and_chasms:necromium_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:chest>]);
<item:caverns_and_chasms:necromium_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:legs>]);
<item:caverns_and_chasms:necromium_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor>, [<constant:equipmentslot:feet>]);

<item:caverns_and_chasms:necromium_helmet>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:head>]);
<item:caverns_and_chasms:necromium_chestplate>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:chest>]);
<item:caverns_and_chasms:necromium_leggings>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:legs>]);
<item:caverns_and_chasms:necromium_boots>.anyDamage().removeGlobalAttribute(<attribute:minecraft:generic.armor_toughness>, [<constant:equipmentslot:feet>]);

<item:caverns_and_chasms:necromium_helmet>.anyDamage().removeGlobalAttribute(<attribute:caverns_and_chasms:weakness_aura>, [<constant:equipmentslot:head>]);
<item:caverns_and_chasms:necromium_chestplate>.anyDamage().removeGlobalAttribute(<attribute:caverns_and_chasms:weakness_aura>, [<constant:equipmentslot:chest>]);
<item:caverns_and_chasms:necromium_leggings>.anyDamage().removeGlobalAttribute(<attribute:caverns_and_chasms:weakness_aura>, [<constant:equipmentslot:legs>]);
<item:caverns_and_chasms:necromium_boots>.anyDamage().removeGlobalAttribute(<attribute:caverns_and_chasms:weakness_aura>, [<constant:equipmentslot:feet>]);

<item:caverns_and_chasms:necromium_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "aff43d16-27ab-43f9-b06d-86115e8556b0", "bone1", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:caverns_and_chasms:necromium_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "641000a6-424a-4b6e-881b-f80c2ee67829", "bone2", 3, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:caverns_and_chasms:necromium_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "7d181cef-3e54-44bc-a244-89c8bfc81d4c", "bone3", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:caverns_and_chasms:necromium_boots>.anyDamage().addGlobalAttributeModifier(<attribute:minecraft:generic.armor>, "8299a4b0-84a5-4fef-b5b4-7b15355d0d21", "bone4", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:caverns_and_chasms:necromium_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:ranged_damage>, "e5f59507-157a-4511-a56c-a72582389cdc", "bone12", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:caverns_and_chasms:necromium_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:ranged_damage>, "7825201f-fef4-4966-a86a-17a42ddca7fb", "bone22", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:caverns_and_chasms:necromium_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:ranged_damage>, "803a208f-0ccc-453f-8bde-20a4c7835222", "bone32", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:caverns_and_chasms:necromium_boots>.anyDamage().addGlobalAttributeModifier(<attribute:puffish_attributes:ranged_damage>, "abc1bb75-c374-4f1d-b0ec-a21fb1b42372", "bone42", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:aether:neptune_helmet>.anyDamage().addGlobalAttributeModifier(<attribute:forge:swim_speed>, "b227b700-8a9d-4820-b829-9b50ec9437f9", "aquamarine1", 0.15, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:aether:neptune_chestplate>.anyDamage().addGlobalAttributeModifier(<attribute:forge:swim_speed>, "ab08d37f-49fa-4389-a74c-2f5f3ddfef1a", "aquamarine2", 0.15, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:aether:neptune_leggings>.anyDamage().addGlobalAttributeModifier(<attribute:forge:swim_speed>, "4e0d4822-5543-496e-837e-e279a0029e36", "aquamarine3", 0.15, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:aether:neptune_boots>.anyDamage().addGlobalAttributeModifier(<attribute:forge:swim_speed>, "c4022598-4416-4896-b6f1-c5e16c890fcc", "aquamarine4", 0.15, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);


//TANZANITE CHANGES (STRONGER STATS AS DURABILITY GOES DOWN)
<item:aether:zanite_helmet>.onlyDamagedAtLeast(42).onlyDamagedAtMost(82).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "0280949e-f861-4e63-b229-de4b7b94a022", "zanitebroken1a", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:aether:zanite_chestplate>.onlyDamagedAtLeast(60).onlyDamagedAtMost(119).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "69c4a598-ab98-4aa1-a116-5745e1b92159", "zanitebroken1b", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:aether:zanite_leggings>.onlyDamagedAtLeast(57).onlyDamagedAtMost(112).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "7b17be3c-224d-4d50-80d3-6f9f123a58fb", "zanitebroken1c", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:aether:zanite_boots>.onlyDamagedAtLeast(48).onlyDamagedAtMost(96).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "6c5d56c4-958d-43e0-9e0f-6478936789bb", "zanitebroken1d", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:aether:zanite_helmet>.onlyDamagedAtLeast(83).onlyDamagedAtMost(123).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "c61f9eb2-52c9-475c-928d-fca339a913d8", "zanitebroken2a", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:aether:zanite_chestplate>.onlyDamagedAtLeast(120).onlyDamagedAtMost(179).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "7adf8360-ad53-4e9c-8844-e3755a697ef7", "zanitebroken2b", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:aether:zanite_leggings>.onlyDamagedAtLeast(113).onlyDamagedAtMost(168).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "aa60df27-9560-4a93-a70c-237741dd7207", "zanitebroken2c", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:aether:zanite_boots>.onlyDamagedAtLeast(97).onlyDamagedAtMost(145).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "bc76a940-3919-4cb3-a194-20a4a4eec148", "zanitebroken2d", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:aether:zanite_helmet>.onlyDamagedAtLeast(124).onlyDamagedAtMost(165).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "2d6d7e18-f7ec-4169-8bcd-613753e83f03", "zanitebroken3a", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:aether:zanite_chestplate>.onlyDamagedAtLeast(180).onlyDamagedAtMost(240).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "abecb79b-b2cc-45fe-a513-926f1b0354b9", "zanitebroken3b", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:aether:zanite_leggings>.onlyDamagedAtLeast(169).onlyDamagedAtMost(225).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "e1ff7299-1bac-4e46-a4f8-88f2bdea8e2e", "zanitebroken3c", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:aether:zanite_boots>.onlyDamagedAtLeast(146).onlyDamagedAtMost(195).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "5f9adc46-c6ec-4083-91a5-a421615f868c", "zanitebroken3d", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);


//EMERALD CHANGES (WEAKER STATS AS DURABILITY GOES DOWN)
<item:deep_aether:skyjade_helmet>.onlyDamagedAtLeast(93).onlyDamagedAtMost(139).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "0280949e-f861-4e63-b229-de4b7b94a022", "emeraldbroken1a", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:deep_aether:skyjade_chestplate>.onlyDamagedAtLeast(137).onlyDamagedAtMost(206).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "69c4a598-ab98-4aa1-a116-5745e1b92159", "emeraldbroken1b", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:deep_aether:skyjade_leggings>.onlyDamagedAtLeast(130).onlyDamagedAtMost(195).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "7b17be3c-224d-4d50-80d3-6f9f123a58fb", "emeraldbroken1c", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:deep_aether:skyjade_boots>.onlyDamagedAtLeast(100).onlyDamagedAtMost(150).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "6c5d56c4-958d-43e0-9e0f-6478936789bb", "emeraldbroken1d", 1, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:deep_aether:skyjade_helmet>.onlyDamagedAtLeast(47).onlyDamagedAtMost(92).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "c61f9eb2-52c9-475c-928d-fca339a913d8", "emeraldbroken2a", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:deep_aether:skyjade_chestplate>.onlyDamagedAtLeast(68).onlyDamagedAtMost(136).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "7adf8360-ad53-4e9c-8844-e3755a697ef7", "emeraldbroken2b", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:deep_aether:skyjade_leggings>.onlyDamagedAtLeast(65).onlyDamagedAtMost(129).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "aa60df27-9560-4a93-a70c-237741dd7207", "emeraldbroken2c", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:deep_aether:skyjade_boots>.onlyDamagedAtLeast(50).onlyDamagedAtMost(99).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "bc76a940-3919-4cb3-a194-20a4a4eec148", "emeraldbroken2d", 1.5, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);

<item:deep_aether:skyjade_helmet>.onlyDamagedAtLeast(0).onlyDamagedAtMost(46).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "2d6d7e18-f7ec-4169-8bcd-613753e83f03", "emeraldbroken3a", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:head>]);
<item:deep_aether:skyjade_chestplate>.onlyDamagedAtLeast(0).onlyDamagedAtMost(67).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "abecb79b-b2cc-45fe-a513-926f1b0354b9", "emeraldbroken3b", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:chest>]);
<item:deep_aether:skyjade_leggings>.onlyDamagedAtLeast(0).onlyDamagedAtMost(64).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "e1ff7299-1bac-4e46-a4f8-88f2bdea8e2e", "emeraldbroken3c", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:legs>]);
<item:deep_aether:skyjade_boots>.onlyDamagedAtLeast(0).onlyDamagedAtMost(49).addGlobalAttributeModifier(<attribute:minecraft:generic.armor_toughness>, "5f9adc46-c6ec-4083-91a5-a421615f868c", "emeraldbroken3d", 2, AttributeOperation.ADDITION, [<constant:equipmentslot:feet>]);