plugins {
    id("dev.kikugie.stonecutter")
}

stonecutter active "26.1.2"

stonecutter parameters {
    replacements {
        // Modernica's advancement-dedup mixins (perf.deduplicate_advancement_predicates, ported from
        // Hydrogen) and the un-ported-elsewhere criterion classes both need this rename on 26.2+.
        string(current.parsed >= "26.2") {
            replace("net.minecraft.advancements.criterion.ItemPredicate", "net.minecraft.advancements.predicates.ItemPredicate")
            replace("net.minecraft.advancements.criterion.MinMaxBounds", "net.minecraft.advancements.predicates.MinMaxBounds")
            replace("net.minecraft.advancements.criterion.InventoryChangeTrigger", "net.minecraft.advancements.triggers.InventoryChangeTrigger")
            replace("net.minecraft.advancements.Criterion", "net.minecraft.advancements.triggers.Criterion")

            // same renames, JVM internal-name form used inside @At(target = "...") descriptor strings
            replace("Lnet/minecraft/advancements/criterion/ItemPredicate;", "Lnet/minecraft/advancements/predicates/ItemPredicate;")
            replace("Lnet/minecraft/advancements/criterion/MinMaxBounds;", "Lnet/minecraft/advancements/predicates/MinMaxBounds;")
            replace("Lnet/minecraft/advancements/criterion/InventoryChangeTrigger;", "Lnet/minecraft/advancements/triggers/InventoryChangeTrigger;")
            replace("Lnet/minecraft/advancements/Criterion;", "Lnet/minecraft/advancements/triggers/Criterion;")

            // 26.2 moved "currently open screen" tracking off Minecraft (a public `screen` field on
            // 26.1.2) and onto Gui instead (a private field with a `screen()` accessor on 26.2).
            // Confirmed via javap against both versions' minecraft-merged.jar during this merge.
            replace(
                "@Shadow public Screen screen;",
                "@Shadow public Gui gui;"
            )
            replace(
                "if(this.screen == null && ModernicaClient.INSTANCE != null) {",
                "if(this.gui.screen() == null && ModernicaClient.INSTANCE != null) {"
            )
            replace(
                "if(minecraft.screen instanceof CreateWorldScreen)",
                "if(minecraft.gui.screen() instanceof CreateWorldScreen)"
            )

            // 26.2 added WorldData.setAllowCommands(boolean) to the interface; 26.1.2 doesn't have it.
            replace(
                "//STONECUTTER_SET_ALLOW_COMMANDS",
                "@Override\n    public void setAllowCommands(boolean allowCommands) {\n\n    }"
            )

            // same screen-tracking split, for the ported force_close_loading_screen mixins
            replace(
                "//STONECUTTER_FCLS_GUI_MIXIN_TARGET\n@Mixin(Minecraft.class)",
                "@Mixin(Gui.class)"
            )
            replace(
                "//STONECUTTER_FCLS_SET_OVERLAY\n            this.minecraft.setOverlay(null);",
                "this.minecraft.gui.setOverlay(null);"
            )

            // GameRenderer's render-target/render-state accessors also moved onto GameRenderer in 26.2
            replace(
                "//STONECUTTER_FCLS_MAIN_RENDER_TARGET\n        final RenderTarget target = client.getMainRenderTarget();",
                "final RenderTarget target = client.gameRenderer.mainRenderTarget();"
            )
            replace(
                "//STONECUTTER_FCLS_GAME_RENDER_STATE\n        final Vector4f fogColor = client.gameRenderer.getGameRenderState().levelRenderState.cameraRenderState.fogData.color;",
                "final Vector4f fogColor = client.gameRenderer.gameRenderState().levelRenderState.cameraRenderState.fogData.color;"
            )
        }
    }
}
