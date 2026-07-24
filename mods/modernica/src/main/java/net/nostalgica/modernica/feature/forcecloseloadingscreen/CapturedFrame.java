package net.nostalgica.modernica.feature.forcecloseloadingscreen;

import com.mojang.blaze3d.pipeline.RenderTarget;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.textures.GpuTexture;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.AbstractTexture;
import net.minecraft.resources.Identifier;
import net.minecraft.util.ARGB;
import org.joml.Vector4f;

/** Ported from kennytv's forcecloseloadingscreen (MIT). */
public final class CapturedFrame {

    public static final Identifier CAPTURED_FRAME_ID = Identifier.fromNamespaceAndPath("modernica", "captured_frame");
    public static boolean initialJoin = true;
    private static volatile int backgroundColor = -16777216;
    private static CapturedFrameTexture texture;

    private CapturedFrame() {}

    public static int width() {
        return texture != null ? texture.getTexture().getWidth(0) : 0;
    }

    public static int height() {
        return texture != null ? texture.getTexture().getHeight(0) : 0;
    }

    public static int backgroundColor() {
        return backgroundColor;
    }

    public static void clearCapturedTexture() {
        texture = null;
        Minecraft.getInstance().getTextureManager().release(CAPTURED_FRAME_ID);
    }

    public static void captureLastFrame() {
        final Minecraft client = Minecraft.getInstance();
        //STONECUTTER_FCLS_MAIN_RENDER_TARGET
        final RenderTarget target = client.getMainRenderTarget();
        final GpuTexture sourceTexture = target.getColorTexture();
        if (sourceTexture == null) {
            return;
        }

        //STONECUTTER_FCLS_GAME_RENDER_STATE
        final Vector4f fogColor = client.gameRenderer.getGameRenderState().levelRenderState.cameraRenderState.fogData.color;
        backgroundColor = ARGB.colorFromFloat(1.0F, fogColor.x, fogColor.y, fogColor.z);

        if (texture == null || texture.needsResize(sourceTexture)) {
            texture = new CapturedFrameTexture(sourceTexture);
            client.getTextureManager().register(CAPTURED_FRAME_ID, texture);
        }

        RenderSystem.getDevice()
                .createCommandEncoder()
                .copyTextureToTexture(sourceTexture, texture.getTexture(), 0, 0, 0, 0, 0, target.width, target.height);
    }

    private static final class CapturedFrameTexture extends AbstractTexture {

        private CapturedFrameTexture(final GpuTexture sourceTexture) {
            this.texture = RenderSystem.getDevice().createTexture(
                    () -> "modernica force close loading screen captured frame",
                    GpuTexture.USAGE_COPY_DST | GpuTexture.USAGE_TEXTURE_BINDING,
                    sourceTexture.getFormat(),
                    sourceTexture.getWidth(0),
                    sourceTexture.getHeight(0),
                    1,
                    1
            );
            this.textureView = RenderSystem.getDevice().createTextureView(this.texture);
        }

        private boolean needsResize(final GpuTexture sourceTexture) {
            return this.texture == null
                    || this.texture.isClosed()
                    || this.texture.getWidth(0) != sourceTexture.getWidth(0)
                    || this.texture.getHeight(0) != sourceTexture.getHeight(0)
                    || this.texture.getFormat() != sourceTexture.getFormat();
        }
    }
}
