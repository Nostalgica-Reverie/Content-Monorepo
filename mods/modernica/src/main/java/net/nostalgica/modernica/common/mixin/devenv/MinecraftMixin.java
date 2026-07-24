package net.nostalgica.modernica.common.mixin.devenv;

import com.mojang.authlib.minecraft.UserApiService;
import com.mojang.authlib.yggdrasil.YggdrasilAuthenticationService;
import net.minecraft.client.Minecraft;
import net.minecraft.client.main.GameConfig;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(Minecraft.class)
@ClientOnlyMixin
public class MinecraftMixin {
    /**
     * @author embeddedt
     * @reason avoid exception stacktrace being printed in dev
     */
    @Overwrite
    private UserApiService createUserApiService(YggdrasilAuthenticationService yggdrasilAuthenticationService, GameConfig arg) {
        return UserApiService.OFFLINE;
    }
}
