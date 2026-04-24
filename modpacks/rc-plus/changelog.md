# Re-Console Plus 26.04.10
A major update for Re-Console Plus, Simply Legacy and Legacy4J. This updates us to Minecraft 26.1.2, and gives us numerous new features.

This changelog encompasses all changelogs from 26.04.2 to 26.04.9, as well.

## Note
- If you're updating from 26.04.1 and earlier, it is recommended to update or reset your configs through Config Manager for the best experience.
  - This can be found in `Modifications > Config Manager`.

### Updated Legacy4J
- More SD UI screens
- Legacy Settings Menus to allow options menus to match LCE
- Legacy Mobcap Limits, Shield Blocking and Offhand Limits
- Legacy Clouds and Cloud Height
- Screenshot Toasts
- Skin Changing Menu
- Decay Potions
- Master Volume is now split into Music and Sound properly
- Improved map tooltip and handling
- Creative Mode Elytra momentum handling
- Advanced Options with Legacy Settings Menus can now show contents from the Merge Advanced Options Mode 
- List buttons now use their own sprite
- By default, the Preset slider will show up with Legacy Settings Menus enabled 
- Various bug fixes
- and more!

### Added Preferred Gamerules
- The new `Legacy Shield Blocking`,  `Legacy Offhand` and `Legacy Mobcap Limits` gamerules have been enabled by default.
- `Locator Bar` and `Announce Advancements` have been disabled by default. 

### Added Cull Fewer Leaves
Replaces MoreCulling.

### Added Gnetum
Buffers framerate on GUI elements to improve performance

### Refactored Options Presets
- `"Retro" LCE` - Replaces `Very Low (XB/PS2*)`, now with more Legacy-style enhancements.
- `Handheld LCE` - Replaces `Handheld (Switch/Vita)`, now with more Legacy-style enhancements.
- `Old-Gen LCE` - Replaces `Low (XB360/PS3)`, now with more Legacy-style enhancements
  - This will be the new default preset.
- `New-Gen LCE - Derived from `Medium (XB1/PS4)`, now with more Legacy-style enhancements.
- `"Next-Gen" LCE` - Derived from `High (XB Series/PS5)`, now with more Legacy-style enhancements.
- `Potato` - Derived from `Very Low (XB/PS2*)`, without the Legacy-style enhancements
- `Very Low` - Derived from `Handheld (Switch/Vita)`, without the Legacy-style enhancements
- `Low` - Derived from `Low (XB360/PS3)`, without the Legacy-style enhancements
- `Medium`, `High`, `Ultra`, `Extreme` use the same settings as previously
- `Default` and `Default+` have been removed, in favor of `Low`

### Removed KryptonFNP Patcher
No longer maintained.

### Removed Tooltips Enhanced
Now that we're on a build of Legacy4J that includes these control icons by default, Tooltips Enhanced has been removed.

### Updated Fabric Loader to 0.19.2
The modpack has been updated to the latest Fabric Loader.

### Adjusted the default SVC HUD
- Voice chat status icons are shown in the top-right corner, just right of the potion/status effect HUD.
- Group icons will populate from the bottom-right corner.

## Temporarily Unavailable
- Pro Placer
- Particle Core
- Polytone
- Legacy Skins
- Console Advancement Sounds

- 8723ff6d chore(simply/rc): bump ver - Cjnator38
- d57e8221 chore(simply/rc): update ca modlist - Cjnator38
- 2d4dfdc6 fix(simply/rc): update fnp patcher - Cjnator38
- 8946306b chore(rc): update golden days - Cjnator38
- b0c38557 fix(simply/rc): update loader on cf - Cjnator38
- 9b174985 feat(simply/rc): re-add minimega - Cjnator38
- 9520e283 fix(rc): cf too - Cjnator38
- 082b08b4 fix(rc): remove unneeded fabric deps file - Cjnator38
- 1033ed62 chore(repo): improve readmes and update licenses and mcmetas - omo50
- 093b7d83 chore(simply/rc): rereredowngrade e4mc - Cjnator38
- c05e75ae chore(simply/rc): update l4j/facapi - Cjnator38
- a238f6ac chore(rc and simply): update mods - omo50
- b034340a chore(rc): update mods - omo50
- 64237b5c chore(rc): delete 26.1.2 unused dir - omo50
- fbcc923e feat(simply/rc): new default pos for svc icons - Cjnator38
- 2d7ac6c1 chore(rc & simply): bump again - omo50
- 532b507f fix(rc): disable dr - omo50
- 5653e19e chore(rc & simply): bump manifest - omo50
- 6359900c chore(rc): changelog - omo50
- 0c04036e chore(simply/rc): refresh - Cjnator38
- 90613f81 chore(simply): update ca modlist (mr), refresh - Cjnator38
- acb6f605 feat(simply/rc): default svc configs - Cjnator38
- 092ad09b chore(rc): remove plasmo configs - Cjnator38
- 938de62e chore(rc): update fixed chest models bio - omo50
- 8b86760a chore(rc and sl): remove deprecated dev folder - omo50
- 5d8e7a10 feat(rc): switch back to svc - omo50
- 12f15dbc actions: auto-update (Auto Update & Refresh) - forgejo-actions[bot]
- fdcca0cb actions: auto-update (Auto Update & Refresh) - forgejo-actions[bot]
- 8dcdc242 chore(simply/rc): bump manifest - Cjnator38
- 7917249b fix(simply/rc): remove minimega actually - Cjnator38
- 7a978f63 chore(simply/rc): bump version/manifest - Cjnator38
- 494e6fbe chore(simply/mr): update ca modlist - Cjnator38
- 257fbd45 fix(simply/rc): correct the file name - Cjnator38
- 4472184c fix(simply/rc): force moreculling config - Cjnator38
- d84d66e3 fix(simply/rc): add special mm build - Cjnator38
- 2afea577 fix(simply/rc): re-downgrade e4mc to 6.0.6 - Cjnator38
- 36f1a881 chore(simply/rc): mirror to modpack defaults - Cjnator38
- 440bb678 fix(simply/mr): make lang stuff consistent - Cjnator38
- 46e3b79c fix(simply/rc): readd legacy skins - Cjnator38
- 0cb6c2e5 chore(rc): update manifest - omo50
- 70e35460 chore(rc): update changelog - omo50
- f8ef9192 fix(rc): fix file name - omo50
- bea5b768 chore(rc): bump ver - omo50
- ccca06b4 fix(rc): minor bugs - omo50
- a5f29819 fix(rc-plus): plasmovoice icon and fac api - omo50
