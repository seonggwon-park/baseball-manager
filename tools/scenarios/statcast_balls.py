"""Measured batted balls for `bm park --balls`: every ball in play of the Statcast sample
(tools/statcast/download.sh: DIR/*.csv) that has an exit speed, a launch angle and hit
coordinates, as `exit_speed_mph,launch_angle_deg,spray_deg,batter_sign,distance_ft,events,
home_team,game_pk` lines. The first five columns are the format of the bm-physics `flight_check`
example. The spray angle comes from the hit coordinates as in
tools/statcast/export_contacts.py (+ toward right field); the sign is +1 for a right-handed
batter; home_team is Statcast's code of the club whose park the ball was hit in, and
game_pk the game (for its weather: mlb_weather.py).

usage: statcast_balls.py DIR OUT.csv"""
import csv
import glob
import math
import os
import sys


def num(row, k):
    try:
        return float(row[k])
    except (KeyError, ValueError):
        return math.nan


def main():
    src, out = sys.argv[1], sys.argv[2]
    n = kept = hr = hr_kept = 0
    with open(out, 'w') as f:
        for fn in sorted(glob.glob(os.path.join(src, '*.csv'))):
            with open(fn, encoding='utf-8-sig') as g:
                for row in csv.DictReader(g):
                    ev, la, hx, hy = num(row, 'launch_speed'), num(row, 'launch_angle'), num(row, 'hc_x'), num(row, 'hc_y')
                    if row.get('type') != 'X' or math.isnan(ev) or math.isnan(la):
                        continue
                    n += 1
                    is_hr = row.get('events') == 'home_run'
                    hr += is_hr
                    if math.isnan(hx) or math.isnan(hy):
                        continue
                    kept += 1
                    hr_kept += is_hr
                    spray = math.degrees(math.atan2(hx - 125.42, 198.27 - hy))
                    sign = 1 if row.get('stand') == 'R' else -1
                    dist = num(row, 'hit_distance_sc')
                    f.write(f"{ev:.1f},{la:.1f},{spray:.2f},{sign},{'' if math.isnan(dist) else f'{dist:.0f}'},{row.get('events', '')},{row.get('home_team', '')},{row.get('game_pk', '')}\n")
    print(f'{kept} balls in play written of {n} with exit speed and launch angle; home runs {hr_kept} of {hr}')


if __name__ == '__main__':
    main()
