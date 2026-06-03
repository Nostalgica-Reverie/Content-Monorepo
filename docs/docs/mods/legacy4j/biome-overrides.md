# Biome Overrides

Biome overrides are a way to replace certain aspects of biomes, that are normally controlled by the server side, with the use of resource packs on the client side, similar to the `biomes_client.json` in the Bedrock Edition.

***

They are defined by the `biome_overrides.json` file in any namespace (directories in assets, like `minecraft`), having an override effect if you use the same namespace used by another resource pack, and an addition effect otherwise.

## Example

```
{
   "minecraft:swamp": {
      "water_color": "#4c6559",
      "water_fog_color": "#4c6559",
      "fog_color": "#4c6559",
      "sky_color": "#4c6559",
      "water_fog_distance": 15,
      "water_transparency": 1.0
    }
}
```
### Description
- `minecraft:swamp` -> the desired biome, defined by the [biome id](https://minecraft.wiki/w/Biome), can be `default` to be applied for all biomes (be careful with this), when there is no biome override for it
- `water_color` -> the water color used by the biome, can be a RGB color hex or number
- `water_fog_color` -> the water fog color used by the biome, can be a RGB color hex or number
- `fog_color` -> the fog color used by the biome, can be a RGB color hex or number
- `sky_color` -> the sky fog color used by the biome, can be a RGB color hex or number
- `water_fog_distance` -> the water fog distance used by the biome, can be a decimal number (default:96)
- `water_transparency` -> the water transparency used by the biome, can be a percentage, i.e. from the scope 0-1

