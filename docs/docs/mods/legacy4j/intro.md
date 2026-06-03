# Intro Sequence

The Legacy4J intro sequence can be modified with resource packs, from the brand textures used, to whether there will be crossfade or not. This is defined by the `legacy:intro.json` file <br> (You can see its default contents [here](https://github.com/Wilyicaro/Legacy-Minecraft/blob/main/src/main/resources/assets/legacy/intro.json)). <br>
The intro sequence can be skipped using the RETURN key on keyboard, clicking with the Left Mouse Button, or pressing `A` on an Xbox gamepad.

<video controls="controls" src="./images/intro.mp4"/>
<sub>Intro sequence seen in Simply Legacy, using assets from built-in resources and Legacy Modpack Resources</sub>

## Components

```json
{
   "brands": [
       "legacy:textures/gui/intro/loading3.webp",
       "legacy:textures/gui/intro/loading.webp",
       "legacy:textures/gui/intro/loading1.webp",
       "legacy:textures/gui/intro/loading2.webp",
       "legacy:textures/gui/intro/loading3.webp",
       "legacy:textures/gui/intro/eula.webp",
       "legacy:textures/gui/intro/lasting.webp",
       "legacy:textures/gui/intro/mojang.webp",
       "legacy:textures/gui/intro/4j.webp",
       "legacy:textures/gui/intro/xbox.webp"
    ],
    "background": "legacy:textures/gui/intro/background.webp",
    "brandDuration": 2500,
    "fadeIn": 0.1,
    "fadeOut": 0.1,
    "crossFade": true
}
```
Components which can be defined in the Intro Sequence include:
- Brand images ([`brands`](#brands))
- Background image ([`background`](#background))
- Delay in milliseconds between images ([`brandDuration`](#brandDuration))
- Percentage of the delay used for the fade in ([`fadeIn`](#fadeIn))
- Percentage of the delay used for the fade out ([`fadeOut`](#fadeOut))
- If there will be crossfade between the images ([`crossFade`](#crossFade))

### `brands`
<sub>(required)</sub><br>
The images used in the intro sequence can be defined by `brands` <br>
::: info
This field is required for the `intro.json` to work
:::
This field will reference assets provided by a [resource location](https://mc.wiki/Resource_location)

### `background`
The background image used can be defined by `background`. <br>
Especially important if [`crossFade`](#crossFade) is set to `false`, but also displays briefly when fading out the last slide in the sequence. Also defined by a [resource location](https://mc.wiki/Resource_location)

### `brandDuration`
The duration of each slide in milliseconds be defined by `brandDuration`. <br> 2500 milliseconds = 2.5 seconds, etc.

### `fadeIn`
The length of the fade in animation can be defined by `fadeIn`. <br>
This is defined as the percentage of the total runtime of the intro sequence.

### `fadeOut`
The length of the fade out animation can be defined by `fadeOut`. <br>
Same as `fadeIn`, this is defined as the percentage of the total runtime of the intro sequence.

### `crossFade`
Whether the images are crossfaded between can be defined by `crossFade`. <br>
Especially useful if you'd like to mesh multiple images together (like the "Loading..." sequence seen in Re-Console+ and Simply Legacy)
