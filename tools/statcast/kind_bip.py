"""Balls in play by pitch type, in and out of the zone: the engine against the 30-day 2025
Statcast sample (download.sh). For each type and zone side: home runs per ball in play, the
mean and spread of launch angle and exit velocity, the shares of launch-angle classes (ground
ball < 10 deg, line drive 10-25, fly ball 25-50, pop-up 50 and up) and of hard balls in the air
(20-40 deg, 100 mph and up).

The zone is the rulebook zone grown by the ball's radius, as `zone_tables.py` and the engine's
`StrikeZone::signed_distance` draw it. Bunts are left out on both sides. Engine rows come from
`bm contacts --out FILE` (rows with `res` X; position and zone in metres, exit speed in m/s,
launch angle in radians).

usage: python3 -I kind_bip.py CONTACTS_CSV STATCAST_DIR"""
import csv
import glob
import math
import os
import sys

KINDS = ['FF', 'SI', 'FC', 'SL', 'ST', 'CU', 'CH', 'FS']
BALL_R = 1.45 / 12
HALF = 8.5 / 12 + BALL_R
FT = 0.3048
MPH = 1609.344 / 3600


def in_zone(x, z, bot, top):
    """Plate position and zone edges in feet."""
    return abs(x) <= HALF and bot - BALL_R <= z <= top + BALL_R


def summary(rows):
    n = len(rows)
    if n == 0:
        return None
    la = [r[0] for r in rows]
    ev = [r[1] for r in rows]

    def mean(v):
        return sum(v) / len(v)

    def sd(v):
        return math.sqrt(max(mean([x * x for x in v]) - mean(v) ** 2, 0.0))

    def share(cond):
        return 100.0 * sum(1 for r in rows if cond(r)) / n

    return (n, share(lambda r: r[2]), mean(la), sd(la), mean(ev), sd(ev), share(lambda r: r[0] < 10), share(lambda r: 10 <= r[0] < 25),
            share(lambda r: 25 <= r[0] < 50), share(lambda r: r[0] >= 50), share(lambda r: 20 <= r[0] < 40 and r[1] >= 100))


def engine(path):
    out = {k: ([], []) for k in KINDS}
    with open(path) as f:
        for r in csv.DictReader(f):
            if r['res'] != 'X' or r['kind'] not in KINDS:
                continue
            x, z, bot, top = (float(r[c]) / FT for c in ('px', 'pz', 'zb', 'zt'))
            row = (math.degrees(float(r['la'])), float(r['ev']) / MPH, r['play'] == 'HomeRun')
            out[r['kind']][0 if in_zone(x, z, bot, top) else 1].append(row)
    return out


def statcast(directory):
    out = {k: ([], []) for k in KINDS}
    for fn in sorted(glob.glob(os.path.join(directory, '*.csv'))):
        with open(fn, encoding='utf-8-sig') as f:
            for r in csv.DictReader(f):
                if r.get('game_type', 'R') != 'R' or not r['description'].startswith('hit_into_play') or r['pitch_type'] not in KINDS:
                    continue
                if 'bunt' in r.get('des', '').lower():
                    continue
                try:
                    x, z, bot, top = float(r['plate_x']), float(r['plate_z']), float(r['sz_bot']), float(r['sz_top'])
                    row = (float(r['launch_angle']), float(r['launch_speed']), r['events'] == 'home_run')
                except ValueError:
                    continue
                out[r['pitch_type']][0 if in_zone(x, z, bot, top) else 1].append(row)
    return out


def main():
    sim, mlb = engine(sys.argv[1]), statcast(sys.argv[2])
    names = ['HR/BIP %', 'LA mean', 'LA SD', 'EV mean', 'EV SD', 'GB %', 'LD %', 'FB %', 'PU %', 'hard air %']
    print('balls in play, sim / MLB 2025:  n, ' + ', '.join(names))
    for k in KINDS:
        for zone in (0, 1):
            a, b = summary(sim[k][zone]), summary(mlb[k][zone])
            if a is None or b is None:
                continue
            cells = '  '.join('%5.1f/%5.1f' % (a[i], b[i]) for i in range(1, 11))
            print('%s %s %6d/%5d  %s' % (k, 'in ' if zone == 0 else 'out', a[0], b[0], cells))


if __name__ == '__main__':
    main()
