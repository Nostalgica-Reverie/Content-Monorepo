# Custom Control Types

It is possible to create a custom Control Type with a custom Minecraft logo and custom Control Icons.

To make custom control types, you need:
- `namespace:font/[id].json` - Specifies the sprites that will be used for the Control Icons, as well as the ID in the filename
- `namespace:control_types.json` - Specifies all the Control Types, with optional fields for using multiple fonts (for different UI Modes) and adding a Minecraft logo for a certain Control Type
- Assets within a namespace folder
  - Typically the structure is `namespace:textures/component/[platform]/[ui-mode]` (e.g. `legacy:textures/component/x360/sd`)

## File Structure
```
.
└── legacy
    ├── font
    │   ├── gamepad.json
    │   ├── gamepad_hd.json
    │   └── gamepad_sd.json
    ├── textures
    │   └── component
    │       └── pad
    │           ├── hd
    │           │   ├── back_button.webp
    │           │   └── [...]
    │           ├── sd
    │           │   ├── back_button.webp
    │           │   └── [...]
    │           ├── back_button.webp
    │           └── [...]
    └── control_types.json
```

## `namespace:font/[id].json`
This JSON determines the 

```json
{
    "providers":[
    {
            "type": "bitmap",
            "file": "legacy:component/pad/hd/down_button.webp",
            "ascent": 11,
            "height": 15,
            "chars": ["\uE735"]
        },
        {
            ...
        }
    ]    
}
```

