# Simply Legacy 26.05-alpha
*This changelog encompasses all changelogs from 26.04.2 to 26.04.7, as well.*

## Notes
- **It is recommended to Update or Reset your configs through Config Manager for the best experience.**
  - This can be found in `Modifications > Config Manager`.
- This update is marked as alpha due to major internal changes between 1.21.10 and 26.1.2, and the relatively early state of the Legacy4J 26.1.2 port as of 1.9 Pre-Release 
- Releases on CurseForge will come at a later date.

### Updated Minecraft from 1.21.10 to 26.1.2
Gameplay changes will be shown in a changelog when clicking `Play Game`.

### Updated Legacy4J
- More SD UI screens
- Legacy Settings Menus to allow options menus to match LCE
- Legacy Mobcap Limits, Shield Blocking and Offhand Limits
- Legacy Clouds and Cloud Height
- Screenshot Toasts
- Built-in Skin System
- Decay Potions
- Master Volume is now split into Music and Sound properly
- Improved map tooltip and Starter Map handling
- Creative Mode Elytra momentum handling
- Advanced Options with Legacy Settings Menus can now show contents from the Merge Advanced Options Mode
- Various bug fixes
- and more!

### Added Preferred Gamerules
- The new `Legacy Shield Blocking`,  `Legacy Offhand` and `Legacy Mobcap Limits` gamerules have been enabled by default.
- `Locator Bar` and `Announce Advancements` have been disabled by default. 

### Refactored Options Presets
- `"Retro" LCE` - Replaces `Very Low (XB/PS2*)`, now with more Legacy-style enhancements.
- `Handheld LCE` - Replaces `Handheld (Switch/Vita)`, now with more Legacy-style enhancements.
- `Old-Gen LCE` - Replaces `Low (XB360/PS3)`, now with more Legacy-style enhancements
  - This will be the new default preset on Simply Legacy.
- `New-Gen LCE` - Derived from `Medium (XB1/PS4)`, now with more Legacy-style enhancements.
- `"Next-Gen" LCE` - Derived from `High (XB Series/PS5)`, now with more Legacy-style enhancements.
- `Potato` - Derived from `Very Low (XB/PS2*)`, without the Legacy-style enhancements
- `Very Low` - Derived from `Handheld (Switch/Vita)`, without the Legacy-style enhancements
- `Low` - Derived from `Low (XB360/PS3)`, without the Legacy-style enhancements
- `Medium`, `High`, `Ultra`, `Extreme` use the same settings as previously
- `Default` and `Default+` have been removed, in favor of `Low`

### Added Simple Voice Chat
This will be mostly in the background unless you actually want to use it.
The default HUD is modified to where:
- Voice chat status icons are shown in the top-right corner, just right of the potion/status effect HUD.
- Group icons will populate from the bottom-right corner.
The default binds are:
- `V` to open VC options
- `Z` to hide VC icons
- `B` to open the VC group management menu.
- `N` to disable voice chat.
- `M` to mute voice chat.

### Removed KryptonFNP Patcher
No longer maintained.

### Removed Tooltips Enhanced
Now that we're on a build of Legacy4J that includes these control icons by default, Tooltips Enhanced has been removed.

### Updated Fabric Loader to 0.19.2
The modpack has been updated to the latest Fabric Loader.

## Temporarily Unavailable
- Console Advancement Sounds
- Legacy Mechanics
- Legacy Nether: Enhanced
- Legacy Skins
