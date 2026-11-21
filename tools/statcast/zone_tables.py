"""Targets for where pitches are and what becomes of them, from the 30-day 2025 Statcast sample
(download.sh): the count-by-result run-value table, swing outcomes and contact quality by attack
region of the plate, pitch types in and out of the zone, and pitch types by region. These are
the MLB columns of `bm validate`'s 'Attack regions' and 'Pitch types in and out of the zone'
sections and the targets of the swing fits (docs/physics-model.md 5.1).

Regions by the signed distance d of the ball's centre from the rulebook zone grown by the ball's
radius (in; negative inside), as the engine's `StrikeZone::signed_distance`, and the edge the
pitch is nearest to or furthest past (inside / outside / bottom / top, inside = toward the
batter):
  0 heart, deep      d < -6
  1 heart, outer     -6 <= d < -2.9
  2-5 shadow, in     -2.9 <= d < 0, by edge
  6-9 shadow, out    0 <= d < 2.9, by edge
  10-13 chase        2.9 <= d < 8.7, by edge
  14 waste           d >= 8.7
Strike groups: 0-1 strikes, 2 strikes. Bunts and automatic or intentional pitches are left out of
everything but the run-value table.

Result classes (as the engine's `tally::pitch_class`): ball, called strike, swinging strike, foul
tip, foul, hit by pitch, out in play, single, double, triple, home run, error. A pitch's run value
is the mean `delta_run_exp` of its result in its count (the table), so that the value of a result
does not depend on the base-out state; balls in play by region are valued with the 0-0 row, so
that it does not depend on the count either.

usage: python3 -I zone_tables.py STATCAST_DIR OUT_JSON"""
import csv
import glob
import json
import math
import os
import sys

BALL_R = 1.45 / 12
HALF = 8.5 / 12 + BALL_R
KINDS = ['FF', 'SI', 'FC', 'SL', 'ST', 'CU', 'CH', 'FS']
NAMES = ['heart deep', 'heart outer'] + ['shadow in ' + e for e in ('inside', 'outside', 'bottom', 'top')] + \
        ['shadow out ' + e for e in ('inside', 'outside', 'bottom', 'top')] + ['chase ' + e for e in ('inside', 'outside', 'bottom', 'top')] + ['waste']
BALL = {'ball', 'blocked_ball', 'pitchout', 'automatic_ball', 'intent_ball'}
CALLED = {'called_strike', 'automatic_strike'}
SWINGING = {'swinging_strike', 'swinging_strike_blocked'}
BUNT = {'foul_bunt', 'missed_bunt', 'bunt_foul_tip'}
NOT_PITCHED = {'pitchout', 'intent_ball', 'automatic_ball', 'automatic_strike'}


def cls(desc, ev):
    """Result class; bunted balls in play are told apart by the caller (from `des`)."""
    if desc in BALL:
        return 0
    if desc in CALLED:
        return 1
    if desc in SWINGING or desc == 'missed_bunt':
        return 2
    if desc in ('foul_tip', 'bunt_foul_tip'):
        return 3
    if desc in ('foul', 'foul_bunt', 'foul_pitchout'):
        return 4
    if desc == 'hit_by_pitch':
        return 5
    if desc.startswith('hit_into_play'):
        return {'single': 7, 'double': 8, 'triple': 9, 'home_run': 10, 'field_error': 11}.get(ev, 6)
    return None


def where(x, z, bot, top, stand):
    """(signed distance in inches, region) for plate_x, plate_z, sz_bot, sz_top in feet (plate_x from
    the catcher's view: negative is inside to a right-handed batter)."""
    inside_x = -x if stand == 'R' else x
    past = [inside_x - HALF, -inside_x - HALF, (bot - BALL_R) - z, z - (top + BALL_R)]
    dx = abs(x) - HALF
    dz = max(past[2], past[3])
    if dx <= 0 and dz <= 0:
        d = max(dx, dz)
    elif dx <= 0:
        d = dz
    elif dz <= 0:
        d = dx
    else:
        d = math.hypot(dx, dz)
    d *= 12
    j = max(range(4), key=lambda i: past[i])
    if d < -6:
        return d, 0
    if d < -2.9:
        return d, 1
    if d < 0:
        return d, 2 + j
    if d < 2.9:
        return d, 6 + j
    if d < 8.7:
        return d, 10 + j
    return d, 14


