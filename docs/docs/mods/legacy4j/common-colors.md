# Common Colors

Common colors are basically variables controlled by resource packs, defining the color of texts and UI elements in general.
They can be accessed via [UI Definitions](https://github.com/Wilyicaro/FactoryAPI/wiki/UI-Definitions) using the format: `commonColor.<namespace>.<path>` or `commonColor.<path>` (if the namespace is `minecraft`, the default)

***

They are defined by the `common_colors.json` file in any namespace (directories in assets, like `minecraft`).

## Defaults

```json
{
  "minecraft:chat_background": "#FF323232",
  "minecraft:inventory_gray_text": "#FF323232",
  "minecraft:widget_text": "#FFFFFFFF",
  "minecraft:highlighted_widget_text": "#FFFFFF00",
  "minecraft:title_text": "#FFFFFFFF",
  "minecraft:title_text_outline": "#FF000000",
  "minecraft:stage_text": "#FFFFFFFF",
  "minecraft:tip_title_text": "#FFFFFFFF",
  "minecraft:tip_text": "#FFFFFFFF",
  "minecraft:action_text": "#FFFFFFFF",
  "minecraft:selected_storage_save": "#FFFFFF00",
  "minecraft:storage_save": "#FF8C9DE2",
  "minecraft:experience_text": "#FF80FF20",
  "minecraft:insufficient_experience_text": "#FFCF1F1D",
  "minecraft:anvil_error_text": "#FFFF6060",
  "minecraft:enchantment_text": "#FF685E4A",
  "minecraft:highlighted_enchantment_text": "#FFFFFF80",
  "minecraft:black": "#FF000000",
  "minecraft:dark_blue": "#FF0000AA",
  "minecraft:dark_green": "#FF00AA00",
  "minecraft:dark_aqua": "#FF00AAAA",
  "minecraft:dark_red": "#FFAA0000",
  "minecraft:dark_purple": "#FFAA00AA",
  "minecraft:gold": "#FFFFAA00",
  "minecraft:gray": "#FFAAAAAA",
  "minecraft:dark_gray": "#FF555555",
  "minecraft:blue": "#FF7878FF",
  "minecraft:green": "#FF55FF55",
  "minecraft:aqua": "#FF55FFFF",
  "minecraft:red": "#FFFF5555",
  "minecraft:light_purple": "#FFFF55FF",
  "minecraft:yellow": "#FFFFFF55",
  "minecraft:white": "#FFFFFFFF",
  "minecraft:block_light": "#FFFFFFFF"
}
```
### Description (not self-explanatory)
- `minecraft:inventory_gray_text` -> the color used in container and panel texts
- `minecraft:title_text` -> the color used in titles, like the one in the Legacy Loading Interface and the Death screen
- `minecraft:title_text_outline` -> the color used in title outlines, like in the Legacy Loading Interface and the Death screen
- `minecraft:stage_text` -> the color used in the stage of the Legacy Loading Interface
- `minecraft:action_text` -> the color used in the action text of control tooltips, like `Inventory`
- `minecraft:<color>` -> the color used in chat formatting
- `minecraft:block_light` -> the color used in the block lighting

