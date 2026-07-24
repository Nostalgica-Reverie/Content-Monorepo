package net.nostalgica.modernica.common.mixin.perf.network_optimizations.encryption;

import com.velocitypowered.natives.encryption.VelocityCipher;
import com.velocitypowered.natives.util.Natives;
import io.netty.channel.Channel;
import net.nostalgica.modernica.network.misc.KryptonPipelineEvent;
import net.nostalgica.modernica.network.ClientConnectionEncryptionExtension;
import net.nostalgica.modernica.network.pipeline.MinecraftCipherDecoder;
import net.nostalgica.modernica.network.pipeline.MinecraftCipherEncoder;
import net.minecraft.network.Connection;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;

import javax.crypto.SecretKey;
import java.security.GeneralSecurityException;

@Mixin(Connection.class)
public class ConnectionMixin implements ClientConnectionEncryptionExtension {

    @Shadow private Channel channel;
    @Unique private boolean kryptonEncryptionEnabled = false;

    @Override
    public void setupEncryption(SecretKey key) throws GeneralSecurityException {
        if (this.kryptonEncryptionEnabled) {
            return;
        }

        VelocityCipher decryption = Natives.cipher.get().forDecryption(key);
        VelocityCipher encryption = Natives.cipher.get().forEncryption(key);

        this.channel.pipeline().addBefore("splitter", "decrypt", new MinecraftCipherDecoder(decryption));
        this.channel.pipeline().addBefore("prepender", "encrypt", new MinecraftCipherEncoder(encryption));
        this.channel.pipeline().fireUserEventTriggered(KryptonPipelineEvent.ENCRYPTION_ENABLED);

        this.kryptonEncryptionEnabled = true;
    }
}