def plate_speed(r):
    """Pitch speed (mph) at the front of the plate, from the constant-acceleration fit (y0 = 50 ft)."""
    try:
        vx0, vy0, vz0, ax, ay, az = (float(r[k]) for k in ('vx0', 'vy0', 'vz0', 'ax', 'ay', 'az'))
    except ValueError:
        return None
    disc = vy0 * vy0 - 2 * ay * (50.0 - 17.0 / 12.0)
    if disc <= 0 or ay == 0:
        return None
    t = (-vy0 - math.sqrt(disc)) / ay
    return math.sqrt((vx0 + ax * t) ** 2 + (vy0 + ay * t) ** 2 + (vz0 + az * t) ** 2) * 3600 / 5280


def rows(directory):
    for fn in sorted(glob.glob(os.path.join(directory, '*.csv'))):
        with open(fn, encoding='utf-8-sig') as f:
            for r in csv.DictReader(f):
                if r.get('game_type', 'R') == 'R':
                    yield r


def mean_se(s, sq, n):
    m = s / max(n, 1)
    return m, math.sqrt(max(sq / max(n, 1) - m * m, 0.0) / max(n, 1))


def main():
    directory, out_path = sys.argv[1], sys.argv[2]
    # Pass 1: the count-by-result table, from every pitch with a count and a result.
    tab = {}
    for r in rows(directory):
        try:
            b, s, d = int(r['balls']), int(r['strikes']), float(r['delta_run_exp'])
        except ValueError:
            continue
        c = cls(r['description'], r['events'])
        if c is None or b > 3 or s > 2:
            continue
        t = tab.setdefault((b * 3 + s, c), [0.0, 0])
        t[0] += d
        t[1] += 1
    table = {k: v[0] / v[1] for k, v in tab.items()}
    w00 = [table.get((0, c), 0.0) for c in range(12)]

    # Pass 2: regions, pitch types in and out of the zone, pitch types by region.
    # region cell: pitches, swings, whiffs, fouls, balls in play, rv sum, rv sq (0-0 row, in play),
    #   ev n, ev sum, ev sq, la sum, la sq, bat n, bat sum, bat sq, efficiency n, sum, sq
    reg = [[[0.0] * 18 for _ in range(2)] for _ in NAMES]
    kz = {(k, z): [0, 0.0, 0.0, 0, 0] for k in KINDS for z in (0, 1)}  # pitches, rv sum, rv sq, swings, whiffs
    kr = {k: [[[0, 0, 0] for _ in range(2)] for _ in NAMES] for k in KINDS}  # pitches, swings, whiffs
    for r in rows(directory):
        desc = r['description']
        if desc in BUNT or desc in NOT_PITCHED:
            continue
        try:
            x, z, bot, top = float(r['plate_x']), float(r['plate_z']), float(r['sz_bot']), float(r['sz_top'])
            b, s = int(r['balls']), int(r['strikes'])
        except ValueError:
            continue
        c = cls(desc, r['events'])
        if c is None or b > 3 or s > 2 or (c >= 6 and 'bunt' in r.get('des', '').lower()):
            continue
        d, i = where(x, z, bot, top, r['stand'])
        g = int(s >= 2)
        swung = c in (2, 3, 4) or c >= 6
        cell = reg[i][g]
        cell[0] += 1
        bs = None
        if swung:
            cell[1] += 1
            try:
                bs = float(r['bat_speed'])
                cell[12] += 1
                cell[13] += bs
                cell[14] += bs * bs
            except ValueError:
                bs = None
        cell[2] += c in (2, 3)
        cell[3] += c == 4
        if c >= 6:
            cell[4] += 1
            cell[5] += w00[c]
            cell[6] += w00[c] * w00[c]
            try:
                ev, la = float(r['launch_speed']), float(r['launch_angle'])
            except ValueError:
                ev = None
            if ev is not None:
                cell[7] += 1
                cell[8] += ev
                cell[9] += ev * ev
                cell[10] += la
                cell[11] += la * la
                vp = plate_speed(r)
                if bs is not None and vp is not None:
                    q = ev / (1.23 * bs + 0.23 * vp)
                    cell[15] += 1
                    cell[16] += q
                    cell[17] += q * q
        k = r['pitch_type']
        if k in KINDS:
            v = table.get((b * 3 + s, c), 0.0)
            a = kz[(k, 0 if d <= 0 else 1)]
            a[0] += 1
            a[1] += v
            a[2] += v * v
            a[3] += swung
            a[4] += c in (2, 3)
            e = kr[k][i][g]
            e[0] += 1
            e[1] += swung
            e[2] += c in (2, 3)

    out = {'source': 'Statcast 2025 regular-season sample (30 dates)', 'table': {'%d,%d' % key: v for key, v in sorted(table.items())},
           'regions': [], 'kind_zone': {}, 'kind_region': {k: kr[k] for k in KINDS}, 'region_names': NAMES}
    print('== attack regions: swing %, whiff % and foul % of swings, bat speed (mph), exit velocity (mph), collision efficiency, launch angle')
    for i, name in enumerate(NAMES):
        pair = []
        for g in range(2):
            c = reg[i][g]
            ev, ev_se = mean_se(c[8], c[9], c[7])
            la, la_se = mean_se(c[10], c[11], c[7])
            bat, bat_se = mean_se(c[13], c[14], c[12])
            eff, eff_se = mean_se(c[16], c[17], c[15])
            rv, rv_se = mean_se(c[5], c[6], c[4])
            cellout = {'pitches': c[0], 'swings': c[1], 'bip': c[4], 'swing': 100 * c[1] / max(c[0], 1), 'whiff': 100 * c[2] / max(c[1], 1),
                       'foul': 100 * c[3] / max(c[1], 1), 'rv_bip': rv, 'rv_bip_se': rv_se, 'ev': ev, 'ev_se': ev_se, 'la': la, 'la_se': la_se,
                       'bat_n': c[12], 'bat': bat, 'bat_se': bat_se, 'eff_n': c[15], 'eff': eff, 'eff_se': eff_se}
            pair.append(cellout)
            print('  %-20s %s  n %6d  swing %5.1f  whiff %5.1f  foul %5.1f  bat %5.1f  EV %5.1f  eff %5.3f  LA %5.1f' % (
                name, '2 str' if g else '0-1  ', c[0], cellout['swing'], cellout['whiff'], cellout['foul'], bat, ev, eff, la))
        out['regions'].append(pair)
    print('== pitch types in / out of the zone: run value per 100 pitches (s.e.), whiffs per swing, swings per pitch')
    for (k, z), (n, sm, sq, sw, wh) in kz.items():
        m, se = mean_se(sm, sq, n)
        p = wh / max(sw, 1)
        key = '%s %s' % (k, 'in' if z == 0 else 'out')
        out['kind_zone'][key] = {'n': n, 'rv100': 100 * m, 'rv100_se': 100 * se, 'whiff': 100 * p, 'whiff_se': 100 * math.sqrt(p * (1 - p) / max(sw, 1)),
                                 'swing': 100 * sw / max(n, 1), 'swings': sw}
        print('  %-6s n %6d  rv/100 %+5.2f (%.2f)  whiff %4.1f  swing %4.1f' % (key, n, 100 * m, 100 * se, 100 * p, 100 * sw / max(n, 1)))
    with open(out_path, 'w') as f:
        json.dump(out, f, indent=1)


if __name__ == '__main__':
    main()
