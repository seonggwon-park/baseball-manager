"""Streaks of the engine's batters over a season, from the game logs (`bm season --game-logs DIR`).

The engine has no slumps, hot hands or streak mechanism: every plate appearance is drawn on its
own from the two players' attributes and the game state. So whatever streaks its batters have
are what luck alone makes at their talent: the baseline of the "slump" topic.

For every batter of one league with at least --min-pa plate appearances it counts:
  - the longest hitting streak in games (a hit in consecutive games; a game with no at-bat
    neither extends nor ends it, as in the official rule for walks, hit-by-pitch and sacrifices),
  - the longest run of games without a hit (games with an at-bat),
  - the longest run of at-bats without a hit. The logs keep each game's totals, not the order
    of its at-bats, so the hits are placed at random among the game's at-bats (exact for a
    batter whose at-bats do not depend on each other, as in the engine) and the longest run is
    averaged over --draws placements.
Then the same counts for the batter's games in random order (--shuffles orders): if the
engine's schedule, parks and opponents made streaks of their own, the observed and shuffled
columns would differ. The shuffled column pools every batter's shuffles, so its mean, median and
90th percentile describe one season in random order. Its maximum is the league's maximum in one
shuffled season, averaged over the shuffles: the number to set against the observed maximum
(the maximum of the batters' averages over shuffles would be smaller, since averaging removes
the luck that makes the longest streak). In a shuffle the hitless at-bats are averaged over
--draws / 5 placements, close to the observed column's --draws.

usage: python3 -I streaks.py BATTING_CSV [--league us-major] [--min-pa 400] [--draws 50]
                             [--shuffles 50] [--seed 1] [--json OUT.json]"""
import argparse
import csv
import json
import random
import statistics as st
from collections import defaultdict


def longest(flags):
    best = run = 0
    for f in flags:
        run = run + 1 if f else 0
        best = max(best, run)
    return best


def hitting_streak(games):
    best = run = 0
    for ab, h in games:
        if ab == 0 and h == 0:
            continue
        run = run + 1 if h > 0 else 0
        best = max(best, run)
    return best


def hitless_games(games):
    return longest(h == 0 for ab, h in games if ab > 0)


def hitless_at_bats(games, rng):
    seq = []
    for ab, h in games:
        slots = [True] * ab
        for i in rng.sample(range(ab), min(h, ab)):
            slots[i] = False
        seq.extend(slots)
    return longest(seq)


def summary(values):
    v = sorted(values)
    q = lambda p: v[min(len(v) - 1, int(p * len(v)))]
    return {'mean': st.mean(v), 'median': q(0.5), 'p90': q(0.9), 'max': v[-1]}


def shuffled_summary(per_batter):
    """per_batter: one list of shuffled values per batter, all of the same length."""
    s = summary([x for v in per_batter for x in v])
    s['max'] = st.mean(max(v[j] for v in per_batter) for j in range(len(per_batter[0])))
    return s


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('batting')
    ap.add_argument('--league', default='us-major')
    ap.add_argument('--min-pa', type=int, default=400)
    ap.add_argument('--draws', type=int, default=50)
    ap.add_argument('--shuffles', type=int, default=50)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--json')
    a = ap.parse_args()
    rng = random.Random(a.seed)
    games, pa, name = defaultdict(list), defaultdict(int), {}
    for r in csv.DictReader(open(a.batting, newline='')):
        if r['league'] != a.league:
            continue
        k = r['player']
        games[k].append((r['date'], int(r['game']), int(r['ab']), int(r['h'])))
        pa[k] += int(r['pa'])
        name[k] = r['name']
    rows = []
    for k, g in games.items():
        if pa[k] < a.min_pa:
            continue
        g.sort()
        g = [(ab, h) for _, _, ab, h in g]
        ab, h = sum(x for x, _ in g), sum(y for _, y in g)
        obs = {'hitting_streak': hitting_streak(g), 'hitless_games': hitless_games(g),
               'hitless_ab': st.mean(hitless_at_bats(g, rng) for _ in range(a.draws))}
        shuf = {'hitting_streak': [], 'hitless_games': [], 'hitless_ab': []}
        for _ in range(a.shuffles):
            s = g[:]
            rng.shuffle(s)
            shuf['hitting_streak'].append(hitting_streak(s))
            shuf['hitless_games'].append(hitless_games(s))
            shuf['hitless_ab'].append(st.mean(hitless_at_bats(s, rng) for _ in range(max(1, a.draws // 5))))
        rows.append({'player': k, 'name': name[k], 'pa': pa[k], 'ab': ab, 'avg': h / ab if ab else 0.0, 'games': len(g),
                     'observed': obs, 'shuffled': {m: st.mean(v) for m, v in shuf.items()}, 'shuffles': shuf})
    if not rows:
        raise SystemExit(f'no batter of {a.league} with {a.min_pa} plate appearances')
    print(f'{len(rows)} batters of {a.league} with at least {a.min_pa} plate appearances '
          f'(batting average {st.mean(r["avg"] for r in rows):.3f} on average, {st.mean(r["pa"] for r in rows):.0f} PA)')
    labels = {'hitting_streak': 'longest hitting streak (games)', 'hitless_games': 'longest run of hitless games',
              'hitless_ab': 'longest run of hitless at-bats'}
    out = {'league': a.league, 'min_pa': a.min_pa, 'batters': len(rows), 'measures': {}}
    for m, label in labels.items():
        o = summary([r['observed'][m] for r in rows])
        s = shuffled_summary([r['shuffles'][m] for r in rows])
        out['measures'][m] = {'observed': o, 'shuffled': s}
        print(f'  {label:<34} mean {o["mean"]:5.1f}  median {o["median"]:5.1f}  90% {o["p90"]:5.1f}  max {o["max"]:5.1f}'
              f'   | games in random order: mean {s["mean"]:5.1f}, median {s["median"]:5.1f}, 90% {s["p90"]:5.1f}, max {s["max"]:5.1f}')
    for k in (15, 20, 25):
        n = sum(r['observed']['hitless_ab'] >= k for r in rows)
        sh = st.mean(x >= k for r in rows for x in r['shuffles']['hitless_ab'])
        print(f'  batters with a run of {k} or more hitless at-bats: {n} of {len(rows)} ({100 * n / len(rows):.0f} %; games in random order {100 * sh:.0f} %)')
        out['measures']['hitless_ab'][f'share_{k}'] = n / len(rows)
        out['measures']['hitless_ab'][f'share_{k}_shuffled'] = sh
    top = sorted(rows, key=lambda r: -r['observed']['hitless_ab'])[:5]
    print('  longest hitless runs: ' + '; '.join(f'{r["name"]} {r["observed"]["hitless_ab"]:.0f} AB (AVG {r["avg"]:.3f})' for r in top))
    top = sorted(rows, key=lambda r: -r['observed']['hitting_streak'])[:5]
    print('  longest hitting streaks: ' + '; '.join(f'{r["name"]} {r["observed"]["hitting_streak"]} games (AVG {r["avg"]:.3f})' for r in top))
    if a.json:
        out['rows'] = [{k: v for k, v in r.items() if k != 'shuffles'} for r in rows]
        json.dump(out, open(a.json, 'w'), indent=1)


if __name__ == '__main__':
    main()
