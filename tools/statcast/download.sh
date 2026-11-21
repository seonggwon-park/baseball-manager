#!/bin/bash
# Download the 30-day 2025 regular-season sample of pitch-level Statcast data (public
# Baseball Savant CSV search) used by docs/physics-model.md, one file per day, into DIR.
# usage: download.sh DIR
OUT="$1"
mkdir -p "$OUT"
for d in 2025-04-05 2025-04-16 2025-04-27 2025-05-08 2025-05-19 2025-05-30 2025-06-10 2025-06-21 2025-07-02 2025-07-12 2025-07-23 2025-08-03 2025-08-14 2025-08-25 2025-09-05 2025-09-16 2025-09-24 2025-04-11 2025-05-03 2025-05-25 2025-06-15 2025-07-06 2025-07-29 2025-08-09 2025-08-20 2025-09-10 2025-04-22 2025-06-04 2025-06-27 2025-08-30; do
  f="$OUT/$d.csv"
  if [ -s "$f" ]; then continue; fi
  for attempt in 1 2 3; do
    curl -sS -m 180 -o "$f" "https://baseballsavant.mlb.com/statcast_search/csv?all=true&hfGT=R%7C&hfSea=2025%7C&player_type=pitcher&game_date_gt=$d&game_date_lt=$d&type=details" && [ -s "$f" ] && break
    sleep 5
  done
  echo "$d $(wc -l < "$f") lines"
  sleep 2
done
