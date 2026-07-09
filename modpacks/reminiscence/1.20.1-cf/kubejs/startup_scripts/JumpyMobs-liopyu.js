//ALL CREDIT GOES TO 'liopyu' FOR THIS SCRIPT - I WROTE NONE OF THIS!!!

let jumpyMobs = [
    "minecraft:pig",    
    "minecraft:cow",
    "minecraft:sheep",
    "minecraft:chicken",
    "minecraft:wolf",
    "minecraft:horse",
    "minecraft:donkey",
    "minecraft:llama",
    "minecraft:goat",
    "minecraft:camel",

    "naturalist:deer",
    "naturalist:alligator",
    "naturalist:boar",
    "naturalist:tortoise",
    "naturalist:giraffe",
    "naturalist:snake",
    "naturalist:snail",

    "minecraft:zombie",
    "minecraft:husk",
    "minecraft:creeper",
    "caverns_and_chasms:peeper",
    "caverns_and_chasms:deeper",
    "minecraft:wolf",
    "minecraft:skeleton",
    "minecraft:spider",
    "minecraft:cave_spider"
]
const DirectionAxis = Java.loadClass("net.minecraft.core.Direction$Axis")
Java.loadClass("net.liopyu.entityjs.builders.modification.ModifyPathfinderMobBuilder")
EntityJSEvents.modifyEntity(event => {
    jumpyMobs.forEach(type => {
        event.modify(type, /**@param {Internal.ModifyPathfinderMobBuilder} modifyBuilder */ modifyBuilder => {
            modifyBuilder.aiStep(entity => global.jumpTick(entity))
        })
    })
})
/**
 * 
 * @param {Internal.PathfinderMob} entity 
 * @returns 
 */
global.jumpTick = entity => {
    try {
        if (!entity.onGround() || !entity.getNavigation().isInProgress()) return
        if (shouldEarlyJump(entity)) {
            entity.jumpControl.jump()
        }
    } catch (error) {
        console.log(error)
    }
}

function shouldEarlyJump(entity) {
    let motion = entity.getDeltaMovement()
    if (motion.lengthSqr() < 0.01) return false
    let level = entity.level
    let stepHeight = entity.getStepHeight()
    let pos = entity.blockPosition()
    for (let dx = -2; dx <= 2; dx++) {
        for (let dz = -2; dz <= 2; dz++) {
            if (dx === 0 && dz === 0) continue
            let checkPos = pos.offset(dx, 0, dz)
            if (!level.loadedAndEntityCanStandOn(checkPos, entity)) continue
            let shape = level.getBlockState(checkPos).getShape(level, checkPos)
            let maxY = shape.max(DirectionAxis.Y)
            if (stepHeight < maxY) {
                if (Math.random() < 0.3) return true
            }
        }
    }
    return false
}