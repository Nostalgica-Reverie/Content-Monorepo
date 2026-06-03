# Configuration
While much of Legacy4J's Settings can be navigated in the same way as on the original Legacy Console Edition, there are a lot of fundamental differences due to the amount of options available from vanilla Java Edition and L4J compared to LCE.

## Show Advanced Options
Many of the options categories will show a `Show Advanced Options` tooltip on the bottom corner if Advanced Options are available. This will give you access to options not available at the top-level options screen.

## Legacy4J Options
This houses all the options accessible through the Options and Advanced Options pages provided by L4J. <br>
Legacy4J Options can be accessed by pressing the `Y` key (by default) in a world or on the main menu, or by going to the mod's configuration page in the Mod Menu. 
<br>
<br>
<br>

# Options
Below is a list of all options (Vanilla or Legacy4J) accessible in `Help & Options > Settings` and the Legacy4J Options screen, with explanations on their function. <br>
Pages prefixed with "Advanced" are Advanced Options pages, and can be accessed in the Settings menus by activating `Show Advanced Options` with the provided key, or by clicking the tooltip.

## Mouse Settings
#### Sensitivity
Sets the sensitivity of the mouse for controlling the camera.
#### Scroll Sensitivity
Sets the sensitivity of the scroll wheel for hotbar selection and Spectator Mode speed.
#### Invert Mouse X
Inverts the camera's horizontal movement.
#### Invert Mouse Y
Inverts the camera's vertical movement.
#### Discrete Scrolling
Forces the game to register each scroll input as a single scroll input, ignoring any scroll acceleration set by the operating system.
#### Touchscreen Mode
Changes certain in-game mouse inputs to work better with a touchscreen.
#### Cursor At First Inventory Slot
Determines if opening an inventory or container will place the cursor in the first hotbar slot.
#### System Cursor
Allows the operating system's mouse cursor to display instead of the in-game cursor.

## Key Binds
#### Reset to Defaults
Resets all the keyboard binds to their defaults.
#### Unbind Conflicting Keys
Automatically removes the older bind when setting a bind with a key that has already been set.
#### Toggle Sneak
Allows sneaking to be triggered by toggling a key rather than holding it.
#### Toggle Sprint
Allows sprinting to be triggered by toggling a key rather than holding it.
#### Toggle Use
Allows item usage to be triggered by toggling a key rather than holding it.
#### Toggle Attack
Allows attacking to be triggered by toggling a key rather than holding it.

## Controller
#### Reset to Defaults
Resets all the controller binds to their defaults.
#### Unbind Conflicting Keys
Automatically removes the older bind when setting a bind with a key that has already been set.
#### Display Connnection Toasts
Displays a pop-up notification when a controller is connected or disconnected.
#### Toggle Sneak
Allows sneaking to be triggered by toggling a button rather than holding it.
#### Toggle Sprint
Allows sprinting to be triggered by toggling a button rather than holding it.
#### Toggle Use
Allows item usage to be triggered by toggling a button rather than holding it.
#### Toggle Attack
Allows attacking to be triggered by toggling a button rather than holding it.
#### Invert Buttons Layout
Flips the `X`/`Y` and `A`/`B` face buttons.
#### Virtual Cursor
Allows the controller to control the cursor independently of the mouse cursor. Helpful for multi-instance gameplay.
#### Legacy Cursor Movement
Allows the controller cursor to more easily lock to straight lines.
#### Limit Cursor to Inventory Bounds
Prevents the controller cursor from leaving the outer bounds of inventory and container interfaces.
#### Double-Click In Inventory
Enables the double-click features for inventory management. Can cause unintentional inputs if activating certain functions too fast.
#### Cursor At First Inventory Slot
Determines if opening an inventory or container will place the cursor in the first hotbar slot.
#### Controller
Sets the active controller. Helpful for multi-instance gameplay with multiple controllers connected.
#### Controller Handler
Sets the active controller library to use, or disables controller support.
#### Sensitivity
Sets the maximum speed of the joystick camera.
#### Left Stick Dead Zone
Sets the inner dead zone for the left joystick.
#### Right Stick Dead Zone
Sets the inner dead zone for the right joystick.
#### Left Trigger Dead Zone
Sets the inner dead zone for the left trigger.
#### Right Trigger Dead Zone
Sets the inner dead zone for the right trigger.
### Movement
#### Invert Look
Inverts the camera's vertical movement.
#### Smooth Movement
Allows gradual movement using the analog stick.
#### Force Smooth Movement
Forces the ability to have gradual movement using the analog stick.
#### Linear Camera Movement
Adjusts the responce curve of the joystick camera to be linear instead of exponential, like in Bedrock Edition.

