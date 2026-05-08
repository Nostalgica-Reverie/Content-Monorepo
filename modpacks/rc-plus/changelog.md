# Re-Console Plus 26.05.3-alpha | Megapatch I
Large patch for Re-Console+ for Minecraft 26.1.2, adding in a suite of new features on top of the modpack.

## Notes
- **If you are updating from a 1.21.10 version, it is recommended to Update or Reset your configs through Config Manager for the best experience.**
  - This can be found in `Modifications > Config Manager`.
- This update is marked as beta due to major internal changes between 1.21.10 and 26.1.2, the relatively early state of the Legacy4J 26.1.2 port as of 1.9 Pre-Release 1, and missing features that may impact worlds created prior to 26.05-alpha
- Releases on CurseForge will come at a later date.

### Added ModernRegSyncFix
A fork of RegSyncFix that works on 26.1.2 and above. Special thanks to DexrnZacAttack for the fork!

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

### Added Gnetum
GUI issues have been fixed, and now the mod should offer a considerable performance uplift, regarding GUIs and HUDS. They are now FPS limited and batched, allowing for higher performance in game, where it actually matters.

### Added Experimentalist
Fixes any experimental warnings on the creation screen.

### Added Async Logger
Improving logging performance, by allowing it to run asynchronously. For long sessions on the SMP, this could improve logging speeds substantially (up to 43x).

### Added Picture Mode
A mod by Icanttellyou, allowing for alpha/beta style isometric screenshots. Compatibility with controller is currently not known and untested. Bound to `CTRL+3`

### Added Longview
Longview is a mod that fixes zfighting issues with mobs far away in render distance.

### Added Cave Fog Stabilizer
This is a mod that fixes a visual quirk where fog in caves is impacted by the sunrise/sunset above ground.

### Added Animatium
Animatium provides many neat utilities and old features back into Minecraft!

### Added WorldThreader
Each dimension is now ran on its own thread, improving performance. This means that for higher threaded CPUS (6+), the game should run significantly smoother.

### Added Krypton & Krypton Reno
Both mods can improve connectivity, especially if you run Re-Console+ via server.

### Added Fast Server Pings
Minor fix that will improve speed of accessing servers, like the Re-Console SMP.

### Added Debugify
Expands upon BugFixerUpper by fixing dozens of vanilla bugs.

### Added RenderScale
Allows for scalable resolutions and screenshots. Bound to `SHIFT+R`

### Added AMECs
Allows for binding of keys to multiple, allowing for keybinds like shift+, ctrl+, and more.

This impacts some default keybinds, please read below!

### Default config changes/fixes
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
Duplicated mods, like Zoomify

### Removed Better Block Entities
The mod has been causing consistent trouble in regards to performance. The mod has now been removed.

### Updated Minimega to 6.1.10
Various Battle changes and bug fixes. See the [changelog](https://modrinth.com/mod/minimega/version/6.1.10).

### Updated Legacy Ports
Additions and bug fixes for 26.1

### Updated Legacy Modpack Resources
Fixes various issues with Option Presets

## Temporarily Unavailable
- Console Advancement Sounds
- Legacy Mechanics
- Legacy Nether: Enhanced
- Legacy Skins
- Particle Core
- Polytone
