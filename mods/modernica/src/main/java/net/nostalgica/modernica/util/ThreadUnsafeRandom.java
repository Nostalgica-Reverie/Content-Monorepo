package net.nostalgica.modernica.util;

import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.BitRandomSource;
import net.minecraft.world.level.levelgen.MarsagliaPolarGaussian;
import net.minecraft.world.level.levelgen.PositionalRandomFactory;

/**
 * A {@link RandomSource} using the same 48-bit linear congruential algorithm as {@code java.util.Random}
 * (documented in its Javadoc as "a linear congruential formula as described by Donald Knuth in The Art
 * of Computer Programming, Volume 2" - the constants below are that public algorithm's, not proprietary
 * to any mod) - which is also what vanilla's own {@code LegacyRandomSource} uses, so output stays
 * bit-compatible with vanilla for anything seeded the same way. The difference is purely that vanilla's
 * version stores its seed in an {@code AtomicLong} and advances it with a compare-and-swap loop so the
 * same {@code RandomSource} instance can be shared safely across threads; per-entity and per-level RNGs
 * are never actually shared across threads in practice, so that CAS overhead buys nothing for them and a
 * plain field is strictly cheaper.
 */
public final class ThreadUnsafeRandom implements BitRandomSource {

    private static final long MULTIPLIER = 0x5DEECE66DL;
    private static final long INCREMENT = 0xBL;
    private static final int SEED_BITS = 48;
    private static final long SEED_MASK = (1L << SEED_BITS) - 1L;

    private long seed;
    private final MarsagliaPolarGaussian gaussianSource = new MarsagliaPolarGaussian(this);

    public ThreadUnsafeRandom(long seed) {
        this.setSeed(seed);
    }

    @Override
    public void setSeed(long seed) {
        this.seed = (seed ^ MULTIPLIER) & SEED_MASK;
        this.gaussianSource.reset();
    }

    @Override
    public int next(int bits) {
        this.seed = (this.seed * MULTIPLIER + INCREMENT) & SEED_MASK;
        return (int) (this.seed >>> (SEED_BITS - bits));
    }

    @Override
    public double nextGaussian() {
        return this.gaussianSource.nextGaussian();
    }

    @Override
    public RandomSource fork() {
        return new ThreadUnsafeRandom(this.nextLong());
    }

    @Override
    public PositionalRandomFactory forkPositional() {
        return new Positional(this.nextLong());
    }

    private static final class Positional implements PositionalRandomFactory {
        private final long seed;

        private Positional(long seed) {
            this.seed = seed;
        }

        @Override
        public RandomSource fromHashOf(String string) {
            return new ThreadUnsafeRandom((long) string.hashCode() ^ this.seed);
        }

        @Override
        public RandomSource fromSeed(long seed) {
            return new ThreadUnsafeRandom(seed);
        }

        @Override
        public RandomSource at(int x, int y, int z) {
            return new ThreadUnsafeRandom(Mth.getSeed(x, y, z) ^ this.seed);
        }

        @Override
        public void parityConfigString(StringBuilder builder) {
            builder.append("ThreadUnsafeRandom$Positional{").append(this.seed).append('}');
        }
    }
}
