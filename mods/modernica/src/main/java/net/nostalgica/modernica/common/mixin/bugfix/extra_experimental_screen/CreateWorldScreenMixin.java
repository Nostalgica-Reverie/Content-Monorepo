package net.nostalgica.modernica.common.mixin.bugfix.extra_experimental_screen;

import com.mojang.serialization.Lifecycle;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.world.level.storage.LevelDataAndDimensions;
import net.minecraft.world.level.storage.PrimaryLevelData;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

import java.lang.reflect.Field;
import java.lang.reflect.Method;

@Mixin(CreateWorldScreen.class)
@ClientOnlyMixin
public class CreateWorldScreenMixin {
    @Unique
    private static volatile boolean mfix$warningMemberSearched;

    @Unique
    private static Method mfix$warningSetter;

    @Unique
    private static Field mfix$warningField;

    /**
     * Fix experimental world dialog still being shown the first time you reopen a world that was created
     * as experimental.
     */
    @ModifyArg(method = "createNewWorld", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gui/screens/worldselection/WorldOpenFlows;createLevelFromExistingSettings(Lnet/minecraft/world/level/storage/LevelStorageSource$LevelStorageAccess;Lnet/minecraft/server/ReloadableServerResources;Lnet/minecraft/core/LayeredRegistryAccess;Lnet/minecraft/world/level/storage/LevelDataAndDimensions$WorldDataAndGenSettings;Ljava/util/Optional;)V"), index = 3)
    private LevelDataAndDimensions.WorldDataAndGenSettings setExperimentalFlag(LevelDataAndDimensions.WorldDataAndGenSettings settings) {
        if (settings.data() instanceof PrimaryLevelData pld && settings.data().worldGenSettingsLifecycle() != Lifecycle.stable()) {
            mfix$setConfirmedWarning(pld);
        }
        return settings;
    }

    @Unique
    private static void mfix$setConfirmedWarning(PrimaryLevelData levelData) {
        if (!mfix$warningMemberSearched) {
            mfix$findWarningMember();
            mfix$warningMemberSearched = true;
        }
        try {
            if (mfix$warningSetter != null) {
                mfix$warningSetter.invoke(levelData, true);
                return;
            }
            if (mfix$warningField != null) {
                mfix$warningField.setBoolean(levelData, true);
            }
        } catch (ReflectiveOperationException ignored) {
        }
    }

    @Unique
    private static void mfix$findWarningMember() {
        String[] preferredMethodNames = {
                "withConfirmedWarning",
                "setConfirmedWarning",
                "setConfirmedExperimentalWarning"
        };
        for (String name : preferredMethodNames) {
            try {
                Method method = PrimaryLevelData.class.getDeclaredMethod(name, boolean.class);
                method.setAccessible(true);
                mfix$warningSetter = method;
                return;
            } catch (ReflectiveOperationException ignored) {
            }
        }

        for (Method method : PrimaryLevelData.class.getDeclaredMethods()) {
            if (method.getParameterCount() == 1
                    && method.getParameterTypes()[0] == boolean.class
                    && method.getName().toLowerCase().contains("confirm")
                    && method.getName().toLowerCase().contains("warning")) {
                method.setAccessible(true);
                mfix$warningSetter = method;
                return;
            }
        }

        for (Field field : PrimaryLevelData.class.getDeclaredFields()) {
            String name = field.getName().toLowerCase();
            if (field.getType() == boolean.class && name.contains("confirm") && name.contains("warning")) {
                field.setAccessible(true);
                mfix$warningField = field;
                return;
            }
        }
    }
}