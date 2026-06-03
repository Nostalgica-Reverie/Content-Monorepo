# Making a Skin Pack
A skin pack is a collection of skins in the CPMMODEL file format.

## Creating CPMMODELs
CPMMODEL files can be created in Customizable Player Models's editor screen. Skins need to be exported as **Models**, and without a UUID lock (the skin does not have to be marked as "**Skin Compatible**"). For more information, refer to the [Customizable Player Models wiki](https://github.com/tom5454/CustomPlayerModels/wiki).

## Packaging your skins
Now that your skins have been successfully exported as CPMMODELs, they now need to be packaged in a [**resource pack**](https://minecraft.wiki/w/Resource_pack).

Once added to the resource pack, they need to be referenced in a skin_packs.json (as seen below) stored in <code>assets/<a href="https://minecraft.wiki/w/Namespace"><i>namespace</i></a>/skin_packs.json</code>.

## Example
```json
{
  "schemaVersion": 1,
  "<namespace>:<packname>": {
    "icon": "<namespace>:textures/skin_packs/<packname>",
    "type": "(default|skin|mashup)",
    "skins": [
      {
        "model": "<namespace>:skinpacks/<packname>/<model1>.cpmmodel"
      },
      {
        "model": "<namespace>:skinpacks/<packname>/<model2>.cpmmodel"
      }
    ]
  },
  "<namespace>:<pack2>": {
    "icon": "<namespace>:textures/skin_packs/<pack2>",
    "type": "(default|skin|mashup)",
    "skins": [
      {
        "model": "<namespace>:skinpacks/<pack2>/<model1>.cpmmodel"
      },
      {
        "model": "<namespace>:skinpacks/<pack2>/<model2>.cpmmodel"
      }
    ]
  }
}
```
