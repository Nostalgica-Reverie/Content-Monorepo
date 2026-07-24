package net.nostalgica.modernica.common.mixin.perf.compress_unihex_font;

import com.llamalad7.mixinextras.injector.ModifyExpressionValue;
import com.mojang.serialization.MapCodec;
import net.minecraft.client.gui.font.providers.GlyphProviderDefinition;
import net.minecraft.client.gui.font.providers.GlyphProviderType;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import net.nostalgica.modernica.render.font.LazyGlyphProvider;
import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

@Mixin(GlyphProviderType.class)
@ClientOnlyMixin
public class GlyphProviderTypeMixin {
    @ModifyExpressionValue(method = "<clinit>", at = @At(value = "FIELD", opcode = Opcodes.GETSTATIC, target = "Lnet/minecraft/client/gui/font/providers/UnihexProvider$Definition;CODEC:Lcom/mojang/serialization/MapCodec;"))
    private static MapCodec<? extends GlyphProviderDefinition> lazyUnihex(MapCodec<? extends GlyphProviderDefinition> codec) {
        return LazyGlyphProvider.wrap(codec);
    }
}
