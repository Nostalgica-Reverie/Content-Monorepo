package net.nostalgica.modernica.fabric.mixin.feature.fast_ip_ping;

import com.google.common.net.InetAddresses;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.net.InetAddress;
import java.net.UnknownHostException;

/** Ported from fast-ip-ping (Fallen_Breath, LGPL-3.0) */
@Mixin(targets = "net/minecraft/client/multiplayer/resolver/ServerAddressResolver")
public interface ServerAddressResolverMixin {
    @Redirect(
            method = "lambda$static$0(Lnet/minecraft/client/multiplayer/resolver/ServerAddress;)Ljava/util/Optional;",
            at = @At(
                    value = "INVOKE",
                    target = "Ljava/net/InetAddress;getByName(Ljava/lang/String;)Ljava/net/InetAddress;"
            ),
            remap = false
    )
    private static InetAddress skipReverseDnsLookupForLiteralIps(String hostName) throws UnknownHostException {
        InetAddress address = InetAddress.getByName(hostName);
        if (InetAddresses.isInetAddress(hostName)) {
            address = InetAddress.getByAddress(address.getHostAddress(), address.getAddress());
        }
        return address;
    }
}
