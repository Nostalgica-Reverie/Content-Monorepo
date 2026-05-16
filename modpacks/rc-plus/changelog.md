# Re-Console Plus 26.05.4 | Megapatch II
Large patch for Re-Console+ for Minecraft 26.1.2, adding in new features, fixes, and QoL changes on top of Megapatch I. 

## Notes
- **If you are updating from a 1.21.10 version, it is recommended to Update or Reset your configs through Config Manager for the best experience.**
  - This can be found in `Modifications > Config Manager`.
- This update is marked as beta due to major internal changes between 1.21.10 and 26.1.2, the relatively early state of the Legacy4J 26.1.2 port as of 1.9 Pre-Release 2, and missing features that may impact worlds created prior to 26.05-alpha
- Releases on CurseForge will come at a later date, as we approach stability on the build.

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

### Updated Legacy4J
- Now on Pre-Release 2
- Fixed a bug when using the Load Save Directly option would delete worlds
- Fixed compatibility issues with Sodium
- Fixed tutorial world not being properly converted to 26.1.2
- Cjnator38 added better translations for some of the added tips
- Cjnator38 reordered the options and unified the size of advanced options panels
Thanks to WilyIcaro, CreeperEater201 and Cjnator38 for this release.

### Updated Minimega to 6.1.20
Various Battle changes and bug fixes. See the [changelog](https://modrinth.com/mod/minimega/version/6.1.20).

### Updated Legacy Ports
Additions and bug fixes for Minecraft 26.1

### Updated Legacy Modpack Resources
Fixes various issues with Option Presets and adds in the new Outrageous preset.

## Temporarily Unavailable
- Console Advancement Sounds
- Legacy Mechanics
- Legacy Nether: Enhanced
- Legacy Skins
- Particle Core
- Polytone