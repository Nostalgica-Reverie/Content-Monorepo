KeyBindEvents.modify(event => {

    //Moving to Misc. Category
    event.modifyCategory('key.nostalgic_tweaks.toggle_fog', 'key.categories.misc');
    event.modifyCategory('key.nostalgic_tweaks.stop_song', 'key.categories.misc');
    event.modifyCategory('key.nostalgic_tweaks.next_song', 'key.categories.misc');

    event.modifyCategory('quark.keybind.camera_mode', 'key.categories.misc');

    //Moving to Inventory Category
    event.modifyCategory('key.inmis.open_backpack', 'key.categories.inventory');
    event.modifyCategory('key.spyglass-improvements.use', 'key.categories.inventory');
    event.modifyCategory('supplementaries.keybind.quiver', 'key.categories.inventory');

    event.modifyCategory('quark.keybind.change_hotbar', 'key.categories.inventory');
    event.modifyCategory('quark.keybind.lock_rotation', 'key.categories.inventory');
    event.modifyCategory('key.aether.invisibility_toggle.desc', 'key.categories.inventory');

    event.modifyCategory('quark.keybind.sort_player', 'key.categories.inventory');
    event.modifyCategory('quark.keybind.sort_container', 'key.categories.inventory');
    event.modifyCategory('quark.keybind.shift_lock', 'key.categories.inventory');
    event.modifyCategory('quark.keybind.transfer_insert', 'key.categories.inventory');
    event.modifyCategory('quark.keybind.transfer_extract', 'key.categories.inventory');

    //Moving to one JEI category
    event.modifyCategory('quark.keybind.sort_player', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.toggleBookmarkOverlay', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.toggleOverlay', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.transferRecipeBookmark', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.clearSearchBar', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.showUses', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.nextRecipePage', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.nextCategory', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.showRecipe2', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.previousCategory', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.bookmark', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.previousPage', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.showUses2', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.recipeBack', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.showRecipe', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.previousSearch', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.previousRecipePage', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.nextSearch', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.focusSearch', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.maxTransferRecipeBookmark', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.nextPage', 'jei.key.category.edit.mode');
    event.modifyCategory('key.jei.toggleEditMode', 'jei.key.category.edit.mode');

    //Removing keybinds
    event.remove('key.deep_aether.stratus_dash_ability.desc');
    event.remove('key.configured.open_mod_list');
    event.remove('key.curios.open.desc');
    event.remove('key.entityculling.toggle');
    event.remove('key.modernfix.config');
    event.remove('key.nostalgic_tweaks.open_config');
    event.remove('key.nostalgic_tweaks.toggle_weather');
    event.remove('placebo.toggleTrails');
    event.remove('placebo.toggleWings');
    event.remove('key.veinmining.activate.desc');
    event.remove('key.toastcontrol.clear');
    event.remove('key.aether.open_accessories.desc');

    event.remove('key.mount_jump');
    event.remove('[Forgery] Toggle/Hold Sprint');
    event.remove('[Forgery] Toggle Stance');
    event.remove('key.dynamic_fps.toggle_forced');
    event.remove('key.dynamic_fps.toggle_disabled');
    event.remove('key.spectatorOutlines');
    event.remove('key.jei.copy.recipe.id');
    event.remove('quark.keybind.narrator_readout');
    event.remove('quark.keybind.narrator_full_readout');

    event.remove('quark.emote.no');
    event.remove('quark.emote.yes');
    event.remove('quark.emote.wave');
    event.remove('quark.emote.salute');
    event.remove('quark.emote.cheer');
    event.remove('quark.emote.clap');
    event.remove('quark.emote.think');
    event.remove('quark.emote.point');
    event.remove('quark.emote.shrug');
    event.remove('quark.emote.headbang');
    event.remove('quark.emote.weep');
    event.remove('quark.emote.facepalm');
    event.remove('quark.keybind.patreon_emote.dance');
    event.remove('quark.keybind.patreon_emote.tpose');
    event.remove('quark.keybind.patreon_emote.dab');
    event.remove('quark.keybind.patreon_emote.jet');
    event.remove('quark.keybind.patreon_emote.exorcist');
    event.remove('quark.keybind.patreon_emote.zombie');
    event.remove('quark.keybind.back');
    event.remove('quark.keybind.autorun');
    event.remove('quark.keybind.variant_selector');

    // event.remove('key.cycleLeft');
    // event.remove('key.cycleRight');
    // event.remove('key.toggleVisualItemContents');
    // event.remove('key.toggleSelectedItemTooltips');
    // event.remove('key.toggleCarriedItemTooltips');

    event.remove('key.jei.closeRecipeGui');

    event.remove('key.jei.cheatOneItem');
    event.remove('key.jei.toggleCheatMode');
    event.remove('key.jei.cheatOneItem2');
    event.remove('key.jei.cheatItemStack');
    event.remove('key.jei.toggleCheatModeConfigButton');
    event.remove('key.jei.cheatItemStack2');

    event.remove('fooxy.sss.key.toggle');
    event.remove('fooxy.sss.key.next');
    event.remove('fooxy.sss.key.prev');

    event.remove('chloride.zoom');

    event.remove('key.immersiveoverlays.toggle');

    



})