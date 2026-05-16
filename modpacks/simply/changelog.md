# Simply Legacy 26.05.6

## Notes
- **If you are updating from a 1.21.10 version, it is recommended to Update or Reset your configs through Config Manager for the best experience.**
  - This can be found in `Modifications > Config Manager`.
- This update is marked as beta due to missing features that may impact worlds created prior to 26.05-alpha. Legacy4J has become relatively stable as of Pre-Release 2.
- Releases on CurseForge will come at a later date.

### Configuration Changes
- The built-in config for those installing C2ME themselves is now mostly synced with Re-Console's config
  - hopefully this doesnt break with manual saving cuz i never check c2me :P

### Notable Updates
- Re-added Legacy Nether: Enhanced
  - Blaze loot and Wither Skeleton spawning should now work again
  - Nether Wart generating on Soul Sand currently does not work
- Updated Minimega from 6.1.20 to 6.2.2
  - Data-driven map loading and map creation tools
  - See the full changelogs [here](https://modrinth.com/mod/minimega/changelog)
- Added Gnetum
  - Improves user interface performance
  - Compared to Re-Console, now only the Experience Level and Info Bar elements have had their caching disabled
    - The Experience Level was the element causing issues with flashing previously
    - The Info Bar has visual issues when caching is enabled
- Added KryptonReno's Fabric Patcher
  - Improves networking performance
- Added WorldThreader
  - Improves multi-dimension performance, especially on higher threaded CPUs
- Added Async Logger
  - Improves logging performance

## Temporarily Unavailable
- Console Advancement Sounds
- Legacy Mechanics
- Legacy Nether: Enhanced
- Legacy Skins
