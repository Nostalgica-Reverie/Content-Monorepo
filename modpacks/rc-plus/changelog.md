# Re-Console Plus 26.04.9
## Note (for Modrinth users)
- If you're updating from 26.04.1 and earlier, it is recommended to update or reset your configs through Config Manager for the best experience.
  - This can be found in `Modifications > Config Manager`.

### Adjusted the default SVC HUD
- Voice chat status icons are shown in the top-right corner, just right of the potion/status effect HUD.
- Group icons will populate from the bottom-right corner.

### Added Minimega (Modrinth)
Most, if not all, of the critical bugs from the previous custom development builds have now been fixed.

### Updated Legacy4J (Modrinth)
- We are now using a proper in-development build, based on a [GitHub Actions run](https://github.com/Wilyicaro/Legacy-Minecraft/actions/runs/24763878310) on the main branch. The L4J build number is now 1.8.7.2616.0
- List buttons now use their own sprite

### Updated Fabric Loader to 0.19.2 (CurseForge)
Fixes an issue that blocked CurseForge users from opening the game, due to Fast Noise requiring a newer version of the Fabric Loader.
