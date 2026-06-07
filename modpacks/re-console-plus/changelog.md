# Re-Console Plus 26.06.1
Curseforge port of 26.1.2, and a few fixes.

This also is the first release using our more modular system of development. If there are any bugs, please report them!

26.06.1 is a small patch removing known problematic mods

# Content Additions

### Added Bedrock Skins
This should integrate with the Skin Change Menu nicely

# CurseForge Changes

### Updated Legacy4J
- Now on Pre-Release 2
- Fixed a bug when using the Load Save Directly option would delete worlds
- Fixed compatibility issues with Sodium
- Fixed tutorial world not being properly converted to 26.1.2
- Cjnator38 added better translations for some of the added tips
- Cjnator38 reordered the options and unified the size of advanced options panels
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

### Adjusted the default SVC HUD
- Voice chat status icons are shown in the top-right corner, just right of the potion/status effect HUD.
- Group icons will populate from the bottom-right corner.

### Refactored Options Presets
- `"Retro" LCE` - Replaces `Very Low (XB/PS2*)`, now with more Legacy-style gui enhancements.
- `Handheld LCE` - Replaces `Handheld (Switch/Vita)`, now with more Legacy-style gui enhancements.
- `Old-Gen LCE` - Replaces `Low (XB360/PS3)`, now with more Legacy-style gui enhancements
- `New-Gen LCE - Derived from `Medium (XB1/PS4)`, now with more Legacy-style gui enhancements.
- `"Next-Gen" LCE` - Derived from `High (XB Series/PS5)`, now with more Legacy-style gui enhancements.
- `Potato` - Derived from `Very Low (XB/PS2*)`, without the Legacy-style enhancements
- `Very Low` - Derived from `Handheld (Switch/Vita)`, without the Legacy-style enhancements
- `Low` - Derived from `Low (XB360/PS3)`, without the Legacy-style enhancements
- `Medium`, `High`, `Ultra`, `Extreme` use the same settings as previously
- `Default` and `Default+` have been removed, in favor of `Low`

### Updated Fabric Loader to 0.19.2
The modpack has been updated to the latest Fabric Loader.

## Content Additions

### Added Picture Mode
A mod by Icanttellyou, allowing for alpha/beta style isometric screenshots. Compatibility with controller is currently not known and untested. Bound to `CTRL+3`

### Added Animatium
Animatium provides many neat utilities and old features back into Minecraft!

### Added Skyboxify
Restores Optifine compatibility in regards to sky-modifying texture packs.

### Added Animatica Refabricated
Restores Optifine compatibility in regards to animated textures.

### Added Advancement Screenshot
Take a screenshot anytime you advance!

### Added Outrageous Preset
Go beyond Extreme with 128 render distance! Requires an extremely beefy computer to run.

### Added RenderScale
Allows for scalable resolutions and screenshots. Bound to `SHIFT+R`

### Added Borderless Mining Updated
A proper borderless fullscreen mod that does its job well, and improves performance vs alternative.

### Added Preferred Gamerules
- The new `Legacy Shield Blocking`,  `Legacy Offhand` and `Legacy Mobcap Limits` gamerules have been enabled by default.
- `Locator Bar` and `Announce Advancements` have been disabled by default. 

### Added Cull Fewer Leaves
Replaces MoreCulling.

### Added Gnetum
Buffers framerate on GUI elements to improve performance.

### Added Ok Zoomer
Mod no-longer incompatible with Minimega.

### Added BugFixerUpper
A minor mod that fixes numerous small bugs in the game.

### Added new Texture Packs
- Added Ultra High-Res Texture Pack
- Added Classic High-Res Texture Pack
- Added Player vs. Player Texture Pack
- Added Ashen Texture Pack
- Added Realistic Texture Pack
- Added Whimsical Texture Pack

### Added new Shader Packs
Following the update to 26.1.2, shader packs had been missing for a while. They have been re-added with new additions.

Re-added packs:
- Complementary Reimagined
- Complementary Unbound
- BSL Shaders
- Rethinking Voxels
- Solas Shaders
- MakeUp
- Bliss Shaders
- Kappa
- Super Duper Vanilla

New Packs:
- Nostalgia Shader
- Shrimple Shader
- Noble Shader
- RenderPearl
- RedHat Shaders

### Added Community Splashes Pack
A pack of community splashes, from the Discord and Re-Console SMP! The current list of splashes:
- Insects with Instruments!
- Your friend wont murdurr!
- Little lad go!
- Big lad go!
- Nocholas_____ hit the ground too hard!
- Eggs!
- Buy MusCola™️!
- Beware of the Evil Re-Console!
- Also check out Simply Legacy!
- Also check out Rekindled Legacy!
- Also check out 2000s Edition!
- Also check out Consoleidated!
- Don’t check out Evil Consoleidated!
- Spears can Jab 1-to-5!
- The steak The Potato
- Bomboclat Wallahi
- Anything but modpack development
- Punkinville!
- Mälmö!
- Petoria!
- The Safe Zone!
- Post Town!
- The Emporium
- Should i mace him?
- The carrot caravan!
- Dexron Zekeatlas!
- Also check out Legacy Edition Minigames!
- Also check out Silver Lining!
- Also check out Reminiscence!
- Check out the Re-Console SMP!
- What is the meaning of "Affogato"?
- Affogato Reverie?
- Nostalgica Reverie!
- Lasting Legacy!
- Violaflower!
- Reverie Projects!
- Where is Gerald?
- If you mash strawberries…
- Pixel Peeping since 2023!
- Born on April 28th 2024!
- Born on April 24th 2024!
- ..When was this pack born?
- Legacy4J!
- Legacy Skins!
- Minimega!
- What is a Re-Console+?
- Not Bug Free!
- smp.nostalgica.net!
- Also check out AydenFYP!
- Also check out TheMinecraftArchitect!

### Added Bisect Hosting Integration
You can now order a server for Re-Console+ or other modpacks in-game, with all proceeds supporting us. This was designed to be unintrusive; only appearing on the multiplayer screen.

### Updated Vanilla Plus Texture Pack
The pack has been updated with new flowers in grass, 3d item drops, and a few smaller things.

## Performance Improvements

### Added Gnetum
GUI issues have been fixed, and now the mod should offer a considerable performance uplift, regarding GUIs and HUDS. They are now FPS limited and batched, allowing for higher performance in game, where it actually matters.

### Added Async Logger
Improving logging performance, by allowing it to run asynchronously. For long sessions on the SMP, this could improve logging speeds substantially (up to 43x).

### Added WorldThreader
Each dimension is now ran on its own thread, improving performance. This means that for higher threaded CPUS (6+), the game should run significantly smoother.

### Added Krypton Reno
This can improve connectivity, especially if you run Re-Console+ via server.

## Quality of Life Improvements

### Added ModernRegSyncFix
A fork of RegSyncFix that works on 26.1.2 and above. Special thanks to DexrnZacAttack for the fork!

### Added Experimentalist
Fixes any experimental warnings on the creation screen.

### Added Longview
Longview is a mod that fixes zfighting issues with mobs far away in render distance.

### Added Cave Fog Stabilizer
This is a mod that fixes a visual quirk where fog in caves is impacted by the sunrise/sunset above ground.

### Added Fast Server Pings
Minor fix that will improve speed of accessing servers, like the Re-Console SMP.

### Added Debugify
Expands upon BugFixerUpper by fixing dozens of vanilla bugs

### Added AMECs
Allows for binding of keys to multiple, allowing for keybinds like shift+, ctrl+, and more.

This impacts some default keybinds, please read below!

## Default config changes/fixes
- The `Medium` Option Preset should now be enabled properly by default
- The default window resolution is now 854x480 to match Simply Legacy
- The Inventory/Crafting keys have been changed from `I`/`E` to `E`/`R`
- The Reload Shaders key for Iris has been changed to `U`, which is also in close proximity to the other Iris hotkeys
- The Simple Voice Chat key has been moved to `CTRL+C`
- The Isometric Screenshot key has been set to `CTRL+3`
- The Change Music button has been set to `SHIFT+F`
- The Pause Music button has been set to `SHIFT+G`
- The Dynamic Resources option in ModernFix is once again enabled, which should provide a large decrease to RAM usage (sometimes over 50%!)
- The Dynamic Languages option in ModernFix is now enabled.
- The F3 config has been improved to add in more system information on the right, alongside providing improved readability when it comes to the left.
- The Gnetum mod now has a default config applied, to fix the GUI issues seen in 26.05.1.
- The window title will now say Re-Console+ for Minecraft 26.1.2 instead of just Minecraft 26.1.2
- There have been fixes applied to the Crash Assistant pop-up, removing leftovers from when the 26.1.2 development process was based upon Simply Legacy.

### Fixed resource packs
Various built-in resource packs have been fixed via changes to their pack.mcmetas. This applies to:
- Fixed Chest Models
- Re-Console+ Resources

### Removed duplicated mods
Duplicated mods, like Zoomify, which were left-overs from an unfinished development build.

### Removed Better Block Entities
The mod has been causing consistent trouble in regards to performance. The mod has now been removed.

This mod had caused issues in regards to Bobby (rendering more chunks than servers allow), issues with mash-up models, and the performance increase is simply not worth the hassle of all of these issues.

### Removed KryptonFNP Patcher
No longer maintained.

### Removed Tooltips Enhanced
Now that we're on a build of Legacy4J that includes these control icons by default, Tooltips Enhanced has been removed.

### Removed BadOptimizations
Mod is more placebo than it is actual benefits.

### Updated Legacy Ports
Additions and bug fixes for Minecraft 26.1

### Updated Legacy Modpack Resources
Fixes various issues with Option Presets and adds in the new Outrageous preset.