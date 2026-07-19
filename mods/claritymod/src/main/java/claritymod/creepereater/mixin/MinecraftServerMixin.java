package claritymod.creepereater.mixin;

import java.util.concurrent.TimeUnit;
import net.minecraft.server.MinecraftServer;
//? if >=26 {
import net.minecraft.util.Util;
//?} else {
/*import net.minecraft.Util;*/
//?}
import org.slf4j.Logger;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftServer.class)
public abstract class MinecraftServerMixin {
	@Unique
	private static final long claritymod$STARTUP_GRACE_PERIOD_NANOS = TimeUnit.MINUTES.toNanos(2L);

	@Unique
	private long claritymod$suppressOverloadWarningsUntilNanos;

	@Inject(method = "runServer", at = @At("HEAD"))
	private void claritymod$startOverloadWarningGracePeriod(CallbackInfo callback) {
		MinecraftServer server = (MinecraftServer)(Object)this;
		if (server.isDedicatedServer()) {
			this.claritymod$suppressOverloadWarningsUntilNanos = Util.getNanos() + claritymod$STARTUP_GRACE_PERIOD_NANOS;
		}
	}

	@Redirect(
		method = "runServer",
		at = @At(
			value = "INVOKE",
			target = "Lorg/slf4j/Logger;warn(Ljava/lang/String;Ljava/lang/Object;Ljava/lang/Object;)V"
		)
	)
	private void claritymod$suppressStartupOverloadWarning(
		Logger logger,
		String message,
		Object millisecondsBehind,
		Object ticksBehind
	) {
		if (Util.getNanos() >= this.claritymod$suppressOverloadWarningsUntilNanos) {
			logger.warn(message, millisecondsBehind, ticksBehind);
		}
	}
}
