package claritymod.creeereater.mixin;

import claritymod.creeereater.compat.GraveModCompat;
import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.damagesource.DamageTypes;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ServerPlayer.class)
public abstract class ServerPlayerMixin {
	@Inject(method = "die", at = @At("TAIL"))
	private void claritymod$remindPlayerAboutVoidGrave(DamageSource source, CallbackInfo callback) {
		if (!source.is(DamageTypes.FELL_OUT_OF_WORLD) || !GraveModCompat.isSupportedGraveModLoaded()) {
			return;
		}

		ServerPlayer player = (ServerPlayer)(Object)this;
		player.sendSystemMessage(Component.translatableWithFallback(
			"claritymod.grave.void_reminder",
			"Your grave still exists despite your void death. Use your grave mod's recovery tools to find and retrieve it!"
		).withStyle(ChatFormatting.GOLD));
	}
}
