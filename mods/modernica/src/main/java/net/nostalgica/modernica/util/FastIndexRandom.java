package net.nostalgica.modernica.util;

import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.BitRandomSource;
import net.minecraft.world.level.levelgen.MarsagliaPolarGaussian;
import net.minecraft.world.level.levelgen.PositionalRandomFactory;

/**
 * Same algorithm as {@link ThreadUnsafeRandom} (see its Javadoc), plus a biased-but-fast
 * {@link #nextInt(int)} using Lemire's public multiply-shift technique
 * (https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/) instead of vanilla's
 * rejection-sampling loop. That bias is a bad trade for a general-purpose, player-facing RNG (loot rolls,
 * damage, mob AI...), which is why it's a separate class from {@link ThreadUnsafeRandom} rather than an
 * override there - this one exists only for internal engine bookkeeping (picking which array slot to
 * check next) where the caller never observes the distribution directly.
 */
public final class FastIndexRandom implements BitRandomSource {

    private static final long MULTIPLIER = 0x5DEECE66DL;
    private static final long INCREMENT = 0xBL;
    private static final int SEED_BITS = 48;
    private static final long SEED_MASK = (1L << SEED_BITS) - 1L;

    private long seed;
    private final MarsagliaPolarGaussian gaussianSource = new MarsagliaPolarGaussian(this);

    public FastIndexRandom(long seed) {
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
    public int nextInt(int bound) {
        if (bound <= 0) {
            throw new IllegalArgumentException("bound must be positive");
        }
        long value = this.next(32) & 0xFFFFFFFFL;
        return (int) ((value * bound) >>> 32);
    }

    @Override
    public double nextGaussian() {
        return this.gaussianSource.nextGaussian();
    }

    @Override
    public RandomSource fork() {
        return new FastIndexRandom(this.nextLong());
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
            return new FastIndexRandom((long) string.hashCode() ^ this.seed);
        }

        @Override
        public RandomSource fromSeed(long seed) {
            return new FastIndexRandom(seed);
        }

        @Override
        public RandomSource at(int x, int y, int z) {
            return new FastIndexRandom(Mth.getSeed(x, y, z) ^ this.seed);
        }

        @Override
        public void parityConfigString(StringBuilder builder) {
            builder.append("FastIndexRandom$Positional{").append(this.seed).append('}');
        }
    }
}
