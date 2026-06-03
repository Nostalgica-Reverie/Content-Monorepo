# Listing-Based Manager
Here you will see about modifying/creating the JSON files of the listings used in the mod, such as `crafting_tab_listing.json`, `creative_tab_listing.json` and `stonecutting_groups.json`

## Legend
- [resource location](https://minecraft.wiki/w/Resource_location)
- [text component](https://minecraft.wiki/w/Raw_JSON_text_format)
- [sprite](https://minecraft.wiki/w/Resource_pack)

## General Concept
**The listings for this mod are generally made to facilitate the addition of resources to interfaces, which clearly do not affect the server-side directly, through resource packs, without the need to modify something as simple as that in code, allowing mods to make their own changes to the Legacy4J interface.**

If the listing is in a tab, its icon can be either a GUI sprite or an item, being mandatory if you want to add a tab. And if it's an item, it can have a nbt (or components in versions after 1.20.4), formatted as in the game commands.


`--> (*explanation) = JSON Entry Note`



- Sprite Icon Examples

```json
"legacy:icon/transport",
```

```json
"type": "sprite", --> (it's not needed in this case, as this is the default value)
"value": "legacy:icon/transport",
```

- Item Icon Examples

```json
"type": "item",
"value": {
   "item": "white_banner",
   "nbt": {BlockEntityTag:{Patterns:[{Pattern: "cr", Color:15}]}}
},
```

```json
"type": "item",
"value": {
   "item": "potion",
   "components": {"minecraft:potion_contents":{"potion": "minecraft:night_vision"}}
},
```

```json
"type": "item",
"value": {
   "item": "potion",
   "nbt": {"Potion": "minecraft:long_night_vision"}}
}.
```

## Creative Tab Listing
**Basically, this uses item listing when adding/modifying Legacy Creative Interface tabs, which is its main function. It's defined by the JSON file named `creative_tab_listing.json` in the assets directory of the desired namespace.**

As usual, I will use a JSON file as an example, in this case part of [legacy:creative_tab_listing.json](https://github.com/Wilyicaro/Legacy-Minecraft/blob/main/src/main/resources/assets/legacy/creative_tab_listing.json5) to make it easier to understand.

- Example

 ```json
   {
      {
        "id": "legacy:building_blocks" --> (a resource location used as an identifier)
        "name": {"translate": "legacy.container.tab.building_blocks"}
        "icon": "legacy:icon/structures",
        "listing": [
        "minecraft:stone", "minecraft:deepslate", "minecraft:basalt", "minecraft:blackstone", "minecraft:grass_block", "minecraft:dirt_path", "minecraft:dirt", "minecraft:cobblestone", "minecraft:cobbled_deepslate", "minecraft:smooth_stone"
        ] --> (its value is an array of item ids, which can contain an nbt as well as the icon if it is an object that has the id in the "item" key and the nbt in the "nbt" key, you can see this in the rest of the file)
      }
   }
```

As you can see, the `"name"` value is the text component that the Creative Interface displays when this tab is selected

***

## Recipe Info Filter Concept
Before we proceed, it's essential to understand the principle of the posterior listings, which are the recipe info filters.

Basically, they're used to select the recipes, either by an id of a specific recipe, like `black_stained_glass_pane_from_glass_pane`, or by an item or block tag (if directly referencing to its type) that references all recipes that the result matches this tag, if it starts with `#`.

- Examples

```json
"legacy:enchanted_golden_apple",
```

```json
"#minecraft:candles",
```

```json
"type": "id", --> (it's not needed in this case, as this is the default value)
"value": "legacy:enchanted_golden_apple",
```

```json
"type": "item_tag", --> (also, it's not needed, as you can simply use a string starting with #)
"value": "minecraft:candles"
```

```json
"type": "block_tag",
"value": "minecraft:mineable/pickaxe"
```

```json
"type": "item_id",
"value": "minecraft:suspicious_stew" --> (selects the first recipe found with this item as a result)
```

```json
"type": "item_id",
"value": {"id": "minecraft:suspicious_stew", "onlyFirstMatch: false} --> (selects all the recipes found with this item as a result)
```

## Crafting Tab Listing
**Basically, this uses recipe info filters listing when adding/modifying Crafting Interface tabs, which is its main function. It's defined by the JSON file named "crafting_tab_listing.json" in the assets directory of the desired namespace.**

The Crafting Tabs, unlike Creative's, have groups for each listing, which the name is only used as an identifier, and you can use the defined name to add recipe info filters to specific groups in the default file, without copying it all, through from other namespaces.

As usual, I will use a JSON file as an example, in this case part of [legacy:crafting_tab_listing.json](https://github.com/Wilyicaro/Legacy-Minecraft/blob/main/src/main/resources/assets/legacy/crafting_tab_listing.json5) to make it easier to understand.

- Example
```json
    {
        "id": "transport", --> (a resource location, used as an identifier)
        "icon": "legacy:icon/transport",
        "name": {"translate": "legacy.container.tab.transport"},
        "listing": [
            {"group": "rail", "recipes": ["rail","powered_rail","detector_rail","activator_rail"]},
            "minecart","chest_minecart","furnace_minecart","tnt_minecart","hopper_minecart",
            {"group": "boat", "recipes": ["oak_boat","birch_boat","spruce_boat","jungle_boat","acacia_boat","dark_oak_boat","mangrove_boat","pale_oak_boar","bamboo_raft","cherry_boat"]},
            {"group": "chest_boat", "recipes": 
["oak_chest_boat","birch_chest_boat","spruce_chest_boat","jungle_chest_boat","acacia_chest_boat","dark_oak_chest_boat","mangrove_chest_boat","pale_oak_chest_boat","bamboo_chest_raft","cherry_chest_boat"]}
       ] --> (its value can be either an object or an array with objects, where the group of recipes is defined by the value of the "group" key, and the recipes by "recipes", this can be seen in the rest of the file)
    }
```


As you can see, the key `name` value is the text component that the Crafting Interface displays when this tab is selected

## Loom Tab Listing
**The only difference between this and the Crafting Tab Listing is that instead of recipe info values, this one uses [banner pattern ids](https://minecraft.wiki/w/Banner_Pattern).**

## Stonecutting Groups
**The most simplistic of all, it's defined by the JSON file namespace:stonecutting_groups.json and only presents groups of recipe info filters, which work as in the Crafting Tab Listing.**

It only presents groups as an array. Another difference is that as there are no tabs, recipes will not be repeated between groups to make the view more compact.  

You can use the default file [legacy:stonecutting_groups.json](https://github.com/Wilyicaro/Legacy-Minecraft/blob/main/src/main/resources/assets/legacy/stonecutting_groups.json) as an example
