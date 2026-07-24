package net.nostalgica.modernica.common.mixin.bugfix.chunk_deadlock;

import com.llamalad7.mixinextras.injector.v2.WrapWithCondition;
import net.minecraft.core.Holder;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.Level;
import net.minecraft.world.phys.Vec3;
import net.nostalgica.modernica.Modernica;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

@Mixin(Entity.class)
public class EntityMixin {
    /**
     * @author embeddedt
     * @reason When an entity is added to the world via the worldgen load path (ChunkMap#postLoadProtoChunk calling
     * ServerLevel#addWorldGenChunkEntities), attempts to add a passenger result in a deadlock when the sculk event
     * tries to raytrace blocks. To fix this, we skip firing the sculk event if the chunk the entity is within is not
     * loaded.
     * Note(DerCommander323): This used to apply to addPassenger, but Mojang removed the event invocation from that
     * function. I found it is in startRiding now, but I'm not sure if it also causes a deadlock.
     */
    @WrapWithCondition(method = "startRiding(Lnet/minecraft/world/entity/Entity;ZZ)Z", at = @At(value = "INVOKE", target = "Lnet/minecraft/world/level/Level;gameEvent(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/core/Holder;Lnet/minecraft/world/phys/Vec3;)V"))
    private boolean onlyAddIfSelfChunkLoaded(Level level, Entity entity, Holder holder, Vec3 vec3) {
        var chunkPos = entity.chunkPosition();
        if (level instanceof ServerLevel serverLevel && serverLevel.getChunkSource().getChunkNow(chunkPos.x(), chunkPos.z()) == null) {
            Modernica.LOGGER.warn("Skipped emitting ENTITY_MOUNT game event for entity {} as it would cause deadlock", entity.toString());
            return false;
        } else {
            return true;
        }
    }
}
