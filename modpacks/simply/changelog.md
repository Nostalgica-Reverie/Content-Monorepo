# Simply Legacy 26.05.7

## Notes
- **If you are updating from a 1.21.10 version, it is recommended to Update or Reset your configs through Config Manager for the best experience.**
  - This can be found in `Modifications > Config Manager`.

# Modrinth

### Notable Updates
- Added Raise Sound Limit Simplified
  - Fixes an issue where sounds would stop during longer gameplay sessions
- Updated Minimega from 6.2.2 to 6.2.3
  - Fixes an issue with the PvP states being inverted on Fistfight
- Updated Legacy World Sizes
  - Fixes the appearance of the Biome Scale and World Size sliders

### Configuration Changes
- The default game difficulty is now Easy

## Temporarily Unavailable
- Console Advancement Sounds
- Legacy Skins

# CurseForge (beta)

### Notable Updates
- Updated Minecraft from 1.21.10 to 26.1.2
  - Gameplay changes will be shown in a changelog when clicking `Play Game`.
- Added Legacy World Sizes
  - Adds Legacy Console Edition-style limited worlds
- Updated Legacy4J
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
  - Reordered Mouse Options and unified the size of Advanced Options panels
  - Ported the Kanji font from the Legacy Console Edition
  - Fixed several issues with the tutorial world
  - Various bug fixes
  - and more!
- Updated Legacy Ports
  - Adds content and fixes issues for 26.1
- Added Preferred Gamerules
  - The new `Legacy Shield Blocking`, `Legacy Offhand` and `Legacy Mobcap Limits` gamerules have been enabled by default.
  - `Locator Bar` and `Announce Advancements` gamerules have been disabled by default.
- Added Simple Voice Chat
  - This will be mostly in the background unless you actually want to use it. The default binds are:
    - `V` to open VC options
    - `Z` to hide VC icons
    - `B` to open the VC group management menu.
    - `N` to disable voice chat.
    - `M` to mute voice chat.
- Added Async Logger
  - Improves logging performance
- Added Gnetum
  - Improves user interface performance
- Added WorldThreader
  - Improves multi-dimension performance, especially on higher threaded CPUs
- Added Raise Sound Limit Simplified
  - Fixes an issue where sounds would stop during longer gameplay sessions
- Removed Better Block Entities
  - Due to various mod and resource pack issues, BBE has been removed
- Removed Legacy Mechanics
  - Much of the functionality from this has been merged into base Legacy4J
- Removed RegSyncFix
  - ModernRegSyncFix will replace this once a CurseForge version comes out
- Removed Stfu
  - Much of the functionality from this has been merged into base Legacy4J
- Removed Tooltips Enhanced
  - Control Icons from this have been merged into base Legacy4J

### Configuration Changes
- Refactored Option Presets
  - `"Retro" LCE` - Replaces `Very Low (XB/PS2*)`, now with more Legacy-style enhancements.
  - `Handheld LCE` - Replaces `Handheld (Switch/Vita)`, now with more Legacy-style enhancements.
  - `Old-Gen LCE` - Replaces `Low (XB360/PS3)`, now with more Legacy-style enhancements
    - This will be the new default preset.
  - `New-Gen LCE` - Derived from `Medium (XB1/PS4)`, now with more Legacy-style enhancements.
  - `"Next-Gen" LCE` - Derived from `High (XB Series/PS5)`, now with more Legacy-style enhancements.
  - `Potato` - Derived from `Very Low (XB/PS2*)`, without the Legacy-style enhancements
  - `Very Low` - Derived from `Handheld (Switch/Vita)`, without the Legacy-style enhancements
  - `Low` - Derived from `Low (XB360/PS3)`, without the Legacy-style enhancements
  - `Medium`, `High`, `Ultra`, `Extreme` use the same settings as previously
  - `Outrageous` - Derived from `Extreme`, but with a 128 chunk render distance.
  - `Default` and `Default+` have been removed, in favor of `Low`
  - Switching from a non-LCE to an LCE preset will display a prompt with a toggle to show the slider.
- The built-in config for those installing C2ME themselves is now mostly synced with Re-Console's config
- Enabled the `Rosenfeld Patch` resource pack in the `Minecraft Classic Texture Pack` resource album
- Disabled the `Legacy Baby Villager Head` by default, since Tiny Takeover increased the baby villager's head size anyways
- The Legacy4J Advanced Options mode is now set to Merge by default
- The default game difficulty is now Easy
- Increased mob activation range
  - The ServerCore mob activation range has been increased from 20 to 96.
- Fixed default fullscreen mode
  - A now properly set Cubes Without Borders default config has been added, so Fullscreen should once again be enabled by default.

## Temporarily Unavailable
- Legacy Skins
