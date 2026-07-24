package net.nostalgica.modernica.util;

import com.google.common.collect.ImmutableSet;
import com.mojang.serialization.Lifecycle;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.world.level.*;
import net.minecraft.world.Difficulty;
import net.minecraft.world.level.storage.WorldData;
import net.minecraft.world.level.storage.ServerLevelData;

import java.util.Collections;
import java.util.Set;

public class DummyServerConfiguration implements WorldData {
    @Override
    public WorldDataConfiguration getDataConfiguration() {
        return null;
    }

    @Override
    public void setDataConfiguration(WorldDataConfiguration arg) {

    }

    @Override
    public Set<String> getRemovedFeatureFlags() {
        return Collections.emptySet();
    }

    @Override
    public boolean wasModded() {
        return true;
    }

    @Override
    public Set<String> getKnownServerBrands() {
        return ImmutableSet.of("forge");
    }

    @Override
    public void setModdedInfo(String name, boolean isModded) {

    }

    @Override
    public ServerLevelData overworldData() {
        return null;
    }

    @Override
    public LevelSettings getLevelSettings() {
        return null;
    }

    @Override
    public CompoundTag createTag(java.util.UUID playerUUID) {
        return null;
    }

    @Override
    public boolean isHardcore() {
        return false;
    }

    @Override
    public int getVersion() {
        return 0;
    }

    @Override
    public String getLevelName() {
        return null;
    }

    @Override
    public GameType getGameType() {
        return null;
    }

    @Override
    public void setGameType(GameType type) {

    }

    @Override
    public boolean isAllowCommands() {
        return false;
    }

    // 26.2 added WorldData.setAllowCommands(boolean); no such method exists on 26.1.2's interface,
    // so it can't be a no-op stub here - Stonecutter substitutes the real override in on 26.2+.
    //STONECUTTER_SET_ALLOW_COMMANDS

    @Override
    public Difficulty getDifficulty() {
        return null;
    }

    @Override
    public void setDifficulty(Difficulty difficulty) {

    }

    @Override
    public boolean isDifficultyLocked() {
        return false;
    }

    @Override
    public void setDifficultyLocked(boolean locked) {

    }

    @Override
    public java.util.UUID getSinglePlayerUUID() {
        return null;
    }

    @Override
    public boolean isFlatWorld() {
        return false;
    }

    @Override
    public boolean isDebugWorld() {
        return false;
    }

    @Override
    public Lifecycle worldGenSettingsLifecycle() {
        return Lifecycle.stable();
    }
}
