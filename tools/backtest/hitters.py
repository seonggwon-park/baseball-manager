"""Backtest: real major-league hitters turned into attributes from their 2025 season, aged one
year with the game's aging curves, and their 2026 rates predicted, against Marcel.

Steps
1. Surrogate. In leagues of generated players (`bm players --json`, several seeds), every
   hitter's season rates are regressed on his hitting attributes (power, contact, timing,
   discipline, recognition, bat control, aggression, speed) and his swing's attack angle: a
   linear model of what the engine makes of the attributes (and of the swing's style traits),
   with the spread a season of that
   size adds (sampling) and the spread the model leaves (misfit).
2. Attributes. For a real hitter the 2025 rates (MLB Stats API counts; Statcast custom
   leaderboard: plate discipline, bat speed, exit velocity, launch angle, batted-ball types,
   sprint speed, attack angle) are inverted through the surrogate with the generated
   hitters' attributes as the prior (Gaussian, so a closed-form posterior mean): a hitter
   with few plate appearances stays near the population.
3. Aging. One year of the game's development table (data/people/aging.json): the drift of
   each attribute at his age plus the expected growth towards his ceiling (the table's mean
   remaining growth at that age times the share closed in a year).
4. Prediction. The surrogate's rates for the aged attributes.
The engine's league is not the real one (its BABIP is .281 against .291, for example), so
observables are moved by the difference between the real 2025 hitters' mean and the generated
hitters' mean before the inversion, and the predicted rates are moved back by the same
difference: the attributes describe a hitter relative to his league, as Marcel's regression
to the league average does.
Marcel (Tango): 2023-2025 weighted 5/4/3, 1200 plate appearances of league average added,
age factor 1 + (29 - age) x 0.006 under 29 and x 0.003 over 29 on every rate.

Evaluation: hitters with at least 200 PA in 2026 and 100 in 2025; error weighted by 2026
plate appearances, for strikeout, walk and home-run rates per PA, BABIP, ISO, OBP and SLG.

usage: [AGING=full|drift|none|world [WORLD_AGING=TABLE.json]] hitters.py POP_DIR DATA_DIR AGING.json OUT.json [TARGET]
  (TARGET default 2026; 2025 is the validation year used to choose the aging mode and the
  carry-over before looking at 2026; AGING=world: tools/backtest/world_aging.py --table)
  POP_DIR: p*.csv and p*.json from `bm players --clubs 60 --games 162 --seed S --json pS.json > pS.csv`
  DATA_DIR: hit_YYYY.json (MLB Stats API season hitting, TARGET-3 .. TARGET) and savant_bat_YYYY.csv
  (Statcast custom leaderboard, TARGET-3 .. TARGET-1)"""
import csv
import glob
import json
import math
import os
import sys

import numpy as np

# What the engine's batter is made of (bm-world convert.rs): attributes, then the swing's style
# traits (player.hit), then the hidden platoon trait.
ATTRS = ['power', 'contact', 'timing', 'discipline', 'recognition', 'bat_control', 'aggression', 'speed', 'strength',
         'anticipation', 'concentration', 'composure', 'acceleration']
STYLE = ['attack_angle', 'barrel_aim', 'timing_bias']  # bat_speed_sd and attack_sd are the same for everyone
LATENT = ATTRS + STYLE + ['platoon']
# Per-PA components (binomial in plate appearances).
PA_RATES = ['so', 'bb', 'hbp', 'single', 'double', 'triple', 'hr']
# Observables used to infer the attributes: name -> kind of sampling error.
OBS = {
    'so': 'pa', 'bb': 'pa', 'hbp': 'pa', 'single': 'pa', 'double': 'pa', 'triple': 'pa', 'hr': 'pa',
    'z_swing': 'zone', 'o_swing': 'out', 'whiff': 'swings', 'z_contact': 'z_swings', 'o_contact': 'o_swings',
    'bat_speed': ('swings', 7.0), 'ev': ('bip', 14.0), 'la': ('bip', 27.0),
    'sweet': 'bip', 'barrel': 'bip', 'hard': 'bip', 'gb': 'bip', 'ld': 'bip', 'fb': 'bip', 'pu': 'bip', 'pull': 'bip',
    'sprint': ('one', 0.4), 'attack_angle': ('swings', 9.0),
}
PITCHES_PER_PA, ZONE_SHARE = 3.88, 0.507


