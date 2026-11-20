"""How much exit velocity a ball in play loses beyond what its vertical miss explains: the engine
against the 2025 Statcast sample, through the same collision model.

`bm contacts --invert` finds for each ball in play the bat-ball offset that sends it off at its
measured launch angle from the measured swing (bat speed, attack angle, direction, tilt), and
the exit velocity (`fit_ev`) the collision model then gives for a ball met on the sweet spot.
The ratio EV / fit_ev is what the vertical miss does not explain. In the engine that is where on
the bat the ball was met (the sweet spot is 5.5 in from the tip; the spread around it is the
batter's `reach_sd` plus the shifts for timing and location). In the sample it also carries the
measurement error of exit velocity, bat speed and launch angle; the engine's ratios are given
that error as a factor exp(N(0, s)) before their quantiles are compared.

Groups by the signed distance of the ball from the zone grown by the ball's radius:
heart (more than 6 in inside), edge (6 in inside to 2.9 in outside), out (beyond).
Rows with bat speed under 60 mph and balls the inversion could not solve are left out.

Prepare the inputs:
  python3 -I tools/statcast/export_contacts.py SAMPLE.npz mlb.csv && bm contacts --invert mlb.csv --out mlb_inv.csv
  bm contacts --out sim.csv --games 3000 [--set w.reach_sd=V] && bm contacts --invert sim.csv --out sim_inv.csv
usage: python3 -I along_check.py MLB_INV_CSV SIM_INV_CSV [SIM_INV_CSV ...]"""
import csv
import math
import random
import sys

BALL_R = 1.45 / 12
HALF = 8.5 / 12 + BALL_R
MPH = 3600 / 1609.344
FT = 0.3048
QUANTILES = (0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95)
NOISE = (0.0, 0.03, 0.05)


def distance_in(x, z, bot, top):
    """Signed distance (in) of the ball's centre outside the zone grown by the ball's radius."""
    dx = abs(x) - HALF
    dz = max((bot - BALL_R) - z, z - (top + BALL_R))
    if dx <= 0 and dz <= 0:
        return max(dx, dz) * 12
    if dx <= 0:
        return dz * 12
    if dz <= 0:
        return dx * 12
    return math.hypot(dx, dz) * 12


def load(path):
    groups = {'heart': [], 'edge': [], 'out': []}
    with open(path) as f:
        for r in csv.DictReader(f):
            if r['res'] != 'X':
                continue
            try:
                bs, ev, fit = float(r['bs']), float(r['ev']), float(r['fit_ev'])
                x, z, bot, top = (float(r[c]) / FT for c in ('px', 'pz', 'zb', 'zt'))
            except ValueError:
                continue
            if bs * MPH < 60 or not (math.isfinite(fit) and fit > 0):
                continue
            d = distance_in(x, z, bot, top)
            groups['heart' if d < -6 else 'edge' if d < 2.9 else 'out'].append(ev / fit)
    return groups


def quantiles(v):
    v = sorted(v)
    return [v[int(q * len(v))] for q in QUANTILES]


def main():
    random.seed(1)
    mlb = load(sys.argv[1])
    sims = [(p.rsplit('/', 1)[-1], load(p)) for p in sys.argv[2:]]
    print('EV / fit_ev quantiles ' + ' '.join('p%d' % round(100 * q) for q in QUANTILES) + ', and their RMS difference from MLB')
    for g in ('heart', 'edge', 'out'):
        target = quantiles(mlb[g])
        print('%s  MLB (n %d)  %s' % (g, len(mlb[g]), ' '.join('%.3f' % x for x in target)))
        for s in NOISE:
            for name, d in sims:
                v = [x * math.exp(random.gauss(0.0, s)) for x in d[g]] if s > 0 else d[g]
                q = quantiles(v)
                rms = math.sqrt(sum((a - b) ** 2 for a, b in zip(q, target)) / len(q))
                print('   noise %.2f  %-24s %s   RMS %.4f' % (s, name, ' '.join('%.3f' % x for x in q), rms))


if __name__ == '__main__':
    main()
