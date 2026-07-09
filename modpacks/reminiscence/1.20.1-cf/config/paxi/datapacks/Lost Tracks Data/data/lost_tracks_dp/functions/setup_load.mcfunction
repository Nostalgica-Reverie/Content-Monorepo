scoreboard objectives add imd_player_id dummy
scoreboard objectives add imd_disc_id dummy
scoreboard objectives add imd_rc_steps dummy
scoreboard objectives add imd_play_time dummy
scoreboard objectives add imd_stop_11_time dummy

advancement revoke @a only lost_tracks_dp:placed_disc
advancement revoke @a only lost_tracks_dp:placed_jukebox
tellraw @a [{"text":"Infinite Music Discs v2.1 by link2_thepast", "color":"gold"}]
tellraw @a [{"text":"Type ", "color":"gold"}, {"text":"/function lost_tracks_dp:help", "color":"yellow"}, {"text":" for help", "color":"gold"}]
