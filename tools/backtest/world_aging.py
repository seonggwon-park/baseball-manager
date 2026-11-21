"""How major leaguers age in the game's world, against the published delta-method curve.

Input: `bm careers --years N --from 0 --seed S --out FILE [--attrs-out ATTRS]`. FILE has one row
per rostered player before each offseason (his age at 1 November, level, roster slot, value and,
for a hitter, his batting and running alone in runs per 162 games as a regular,
`ability::batting`); ATTRS has every major leaguer's attributes at the same moments.

A pair is a player in a major-league lineup, bench, rotation or bullpen before one offseason, seen
again before the next one; his change is what one offseason of development and aging did to his
true ability (there are no games, so no luck). Pairs are weighted by playing time (lineup 600
plate appearances, bench 200; starter 1, reliever 0.4) and grouped by his age in the season just
played (age at 1 November minus 0.34).

1. Hitters' batting, in runs per 500 PA (675 PA a season as a regular: 162 games x 37.5 / 9).
   - `all`: every such hitter still in the world a year later, whatever his level (the world
     knows his true ability even if he was sent down: no survivor bias except retirement).
   - `stayed`: only those back in a major-league lineup or on its bench (the survivors an
     uncorrected delta method sees).
   Reference: Lichtman (2009, part 2), survivor-corrected delta method, all hitters 1950-2008,
   linear-weights runs per 500 PA by age pair (docs/research/world/aging.md 1B).
2. Pitchers' value (runs saved over an average pitcher's share of a team's innings, 1/13 of
   them, about 111 innings), shown without a published counterpart in the same unit.
3. With --attrs and --table: the mean one-year change of each aging group of
   data/people/aging.json (the mean over its attributes), by age, for hitters and for pitchers:
   the aging of major leaguers as it happens in the world, which the backtests use with
   AGING=world (WORLD_AGING=TABLE). Ages with fewer than 50 pairs take the nearest age's value.

usage: world_aging.py OUT_FILE... [--attrs ATTRS_FILE... --table TABLE.json [--aging AGING.json]]"""
import argparse
import collections
import csv
import json
import math

LICHTMAN = {21: 14.2, 22: 5.3, 23: 6.1, 24: 2.8, 25: 2.1, 26: 1.1, 27: 0.0, 28: -1.9, 29: -1.3, 30: -1.8, 31: -2.3,
            32: -2.3, 33: -3.8, 34: -4.1, 35: -4.4, 36: -4.7, 37: -3.9, 38: -6.8, 39: -4.9}
PA_WEIGHT = {'lineup': 600.0, 'bench': 200.0}
PITCH_WEIGHT = {'starter': 1.0, 'reliever': 0.4}
PER_500 = 500.0 / 675.0


def load(path):
    with open(path) as f:
        return {(int(r['year']), int(r['id'])): r for r in csv.DictReader(f)}


def season_age(row):
    return int(math.floor(float(row['age']) - 0.34))


def curves(paths):
    """Weighted mean change of hitters' batting (all / stayed) and pitchers' value, by age."""
    hit = {'all': collections.defaultdict(lambda: [0.0, 0.0, 0]), 'stayed': collections.defaultdict(lambda: [0.0, 0.0, 0])}
    pit = collections.defaultdict(lambda: [0.0, 0.0, 0])
    for path in paths:
        rows = load(path)
        for (year, pid), r in rows.items():
            nxt = rows.get((year + 1, pid))
            if r['level'] != 'mlb' or nxt is None:
                continue
            age = season_age(r)
            if r['pitcher'] == '0' and r['slot'] in PA_WEIGHT:
                d = (float(nxt['batting']) - float(r['batting'])) * PER_500
                kinds = ['all'] + (['stayed'] if nxt['level'] == 'mlb' and nxt['slot'] in PA_WEIGHT else [])
                for k in kinds:
                    acc = hit[k][age]
                    acc[0] += PA_WEIGHT[r['slot']] * d
                    acc[1] += PA_WEIGHT[r['slot']]
                    acc[2] += 1
            elif r['pitcher'] == '1' and r['slot'] in PITCH_WEIGHT:
                acc = pit[age]
                acc[0] += PITCH_WEIGHT[r['slot']] * (float(nxt['value']) - float(r['value']))
                acc[1] += PITCH_WEIGHT[r['slot']]
                acc[2] += 1
    return hit, pit


def group_table(aging_path, paths):
    """Mean one-year change of each aging group's attributes among major leaguers, by age."""
    groups = json.load(open(aging_path))['groups']
    acc = {side: collections.defaultdict(lambda: collections.defaultdict(lambda: [0.0, 0.0, 0])) for side in ('hitters', 'pitchers')}
    for path in paths:
        rows = load(path)
        for (year, pid), r in rows.items():
            nxt = rows.get((year + 1, pid))
            if nxt is None:
                continue
            pitcher = r['pitcher'] == '1'
            w = (PITCH_WEIGHT if pitcher else PA_WEIGHT)[r['slot']]
            age = season_age(r)
            for g, spec in groups.items():
                keys = [k for k in spec['attrs'] if k in r]
                if not keys:
                    continue
                a = acc['pitchers' if pitcher else 'hitters'][g][age]
                a[0] += w * sum(float(nxt[k]) - float(r[k]) for k in keys) / len(keys)
                a[1] += w
                a[2] += 1
    out = {}
    for side, by_group in acc.items():
        out[side] = {}
        for g, by_age in by_group.items():
            good = {age: v[0] / v[1] for age, v in by_age.items() if v[2] >= 50}
            if good:
                lo, hi = min(good), max(good)
                out[side][g] = {str(age): good.get(age, good[min(good, key=lambda x: abs(x - age))]) for age in range(lo, hi + 1)}
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('out_files', nargs='+')
    ap.add_argument('--attrs', nargs='+', default=[])
    ap.add_argument('--table')
    ap.add_argument('--aging', default='data/people/aging.json')
    args = ap.parse_args()

    hit, pit = curves(args.out_files)
    print('Hitters in a major-league lineup or on its bench: change in batting, runs per 500 PA, by age in the season')
    print('age     all (pairs)      stayed (pairs)   Lichtman 2009 corrected')
    for age in range(20, 41):
        a, s = hit['all'].get(age), hit['stayed'].get(age)
        if not a or a[2] < 30:
            continue
        ds = f'{s[0] / s[1]:+6.2f} ({s[2]:5d})' if s and s[1] > 0 else '     -        '
        lt = LICHTMAN.get(age)
        print(f'{age}-{age + 1}  {a[0] / a[1]:+6.2f} ({a[2]:5d})   {ds}   ' + (f'{lt:+5.1f}' if lt is not None else '  -'))
    print('\nPitchers in a major-league rotation or bullpen: change in value, runs per 1/13 of the team innings, by age')
    for age in range(20, 42):
        p = pit.get(age)
        if p and p[2] >= 30:
            print(f'{age}-{age + 1}  {p[0] / p[1]:+6.2f} ({p[2]:5d})')
    if args.attrs and args.table:
        table = group_table(args.aging, args.attrs)
        json.dump(table, open(args.table, 'w'), indent=1)
        for side in ('hitters', 'pitchers'):
            print(f'\n{side}: mean one-year change of each aging group (SD) at ages 22 / 25 / 28 / 31 / 34')
            for g, by_age in table[side].items():
                print(f'  {g:12} ' + '  '.join(f"{by_age[str(a)]:+.3f}" if str(a) in by_age else '   -  ' for a in (22, 25, 28, 31, 34)))


if __name__ == '__main__':
    main()
