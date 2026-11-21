"""Compare the engine's tactics tables (`bm tactics --json`) with measured ones and work out what
follows for the manager's decisions.

Measured tables:
* Tango, RE24 and the chance of scoring by base-out state, 2010-2015 (Retrosheet, innings 1-8):
  http://www.tangotiger.net/re24.html (values copied below).
* The 2025 Statcast sample, innings 1-8 (tools/tactics/re24_statcast.py), with standard errors.

Decision values from each table:
* break-even success rate of a steal: (RE before - RE if caught) / (RE if safe - RE if caught);
* the change in run expectancy (RE) and in the chance of scoring (R1) of a successful sacrifice
  bunt and of an intentional walk.

The 2023 shift restriction: the engine played with and without it (`--shift-rule on|off`),
compared with the difference-in-differences estimate of Kennedy-Shaffer (2024), "Panel Data
Methods to Evaluate the Impact of Rule Changes" (CMSAC 2024), Table 1: BABIP with the bases
empty, 2023 against 2022, left-handed batters +0.012, right-handed +0.003, difference +0.009.

usage: compare.py ENGINE_ON.json REAL.json [ENGINE_OFF.json]"""
import json
import math
import sys

STATES = ['___', '1__', '_2_', '12_', '__3', '1_3', '_23', '123']
# Tango, 2010-2015, rows by STATES, columns 0, 1, 2 outs.
TANGO_RE = [[0.481, 0.254, 0.098], [0.859, 0.509, 0.224], [1.100, 0.664, 0.319], [1.437, 0.884, 0.429],
            [1.350, 0.950, 0.353], [1.784, 1.130, 0.478], [1.964, 1.376, 0.580], [2.292, 1.541, 0.752]]
TANGO_R1 = [[0.268, 0.155, 0.067], [0.416, 0.265, 0.127], [0.614, 0.397, 0.216], [0.610, 0.406, 0.222],
            [0.843, 0.660, 0.257], [0.860, 0.634, 0.270], [0.852, 0.676, 0.260], [0.861, 0.657, 0.316]]
SHIFT_DID = {'left': 0.012, 'right': 0.003, 'did': 0.009}


def grid(cells, key='re'):
    return [[cells[f'{b}-{o}'][key] for o in range(3)] for b in range(8)]


def re_at(t, b, o):
    return 0.0 if o >= 3 else t[b][o]


def be(t, kind, outs):
    """Break-even success rate of a steal: of second (runner on first only) or of third (runner on
    second only); a caught runner leaves the bases empty."""
    if kind == 'second':
        before, safe = re_at(t, 1, outs), re_at(t, 2, outs)
    else:
        before, safe = re_at(t, 2, outs), re_at(t, 4, outs)
    caught = re_at(t, 0, outs + 1)
    return (before - caught) / (safe - caught)


