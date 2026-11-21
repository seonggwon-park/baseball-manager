"""Situational rates behind the engine's runner-on-third rule and the AI manager's tactic
frequencies (docs/tactics.md 7), measured from the Statcast sample
(tools/statcast/download.sh: DIR/*.csv).

Every plate appearance is taken at its first pitch (outs, runners, score, inning) and its last
pitch (event, batted-ball type, fielder). "A run scored" means the batting team's score went up
during the plate appearance.

1. Plate appearances that start with a runner on third and fewer than two outs, innings 1-8:
   how they ended and how often a run scored.
2. Ground-ball outs in that situation (field outs, force outs, fielder's choices, double plays):
   how often a run scored, by fielder and by whether the runner on third was forced; with
   first base open split by the middle infielders against the rest; with a runner on first
   (and none on second) split into double plays and single outs.
3. Bunt attempts and intentional walks per plate appearance: by base state and outs in innings
   1-8, and by inning group (1-3, 4-6, 7-8, 9 and later; extra innings start with a runner on
   second, which is why the bunt rate with nobody out is so high from the ninth on).

usage: situations.py DIR"""
import collections
import csv
import glob
import os
import sys

GROUND_OUTS = {'force_out', 'grounded_into_double_play', 'fielders_choice', 'fielders_choice_out',
               'double_play', 'field_out'}
BUNT_PITCHES = {'foul_bunt', 'missed_bunt', 'bunt_foul_tip'}
FIELDERS = {'1': 'P', '2': 'C', '3': '1B', '4': '2B', '5': '3B', '6': 'SS'}


def occupied(row, key):
    return row.get(key, '') not in ('', 'NA', 'nan')


def inning_group(inning):
    return '1-3' if inning <= 3 else '4-6' if inning <= 6 else '7-8' if inning <= 8 else '9+'


def load(src):
    pas = {}  # (game, at_bat) -> [(pitch_number, first row), (pitch_number, last row)]
    bunt = set()
    for fn in sorted(glob.glob(os.path.join(src, '*.csv'))):
        with open(fn, encoding='utf-8-sig') as f:
            for r in csv.DictReader(f):
                key = (r['game_pk'], int(r['at_bat_number']))
                pn = int(r['pitch_number'])
                p = pas.setdefault(key, [None, None])
                if p[0] is None or pn < p[0][0]:
                    p[0] = (pn, r)
                if p[1] is None or pn > p[1][0]:
                    p[1] = (pn, r)
                d = r.get('description', '')
                if d in BUNT_PITCHES or (d == 'hit_into_play' and 'bunt' in r.get('des', '').lower()):
                    bunt.add(key)
    return {k: (v[0][1], v[1][1]) for k, v in pas.items()}, bunt


def rate(scored, n):
    return f'{scored / n:.2f} of {n}' if n else '-'


def runner_on_third(pas):
    print('1. Runner on third, fewer than two outs, innings 1-8: how the plate appearance ended')
    n = 0
    ends = collections.Counter()
    ends_scored = collections.Counter()
    ground = collections.Counter()
    ground_scored = collections.Counter()
    for first, last in pas.values():
        if int(first['inning']) > 8 or int(first['outs_when_up']) > 1 or not occupied(first, 'on_3b'):
            continue
        scored = int(last['post_bat_score']) > int(first['bat_score'])
        ev = last['events'] or 'none'
        if ev == 'field_out':
            ev = 'field_out, ' + (last.get('bb_type') or 'other')
        n += 1
        ends[ev] += 1
        ends_scored[ev] += scored
        if last['events'] not in GROUND_OUTS or last.get('bb_type') != 'ground_ball':
            continue
        r1, r2 = occupied(first, 'on_1b'), occupied(first, 'on_2b')
        fielder = FIELDERS.get((last.get('hit_location') or '?').split('.')[0], '?')
        outs = first['outs_when_up']
        double = last['events'] in ('grounded_into_double_play', 'double_play')
        keys = []
        if r1 and r2:
            keys += [('forced', fielder), ('forced', 'all')]
        else:
            keys += [('not forced', fielder), ('not forced', 'all')]
            if not r1:
                keys.append(('first open', 'middle' if fielder in ('2B', 'SS') else 'others'))
                keys.append(('first open', outs + ' out'))
            else:
                keys.append(('runner on first', outs + ' out, ' + ('double play' if double else 'one out')))
        for k in keys:
            ground[k] += 1
            ground_scored[k] += scored
    print(f'   {n} plate appearances; a run scored during {sum(ends_scored.values()) / n:.3f} of them')
    for ev, c in ends.most_common():
        print(f'   {ev:32} {c / n:.3f} of them, a run scored in {ends_scored[ev] / c:.2f}')
    print('2. Ground-ball outs in that situation: share during which a run scored')
    for group in ('not forced', 'forced', 'first open', 'runner on first'):
        keys = sorted(k for k in ground if k[0] == group)
        print(f'   {group}: ' + ', '.join(f'{k[1]} {rate(ground_scored[k], ground[k])}' for k in keys))


def tactic_rates(pas, bunt):
    print('3. Bunt attempts and intentional walks per plate appearance (by state: innings 1-8)')
    games = len({k[0] for k in pas})
    n = collections.Counter()
    hit = collections.Counter()
    for key, (first, last) in pas.items():
        inning, outs = int(first['inning']), int(first['outs_when_up'])
        r1, r2, r3 = (occupied(first, k) for k in ('on_1b', 'on_2b', 'on_3b'))
        group = inning_group(inning)
        state = ('1' if r1 else '_') + ('2' if r2 else '_') + ('3' if r3 else '_')
        early = inning <= 8
        cells = []
        if (r1 or r2) and not r3 and outs < 2:
            if early:
                cells += [('sacrifice situation', f'{state}, {outs} out')]
            if outs == 0:
                cells += [('sacrifice situation, nobody out', 'innings ' + group)]
        if early and not (r1 or r2 or r3):
            cells += [('bases empty', f'{outs} out')]
        elif early and outs == 2:
            cells += [('runners on, two outs', 'all')]
        if not r1 and (r2 or r3):
            cells += [('first open, runner in scoring position', 'innings ' + group)]
            if early:
                cells += [('first open, runner in scoring position', state)]
        if early and r1 and (r2 or r3) and outs == 2:
            cells += [('first taken, runner in scoring position, two outs', state)]
        for c in cells:
            n[c] += 1
            if c[0] in ('first open, runner in scoring position', 'first taken, runner in scoring position, two outs'):
                hit[c] += last.get('events') == 'intent_walk'
            else:
                hit[c] += key in bunt
    ibb = sum(1 for f, l in pas.values() if l.get('events') == 'intent_walk')
    print(f'   {games} games; intentional walks {ibb} ({ibb / games:.3f} per game)')
    for title in ('sacrifice situation', 'sacrifice situation, nobody out', 'bases empty', 'runners on, two outs'):
        keys = sorted(k for k in n if k[0] == title)
        print(f'   bunt attempts, {title}: ' + ', '.join(f'{k[1]} {hit[k] / n[k]:.4f} ({hit[k]} of {n[k]})' for k in keys))
    for title in ('first open, runner in scoring position', 'first taken, runner in scoring position, two outs'):
        keys = sorted(k for k in n if k[0] == title)
        print(f'   intentional walks, {title}: ' + ', '.join(f'{k[1]} {hit[k] / n[k]:.4f} ({hit[k]} of {n[k]})' for k in keys))


def main():
    pas, bunt = load(sys.argv[1])
    runner_on_third(pas)
    tactic_rates(pas, bunt)


if __name__ == '__main__':
    main()