## Game Options
#### Auto-Jump
Allows the player to automatically jump when approaches a block with a block of air above it.
#### View Bobbing
Displays bobbing animations for the viewmodel hand and camera.
#### Hints
Displays hints for newly discovered blocks and items when in-game.
#### Flying View Rolling
Allows the camera to have a "rolling" animation when flying with an Elytra.
#### Autosave Interval
Adjusts the amount of time between autosaves in five minute intervals. OFF disables autosave, and will fallback to manual saving if [Save Cache](#save-cache) is enabled.
#### Languages
Opens the language selection screen.
#### Mods
Opens the mod list, with mod configuration screens available if [Mod Menu](https://modrinth.com/mod/modmenu) is installed.

## Advanced Game Options
### In-Game Settings
#### Force Active Window
Allows an assigned gamepad to control the game, even when the window is unfocused. Helpful for multi-instance gameplay.
#### Inverted Front Camera Pitch
Makes the player's head look towards the camera when in the Third Person Front camera mode.
#### Head Follows The Camera
Makes the player's head lock to the camera position, rather than lagging behind.
#### Vehicle Camera Rotation
Allows the camera to rotate with certain types of entities:
- NONE - The camera will not rotate when on any mount.
- All Entities - The camera will rotate when on any mount.
- Only Non-Living Entities - The camera will only rotate when on a non-living entity, such as a Minecart.
- Only Living Entities - The camera will only rotate when on a living entity, such as a Horse or Llama.
#### Rotate with Minecarts
This option only applies to worlds with the [Minecart Improvements](https://minecraft.wiki/w/Minecart_Improvements) data pack enabled. This will not affect Legacy4J's Vehicle Camera Rotation in worlds without it installed.
#### Legacy Creative Block Placing
Allows block placement in Creative Mode to match the speed of the player.
#### Maps With Coordinates
Displays coordinate information on any Map item.
#### Vanilla Tutorial
Displays the tutorial pop-ups from vanilla (i.e. "Press WASD to move")
#### Legacy Sword Blocking
Enables sword blocking mechanics.
#### Squared View Distance
Allows chunks on the edge of Render Distance to render in a square shape, rather than a circle.
#### Legacy Combat
Adjusts various tool and weapon attributes and disables weapon cooldowns.

### User Interface Settings
#### Skip Intro
Skips the intro sequence once the game is finished loading.
#### Skip Initial Save Warning
Skips the `This game has a level autosave feature` screen shown after the intro sequence.
#### Lock Control Type Change
Prevents the `Java Edition` Control Type taking over when using a Keyboard & Mouse.
#### Control Type
Changes the Control Icons and Minecraft logo (if available).
#### Cursor Mode
Controls how the mouse cursor should be shown. (see [Controls: Cursor Mode](./controls#cursor-mode))
#### Default Show Craftable Recipes
Determines if `Show Craftable Recipes` is shown first when opening a Legacy Crafting interface.

### Accessibility Settings
#### Notification Time
Determines the length of notification pop-ups (such as Advancement toasts).
#### Panorama Scroll Speed
Adjusts the speed of the panorama. Affects both the Vanilla and Legacy Panorama.
#### Narrator
Allows the narrator to narrate various parts of the user interface.
#### Narrator Hotkey
Allows the `Ctrl`+`B` keyboard shortcut for toggling the narrator.
#### Monochrome Logo
Makes the background for the loading screen black instead of red. Only works if [Legacy Loading & Reloading](./configuration#legacy-intro-reloading) is disabled.
#### High Contrast
Enables the High Contrast resource pack.
#### Hide Sky Flashes
Prevents Lightning Bolts or other environmental effects from making the sky flash. The sources of flashes themselves will still be visible.
#### Damage Tilt
Adjusts the intensity of the camera tilt from taking damage.
#### Distortion Effects
Adjusts the intensity of the Nether Portal and Nausea camera effects.
#### FOV Effects
Adjusts the intensity of various field-of-view gameplay effects, such as sprinting and potion effects.
#### Darkness Pulsing
Adjusts the intensity of the Darkness effect given by a Warden or Sculk Shrieker.
#### Glint Speed
Adjusts the speed of the enchantment glint.
#### Glint Strength
Adjusts the intensity of the enchantment glint.

### Save Settings
#### Autosave When Paused
Allows autosaves to trigger when pausing the game.
#### Load Save Directly
Allows joining a world without showing the `Load Save` screen.
#### Save Cache
Enables the manual saving system.

### Miscellaneous
#### Realms News & Invites
Toggles Realms notifications.
#### Allow Server Listings
Servers may list online players as part of the public status. With this option off, your name will not show up in such lists.
#### Reset Known Blocks
Resets the list of blocks already shown as an in-game hint.
#### Reset Known Entities
Resets the list of entities already shown as an in-game hint.

## Audio
#### Music
Adjusts the volume for background music.
#### Sound (Master Volume)
Adjusts the volume for all in-game content, including background music.
#### Cave Sounds
Toggles whether ambient cave sounds can be heard.
#### Minecart Sounds
Toggles whether Minecart travelling sounds can be heard.

## Advanced Audio
#### Device
Sets the audio output for the game.
#### Back Sound
Toggles whether a sound is played when backing out of interfaces.
#### Hover Focus Sound
Toggles whether the navigation sound is heard when hovering over UI elements with the mouse.
#### Inventory Hover Focus Sound
Toggles whether the navigation sound is heard when navigating within an inventory or container UI.
#### Closed Captions
Toggles whether subtitles are shown in the UI.
#### Directional Audio
Uses HRTF-based directional audio to improve the simulation of 3D sound. Requires HRTF compatible audio hardware, and is best experienced with headphones.
#### Music Frequency
Adjusts the frequency of the background music.
#### Show Music Toast
Shows a notification when a new background music track plays, with the song artist and title.
#### Jukebox/Note Blocks
Adjusts volume for the Jukebox and Note Blocks
#### Weather
Adjusts volume for various weather events.
#### Blocks
Adjusts volume for various block interactions.
#### Hostile Mobs
Adjusts volume for hostile mobs.
#### Friendly Mobs
Adjusts volume for passive mobs and non-living entities.
#### Players
Adjusts volume for player interactions (such as damage and item pickup).
#### Ambient/Environment
Adjusts volume for various ambient sounds, entering/exiting water, and fireworks.
#### Narrator/Voice
Adjusts volume for the narrator.
#### UI
Adjusts volume for user interface interactions.

## Graphics
#### Clouds
Adjusts how clouds are rendered.
#### Preset
Adjusts the selected options preset.
#### Display Gamma
Toggles whether the LCE Gamma should be displayed.
#### Gamma
Adjusts the intensity of the gamma shown over the display.
#### Brightness
Adjusts the brightness of in-game surfaces and shadows.
#### Smooth Lighting
Toggles ambient occlusion, which smooths the game's lighting.
#### Global Resource Packs
Sets the resource packs that will always be applied, regardless of Resource Album. Applies on top of albums by default. (see [Resource Pack Management](./resource-pack-management))
#### Resource Albums
Sets a profile of multiple resource packs to be applied at once. (see [Resource Pack Management](./resource-pack-management))

## Advanced Graphics
### Video Settings
#### Fullscreen Resolution
Sets the resolution the display will use when in Fullscreen mode.
#### Fullscreen
Toggles fullscreen mode.
#### Graphics
Sets the game's graphics mode.
#### VSync
Toggles whether the game should sync its framerate to the monitor's refresh rate.
#### Max Framerate
Sets the maximum frames-per-second the game can run at.
#### FOV
Sets the camera's vertical field-of-view.
#### Reduce FPS when
Sets the condition that would reduce the framerate.
#### Render Distance
Sets the maximum distance the player is able to see in chunks.
#### Simulation Distance
Sets the maximum distance blocks and entities can tick.
#### Cloud Distance
Sets the maximum distance clouds can render.
#### Chunk Builder
Determines how chunks should be synchronized during a single frame.
#### Biome Blend
Sets the radius for the transition between biomes.
#### Entity Distance
Sets the distance entities can be seen at.
#### Entity Shadows
Allow entities to display a circular shadow texture below themselves.

### Legacy Settings
#### Override Terrain Fog Start
Allows changing the starting distance for environmental fog.
#### Terrain Fog Start
Sets the starting distance for environmental fog.
#### Override Terrain Fog End
Allows changing the end distance for environmental fog.
#### Terrain Fog End
Sets the end distance for environmental fog.
#### Legacy Sky Shape
Adjusts the sky shape to match LCE.
#### Fast Leaves When Blocked
Enables the culling of leaf blocks when blocked off on all sides.
#### Fast Leaves Custom Models
Displays a custom block model for fast leaves.
#### Display Name Tag Border
Displays a colored border around player name tags, corresponding to the player's color.
#### Display Held Item Lighting
Lights up the viewmodel (the hands) if any light-emitting block is held.
#### Enhanced Item Translucency
Renders the transparency in places such as the hotbar in 2D instead of 3D, so you wouldn't see the inside of a chest model, for example.
#### Display Loyalty Effect
Displays a reeling effect when a Trident with the Loyalty effect returns to you.
#### Display Trading Icon
Displays an icon above villagers who can be traded with.
#### Legacy Baby Villager Head
Makes the head on baby villagers bigger.
#### Legacy Fireworks
Renders fireworks as a 3D model with a different texture rather than a 2D sprite.
#### Legacy Evoker Fangs
Makes Evoker Fangs bigger to match LCE.
#### Legacy Drowned Animation
Renders a different swimming animation for the Drowned to match LCE.
#### Legacy Entity Fire Tint
Tints burning entities orange, similar to LCE and Bedrock Edition.
#### Legacy Item Pickup
Gives the item pickup animation variable speed, like in later versions of LCE.
#### Enhanced Piston Moving Renderer
Allows block entities to show during the piston's moving animation.
#### Legacy Potions Bar
Adds a level meter to all potions, similar to older versions of LCE.
#### Random Block Rotations (FactoryAPI)
Re-enables variable block rotation.
#### Default Particle Physics
Re-enables particle collision.
#### Bubbles Outside Water
Prevents bubble particles from disappearing when outside water.
#### Nearest Mipmap Scaling (FactoryAPI)
Makes the distant mipmaps use Nearest Neighbor scaling, achieving a look similar to LCE.
#### Mipmap Levels
Adjusts how many mipmap texture iterations can render. Disables when Nearest Mipmap Scaling is enabled.

### Mixins
#### Legacy Enchanting Screen
Enables the Legacy-style screen for the Enchanting Table.
#### Legacy Classic Merchant Screen
Enables the classic trading screen from the PS4 Edition.
#### Legacy Container-like Screen
Enables the Legacy-style screens for Chests and other containers.
#### Legacy Boss Health
Enables the Legacy-style boss health bar.
#### Legacy Title Screen
Enables the Legacy-style title screen.
#### Legacy Brewing Stand Screen
Enables the Legacy-style screen for the Brewing Stand.
#### Legacy Cartography Screen
Enables the Legacy-style screen for the Cartography Table.
#### Legacy Pause Screen
Enables the Legacy-style pause screen.
#### Legacy Classic Stonecutter Screen
Enables the classic Stonecutter screen from the PS4 Edition.
#### Legacy Gui
Enables the Legacy-style heads-up display.
#### Legacy Witches
Allows rendering of the Witch's purple cowl, seen in its texture map.
#### Legacy Book Screen
Enables the Legacy-style Book/Book & Quill screens.
#### Legacy Crafter Screen
Enables a Legacy-style screen for the Crafter.
#### Legacy Drowned
Allows the Legacy Drowned Animation to be applied.
#### Legacy Classic Loom Screen
Enables the classic Loom screen from the PS4 Edition.
#### Legacy Beacon Screen
Enables the Legacy-style screen for the Beacon.
#### Legacy Classic Crafting Screen
Enables the Legacy-style Classic Crafting screens.
#### Legacy Create World Screen
Enables the Legacy-style Create World screen.
#### Legacy Chat
Enables a Legacy-style chat screen.
#### Legacy Furnace Screen
Enables the Legacy-style screen for the Furnace.
#### Legacy Anvil Screen
Enables the Legacy-style screen for the Anvil.
#### Legacy Sign Screen
Enables a Legacy-style screen for all Sign blocks.
#### Legacy Pack Selection Screen
Enables a Legacy-style resource pack selection screen.
#### Legacy Smithing Screen
Enables a Legacy-style screen for the Smithing Table.
#### Legacy Grindstone Screen
Enables the Legacy-style screen for the Grindstone.
#### Legacy Inventory Screen
Enables the Legacy-style screen for the Player Inventory.

## User Interface
#### Display HUD
Toggles the visibility of the heads-up display. Separate from pressing F1.
#### Display Hand
Toggles the visibility of the viewmodel (hands/items). Separate from pressing F1.
#### Display Save Icon
Toggles the visibility of the save chest icon.
#### Display Vanilla Recipe Book
Toggles the visibility of the Recipe Book in certain screens.
#### Display Tooltip Boxes
Toggles the visibility of information sidebars in certain screens.
#### Attack Indicator
Toggles the visibility, or sets the location, of the attack cooldown indicator.
#### HUD Size
Sets the size of the heads-up display.
#### Interface Opacity
Sets the opacity of the Control Icons and heads-up display.
#### Interface Distance
Sets the distance for certain user interface elements.
#### GUI Scale
Sets the scale for the entire user interface.
#### UI Mode
Sets the user interface mode between Full HD (1080p), HD (720p) and SD (480p).
#### Interface Sensitivity
Sets the controller's cursor sensitivity.
#### Legacy Item Tooltips
Gives item tooltips a look similar to LCE.
#### Legacy Item Tooltips Scaling
Scales item tooltips depending on UI Mode.
#### In-Game Tooltips
Displays the Control Icons in the heads-up display.
#### Animated Character
Displays a paper doll in the corner of the heads-up display.
#### Smooth Animated Character
Renders the paper doll at the game's refresh rate.
#### Classic Crafting
Replaces the Crafting interface with one similar to vanilla Java Edition. This merges the Crafting and Inventory screens.
#### Classic Stonecutting
Replaces the Stonecutter interface with one similar to vanilla Java Edition.
#### Classic Loom
Replaces the Loom interface with one similar to vanilla Java Edition.
#### Classic Trading
Replaces the Villager Trading interface with one similar to vanilla Java Edition.
#### Force Mixed Crafting
Forces the Mixed Crafting UI (a recipe book frontend) to be used in place of the Legacy Crafting UI.
#### Mod Crafting Tabs
Adds Crafting and Creative tabs for mods with registered recipes.
#### Display Vanilla Tabs
Adds tabs from the Recipe Book and vanilla Creative Inventory to their respective interfaces.
#### Search Creative Tab
Adds a search tab to the Legacy Creative Inventory.
#### Operator Items Tab
Adds a tab with operator items to the Creative Inventory.
#### Vignette
Renders a vignette when in-game.
#### Display Control Tooltips
Toggles the visibility of Control Icons in all interfaces.

## Advanced User Interface
#### Advanced Options Mode ([Legacy4J Options](./configuration#legacy4j-options) only)
Determines whether `Show Advanced Options` is displayed or not, or if the Advanced Options interfaces would combine into the main option screens.
### In-Game Settings
#### Display Inverted Crosshair
Re-enables the inverted crosshair in the heads-up display.
#### Legacy Creative Interface
Enables the Legacy-style Creative Inventory.
#### Legacy Advancements Interface
Enables the Legacy-style Advancements menu.
#### Leaderboards Interface
Enables the Legacy-style Statistics menu.
#### Legacy Overstacked Items
Shows `99+` instead of the actual number of items in the stack if the stack size is above 99.
#### Legacy Health Flash
Renders the healthbar flash animation faster when regenerating health.
#### HUD Load Delay
Sets the amount of time the heads-up display is hidden when leaving a screen.
#### Display System Messages As Overlay
Re-enables system messages (such as sleep warnings) displaying in the action bar.
#### Autosave Countdown
Allows a five second countdown to display before an autosave.
#### Advanced Held Item Tooltip
Enables Advanced tooltips specifically for the held item tooltip.
#### Selected Item Tooltip Ellipsis
Adds ellipsis to the end of the held item tooltip if more information is available but cannot be shown.
#### Selected Item Tooltip Lines
Adjusts the amount of lines used in the held item tooltip.
#### Selected Item Tooltip Spacing
Adjusts the line spacing for the held item tooltip.
#### Control Tooltip Display
Sets which side of the screen the Control Icons should render at.

### Menu Settings
#### Title Screen Fade
Enables the fade animation when loading into the title screen.
#### Title Screen Version Text
Allows the version information to be displayed in the corner of the title screen.
#### Menus With Background
Allows non-Legacy interfaces to render a background overlay (like the dark background of Inventory menus in vanilla)
#### Legacy Intro & Reloading
Replaces the Mojang Studios logo with a Legacy-style intro sequence on game launch, and a black screen with a loading animation for resource reloading.
#### Legacy Loading & Connecting
Replaces the world loading and server connecting screen with a Legacy-style loading screen.
#### Legacy Panorama
Displays a 2D scrolling image as the background panorama.
#### Fake Autosave Screen
Displays a fake `Preparing to Autosave Level` screen when an autosave is triggered.
### Chat Settings
#### Reduced Debug Information
Blocks the visibility of certain information in the debug screen and debug renderers.
#### Chat
Toggles between what is visible within the in-game text chat.
#### Chat Text Opacity
Adjusts the opacity of text inside of chat.
#### Text Background Opacity
Adjusts the opacity of the background for chat.
#### Chat Text Size
Adjusts the size of text inside of chat.
#### Line Spacing
Adjusts the line spacing for chat.
#### Chat Delay
Adjusts the minimum amount of time allowed between chat messages.
#### Width
Adjusts the width of the in-game chat.
#### Focused Height
Adjusts the maximum height of the chat when the chat screen is active.
#### Unfocused Height
Adjusts the maximum height of the chat when outside of the chat screen.
#### Colors
Allows colors to be displayed in chat.
#### Web Links
Allows web links to be visible in chat.
#### Prompt on Links
Shows a confirmation prompt when clicking on a web link.
#### Text Background
When disabled, shows a background on any text that doesn't already have a background.
#### Command Suggestions
Allows suggestions to display automatically when typing commands.
#### Hide Matched Names
If a player is hidden through the Social Interactions menu, the game will attempt to hide messages based on the name of the chat sender.
#### Only Show Secure Chat
Only display messages from other players that can be verified to have been sent by that player, and have not been modified.
