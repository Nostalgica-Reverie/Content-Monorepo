# Re-Console Plus 26.05.3-alpha | Megapatch I
Large patch for Re-Console+ 26.1.2

## Notes
- **If you are updating from a 1.21.10 version, it is recommended to Update or Reset your configs through Config Manager for the best experience.**
  - This can be found in `Modifications > Config Manager`.
- This update is marked as beta due to major internal changes between 1.21.10 and 26.1.2, the relatively early state of the Legacy4J 26.1.2 port as of 1.9 Pre-Release 1, and missing features that may impact worlds created prior to 26.05-alpha
- Releases on CurseForge will come at a later date.

### Added ModernRegSyncFix
A fork of RegSyncFix that works on 26.1.2 and above. Special thanks to DexrnZacAttack for the fork!

### Added Community Splashes Pack
A pack of community splashes, from the Discord and Re-Console SMP!

### Added Gnetum
GUI issues have been fixed, and now the mod should offer a considerable performance uplift, regarding GUI's and HUDS. They are now FPS limited and batched, allowing for higher performance in game.

### Added Async Logger
Improving logging performance, by allowing it to run asynchronously. For long sessions on the SMP, this could improve logging speeds substantially and potentially fix minor lag issues.

### Added Picture Mode
A mod by Icanttellyou, allowing for alpha/beta style isometric screenshots. Compatibility with controller is currently not known.

### Added Longview
Longview is a mod that fixes zfighting issues with mobs far away in render distance.

### Added Cave Fog Stabilizer
This is a mod that fixes a visual quirk where fog in caves is impacted by the sunrise/sunset above ground.

### Added Animatium
Animatium provides many neat utilities and old features back into Minecraft!

### Added WorldThreader
Each dimension is now ran on its own thread, improving performance. This means that for higher threaded CPUS (6+), the game should run significantly smoother.

### Added Fast Server Pings
Minor fix that will improve speed of accessing servers, like the Re-Console SMP.

### Added Debugify
Expands upon BugFixerUpper by fixing dozens of vanilla bugs.

### Default config changes/fixes
- Cubes Without Borders now has a working default config that should fix Fullscreen being disabled by default
- The `Medium` Option Preset should now be enabled properly
- The default window resolution is now 854x480 to match Simply Legacy
- The Inventory/Crafting keys have been changed from `I`/`E` to `E`/`R`
- The Reload Shaders key for Iris has been changed to `U`, which is also in close proximity to the other Iris hotkeys
- The Simple Voice Chat key has been moved to `CTRL+B`
- The Isometric Screenshot key has been set to `CTRL+3`
- The Change Music button has been set to `SHIFT+F`
- The Dynamic Resources option in ModernFix is once again enabled, which should provide a large decrease to RAM usage
- The F3 config has been improved to add in more system information on the right, alongside providing improved readability when it comes to the left.
- The Gnetum mod now has a default config applied, to fix the GUI issues seen in 26.05.1.

### Fixed numerous resource packs
Various built-in resource packs have been fixed via changes to their pack.mcmetas. This applies to:
- Fixed Chest Models
- Re-Console+ Resources

### Removed duplicated mods
Duplicated mods, like Zoomify

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
