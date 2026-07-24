package net.nostalgica.modernica.common.mixin.perf.fast_block_entity_removal;

import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.core.BlockPos;
import net.minecraft.world.TickRateManager;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.LevelAccessor;
import net.minecraft.world.level.block.entity.TickingBlockEntity;

import java.util.Iterator;
import java.util.List;

/**
 * Vanilla removes finished block-entity tickers from {@code blockEntityTickers} one at a time via
 * {@code Iterator#remove()}, which is O(n) per removal (an {@code ArrayList} shifts every trailing
 * element down) - O(n^2) total for a tick that removes several. Backing the list with fastutil's
 * {@link ObjectArrayList} exposes its raw backing array, so removed entries can instead be compacted
 * forward in a single O(n) pass: entries survive by being copied earlier in the same array, and the list
 * is truncated to the surviving count once, instead of shrinking on every removal.
 * <p>
 * The tick loop itself is still driven by vanilla's own {@code for (TickingBlockEntity ... : blockEntityTickers)}
 * (redirecting only the iterator's {@code hasNext()} to run the compacted loop and always report "done"),
 * so the pending-ticker merge and everything else {@code tickBlockEntities} does around that loop is
 * untouched.
 */
@Mixin(Level.class)
abstract class LevelMixin implements LevelAccessor {

    @Shadow
    protected List<TickingBlockEntity> blockEntityTickers;

    @Shadow
    private List<TickingBlockEntity> pendingBlockEntityTickers;

    @Shadow
    public abstract TickRateManager tickRateManager();

    @Shadow
    public abstract boolean shouldTickBlocksAt(BlockPos pos);

    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$useArrayBackedTickerLists(CallbackInfo ci) {
        this.blockEntityTickers = new ObjectArrayList<>();
        this.pendingBlockEntityTickers = new ObjectArrayList<>();
    }

    @SuppressWarnings("unchecked")
    @Redirect(
            method = "tickBlockEntities",
            at = @At(value = "INVOKE", target = "Ljava/util/Iterator;hasNext()Z", ordinal = 0)
    )
    private boolean mfh$compactRemovalTick(Iterator<TickingBlockEntity> vanillaIterator) {
        boolean doTick = this.tickRateManager().runsNormally();
        ObjectArrayList<TickingBlockEntity> tickers = (ObjectArrayList<TickingBlockEntity>) this.blockEntityTickers;
        TickingBlockEntity[] elements = tickers.elements();
        int len = tickers.size();
        int writeIndex = 0;
        int readIndex = 0;
        try {
            for (; readIndex < len; readIndex++) {
                TickingBlockEntity ticker = elements[readIndex];
                if (ticker.isRemoved()) {
                    continue;
                }
                if (doTick && this.shouldTickBlocksAt(ticker.getPos())) {
                    ticker.tick();
                }
                elements[writeIndex++] = ticker;
            }
        } finally {
            // on a mid-loop exception, readIndex/writeIndex stop early; shift the untouched tail
            // down to keep the array compacted and the list's reported size accurate either way
            if (readIndex != writeIndex) {
                System.arraycopy(elements, readIndex, elements, writeIndex, len - readIndex);
                tickers.size(len - (readIndex - writeIndex));
            }
        }
        return false;
    }
}
