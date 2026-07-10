execute as @e[type=ocelot,sort=nearest,limit=1] at @s run function co2_oldocelot:cat_replace
data modify entity @e[type=cat,tag=co2newCat,sort=nearest,limit=1] Owner set from entity @s UUID
tag @e[type=cat,tag=co2newCat,sort=nearest,limit=1] remove co2newCat