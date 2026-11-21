"""How far balls carry in each major-league park beyond what its elevation and the game's
temperature explain, measured on the Statcast sample, and the batted-ball drag factor of
data/leagues/us.json (`batted_drag`) that the game uses for it.

Every fly ball of the sample (exit velocity 85 mph or more, launch angle 20-40 deg, whatever
became of it, the Rays' 2025 park left out) is flown with the engine's batted-ball model at its
park's elevation and its game's temperature, without wind (the bm-physics example
`carry_by_ball`). Its residual is the measured distance (Statcast's projected distance) less the
model's. A park's carry is its mean residual less the mean over all parks, so the factors leave
the league as it was. The parks' carries are shrunk toward zero by their noise (empirical Bayes:
the between-park variance over that plus the park's squared standard error), and turned into a
drag factor with the model's own sensitivity (the mean distance change when the drag is raised
by 2 %).

Outcomes are not filtered: a caught ball and a hit are both kept, so the sample is not chosen by
how far the ball went. The reported wind is not used; with it the parks spread four times as much
as without (docs/physics-model.md 7).

usage: park_carry.py CARRY_BY_BALL BALLS.csv WEATHER.json OUTDIR [--write]
  CARRY_BY_BALL: cargo build --release --example carry_by_ball -p bm-physics (target/release/examples/carry_by_ball)
  BALLS.csv: statcast_balls.py; WEATHER.json: mlb_weather.py"""
import json
import math
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from park_balls_check import CLUBS, REPO, weather  # noqa: E402

PROBE = 1.02  # drag factor of the sensitivity run


def flights(binary, path, drag=1.0):
    out = subprocess.run([binary, path, f'{drag}'], check=True, capture_output=True, text=True).stdout
    return [line.split(',') for line in out.splitlines()]


def fly_ball(x):
    ev, la, dist = float(x[0]), float(x[1]), float(x[4])
    return ev >= 85 and 20 <= la <= 40 and dist > 0


def main():
    binary, balls, weather_file, outdir = sys.argv[1:5]
    os.makedirs(outdir, exist_ok=True)
    data_path = os.path.join(REPO, 'data/leagues/us.json')
    clubs = json.load(open(data_path))['clubs']
    parks = {c['id']: c['park'] for c in clubs}
    games = json.load(open(weather_file))
    flown = os.path.join(outdir, 'balls_air.csv')
    n = 0
    with open(flown, 'w') as f:
        for line in open(balls):
            x = line.rstrip('\n').split(',')
            if x[6] not in CLUBS:
                continue
            w = weather(games.get(x[7]))
            if w is None:
                continue
            f.write(f"{','.join(x[:5])},{w[0]:.1f},{parks[CLUBS[x[6]]].get('elevation_m') or 0},{x[6]}\n")
            n += 1
    base = flights(binary, flown)
    probe = flights(binary, flown, PROBE)
    # The model's own sensitivity: feet per 1 % of drag over the fly balls.
    d = [float(b[8]) - float(a[8]) for a, b in zip(base, probe) if fly_ball(a)]
    ft_per_pct = sum(d) / len(d) / (100 * (PROBE - 1))
    by = {}
    for x in base:
        if fly_ball(x):
            by.setdefault(x[7], []).append(float(x[4]) - float(x[8]))
    every = [r for v in by.values() for r in v]
    league = sum(every) / len(every)
    stats = {}
    for code, v in by.items():
        m = sum(v) / len(v)
        se = math.sqrt(sum((a - m) ** 2 for a in v) / (len(v) - 1) / len(v))
        stats[code] = (m - league, se, len(v))
    # Between-park variance of the true carries: the spread of the park means less their noise.
    tau2 = max(sum(s[0] ** 2 for s in stats.values()) / len(stats) - sum(s[1] ** 2 for s in stats.values()) / len(stats), 0.0)
    factors = {}
    print(f'{n} balls of the sample flown, {len(every)} fly balls; league mean residual {league:+.1f} ft; '
          f'{ft_per_pct:+.2f} ft per 1 % of drag; between-park SD of the carry {math.sqrt(tau2):.1f} ft')
    print('| park | fly balls | carry (ft, measured - model, against the league) | shrunk | batted-ball drag |')
    print('|---|---|---|---|---|')
    for code in sorted(stats, key=lambda c: stats[c][0]):
        carry, se, k = stats[code]
        shrunk = carry * tau2 / (tau2 + se * se)
        factor = 1.0 + shrunk / ft_per_pct / 100.0
        factors[CLUBS[code]] = round(factor, 3)
        print(f'| {code} | {k} | {carry:+.1f} ± {se:.1f} | {shrunk:+.1f} | {factor:.3f} |')
    chi = sum((s[0] / s[1]) ** 2 for s in stats.values())
    print(f'sum of squared z of the carries over {len(stats)} parks: {chi:.0f}')
    json.dump({'ft_per_pct_drag': ft_per_pct, 'tau_ft': math.sqrt(tau2), 'parks': {c: list(s) for c, s in stats.items()}, 'batted_drag': factors},
              open(os.path.join(outdir, 'park_carry.json'), 'w'), indent=1)
    if '--write' in sys.argv:
        text = open(data_path).read()
        doc = json.loads(text)
        for c in doc['clubs']:
            if c['id'] in factors:
                c['park']['batted_drag'] = factors[c['id']]
        open(data_path, 'w').write(json.dumps(doc, ensure_ascii=False, indent=1) + '\n')
        print(f'wrote batted_drag for {len(factors)} parks into {data_path}')


if __name__ == '__main__':
    main()
