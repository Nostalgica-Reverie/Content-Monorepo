package net.nostalgica.modernica.feature.forcecloseloadingscreen;

import com.mojang.blaze3d.pipeline.RenderPipeline;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.Connection;
import net.minecraft.network.chat.Component;

/** Ported from kennytv's forcecloseloadingscreen (MIT). */
public final class ReconfigBridgeScreen extends Screen {
    private static final RenderPipeline CAPTURED_FRAME_PIPELINE = RenderPipelines.GUI_OPAQUE_TEXTURED_BACKGROUND;
    private final Connection connection;

    public ReconfigBridgeScreen(final Connection connection) {
        super(Component.literal(""));
        this.connection = connection;
    }

    @Override
    public void extractRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
        final int textureWidth = CapturedFrame.width();
        final int textureHeight = CapturedFrame.height();
        final Minecraft minecraft = Minecraft.getInstance();
        final int guiScale = minecraft.getWindow().getGuiScale();
        final int framebufferWidth = minecraft.getWindow().getWidth();
        final int framebufferHeight = minecraft.getWindow().getHeight();
        if (framebufferWidth <= 0 || framebufferHeight <= 0) {
            return;
        }

        final int sourceWidth = Math.round((float) textureWidth * this.width * guiScale / framebufferWidth);
        final int sourceHeight = Math.round((float) textureHeight * this.height * guiScale / framebufferHeight);
        graphics.fill(0, 0, this.width, this.height, CapturedFrame.backgroundColor());
        graphics.blit(CAPTURED_FRAME_PIPELINE, CapturedFrame.CAPTURED_FRAME_ID, 0, 0, 0.0F, textureHeight, this.width, this.height, sourceWidth, -sourceHeight, textureWidth, textureHeight);
    }

    @Override
    public void extractBackground(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
    }

    @Override
    public void tick() {
        if (this.connection.isConnected()) {
            this.connection.tick();
        } else {
            this.connection.handleDisconnection();
        }
    }

    @Override
    public boolean isPauseScreen() {
        return false;
    }

    @Override
    public boolean shouldCloseOnEsc() {
        return false;
    }

    @Override
    protected boolean shouldNarrateNavigation() {
        return false;
    }
}
