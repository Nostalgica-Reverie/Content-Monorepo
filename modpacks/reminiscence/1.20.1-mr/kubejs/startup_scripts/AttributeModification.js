//NONE OF THIS WAS WRITTEN BY ME

// let $Attributes = Java.loadClass('net.minecraft.world.entity.ai.attributes.Attributes')
// let $AttributeModifier = Java.loadClass('net.minecraft.world.entity.ai.attributes.AttributeModifier')
// let $ISSAttributeRegistry = Java.loadClass('io.redspace.ironsspellbooks.api.registry.AttributeRegistry')



// ForgeEvents.onEvent('net.minecraftforge.event.ItemAttributeModifierEvent', event => {
//     if (event.itemStack.id == 'aether:valkyrie_lance' && event.slotType == 'mainhand') {
//         //Inputting a ResourceLocation or a string of an attribute ID also works, but I just use the direct attribute
//         event.addModifier($Attributes.ATTACK_DAMAGE, new $AttributeModifier(UUID.fromString('CB3F55D3-645C-4F38-A497-9C13A33DB5CF'), 'aether_sword_buff', 6.0, 'addition'))
//     }
// })