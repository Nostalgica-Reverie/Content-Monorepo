execute as @e[type=marker,tag=imd_jukebox_marker] at @s unless block ~ ~ ~ minecraft:jukebox run function lost_tracks_dp:destroy_jukebox_marker
execute as @e[type=marker,tag=imd_jukebox_marker] at @s if block ~ ~ ~ minecraft:jukebox run function lost_tracks_dp:jukebox_check_playing
execute as @e[type=marker,tag=imd_jukebox_marker,tag=imd_is_playing,tag=imd_has_custom_disc] at @s run function lost_tracks_dp:jukebox_tick_timers
