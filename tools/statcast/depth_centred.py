"""Contact by depth around each sample's own centre: the 2025 Statcast sample (depth =
intercept_ball_minus_batter_pos_y less its mean over balls in play) against `bm validate --json`
(its `depth_fine` histogram, centred on the engine's own mean over balls in play). Per bin of
depth: whiff (caught foul tips included) and foul shares of full swings (bat speed 60 mph and
up), and the exit velocity and launch angle of balls in play.

The report of `bm validate` puts the MLB bins at 30 in and the engine's at its usual contact
point; the two means differ by about 2 in, which this comparison removes.

usage: depth_centred.py SAMPLE.npz VALIDATE.json   (SAMPLE.npz from load.py)"""
import json
import sys

import numpy as np

BINS = [(-25, -15), (-15, -8), (-8, -3), (-3, 1), (1, 5), (5, 10), (10, 17), (17, 25)]

d = np.load(sys.argv[1], allow_pickle=True)
ty, desc, iy, bs = d['type'], d['description'], d['intercept_ball_minus_batter_pos_y_inches'], d['bat_speed']
ev, la = d['launch_speed'], d['launch_angle']
sw = np.isfinite(iy) & np.isfinite(bs) & (bs >= 60)
whiff = np.isin(desc, ['swinging_strike', 'swinging_strike_blocked', 'foul_tip'])
foul = desc == 'foul'
bip = (ty == 'X') & np.isfinite(ev) & np.isfinite(la)
dy = iy - np.mean(iy[sw & bip])

doc = json.load(open(sys.argv[2]))
h, origin = doc['depth_fine'], doc['depth_fine_from_in']
n_bip = sum(r[3] for r in h)
centre = sum((origin + i + 0.5) * r[3] for i, r in enumerate(h)) / n_bip
print('engine centre %.2f in from its usual contact point; MLB centre %.2f in of intercept' % (centre, np.mean(iy[sw & bip])))
print('  depth (in)   whiff %        foul %         EV (mph)       LA (deg)        engine / MLB')
for lo, hi in BINS:
    k = sw & (dy >= lo) & (dy < hi)
    b = k & bip
    rows = [r for i, r in enumerate(h) if lo <= origin + i + 0.5 - centre < hi]
    s, ip = sum(r[0] for r in rows), sum(r[3] for r in rows)
    print('  [%3d,%3d)   %5.1f/%5.1f    %5.1f/%5.1f    %5.1f/%5.1f    %5.1f/%5.1f' % (
        lo, hi, 100 * sum(r[1] for r in rows) / s, 100 * whiff[k].mean(), 100 * sum(r[2] for r in rows) / s, 100 * foul[k].mean(),
        sum(r[4] for r in rows) / ip, ev[b].mean(), sum(r[5] for r in rows) / ip, la[b].mean()))
