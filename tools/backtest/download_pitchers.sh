#!/bin/bash
# Data for tools/backtest/pitchers.py, into DIR: MLB Stats API season pitching lines (one request
# per season) and the Statcast custom leaderboard for pitchers (one request per season).
# usage: download_pitchers.sh DIR
OUT="$1"
mkdir -p "$OUT"
for y in 2022 2023 2024 2025 2026; do
  f="$OUT/pit_$y.json"
  [ -s "$f" ] || curl -sS -m 120 -o "$f" "https://statsapi.mlb.com/api/v1/stats?stats=season&group=pitching&season=$y&sportId=1&playerPool=All&limit=3000"
  sleep 1
done
SEL="player_age,pa,k_percent,bb_percent,batting_avg,slg_percent,on_base_percent,babip,home_run,woba,xwoba,p_formatted_ip,p_game,z_swing_percent,oz_swing_percent,whiff_percent,swing_percent,in_zone_percent,iz_contact_percent,oz_contact_percent,exit_velocity_avg,launch_angle_avg,sweet_spot_percent,barrel_batted_rate,hard_hit_percent,groundballs_percent,flyballs_percent,linedrives_percent,popups_percent,pull_percent,arm_angle,pitch_hand,fastball_avg_speed,n_ff_formatted,ff_avg_speed,ff_avg_spin,ff_avg_break_z_induced,ff_avg_break_x,n_si_formatted,si_avg_speed,n_fc_formatted,n_sl_formatted,n_st_formatted,n_cu_formatted,n_ch_formatted,n_fs_formatted,n_sv_formatted,n_kn_formatted"
for y in 2023 2024 2025; do
  f="$OUT/savant_pit_$y.csv"
  [ -s "$f" ] || curl -sS -m 180 -o "$f" "https://baseballsavant.mlb.com/leaderboard/custom?year=$y&type=pitcher&filter=&min=1&selections=$SEL&chart=false&x=pa&y=pa&r=no&chartType=beeswarm&sort=pa&sortDir=desc&csv=true"
  sleep 2
done