def main():
    eng = json.load(open(sys.argv[1]))
    real = json.load(open(sys.argv[2]))
    off = json.load(open(sys.argv[3])) if len(sys.argv) > 3 else None
    games_real = float(real['games'])
    e_re, e_r1 = grid(eng['cells']), grid(eng['cells'], 'r1')
    r_re, r_r1, r_se = grid(real['cells']), grid(real['cells'], 'r1'), grid(real['cells'], 'se')
    out = {}

    print(f"Engine: {eng['games']:.0f} games, {eng['runs_per_team_game']:.3f} runs per team-game. "
          f"Statcast 2025 sample: {real['plate_appearances']} plate appearances. Tango 2010-2015: Retrosheet.\n")
    print('### Run expectancy to the end of the inning (engine / Tango 2010-2015 / 2025 sample +- SE)\n')
    print('| bases | 0 outs | 1 out | 2 outs |')
    print('|---|---|---|---|')
    for b in range(8):
        cols = [f'{e_re[b][o]:.3f} / {TANGO_RE[b][o]:.3f} / {r_re[b][o]:.3f} ± {r_se[b][o]:.3f}' for o in range(3)]
        print(f'| {STATES[b]} | ' + ' | '.join(cols) + ' |')
    d_t = [e_re[b][o] - TANGO_RE[b][o] for b in range(8) for o in range(3)]
    z_r = [(e_re[b][o] - r_re[b][o]) / r_se[b][o] for b in range(8) for o in range(3)]
    rms_t = math.sqrt(sum(x * x for x in d_t) / 24)
    chi = sum(z * z for z in z_r)
    print(f'\nEngine - Tango: mean {sum(d_t) / 24:+.3f}, RMS {rms_t:.3f} runs; largest {max(d_t, key=abs):+.3f}. '
          f'Engine against the 2025 sample: sum of squared z = {chi:.1f} over 24 cells, {sum(abs(z) > 2 for z in z_r)} cells beyond 2 SE.')
    out['re'] = {'rms_vs_tango': rms_t, 'mean_vs_tango': sum(d_t) / 24, 'chi2_vs_2025': chi}

    print('\n### Chance of scoring at least once (engine / Tango 2010-2015 / 2025 sample)\n')
    print('| bases | 0 outs | 1 out | 2 outs |')
    print('|---|---|---|---|')
    for b in range(8):
        print(f'| {STATES[b]} | ' + ' | '.join(f'{e_r1[b][o]:.3f} / {TANGO_R1[b][o]:.3f} / {r_r1[b][o]:.3f}' for o in range(3)) + ' |')
    d1 = [e_r1[b][o] - TANGO_R1[b][o] for b in range(8) for o in range(3)]
    print(f'\nEngine - Tango: mean {sum(d1) / 24:+.3f}, RMS {math.sqrt(sum(x * x for x in d1) / 24):.3f}.')

    print('\n### Break-even success rate of a steal (engine / Tango 2010-2015 / 2025 sample)\n')
    print('| steal | 0 outs | 1 out | 2 outs |')
    print('|---|---|---|---|')
    out['break_even'] = {}
    for kind in ('second', 'third'):
        row = []
        for o in range(3):
            vals = [be(t, kind, o) for t in (e_re, TANGO_RE, r_re)]
            out['break_even'][f'{kind}-{o}'] = vals
            row.append(' / '.join(f'{100 * v:.1f}%' for v in vals))
        print(f'| {kind} | ' + ' | '.join(row) + ' |')
    for kind, key in (('second', 'steal_second'), ('third', 'steal_third')):
        s = eng[key]
        print(f"\nEngine steals of {kind}: {s['attempts']:.0f} attempts, {100 * s['safe'] / max(s['attempts'], 1):.1f}% safe.", end='')
    print()

    print('\n### A successful sacrifice bunt and an intentional walk: change in RE and in the chance of scoring (engine / Tango / 2025 sample)\n')
    print('| move | change in RE | change in chance of scoring |')
    print('|---|---|---|')
    moves = [('sacrifice, runner on first, 0 outs -> runner on second, 1 out', (1, 0), (2, 1)),
             ('sacrifice, runners on first and second, 0 outs -> second and third, 1 out', (3, 0), (6, 1)),
             ('intentional walk, runner on second, 1 out', (2, 1), (3, 1)),
             ('intentional walk, runners on second and third, 1 out', (6, 1), (7, 1)),
             ('intentional walk, runner on second, 2 outs', (2, 2), (3, 2)),
             ('intentional walk, runners on second and third, 2 outs', (6, 2), (7, 2))]
    out['moves'] = {}
    for name, (b0, o0), (b1, o1) in moves:
        d_re = [t[b1][o1] - t[b0][o0] for t in (e_re, TANGO_RE, r_re)]
        d_r1 = [t[b1][o1] - t[b0][o0] for t in (e_r1, TANGO_R1, r_r1)]
        out['moves'][name] = {'re': d_re, 'r1': d_r1}
        print(f'| {name} | ' + ' / '.join(f'{x:+.3f}' for x in d_re) + ' | ' + ' / '.join(f'{x:+.3f}' for x in d_r1) + ' |')

    print('\n### What bunt attempts returned: runs to the end of the inning after a plate appearance with a bunt attempt, and the state\'s RE\n')
    print('| state | engine: attempts per game, runs after, RE | 2025 sample: attempts per game, runs after ± SE, RE |')
    print('|---|---|---|')
    for k in ('0-0', '0-1', '1-0', '2-0', '3-0'):
        b, o = int(k[0]), int(k[2])
        es, rs = eng['sacrifice_bunt'].get(k), real['bunt_attempt'].get(k)
        ecol = f"{es['n'] / eng['games']:.3f}, {es['re']:.3f}, {e_re[b][o]:.3f}" if es else '-'
        rcol = f"{rs['n'] / games_real:.3f}, {rs['re']:.3f} ± {rs['se']:.3f}, {r_re[b][o]:.3f}" if rs else '-'
        print(f'| {STATES[b]}, {o} out{"s" if o != 1 else ""} | {ecol} | {rcol} |')
    tot_e = sum(v['n'] for v in eng['sacrifice_bunt'].values()) / eng['games']
    tot_r = sum(v['n'] for v in real['bunt_attempt'].values()) / games_real
    one_out_e = sum(v['n'] for k, v in eng['sacrifice_bunt'].items() if k.endswith('-1') and k != '0-1') / eng['games']
    one_out_r = sum(v['n'] for k, v in real['bunt_attempt'].items() if k.endswith('-1') and k != '0-1') / games_real
    two_out_e = sum(v['n'] for k, v in eng['sacrifice_bunt'].items() if k.endswith('-2')) / eng['games']
    two_out_r = sum(v['n'] for k, v in real['bunt_attempt'].items() if k.endswith('-2')) / games_real
    print(f'\nAll bunt-attempt plate appearances per game (innings 1-8): engine {tot_e:.3f}, 2025 sample {tot_r:.3f}. '
          f'With runners on and one out: {one_out_e:.3f} / {one_out_r:.3f}. With two outs: {two_out_e:.3f} / {two_out_r:.3f}.')
    ibb_e = sum(v['n'] for v in eng['intentional_walk'].values()) / eng['games']
    ibb_r = sum(v['n'] for v in real['intentional_walk'].values()) / games_real
    print(f'Intentional walks per game, innings 1-8: engine {ibb_e:.3f}, 2025 sample {ibb_r:.3f}.')
    out['frequency'] = {'bunt_attempts': [tot_e, tot_r], 'bunt_one_out_runners_on': [one_out_e, one_out_r], 'bunt_two_outs': [two_out_e, two_out_r], 'ibb_1_8': [ibb_e, ibb_r]}

    if off:
        print('\n### The 2023 shift restriction: BABIP with the bases empty, by batter hand\n')
        print('| batters | shifts allowed (2022 AI) | restricted (2025 AI) | change | measured change 2022 -> 2023 |')
        print('|---|---|---|---|---|')
        ch = {}
        for h in ('left', 'right'):
            a, b = off['babip'][h]['bases_empty'], eng['babip'][h]['bases_empty']
            se = math.sqrt(a['babip'] * (1 - a['babip']) / a['balls_in_play'] + b['babip'] * (1 - b['babip']) / b['balls_in_play'])
            ch[h] = (b['babip'] - a['babip'], se)
            print(f"| {h}-handed (full shift {100 * off['babip'][h]['all']['full_shift']:.1f}% of PA) | {a['babip']:.4f} | {b['babip']:.4f} | "
                  f"{ch[h][0]:+.4f} ± {se:.4f} | {SHIFT_DID[h]:+.3f} |")
        did = ch['left'][0] - ch['right'][0]
        did_se = math.hypot(ch['left'][1], ch['right'][1])
        print(f"\nDifference in differences: engine {did:+.4f} ± {did_se:.4f}, Kennedy-Shaffer (2024) {SHIFT_DID['did']:+.3f}.")
        out['shift'] = {'left': ch['left'], 'right': ch['right'], 'did': [did, did_se]}
    return out


if __name__ == '__main__':
    main()
