EntityJSEvents.modifyEntity(event => {

    // SNAIL
    event.modify("naturalist:snail", e => {
        e.setHurtSound(() => "nostalgic_tweaks:blank")
    })

    // TORTOISE
    event.modify("naturalist:tortoise", e => {
        e.setDeathSound("minecraft:entity.turtle.death")
    })

    // DEER
    event.modify("naturalist:deer", e => {
        e.setDeathSound("nostalgic_tweaks:blank")
    })

    // SNAKE
    event.modify("naturalist:snake", e => {
        e.setDeathSound("nostalgic_tweaks:blank")
    })

    // BUTTERFLY
    event.modify("naturalist:butterfly", e => {
        e
            .setDeathSound("nostalgic_tweaks:blank")
            .setHurtSound(() => "nostalgic_tweaks:blank")
    })

    // GIRAFFE
    event.modify("naturalist:giraffe", e => {
        e
            .setDeathSound("nostalgic_tweaks:blank")
            .setHurtSound(() => "nostalgic_tweaks:blank")
    })

})
