
StartupEvents.registry('item', event => {

    //Copper Knife
    event.create("copper_knife", 'farmersdelight:knife').displayName("Knife").tier('copper').attackDamageBaseline(0)

    //Frog Leg (Cooked & Golden)
    event.create("frog_leg").displayName("Frog Leg")
    event.create("cooked_frog_leg").displayName("Cooked Frog Leg")
    event.create("golden_frog_leg").displayName("Golden Frog Leg")

    //Raw Titanium
    event.create("raw_titanium").displayName("Raw Titanium")

    //New Materials
    event.create("chain_mesh").displayName("Chain Mesh")
    event.create("aquamarine").displayName("Aquamarine")
    event.create("pearl").displayName("Pearl")

    //New Fruits
    event.create("cherry").displayName("Cherry")
    event.create("chestnut").displayName("Chestnut")
    event.create("cranberries").displayName("Cranberries")
    event.create("durian").displayName("Durian")
    event.create("mango").displayName("Mango")
    event.create("pear").displayName("Pear")
    event.create("plum").displayName("Plum")

    //New Flawless Drops
    event.create("valkyrie_feather").displayName("Valkyrie Feather")
    event.create("slider_rubble").displayName("Slider Rubble")
    event.create("serpant_horn").displayName("Serpent Horn")

    //Meteorite Stuff
    event.create("raw_meteorite").displayName("Raw Meteorite")
    event.create("meteorite_ingot").displayName("Meteorite Ingot")

    //Antler - replaces Naturalist Antlers due to an unremovable potion
    event.create("antler").displayName("Antler")
    
  })