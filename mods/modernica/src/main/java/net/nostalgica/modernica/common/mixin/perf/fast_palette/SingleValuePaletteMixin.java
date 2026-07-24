package net.nostalgica.modernica.common.mixin.perf.fast_palette;

import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import net.minecraft.world.level.chunk.Palette;
import net.minecraft.world.level.chunk.PaletteResize;
import net.minecraft.world.level.chunk.SingleValuePalette;
import net.nostalgica.modernica.perf.fast_palette.FastPalette;
import net.nostalgica.modernica.perf.fast_palette.FastPaletteData;

/** A single-value palette only ever has one entry; lazily build a 1-element array wrapping it and keep
 * that array's contents (not the array reference itself, so a cached copy elsewhere stays in sync)
 * updated whenever the value changes. */
@Mixin(SingleValuePalette.class)
abstract class SingleValuePaletteMixin<T> implements Palette<T>, FastPalette<T> {

    @Shadow
    private T value;

    @Unique
    private T[] mfh$rawPalette;

    @Override
    @SuppressWarnings("unchecked")
    public final T[] mfh$getRawPalette(FastPaletteData<T> owner) {
        if (this.mfh$rawPalette == null) {
            this.mfh$rawPalette = (T[]) new Object[] { this.value };
        }
        return this.mfh$rawPalette;
    }

    @Inject(
            method = "idFor",
            at = @At(value = "FIELD", opcode = Opcodes.PUTFIELD, target = "Lnet/minecraft/world/level/chunk/SingleValuePalette;value:Ljava/lang/Object;")
    )
    private void mfh$syncOnIdFor(T object, PaletteResize<T> resize, CallbackInfoReturnable<Integer> cir) {
        if (this.mfh$rawPalette != null) {
            this.mfh$rawPalette[0] = object;
        }
    }

    @Redirect(
            method = "read",
            at = @At(value = "FIELD", opcode = Opcodes.PUTFIELD, target = "Lnet/minecraft/world/level/chunk/SingleValuePalette;value:Ljava/lang/Object;")
    )
    private void mfh$syncOnRead(SingleValuePalette<T> instance, T value) {
        ((SingleValuePaletteMixin<T>) (Object) instance).value = value;
        if (this.mfh$rawPalette != null) {
            this.mfh$rawPalette[0] = value;
        }
    }
}
