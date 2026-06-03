# Predicate-Based Manager
It may seem like a strange name at first glance, but it is a way of encompassing the standard operation of the following managers, which are based on registry predicates of ids and [tags](https://minecraft.wiki/w/Tag), such as item, block and entity_type.

I will use an example of a part of a JSON file to explain this feature

```
"items" --> (the name of the registry type in the plural, such as items, blocks, or entity_types, accept as value an array of ids, written normally, or the registry-related tags, starting with "#", or starting with "!" to exclude specific ids): ["red_mushroom_block","brown_mushroom_block","mushroom_stem"],
"item" --> (accepts the same as the plural key, useful for when you don't need multiple values) : "#logs",
"nbt" --> (optional key, used only for items, which uses the nbt value, in the same format that the game uses in commands, comparing with the nbt of the specified item): "{\"instrument\": \"minecraft:call_goat_horn\"}"
```

# Tip Overrides
**Basically, as the name suggests, they are JSON files in the directory [namespace:tip_overrides](https://github.com/Wilyicaro/Legacy-Minecraft/tree/master/common/src/main/resources/assets/legacy/tip_overrides) with the name used only to identify, that, through this predicate, replace tips that are based on the id of the block, item or entity_type. These files must in the namespace:control_tooltips/gui directory, and their names are also for identification purposes only.**  

For this one, I will use as an example part taken from the file [legacy:tip_overrides/colored_items.json](https://github.com/Wilyicaro/Legacy-Minecraft/blob/master/common/src/main/resources/assets/legacy/tip_overrides/colored_items.json)

    {  
     "overrides" --> (can have the value of an array or just one object) : [  
        {  
          "items": ["#minecraft:terracotta","!minecraft:terracotta"],  
          "tip" : "block.minecraft.stained_terracotta.tip"  
        },  
        {  
          "block": "#minecraft:shulker_boxes",  
          "tip": "block.minecraft.shulker_box.tip"  
        },  
      ]  
    }  
As you can see, the "tip" key is the [translatable](https://minecraft.fandom.com/wiki/Resource_pack#Language) component key that the tip displays, and each element of the array, if it's present, is a different tip override.

# Control Tooltips
**It's a WIP feature, similar to the **Tip Overrides**, and allows you to add control tooltips to the In-Game GUI based on this predicate used on the block or entity you are aiming at, and also on the item you are holding. It's done through JSON files located in the [namespace:control_tooltips/gui](https://github.com/Wilyicaro/Legacy-Minecraft/tree/master/common/src/main/resources/assets/legacy/control_tooltips/gui) directory, and their names are also just for identification.**

As in previous themes, I will use a JSON file as an example, in this case [legacy:control_tooltips/gui/all.json](https://github.com/Wilyicaro/Legacy-Minecraft/blob/master/common/src/main/resources/assets/legacy/control_tooltips/gui/all.json)


    {
      "tooltips": [
        {
          "keyMapping" --> (keybind name, the one used in the options.txt file and in its component key) : "key.attack",
          "action": "legacy.action.hit",
          "usedItem" --> (can be either an item predicate referring to the item currently being used by the player, or a boolean value, which determines whether to allow, if true, any item in the hand) : false,
          "hitEntity" --> (can be either an entity_type predicate referring to the entity currently targeted by the player, or a boolean value, which determines whether to allow, if true, any entity being targeted) : true,
          "hitBlock" --> (can be either a block predicate referring to the block currently targeted by the player, or a boolean value, which determines whether to allow, if true, any block being targeted) : false
        }
      ]
    }
As you can see, the "action" key is the [translatable](https://minecraft.fandom.com/wiki/Resource_pack#Language) component key that the control tooltip displays in the GUI, and each element of the array is a different control tooltip. All predicate values are optional and true by default.




