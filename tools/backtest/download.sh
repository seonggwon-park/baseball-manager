#!/bin/bash
# Data for tools/backtest/hitters.py, into DIR: MLB Stats API season hitting lines (one request
# per season) and the Statcast custom leaderboard for hitters (one request per season).
# usage: download.sh DIR
OUT="$1"
mkdir -p "$OUT"
for y in 2022 2023 2024 2025 2026; do
  f="$OUT/hit_$y.json"
  [ -s "$f" ] || curl -sS -m 120 -o "$f" "https://statsapi.mlb.com/api/v1/stats?stats=season&group=hitting&season=$y&sportId=1&playerPool=All&limit=3000"
  sleep 1
done
SEL="player_age,pa,k_percent,bb_percent,batting_avg,slg_percent,on_base_percent,babip,home_run,woba,xwoba,z_swing_percent,oz_swing_percent,whiff_percent,swing_percent,iz_contact_percent,oz_contact_percent,avg_swing_speed,exit_velocity_avg,launch_angle_avg,sweet_spot_percent,barrel_batted_rate,hard_hit_percent,groundballs_percent,flyballs_percent,linedrives_percent,popups_percent,pull_percent,sprint_speed,squared_up_swing,avg_swing_length,attack_angle"
for y in 2023 2024 2025; do
  f="$OUT/savant_bat_$y.csv"
  [ -s "$f" ] || curl -sS -m 180 -o "$f" "https://baseballsavant.mlb.com/leaderboard/custom?year=$y&type=batter&filter=&min=1&selections=$SEL&chart=false&x=pa&y=pa&r=no&chartType=beeswarm&sort=pa&sortDir=desc&csv=true"
  sleep 2
done
