#replace ocelot
tag @e[type=ocelot,limit=1,sort=nearest] add co2replaced_ocelot
summon cat ~ ~ ~
execute at @s run tag @e[type=cat,tag=!co2newCat,sort=nearest,limit=1] add co2newCat
execute as @e[type=ocelot,tag=co2replaced_ocelot,sort=nearest,limit=1,predicate=co2_oldocelot:is_child] run data modify entity @e[type=cat,tag=co2newCat,sort=nearest,limit=1] Age set value -1000
tp @e[type=ocelot,tag=co2replaced_ocelot,sort=nearest,limit=1] ~ ~-1000 ~