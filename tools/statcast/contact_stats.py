"""Regression summaries of balls in play run backwards through the collision model
(`bm contacts --invert`): the impact parameter and the attack angle against pitch height,
vertical movement, depth of contact, location and pitch type, plus launch-angle
summaries. Offsets in cm, angles in degrees.

usage: contact_stats.py INVERTED.csv [--sim]   (--sim: simulated rows, whose depth is
already measured from each batter's usual contact point; otherwise depth is centred on
each batter's own mean, as for the Statcast sample)"""
import csv
import numpy as np

KINDS = ['FF', 'SI', 'FC', 'SL', 'ST', 'CU', 'CH', 'FS']
DEPTH_BINS = [(-25, -15), (-15, -8), (-8, -3), (-3, 1), (1, 5), (5, 10), (10, 17), (17, 25)]
HEIGHT_BINS = [(-1.5, -0.5), (-0.5, -0.25), (-0.25, 0), (0, 0.25), (0.25, 0.5), (0.5, 1.5)]


def load(fn):
    rows = list(csv.DictReader(open(fn)))
    out = {}
    for k in rows[0]:
        vals = [x[k] for x in rows]
        try:
            out[k] = np.array([float(v) if v not in ('', 'nan') else np.nan for v in vals])
        except ValueError:
            out[k] = np.array(vals, dtype=object)
    return out


def ols(X, y):
    b, *_ = np.linalg.lstsq(X, y, rcond=None)
    r = y - X @ b
    return b, r.std()


def stats(d, per_batter):
    ok = np.isfinite(d['fit_offset']) & (d['bs'] >= 60 * 0.44704) & np.isin(d['kind'], KINDS) & np.isfinite(d['depth']) & np.isfinite(d['move_z'])
    if 'res' in d:
        ok &= d['res'] == 'X'
    if per_batter:
        dy = d['depth'].copy(); b = d['batter']
        for bid in np.unique(b[ok]):
            s = ok & (b == bid)
            dy[b == bid] -= np.mean(d['depth'][s]) if s.sum() >= 20 else np.nanmean(d['depth'][ok])
    else:
        dy = d['depth'] - np.nanmean(d['depth'][ok])
    h = d['pz'] - 0.5 * (d['zb'] + d['zt'])
    ride = d['move_z'].copy()
    for k in KINDS:
        s = d['kind'] == k
        ride[s] -= np.nanmean(d['move_z'][s & ok])
    inside = -np.where(d['hand'] == 'R', 1.0, -1.0) * d['px']
    dummies = [(d['kind'] == k).astype(float) for k in KINDS[1:]]
    out = {}
    # offset (fitted, cm)
    X = np.stack([np.ones_like(h), h, ride, dy, dy * dy] + dummies, 1)[ok]
    y = d['fit_offset'][ok] * 100
    b, sd = ols(X, y)
    out['off.const'] = b[0]; out['off.height'] = b[1] / 100; out['off.ride'] = b[2] / 100
    out['off.depth'] = b[3] / 100; out['off.depth2'] = b[4] / 100
    for i, k in enumerate(KINDS[1:]):
        out['off.' + k] = b[5 + i]
    out['off.sd'] = sd
    out['off.mean'] = y.mean()
    for q in (10, 25, 50, 75, 90):
        out['off.p%d' % q] = np.percentile(y, q)
    zrel = (h / (d['zt'] - d['zb']))
    for lo, hi in DEPTH_BINS:
        s = ok & (dy / 0.0254 >= lo) & (dy / 0.0254 < hi)
        out['offdep[%d,%d)' % (lo, hi)] = np.mean(d['fit_offset'][s]) * 100
    # attack angle (deg)
    X = np.stack([np.ones_like(h), dy / 0.0254, h / 0.3048, inside / 0.0254] + dummies, 1)[ok]
    y = np.degrees(d['aa'][ok])
    b, sd = ols(X, y)
    out['aa.const'] = b[0]; out['aa.depth'] = b[1]; out['aa.height'] = b[2]; out['aa.inside'] = b[3]
    for i, k in enumerate(KINDS[1:]):
        out['aa.' + k] = b[4 + i]
    out['aa.sd'] = sd
    out['aa.mean'] = y.mean(); out['aa.total_sd'] = y.std()
    # outcomes
    la = np.degrees(d['la'][ok])
    out['la.mean'] = la.mean(); out['la.sd'] = la.std()
    out['gb%'] = 100 * np.mean(la < 10); out['ld%'] = 100 * np.mean((la >= 10) & (la < 25)); out['fb%'] = 100 * np.mean((la >= 25) & (la < 50)); out['pu%'] = 100 * np.mean(la >= 50)
    for lo, hi in HEIGHT_BINS:
        s = ok & (zrel >= lo) & (zrel < hi)
        out['la.h[%.2f,%.2f)' % (lo, hi)] = np.degrees(np.mean(d['la'][s]))
    for k in KINDS:
        s = ok & (d['kind'] == k)
        out['la.' + k] = np.degrees(np.mean(d['la'][s]))
    return out


if __name__ == '__main__':
    import json, sys
    st = stats(load(sys.argv[1]), '--sim' not in sys.argv[2:])
    print(json.dumps(st, indent=1))