def f(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return math.nan


def population(pop_dir):
    """Generated hitters: latent attributes, observables and their sample sizes."""
    lat, obs, size = [], [], []
    for path in sorted(glob.glob(os.path.join(pop_dir, 'p*.csv'))):
        players = {}
        for line in open(path[:-4] + '.json'):
            d = json.loads(line)
            players[(d['club'], d['slot'])] = d['player']
        for r in csv.DictReader(open(path)):
            if r['kind'] != 'B' or f(r['pa']) < 100:
                continue
            p = players[(int(r['club']), int(r['slot']))]
            pa, ab, h, d2, d3, hr = (f(r[k]) for k in ('pa', 'ab', 'h', 'doubles', 'triples', 'hr'))
            bip, swings, zone, pitches = f(r['bip']), f(r['swings']), f(r['zone']), f(r['pitches'])
            o = {'so': f(r['so']) / pa, 'bb': f(r['bb']) / pa, 'hbp': f(r['hbp']) / pa, 'single': (h - d2 - d3 - hr) / pa,
                 'double': d2 / pa, 'triple': d3 / pa, 'hr': hr / pa,
                 'z_swing': f(r['z_swings']) / zone, 'o_swing': f(r['o_swings']) / (pitches - zone), 'whiff': f(r['whiffs']) / swings,
                 'z_contact': f(r['z_contact']) / max(f(r['z_swings']), 1), 'o_contact': f(r['o_contact']) / max(f(r['o_swings']), 1),
                 'bat_speed': f(r['bat_speed']) / max(f(r['bat_n']), 1), 'ev': f(r['ev']) / bip, 'la': f(r['la']) / bip,
                 'sweet': f(r['sweet']) / bip, 'barrel': f(r['barrel']) / bip, 'hard': f(r['hard']) / bip, 'gb': f(r['gb']) / bip,
                 'ld': f(r['ld']) / bip, 'fb': f(r['fb']) / bip, 'pu': f(r['pu']) / bip, 'pull': f(r['pull']) / bip,
                 'sprint': f(r['sprint']), 'attack_angle': f(r['attack_angle'])}
            n = {'pa': pa, 'zone': zone, 'out': pitches - zone, 'swings': swings, 'z_swings': f(r['z_swings']), 'o_swings': f(r['o_swings']),
                 'bip': bip, 'one': 1.0}
            a = [p['attrs'][k] for k in ATTRS] + [p['hit'][k] for k in STYLE] + [p['hidden']['platoon']]
            lat.append(a)
            obs.append(o)
            size.append(n)
    return np.array(lat), obs, size


def sampling_var(name, mean, n):
    """Sampling variance of an observable over a season of size `n`, at the population's mean
    rate `mean` (the observed value would make a hitter with no triples look certain)."""
    kind = OBS[name]
    if isinstance(kind, tuple):
        size, sd = kind
        return sd * sd / max(n[size], 1.0)
    return mean * (1 - mean) / max(n[kind], 1.0)


def fit_surrogate(lat, obs, size):
    """Linear model per observable: value = b + W . latent; misfit variance beyond sampling."""
    X = np.hstack([np.ones((len(lat), 1)), lat])
    model = {}
    for name in OBS:
        y = np.array([o[name] for o in obs])
        mean = float(np.nanmean(y))
        sv = np.array([sampling_var(name, mean, n) for n in size])
        ok = np.isfinite(y) & np.isfinite(sv)
        w = 1.0 / sv[ok]
        coef, *_ = np.linalg.lstsq(X[ok] * np.sqrt(w)[:, None], y[ok] * np.sqrt(w), rcond=None)
        resid = y[ok] - X[ok] @ coef
        misfit = max(float(np.mean(resid ** 2) - np.mean(sv[ok])), 0.0)
        r2 = 1 - np.var(resid) / np.var(y[ok])
        model[name] = {'b': float(coef[0]), 'w': coef[1:], 'misfit': misfit, 'r2': float(r2), 'mean': mean}
    return model


def stats_api(path):
    """Season hitting lines by player id (traded players: the season total)."""
    out = {}
    for sp in json.load(open(path))['stats'][0]['splits']:
        pid, s = sp['player']['id'], sp['stat']
        line = {k: float(s.get(k, 0) or 0) for k in ('plateAppearances', 'atBats', 'hits', 'doubles', 'triples', 'homeRuns', 'baseOnBalls',
                                                     'strikeOuts', 'hitByPitch', 'sacFlies', 'sacBunts', 'intentionalWalks')}
        line['age'] = float(s.get('age', math.nan) or math.nan)
        if pid in out:  # one split per club: add them up
            for k in line:
                if k != 'age':
                    out[pid][k] += line[k]
        else:
            out[pid] = line
    return out


def components(line):
    pa = max(line['plateAppearances'], 1.0)
    single = line['hits'] - line['doubles'] - line['triples'] - line['homeRuns']
    return {'so': line['strikeOuts'] / pa, 'bb': line['baseOnBalls'] / pa, 'hbp': line['hitByPitch'] / pa, 'single': single / pa,
            'double': line['doubles'] / pa, 'triple': line['triples'] / pa, 'hr': line['homeRuns'] / pa,
            'sf': line['sacFlies'] / pa, 'sh': line['sacBunts'] / pa}


def slash(c):
    """BABIP, ISO, OBP, SLG from per-PA components."""
    h = c['single'] + c['double'] + c['triple'] + c['hr']
    ab = max(1 - c['bb'] - c['hbp'] - c['sf'] - c['sh'], 1e-6)
    tb = c['single'] + 2 * c['double'] + 3 * c['triple'] + 4 * c['hr']
    babip = (h - c['hr']) / max(ab - c['so'] - c['hr'] + c['sf'], 1e-6)
    return {'k': c['so'], 'bb': c['bb'], 'hr': c['hr'], 'babip': babip, 'iso': (tb - h) / ab, 'obp': (h + c['bb'] + c['hbp']) / max(ab + c['bb'] + c['hbp'] + c['sf'], 1e-6), 'slg': tb / ab}


AGING_MODE = os.environ.get('AGING', 'full')  # full: drift + expected growth; drift: drift only; none; world
# AGING=world: the mean one-year change of each aging group among the world's major leaguers, by
# age (tools/backtest/world_aging.py --table), instead of the table's drift and growth.
WORLD = json.load(open(os.environ['WORLD_AGING']))['hitters'] if AGING_MODE == 'world' else None


def world_delta(group, age):
    by_age = WORLD.get(group)
    if not by_age:
        return 0.0
    ages = sorted(int(k) for k in by_age)
    return by_age[str(min(max(int(math.floor(age)), ages[0]), ages[-1]))]


def aging_delta(table, age):
    """Expected one-year change of each latent at `age` (style traits: none)."""
    if AGING_MODE == 'none':
        return np.zeros(len(LATENT))
    from_age = table['from_age']

    def at(arr, a):
        i = int(math.floor(a)) - from_age
        return arr[min(max(i, 0), len(arr) - 1)] if arr else 0.0
    growth, remaining = table['growth'], table['remaining_growth']
    group_of = {a: g for g, spec in table['groups'].items() for a in spec['attrs']}
    if AGING_MODE == 'world':
        return np.array([world_delta(group_of[name], age) if name in group_of else 0.0 for name in LATENT])
    out = []
    for name in LATENT:
        if name not in group_of:
            out.append(0.0)
            continue
        g = table['groups'][group_of[name]]
        drift = at(g['drift'], age)
        delay = g.get('growth_delay', 0.0)
        grow = at(growth, age - delay) * at(remaining, age) if AGING_MODE == 'full' else 0.0
        out.append(drift + grow)
    return np.array(out)


def marcel(lines, ages, league):
    """Per-PA components for the next season from up to three seasons (newest first)."""
    weights = [5, 4, 3]
    wpa = sum(w * l['plateAppearances'] for w, l in zip(weights, lines) if l)
    if wpa == 0:
        return None
    proj = {}
    for k in PA_RATES + ['sf', 'sh']:
        count = sum(w * components(l)[k] * l['plateAppearances'] for w, l in zip(weights, lines) if l)
        lg = sum(w * l['plateAppearances'] * lgy[k] for w, l, lgy in zip(weights, lines, league) if l) / wpa
        proj[k] = (count + 1200 * lg) / (wpa + 1200)
    age = ages + 1
    factor = 1 + (29 - age) * (0.006 if age < 29 else 0.003)
    return {k: v * factor for k, v in proj.items()}


def main():
    pop_dir, data_dir, aging_path, out_path = sys.argv[1:5]
    T = int(sys.argv[5]) if len(sys.argv) > 5 else 2026
    LAST = T - 1
    lat, obs, size = population(pop_dir)
    model = fit_surrogate(lat, obs, size)
    mu, cov = lat.mean(axis=0), np.cov(lat.T)
    prior_inv = np.linalg.inv(cov)
    print(f'Generated hitters with 100+ PA: {len(lat)}')
    print('surrogate R^2: ' + ', '.join(f"{k} {m['r2']:.2f}" for k, m in model.items()))

    seasons = {y: stats_api(os.path.join(data_dir, f'hit_{y}.json')) for y in (T - 3, T - 2, T - 1, T)}
    league = {}
    for y, players in seasons.items():
        tot = {k: 0.0 for k in ('plateAppearances', 'atBats', 'hits', 'doubles', 'triples', 'homeRuns', 'baseOnBalls', 'strikeOuts', 'hitByPitch', 'sacFlies', 'sacBunts')}
        for l in players.values():
            for k in tot:
                tot[k] += l[k]
        league[y] = components(tot)
    savant = {}
    for y in (T - 3, T - 2, T - 1):
        savant[y] = {}
        path = os.path.join(data_dir, f'savant_bat_{y}.csv')
        if os.path.exists(path):
            with open(path, encoding='utf-8-sig') as fh:
                for r in csv.DictReader(fh):
                    savant[y][int(r['player_id'])] = r
    table = json.load(open(aging_path))
    group_of = {a: g for g, spec in table['groups'].items() for a in spec['attrs']}
    # Yearly random change of each latent (the aging table's noise; style traits do not change).
    yearly_var = np.array([table['groups'][group_of[k]]['noise'] ** 2 if k in group_of else 0.0 for k in LATENT])

    def observe(pid, year):
        """A real hitter's observables in one season and their sample sizes, or None."""
        line = seasons[year].get(pid)
        if not line or line['plateAppearances'] < 1:
            return None
        c = components(line)
        pa = line['plateAppearances']
        bip = max(line['atBats'] - line['strikeOuts'] + line['sacFlies'], 1.0)
        pitches = PITCHES_PER_PA * pa
        zone = ZONE_SHARE * pitches
        sv = savant[year].get(pid, {})
        pct = lambda k: f(sv.get(k)) / 100.0
        o = dict((k, c[k]) for k in PA_RATES)
        o.update({'z_swing': pct('z_swing_percent'), 'o_swing': pct('oz_swing_percent'), 'whiff': pct('whiff_percent'),
                  'z_contact': pct('iz_contact_percent'), 'o_contact': pct('oz_contact_percent'), 'bat_speed': f(sv.get('avg_swing_speed')),
                  'ev': f(sv.get('exit_velocity_avg')), 'la': f(sv.get('launch_angle_avg')), 'sweet': pct('sweet_spot_percent'),
                  'barrel': pct('barrel_batted_rate'), 'hard': pct('hard_hit_percent'), 'gb': pct('groundballs_percent'),
                  'ld': pct('linedrives_percent'), 'fb': pct('flyballs_percent'), 'pu': pct('popups_percent'), 'pull': pct('pull_percent'),
                  'sprint': f(sv.get('sprint_speed')), 'attack_angle': f(sv.get('attack_angle'))})
        swing_share = pct('swing_percent') if math.isfinite(pct('swing_percent')) else 0.47
        n = {'pa': pa, 'zone': zone, 'out': pitches - zone, 'swings': swing_share * pitches,
             'z_swings': (o['z_swing'] if math.isfinite(o['z_swing']) else 0.67) * zone,
             'o_swings': (o['o_swing'] if math.isfinite(o['o_swing']) else 0.28) * (pitches - zone), 'bip': bip, 'one': 1.0}
        return o, n

    # Real league means of the observables, each season (hitters with 100+ PA, weighted by the
    # sample size of each observable), against the generated hitters' means.
    shift = {}
    for y in (T - 3, T - 2, T - 1):
        shift[y] = {}
        for name in OBS:
            num = den = 0.0
            for pid, l in seasons[y].items():
                if l['plateAppearances'] < 100:
                    continue
                got = observe(pid, y)
                if got and math.isfinite(got[0][name]):
                    kind = OBS[name][0] if isinstance(OBS[name], tuple) else OBS[name]
                    num += got[0][name] * got[1][kind]
                    den += got[1][kind]
            shift[y][name] = num / den - model[name]['mean'] if den else 0.0
    print(f'real {LAST} minus engine means: ' + ', '.join(f'{k} {v:+.4f}' for k, v in shift[LAST].items()))

    def infer(pid, years, age25):
        """Posterior mean of the latents in the last season before the target from the given
        seasons; also, per PA component, the evidence on the hitter's own deviation from the
        surrogate (sum of e/s, sum of 1/s). `age25`: his age in that last season."""
        A, v = prior_inv.copy(), prior_inv @ mu
        seen = []
        for y in years:
            got = observe(pid, y)
            if not got:
                continue
            o, n = got
            back = LAST - y
            # Aging from that season to 2025, at the ages he had then.
            delta = sum((aging_delta(table, age25 - back + t) for t in range(back)), np.zeros(len(LATENT)))
            for name, m in model.items():
                val = o.get(name, math.nan)
                if not math.isfinite(val):
                    continue
                r = m['misfit'] + sampling_var(name, m['mean'], n) + back * float(m['w'] ** 2 @ yearly_var)
                adj = val - shift[y][name] + m['w'] @ delta
                A += np.outer(m['w'], m['w']) / r
                v += m['w'] * (adj - m['b']) / r
                seen.append((name, adj, r - m['misfit']))
        latent = np.linalg.solve(A, v)
        own = {}
        for name, adj, s_var in seen:
            e = adj - model[name]['b'] - model[name]['w'] @ latent
            a, b = own.get(name, (0.0, 0.0))
            own[name] = (a + e / s_var, b + 1.0 / s_var)
        return latent, own

    def predict(latent25, own, age25, carry, last):
        latent26 = latent25 + aging_delta(table, age25)
        pred = {}
        for k in PA_RATES:
            m = model[k]
            pred[k] = m['b'] + m['w'] @ latent26 + shift[LAST][k]
            if carry and k in own:
                es, inv = own[k]
                pred[k] += es / (1.0 / max(m['misfit'], 1e-12) + inv)
        pred['sf'], pred['sh'] = last['sf'], last['sh']
        return pred

    rows = []
    for pid, l26 in seasons[T].items():
        l25 = seasons[LAST].get(pid)
        if not l25 or l26['plateAppearances'] < 200 or l25['plateAppearances'] < 100:
            continue
        age25 = l25['age']
        c25 = components(l25)
        one, own1 = infer(pid, [LAST], age25)
        three, own3 = infer(pid, [T - 3, T - 2, LAST], age25)
        lines = [l25, seasons[T - 2].get(pid), seasons[T - 3].get(pid)]
        lgs = [league[LAST], league[T - 2], league[T - 3]]
        # For the statistical baseline (regression_baseline.py): the last season's observables
        # relative to that season's league, the age, and Marcel's components.
        o_last = observe(pid, LAST)[0]
        mar3 = marcel(lines, age25, lgs)
        feat = [o_last[k] - shift[LAST][k] - model[k]['mean'] if math.isfinite(o_last[k]) else 0.0 for k in OBS]
        feat += [1.0 if math.isfinite(o_last[k]) else 0.0 for k in OBS]
        feat += [age25, math.log(l25['plateAppearances'])] + [mar3[k] for k in PA_RATES]
        rows.append({'pid': pid, 'pa26': l26['plateAppearances'], 'age25': age25, 'actual': slash(components(l26)),
                     'actual_comp': components(l26), 'feat': feat, 'last_sf_sh': [c25['sf'], c25['sh']], 'marcel3_comp': mar3,
                     'engine1': slash(predict(one, own1, age25, False, c25)),
                     'engine3': slash(predict(three, own3, age25, False, c25)),
                     'engine3+own': slash(predict(three, own3, age25, True, c25)),
                     'marcel1': slash(marcel([l25, None, None], age25, lgs)),
                     'marcel3': slash(marcel(lines, age25, lgs)),
                     'raw2025': slash(c25), 'league': slash(league[LAST])})

    methods = ['engine1', 'engine3', 'engine3+own', 'marcel1', 'marcel3', 'raw2025', 'league']
    print(f'Target {T}: hitters with 200+ PA in {T} and 100+ in {LAST}: {len(rows)}; aging mode {AGING_MODE}')
    w = np.array([r['pa26'] for r in rows])
    result = {}
    print(f'PA-weighted RMSE (correlation with the actual {T} value); raw2025 = the last season as it was')
    print(f"{'stat':6} " + ' '.join(f'{m:>16}' for m in methods))
    for stat in ('k', 'bb', 'hr', 'babip', 'iso', 'obp', 'slg'):
        actual = np.array([r['actual'][stat] for r in rows])
        cells = []
        result[stat] = {}
        for m in methods:
            p = np.array([r[m][stat] for r in rows])
            rmse = math.sqrt(float(np.sum(w * (p - actual) ** 2) / np.sum(w)))
            corr = float(np.corrcoef(p, actual)[0, 1]) if np.std(p) > 0 else float('nan')
            bias = float(np.sum(w * (p - actual)) / np.sum(w))
            result[stat][m] = {'rmse': rmse, 'corr': corr, 'bias': bias}
            cells.append(f'{rmse:.4f} ({corr:+.2f})')
        print(f'{stat:6} ' + ' '.join(f'{c:>16}' for c in cells))
    json.dump(rows, open(out_path[:-5] + '_rows.json', 'w'))
    json.dump({'target': T, 'aging': AGING_MODE, 'n': len(rows), 'surrogate_r2': {k: m['r2'] for k, m in model.items()}, 'shift_last': shift[LAST], 'result': result},
              open(out_path, 'w'), indent=1)


if __name__ == '__main__':
    main()
