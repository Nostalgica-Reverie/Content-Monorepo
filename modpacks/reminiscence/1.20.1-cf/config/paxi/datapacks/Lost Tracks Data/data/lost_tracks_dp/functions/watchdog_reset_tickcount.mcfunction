execute as @e[type=marker,tag=imd_jukebox_marker,tag=imd_is_playing,tag=imd_has_custom_disc] at @s run data merge block ~ ~ ~ {TickCount:0L}
schedule function lost_tracks_dp:watchdog_reset_tickcount 10s replace
