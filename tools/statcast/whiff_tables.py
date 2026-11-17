"""Whiff share of full swings (bat speed 60 mph and up) by the straight-line vertical error
(tan descent - tan attack) x depth and by depth x (attack - descent), for the Statcast
sample and for simulated swings (`bm contacts --swings --out FILE`).

usage: whiff_tables.py SAMPLE.npz [SIMULATED_SWINGS.csv ...]"""
import sys, csv, json
import numpy as np
M_BINS = [(-30, -8), (-8, -5), (-5, -3), (-3, -1.5), (-1.5, -0.5), (-0.5, 0.5), (0.5, 1.5), (1.5, 3), (3, 5), (5, 30)]
D_ROWS = [(-40, -10), (-10, -4), (-4, 4), (4, 10), (10, 40)]
A_COLS = [(-40, 0), (0, 4), (4, 8), (8, 12), (12, 18), (18, 60)]

def mlb(npz):
    d = np.load(npz, allow_pickle=True); d = {k: d[k] for k in d.files}
    aa = d['attack_angle']; bs = d['bat_speed']; ty = d['type']; desc = d['description']; iy = d['intercept_ball_minus_batter_pos_y_inches']
    vy0, vz0, ay, az = d['vy0'], d['vz0'], d['ay'], d['az']
    t = (-vy0 - np.sqrt(vy0 ** 2 - 2 * ay * (50 - 17 / 12)))/ay
    de = np.degrees(np.arctan(-(vz0 + az * t) / np.abs(vy0 + ay * t)))
    sw = np.isfinite(aa) & np.isfinite(bs) & (bs >= 60) & np.isfinite(iy) & np.isfinite(de)
    whiff = np.isin(desc, ['swinging_strike', 'swinging_strike_blocked'])
    dy_in = iy - np.nanmean(iy[sw & (ty == 'X')])
    return sw, whiff, dy_in, aa, de

def sim(fn):
    rows = list(csv.DictReader(open(fn)))
    g = lambda k: np.array([float(r[k]) for r in rows])
    res = np.array([r['result'] for r in rows])
    bs = g('bs') / 0.44704; geo = g('geo') > 0
    aa = np.degrees(g('aa')); de = np.degrees(g('descent')); depth = g('depth') / 0.0254
    sw = (bs >= 60) & geo
    whiff = res == 'W'
    dy_in = depth - np.mean(depth[sw & (res == 'X')])
    return sw, whiff, dy_in, aa, de

def tables(sw, whiff, dy_in, aa, de):
    m = (np.tan(np.radians(de)) - np.tan(np.radians(aa))) * dy_in * 2.54
    out = {'whiff_all': 100 * np.mean(whiff[sw])}
    for lo, hi in M_BINS:
        k = sw & (m >= lo) & (m < hi)
        out['m[%g,%g)' % (lo, hi)] = (100 * np.mean(whiff[k]), int(k.sum()))
    for lo, hi in D_ROWS:
        for a, b in A_COLS:
            k = sw & (dy_in >= lo) & (dy_in < hi) & (aa - de >= a) & (aa - de < b)
            out['d[%d,%d)a[%d,%d)' % (lo, hi, a, b)] = (100 * np.mean(whiff[k]) if k.sum() else float('nan'), int(k.sum()))
    return out

if __name__ == '__main__':
    a = tables(*mlb(sys.argv[1]))
    runs = [(fn.split('/')[-1][:12], tables(*sim(fn))) for fn in sys.argv[2:]]
    print('%-22s %14s' % ('', 'MLB') + ''.join('%16s' % n for n, _ in runs))
    for k in a:
        v = a[k]
        if isinstance(v, tuple):
            print('%-22s %7.1f (%5d)' % (k, v[0], v[1]) + ''.join('%9.1f (%5d)' % r[k] for _, r in runs))
        else:
            print('%-22s %14.1f' % (k, v) + ''.join('%16.1f' % r[k] for _, r in runs))
