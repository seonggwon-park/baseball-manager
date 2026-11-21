"""Balls in the air that fall for hits, by how far the nearest fielder stands from where
the ball comes down and how long it hangs: the Statcast sample against the engine.

Both sets are flown the same way (crates/bm-physics/examples/fly_list.rs: the engine's
batted-ball drag and lift, Nathan's spin regressions, average air) from exit speed,
launch angle and the bearing of the spot where the ball was fielded, and both use the
engine's standard fielder positions leaned toward the batter's pull side. Only the
outcomes come from the two sources, so a difference in a cell is a difference in how
often such balls are turned into outs. Home runs are left out.

usage: catch_check.py SAMPLE.npz CONTACTS.csv FLY_LIST_BINARY
  SAMPLE.npz    Statcast pitches (tools/statcast/load.py)
  CONTACTS.csv  `bm contacts` output (balls in play with play and fielder columns)
"""
import csv
import os
import subprocess
import sys
import tempfile

import numpy as np

FT = 0.3048
MPH = 0.44704
# Engine standard positions (crates/bm-engine/src/fielding.rs): distance ft, bearing deg
# (+ toward first base), and the lean toward the pull side (corners 3, middle 5, outfield 0).
FIELDERS = {3: (110, 34, 3), 4: (147, 13, 5), 5: (118, -31, 3), 6: (147, -13, 5), 7: (297, -27, 0), 8: (321, 0, 0), 9: (294, 27, 0)}
LA_BANDS = [(10, 25), (25, 40), (40, 50), (50, 60), (60, 90)]
D_BINS = [0, 5, 10, 15, 20, 25, 30, 40, 200]
T_BINS = [0, 2, 3, 4, 5, 6, 9]


def mlb(path):
    d = np.load(path, allow_pickle=True)
    d = {k: d[k] for k in d.files}
    ev = d['launch_speed'].astype(float)
    la = d['launch_angle'].astype(float)
    hx, hy = d['hc_x'].astype(float), d['hc_y'].astype(float)
    ok = (d['type'] == 'X') & np.isfinite(ev) & np.isfinite(la) & np.isfinite(hx) & (d['events'] != 'home_run')
    direction = np.degrees(np.arctan2(hx - 125.42, 198.27 - hy))
    sign = np.where(d['stand'] == 'R', 1.0, -1.0)
    hit = np.isin(d['events'], ['single', 'double', 'triple'])
    return ev[ok], la[ok], direction[ok], sign[ok], hit[ok]


def sim(path):
    rows = [r for r in csv.DictReader(open(path)) if r['res'] == 'X' and r['play'] not in ('-', 'HomeRun', 'GroundRuleDouble', 'Foul')]
    ev = np.array([float(r['ev']) / MPH for r in rows])
    la = np.degrees([float(r['la']) for r in rows])
    sign = np.array([1.0 if r['hand'] == 'R' else -1.0 for r in rows])
    # fbear is + toward the pull side; the pull side of a right-handed batter is third base.
    direction = np.degrees([float(r['fbear']) for r in rows]) * -sign
    hit = np.array([r['play'] in ('Single', 'Double', 'Triple') for r in rows])
    return ev, la, direction, sign, hit


def fly(binary, ev, la, direction, sign):
    with tempfile.TemporaryDirectory() as td:
        src = os.path.join(td, 'in.csv')
        with open(src, 'w') as f:
            for row in zip(ev, la, direction, sign):
                f.write('%.3f,%.3f,%.3f,%.0f\n' % row)
        out = subprocess.run([binary, src], capture_output=True, text=True, check=True).stdout.splitlines()
    vals = np.array([[float(v) for v in line.split(',')] for line in out[1:]])
    return vals[:, 0], vals[:, 1], vals[:, 2]


def nearest(x, y, sign):
    best = np.full(len(x), np.inf)
    who = np.zeros(len(x), int)
    for pos, (r, b, lean) in FIELDERS.items():
        # The pull side of a right-handed batter is the third-base side (negative bearing).
        a = np.radians(b - lean * sign)
        fx, fy = r * FT * np.sin(a), r * FT * np.cos(a)
        dist = np.hypot(x - fx, y - fy)
        closer = dist < best
        best[closer] = dist[closer]
        who[closer] = pos
    return best, who


def table(name, cells, bins, label):
    print('%s, hit rate sim / MLB (n sim / n MLB)' % name)
    for lo, hi in LA_BANDS:
        parts = []
        for i in range(len(bins) - 1):
            s, m = cells['sim'], cells['mlb']
            sel_s = (s['la'] >= lo) & (s['la'] < hi) & (s[label] >= bins[i]) & (s[label] < bins[i + 1])
            sel_m = (m['la'] >= lo) & (m['la'] < hi) & (m[label] >= bins[i]) & (m[label] < bins[i + 1])
            if sel_m.sum() < 20:
                continue
            parts.append('[%g,%g) %.3f/%.3f (%d/%d)' % (bins[i], bins[i + 1], s['hit'][sel_s].mean() if sel_s.any() else np.nan, m['hit'][sel_m].mean(), sel_s.sum(), sel_m.sum()))
        print('  LA %2d-%2d  ' % (lo, hi) + '  '.join(parts))


def main():
    sample, contacts, binary = sys.argv[1:4]
    cells = {}
    for key, (ev, la, direction, sign, hit) in (('mlb', mlb(sample)), ('sim', sim(contacts))):
        air = la >= 10
        ev, la, direction, sign, hit = ev[air], la[air], direction[air], sign[air], hit[air]
        x, y, t = fly(binary, ev, la, direction, sign)
        d, who = nearest(x, y, sign)
        cells[key] = {'la': la, 'hit': hit, 'd': d, 't': t, 'who': who}
    table('Nearest fielder to the landing point (m)', cells, D_BINS, 'd')
    table('Hang time (s)', cells, T_BINS, 't')
    # One number for the whole comparison: squared differences in standard errors.
    chi, cells_n = 0.0, 0
    for lo, hi in LA_BANDS:
        for i in range(len(D_BINS) - 1):
            sel = [(c['la'] >= lo) & (c['la'] < hi) & (c['d'] >= D_BINS[i]) & (c['d'] < D_BINS[i + 1]) for c in (cells['sim'], cells['mlb'])]
            ns, nm = sel[0].sum(), sel[1].sum()
            if nm < 20 or ns < 20:
                continue
            ps, pm = cells['sim']['hit'][sel[0]].mean(), cells['mlb']['hit'][sel[1]].mean()
            var = max(pm * (1 - pm), 0.01) / nm + max(ps * (1 - ps), 0.01) / ns
            chi += (ps - pm) ** 2 / var
            cells_n += 1
    print('chi-square over %d cells: %.1f' % (cells_n, chi))
    for key in ('sim', 'mlb'):
        c = cells[key]
        for lo, hi in LA_BANDS:
            s = (c['la'] >= lo) & (c['la'] < hi)
            print('%s LA %2d-%2d: n %6d  hit %.3f  mean nearest %.1f m  mean hang %.2f s' % (key, lo, hi, s.sum(), c['hit'][s].mean(), c['d'][s].mean(), c['t'][s].mean()))


if __name__ == '__main__':
    main()
