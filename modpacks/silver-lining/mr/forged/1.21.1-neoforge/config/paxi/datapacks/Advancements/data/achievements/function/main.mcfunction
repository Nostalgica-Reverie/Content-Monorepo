# on a rail
execute as @a at @s unless entity @n[type=minecraft:minecart,distance=..2] run scoreboard players reset @s ach.distance_minecart
execute as @a[limit=1] if score @s ach.distance_minecart matches 100000.. run advancement grant @s only achievements:on_a_rail
# when pigs fly
execute as @a at @s store result score @s ach.pig run data get entity @n[type=minecraft:pig,distance=..1.5,nbt={Saddle:1b}] FallDistance
execute as @a[scores={ach.pig=5..},limit=1] run advancement grant @s only achievements:when_pigs_fly