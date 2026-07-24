package net.nostalgica.modernica.common.mixin.perf.network_enhancements;

import io.netty.buffer.ByteBuf;
import net.minecraft.network.VarLong;
import net.nostalgica.modernica.network.util.VarLongUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

/**
 * Independently written network-protocol micro-optimizations for {@code perf.network_enhancements}.
 * <p>
 * These are <b>not</b> ports of KryptonReno-Fabric (or any other third party's code) - none of that
 * project's source was read while building this monorepo's onboarding of the Krypton family (see
 * {@code docs/research/modernica-krypton-fastsuite-plan.md} and mods/kreno-fpatcher's own
 * README for why: its README points to an additional "404Setup Public License" that restricts folding
 * its code into a different mod's binary). This group exists because the user specifically asked for
 * network-optimization *ideas* in that spirit to be reimplemented cleanly, based on public,
 * independently-documented techniques (Minecraft's VarInt wire-format encoding, standard Java 21+
 * virtual-thread patterns, Paper's long-published {@code particle-tracking-range} setting) - not on
 * KryptonReno's actual implementation, which this project's authors never saw.
 * <p>
 * This class extends the exact same lookup-table + single/double-byte-peeling technique Krypton's own
 * (merged-in, LGPL-3.0) {@code VarIntMixin} already applies to 32-bit VarInts, to Minecraft's 64-bit
 * VarLong wire format instead - the two encodings share an identical 7-bits-per-byte,
 * high-bit-continuation scheme, so the same technique applies directly.
 */
@Mixin(VarLong.class)
public class VarLongMixin {
    /**
     * @author modernica
     * @reason optimized version, extending Krypton's VarInt lookup-table technique to VarLong
     */
    @Overwrite
    public static int getByteSize(long value) {
        return VarLongUtil.getVarLongLength(value);
    }

    /**
     * @author modernica
     * @reason optimized version, peeling the single-byte fast path (the most common case for small
     * counts/ids) before falling back to a generic loop for larger values
     */
    @Overwrite
    public static ByteBuf write(ByteBuf buf, long value) {
        if ((value & (0xFFFFFFFFFFFFFFFFL << 7)) == 0) {
            buf.writeByte((int) value);
            return buf;
        }
        writeVarLongFull(buf, value);
        return buf;
    }

    private static void writeVarLongFull(ByteBuf buf, long value) {
        while (true) {
            if ((value & ~0x7FL) == 0) {
                buf.writeByte((int) value);
                return;
            }
            buf.writeByte((int) ((value & 0x7FL) | 0x80L));
            value >>>= 7;
        }
    }
}
