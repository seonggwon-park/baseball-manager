"""Run expectancy by base-out state (RE24) and the chance of scoring at least once (R1),
measured from the Statcast sample (tools/statcast/download.sh: DIR/*.csv).

Every plate appearance is taken at its first pitch: outs, runners on first, second and third,
and the batting team's score. Runs to the end of the half-inning are the team's score after
the half-inning's last pitch minus its score at that first pitch. Innings 1-8 only, so that no
half-inning is cut short by a walk-off (the usual convention; extra innings also start with a
runner on second).

The same runs are also kept for two kinds of plate appearance: those with a bunt attempt
(a bunted foul or miss, or a ball in play the description calls a bunt) and intentional walks,
by the state they started in.

usage: re24_statcast.py DIR OUT.json"""
import csv
import glob
import json
import math
import os
import sys


def main():
    src, out = sys.argv[1], sys.argv[2]
    first = {}   # (game, at_bat) -> (inning, top, outs, bases, score, pitch_number)
    end = {}     # (game, inning, top) -> score after the half-inning's last pitch
    bunt, ibb = set(), set()  # (game, at_bat) with a bunt attempt / ending in an intentional walk
    for fn in sorted(glob.glob(os.path.join(src, '*.csv'))):
        with open(fn, encoding='utf-8-sig') as f:
            for row in csv.DictReader(f):
                try:
                    game, ab, pn = int(row['game_pk']), int(row['at_bat_number']), int(row['pitch_number'])
                    inning, outs = int(row['inning']), int(row['outs_when_up'])
                    score, post = int(row['bat_score']), int(row['post_bat_score'])
                except (KeyError, ValueError):
                    continue
                top = row['inning_topbot'] == 'Top'
                half = (game, inning, top)
                end[half] = max(end.get(half, 0), post)
                bases = sum(1 << i for i, k in enumerate(('on_1b', 'on_2b', 'on_3b')) if row.get(k, '') not in ('', 'NA', 'nan'))
                key = (game, ab)
                desc = row.get('description', '')
                if desc in ('foul_bunt', 'missed_bunt', 'bunt_foul_tip') or (desc == 'hit_into_play' and 'bunt' in row.get('des', '').lower()):
                    bunt.add(key)
                if row.get('events') == 'intent_walk':
                    ibb.add(key)
                if key not in first or pn < first[key][5]:
                    first[key] = (inning, top, outs, bases, score, pn)
    cells, bunt_cells, ibb_cells = {}, {}, {}
    for key, (inning, top, outs, bases, score, _) in first.items():
        if inning > 8 or outs > 2:
            continue
        runs = end[(key[0], inning, top)] - score
        for group, member in ((cells, True), (bunt_cells, key in bunt), (ibb_cells, key in ibb)):
            if not member:
                continue
            c = group.setdefault(f'{bases}-{outs}', [0, 0.0, 0.0, 0])
            c[0] += 1
            c[1] += runs
            c[2] += runs * runs
            c[3] += runs > 0

    def summary(group):
        table = {}
        for k, (n, s, s2, any_run) in sorted(group.items()):
            mean = s / n
            sd = math.sqrt(max(s2 / n - mean * mean, 0.0))
            table[k] = {'n': n, 're': mean, 'se': sd / math.sqrt(n), 'r1': any_run / n}
        return table

    table = summary(cells)
    json.dump({'source': 'Statcast sample, innings 1-8', 'games': len({g for g, _ in first}), 'plate_appearances': sum(c['n'] for c in table.values()), 'cells': table,
               'bunt_attempt': summary(bunt_cells), 'intentional_walk': summary(ibb_cells)}, open(out, 'w'), indent=1)
    print(f"{sum(c['n'] for c in table.values())} plate appearances")
    print('bases  outs:   0              1              2')
    for b in range(8):
        name = ''.join(x if b >> i & 1 else '_' for i, x in enumerate('123'))
        print(f'{name}   ' + '  '.join(f"{table[f'{b}-{o}']['re']:.3f}±{table[f'{b}-{o}']['se']:.3f}" for o in range(3)))


if __name__ == '__main__':
    main()
