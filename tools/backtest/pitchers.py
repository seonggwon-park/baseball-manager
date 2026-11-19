"""Backtest: real major-league pitchers turned into attributes from their seasons, aged one year
with the game's aging curves, and their next-season rates predicted, against Marcel. The pitcher
version of tools/backtest/hitters.py.

Steps
1. Surrogate. In leagues of generated players (`bm players --json`, several seeds), every
   pitcher's season rates are regressed on what the engine's pitcher is made of (bm-world
   convert.rs `pitcher_with`): his pitching attributes (velocity, stuff, control, command,
   deception, stamina, concentration, composure, hold runners), his arsenal (the usage-weighted
   grade of the pitches other than the main fastball, whose grade is his stuff, and each pitch
   type's share of his pitches) and the style traits of his delivery and pitches (arm angle,
   extension, height, throwing hand, the four-seamer's and the sinker's spin, the other pitches'
   shapes, the release noise): a linear model of what the engine makes of them, with the spread
   a season of that size adds (sampling) and the spread the model leaves (misfit).
2. Attributes. For a real pitcher his seasons (MLB Stats API counts; Statcast custom leaderboard:
   plate discipline and batted balls against, pitch mix, four-seam velocity and induced vertical
   break, arm angle, throwing hand) are inverted through the surrogate with the generated
   pitchers' latents as the prior (Gaussian, so a closed-form posterior mean): a pitcher who
   faced few batters stays near the population. Batters faced per game stand for the role.
3. Aging. One year of the game's development table (data/people/aging.json): the drift of each
   attribute at his age plus, with AGING=full, the expected growth towards his ceiling. Pitch
   grades follow the stuff group (develop.rs); the mix and the style traits do not age.
4. Prediction. The surrogate's rates for the aged latents, per batter faced: strikeouts, walks,
   hit batters, home runs and the other hits (the components the latents are inferred from), and
   runs, earned runs and outs (predicted only: they repeat what the components say). The `+own`
   versions also carry the pitcher's own deviation from the surrogate in each component, shrunk
   by the misfit, and through the components' run values (fitted on the generated pitchers) into
   runs, earned runs and outs.
The engine's league is not the real one, so observables are moved by the difference between the
real season's mean (pitchers with 100+ BF, weighted by each observable's sample size) and the
generated pitchers' mean (weighted the same way) before the inversion, and the predicted rates
are moved back by the last season's difference: the latents describe a pitcher relative to his
league, as Marcel's regression to the league average does. The throwing hand and batters faced
per game are not moved (the roles are the same in both leagues; the pooled means differ by the
mix of starters and relievers). With MISFIT=real the misfit of each observable is raised by the
extra spread of the last season's real pitchers around the surrogate (an option tried on the
validation year and not chosen).

Marcel for pitchers (Tango, "The 2004 Marcels", comment #28: the batting process with seasons
weighted 3/2/1, non-pitchers' pitching removed from the league, innings for playing time, age
adjustment and rebaselining the same): rates per out from the last three seasons, the player's
league average (the league's rate each season, weighted by his weighted outs) added for 800 outs
(266.7 innings; the page does not give the amount: the reliability column of his 2004 pitcher
file, reliability = weighted innings / (weighted innings + X), gives X = 262-270 innings for
Halladay .84, Zito .84, Rivera .59, Gagne .68), age factor 1 + (29 - age) x 0.006 under 29 and
x 0.003 over 29 multiplying strikeouts and dividing the other events (the direction his 2004
file shows for league-average forecasts at different ages), then (step 6, which hitters.py leaves
out) every component rescaled so that the projected league, weighted by the projected innings
0.5 IP1 + 0.1 IP2 + 25 for a reliever .. 60 for a starter (by the share of games started),
comes out at the last season's league average. ERA: half Marcel's earned runs, half BaseRuns on
Marcel's components (his ERA column; BaseRuns' B multiplier set so that the last season's league
comes out right, and runs turned into earned runs at the league's ratio). Runs allowed per nine:
Marcel's runs.

Evaluation: pitchers with at least 100 BF in the target year and 50 in the year before; error
weighted by the target year's batters faced, for strikeouts, walks and home runs per batter
faced, BABIP against, runs allowed per nine innings and ERA.

usage: [AGING=full|drift|none] [MISFIT=engine|real] pitchers.py POP_DIR DATA_DIR AGING.json OUT.json [TARGET]
  (TARGET default 2026; 2025 is the validation year used to choose the aging mode, the misfit and
  the carry-over before looking at 2026: AGING=drift, MISFIT=engine, engine3+own)
  POP_DIR: p*.csv and p*.json from `bm players --clubs 60 --games 162 --seed S --json pS.json > pS.csv`
  DATA_DIR: pit_YYYY.json (MLB Stats API season pitching, TARGET-3 .. TARGET) and savant_pit_YYYY.csv
  (Statcast custom leaderboard, TARGET-3 .. TARGET-1; see download_pitchers.sh)"""
import csv
import glob
import json
import math
import os
import sys

import numpy as np

# What the engine's pitcher is made of (bm-world convert.rs `pitcher_with`): attributes, then the
# arsenal, then the style traits of the delivery and the pitches (player.pitching).
ATTRS = ['velocity', 'stuff', 'control', 'command', 'deception', 'stamina', 'concentration', 'composure', 'hold_runners']
KINDS = ['FF', 'SI', 'FC', 'SL', 'ST', 'CU', 'CH', 'FS']
# grade2: usage-weighted grade of the pitches other than the main fastball; mix_*: share of each
# type in his usage (four-seamers: the rest).
ARSENAL = ['grade2'] + ['mix_' + k.lower() for k in KINDS[1:]]
# The pitches' aim height is 0 for everyone (generate.rs AIM_HEIGHT) and a fastball's speed has
# no style of its own; the style draw of a pitch has no value (convert.rs `neutralise`), which
# makes a fastball's tilt a function of its spin and active spin, so only those two enter.
STYLE = ['arm_angle', 'extension', 'height', 'left', 'ff_spin', 'ff_active', 'si_spin', 'si_active',
         'sec_speed', 'sec_spin', 'sec_active', 'sec_tilt', 'speed_sd', 'spin_sd', 'tilt_sd']
LATENT = ATTRS + ARSENAL + STYLE
# Per-BF components (binomial in batters faced) the latents are inferred from; and the ones only
# predicted.
BF_RATES = ['so', 'bb', 'hbp', 'hr', 'hip']  # hip: hits other than home runs
RUN_RATES = ['runs', 'er', 'outs']
MIX_OBS = {'mix_' + k.lower(): ('one', 0.03) for k in KINDS[1:]}
# Observables used to infer the latents: name -> kind of sampling error.
OBS = {
    'so': 'pa', 'bb': 'pa', 'hbp': 'pa', 'hr': 'pa', 'hip': 'pa',
    'zone': 'pitches', 'z_swing': 'zone', 'o_swing': 'out', 'whiff': 'swings', 'z_contact': 'z_swings', 'o_contact': 'o_swings',
    'ev': ('bip', 14.0), 'la': ('bip', 27.0),
    'sweet': 'bip', 'barrel': 'bip', 'hard': 'bip', 'gb': 'bip', 'ld': 'bip', 'fb': 'bip', 'pu': 'bip', 'pull': 'bip',
    'ff_speed': ('ff_n', 1.0), 'ff_ivb': ('ff_n', 2.0), 'arm_angle': ('one', 2.0), 'left': ('one', 0.02),
    **MIX_OBS, 'bf_g': ('games', 3.0),
}
FIT = {**OBS, **{k: 'pa' for k in RUN_RATES}}
NO_SHIFT = {'left', 'bf_g'}
# Year-to-year change of the latents the aging table does not cover, from real pitchers with 100+
# BF in 2023 and 2024 (Statcast): pitch shares 0.058 (pooled over types), arm angle 3.25 degrees.
STYLE_YEARLY_SD = {**{k: 0.058 for k in MIX_OBS}, 'arm_angle': 3.25}
PITCHES_PER_BF, ZONE_SHARE = 3.89, 0.50


def f(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return math.nan


def latents(p):
    """A generated pitcher's latents (LATENT order) and the sum of his secondary pitches' squared
    usage weights (the share of a pitch's own yearly noise that reaches grade2)."""
    a, pit = p['attrs'], p['pitching']
    ps = pit['pitches']
    total = sum(s['usage'] for s in ps)
    main = None  # the fastball he throws most carries the stuff attribute (convert.rs main_fastball)
    for i, s in enumerate(ps):
        if s['kind'] in ('FF', 'SI') and (main is None or s['usage'] > ps[main]['usage']):
            main = i
    other = [(s['usage'] / total, s) for i, s in enumerate(ps) if i != main]
    w_other = sum(w for w, _ in other)
    share = {k: 0.0 for k in KINDS}
    for s in ps:
        share[s['kind']] += s['usage'] / total
    ff = next((s for s in ps if s['kind'] == 'FF'), None)
    si = next((s for s in ps if s['kind'] == 'SI'), None)
    lat = {k: a[k] for k in ATTRS}
    lat['grade2'] = sum(w * s['grade'] for w, s in other) / w_other if w_other > 0 else 0.0
    lat.update({'mix_' + k.lower(): share[k] for k in KINDS[1:]})
    lat.update({'arm_angle': math.degrees(pit['arm_angle']), 'extension': pit['extension'], 'height': p['height'],
                'left': 1.0 if p['throws'] == 'left' else 0.0,
                'ff_spin': ff['spin'] if ff else 0.0, 'ff_active': ff['active'] if ff else 0.0,
                'si_spin': si['spin'] if si else 0.0, 'si_active': si['active'] if si else 0.0,
                'sec_speed': sum(w * s['speed'] for w, s in other), 'sec_spin': sum(w * s['spin'] for w, s in other),
                'sec_active': sum(w * s['active'] for w, s in other), 'sec_tilt': sum(w * s['tilt'] for w, s in other),
                'speed_sd': pit['speed_sd'], 'spin_sd': pit['spin_sd'], 'tilt_sd': pit['tilt_sd']})
    sq = sum(w * w for w, _ in other) / w_other ** 2 if w_other > 0 else 1.0
    return [lat[k] for k in LATENT], sq


def population(pop_dir):
    """Generated pitchers: latents, observables, their sample sizes, and the mean share of a
    pitch's own yearly noise in grade2."""
    lat, obs, size, sq = [], [], [], []
    for path in sorted(glob.glob(os.path.join(pop_dir, 'p*.csv'))):
        players = {}
        for line in open(path[:-4] + '.json'):
            d = json.loads(line)
            players[(d['club'], d['slot'])] = d['player']
        for r in csv.DictReader(open(path)):
            if r['kind'] != 'P' or f(r['pa']) < 100:
                continue
            p = players[(int(r['club']), int(r['slot']))]
            a, s2 = latents(p)
            la = dict(zip(LATENT, a))
            bf, h, hr = f(r['pa']), f(r['h']), f(r['hr'])
            bip, swings, zone, pitches, ff_n = f(r['bip']), f(r['swings']), f(r['zone']), f(r['pitches']), f(r['ff_n'])
            o = {'so': f(r['so']) / bf, 'bb': f(r['bb']) / bf, 'hbp': f(r['hbp']) / bf, 'hr': hr / bf, 'hip': (h - hr) / bf,
                 'runs': f(r['runs']) / bf, 'er': f(r['earned']) / bf, 'outs': f(r['outs']) / bf,
                 'zone': zone / pitches, 'z_swing': f(r['z_swings']) / zone, 'o_swing': f(r['o_swings']) / (pitches - zone),
                 'whiff': f(r['whiffs']) / swings, 'z_contact': f(r['z_contact']) / max(f(r['z_swings']), 1),
                 'o_contact': f(r['o_contact']) / max(f(r['o_swings']), 1), 'ev': f(r['ev']) / bip, 'la': f(r['la']) / bip,
                 'sweet': f(r['sweet']) / bip, 'barrel': f(r['barrel']) / bip, 'hard': f(r['hard']) / bip, 'gb': f(r['gb']) / bip,
                 'ld': f(r['ld']) / bip, 'fb': f(r['fb']) / bip, 'pu': f(r['pu']) / bip, 'pull': f(r['pull']) / bip,
                 'ff_speed': f(r['ff_speed']) / ff_n if ff_n > 0 else math.nan, 'ff_ivb': f(r['ff_ivb']) / ff_n if ff_n > 0 else math.nan,
                 'arm_angle': f(r['arm_angle']), 'left': la['left'], 'bf_g': bf / f(r['games'])}
            o.update({k: la[k] for k in MIX_OBS})
            n = {'pa': bf, 'pitches': pitches, 'zone': zone, 'out': pitches - zone, 'swings': swings, 'z_swings': f(r['z_swings']),
                 'o_swings': f(r['o_swings']), 'bip': bip, 'ff_n': ff_n, 'games': f(r['games']), 'one': 1.0}
            lat.append(a)
            obs.append(o)
            size.append(n)
            sq.append(s2)
    return np.array(lat), obs, size, float(np.mean(sq))


def kind_of(name):
    kind = FIT[name]
    return kind[0] if isinstance(kind, tuple) else kind


def sampling_var(name, mean, n):
    """Sampling variance of an observable over a season of size `n`, at the population's mean
    rate `mean` (the observed value would make a pitcher who allowed no home run look certain)."""
    kind = FIT[name]
    if isinstance(kind, tuple):
        size, sd = kind
        return sd * sd / max(n[size], 1.0)
    return mean * (1 - mean) / max(n[kind], 1.0)


def fit_surrogate(lat, obs, size):
    """Linear model per observable: value = b + W . latent; misfit variance beyond sampling. Also
    the mean weighted by the observable's sample size, the reference of the league shift."""
    X = np.hstack([np.ones((len(lat), 1)), lat])
    model = {}
    for name in FIT:
        y = np.array([o[name] for o in obs])
        mean = float(np.nanmean(y))
        sv = np.array([sampling_var(name, mean, n) for n in size])
        ok = np.isfinite(y) & np.isfinite(sv)
        w = 1.0 / sv[ok]
        coef, *_ = np.linalg.lstsq(X[ok] * np.sqrt(w)[:, None], y[ok] * np.sqrt(w), rcond=None)
        resid = y[ok] - X[ok] @ coef
        misfit = max(float(np.mean(resid ** 2) - np.mean(sv[ok])), 0.0)
        r2 = 1 - np.var(resid) / np.var(y[ok])
        nk = np.array([n[kind_of(name)] for n in size])[ok]
        model[name] = {'b': float(coef[0]), 'w': coef[1:], 'misfit': misfit, 'r2': float(r2), 'mean': mean,
                       'wmean': float(np.sum(y[ok] * nk) / np.sum(nk))}
    # What each BF component is worth in runs, earned runs and outs per BF (the generated pitchers'
    # seasons, weighted by BF): carries a pitcher's own deviations into those.
    C = np.hstack([np.ones((len(obs), 1)), np.array([[o[k] for k in BF_RATES] for o in obs])])
    bf = np.sqrt([n['pa'] for n in size])
    for name in RUN_RATES:
        y = np.array([o[name] for o in obs])
        coef, *_ = np.linalg.lstsq(C * bf[:, None], y * bf, rcond=None)
        model[name]['value'] = dict(zip(BF_RATES, coef[1:]))
    return model


STAT_KEYS = ('battersFaced', 'outs', 'hits', 'doubles', 'triples', 'homeRuns', 'baseOnBalls', 'strikeOuts', 'hitByPitch', 'runs',
             'earnedRuns', 'sacBunts', 'sacFlies', 'catchersInterference', 'atBats', 'gamesPitched', 'gamesStarted', 'numberOfPitches')


def stats_api(path):
    """Season pitching lines by player id (traded players: the season total), and whether he is
    a pitcher (position players who pitched are left out of the league, Tango's step 2)."""
    out = {}
    for sp in json.load(open(path))['stats'][0]['splits']:
        pid, s = sp['player']['id'], sp['stat']
        line = {k: float(s.get(k, 0) or 0) for k in STAT_KEYS}
        line['age'] = float(s.get('age', math.nan) or math.nan)
        line['pitcher'] = sp.get('position', {}).get('abbreviation') in ('P', 'TWP')
        if pid in out:  # one split per club: add them up
            for k in STAT_KEYS:
                out[pid][k] += line[k]
        else:
            out[pid] = line
    return out


def components(line):
    """Per-BF components of a season line."""
    bf = max(line['battersFaced'], 1.0)
    return {'so': line['strikeOuts'] / bf, 'bb': line['baseOnBalls'] / bf, 'hbp': line['hitByPitch'] / bf,
            'hr': line['homeRuns'] / bf, 'hip': (line['hits'] - line['homeRuns']) / bf, 'runs': line['runs'] / bf,
            'er': line['earnedRuns'] / bf, 'outs': line['outs'] / bf, 'sh': line['sacBunts'] / bf,
            'ci': line['catchersInterference'] / bf}


def rates(c):
    """K, BB, HR per batter faced, BABIP against ((H - HR) / (AB - SO - HR + SF)), runs allowed and
    earned runs per nine innings, from per-BF components."""
    bip = max(1 - c['so'] - c['bb'] - c['hbp'] - c['hr'] - c['sh'] - c['ci'], 1e-6)
    outs = max(c['outs'], 1e-6)
    return {'k': c['so'], 'bb': c['bb'], 'hr': c['hr'], 'babip': c['hip'] / bip, 'ra9': 27 * c['runs'] / outs, 'era': 27 * c['er'] / outs}


AGING_MODE = os.environ.get('AGING', 'full')  # full: drift + expected growth; drift: drift only; none
# engine: the misfit the generated pitchers leave; real: raised by the extra spread of real pitchers.
MISFIT_MODE = os.environ.get('MISFIT', 'engine')


def group_map(table):
    """Aging group of each latent (pitch grades follow the stuff group; the arsenal's mix and the
    style traits are in none)."""
    group_of = {a: g for g, spec in table['groups'].items() for a in spec['attrs']}
    group_of['grade2'] = 'stuff'
    return group_of


def aging_delta(table, age):
    """Expected one-year change of each latent at `age` (mix and style traits: none)."""
    if AGING_MODE == 'none':
        return np.zeros(len(LATENT))
    from_age = table['from_age']

    def at(arr, a):
        i = int(math.floor(a)) - from_age
        return arr[min(max(i, 0), len(arr) - 1)] if arr else 0.0
    growth, remaining = table['growth'], table['remaining_growth']
    group_of = group_map(table)
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


# Marcel for pitchers: counts per out, newest season first.
MARCEL = ['so', 'bb', 'hbp', 'hr', 'hip', 'd2', 'd3', 'runs', 'er', 'sh', 'ci', 'bf']
MARCEL_BAD = ['bb', 'hbp', 'hr', 'hip', 'd2', 'd3', 'runs', 'er']
MARCEL_WEIGHTS = [3, 2, 1]
MARCEL_OUTS = 800.0


def counts(line):
    return {'so': line['strikeOuts'], 'bb': line['baseOnBalls'], 'hbp': line['hitByPitch'], 'hr': line['homeRuns'],
            'hip': line['hits'] - line['homeRuns'], 'd2': line['doubles'], 'd3': line['triples'], 'runs': line['runs'],
            'er': line['earnedRuns'], 'sh': line['sacBunts'], 'ci': line['catchersInterference'], 'bf': line['battersFaced']}


def per_out(total):
    return {k: v / max(total['outs'], 1.0) for k, v in counts(total).items()}


def marcel_raw(lines, age_last, league):
    """Per-out rates for the next season from up to three seasons (newest first), before the
    rebaselining: weighted 3/2/1, 800 outs of his league average added, aged."""
    wout = sum(w * l['outs'] for w, l in zip(MARCEL_WEIGHTS, lines) if l)
    if wout <= 0:
        return None
    proj = {}
    for k in MARCEL:
        count = sum(w * counts(l)[k] for w, l in zip(MARCEL_WEIGHTS, lines) if l)
        lg = sum(w * l['outs'] * lgy[k] for w, l, lgy in zip(MARCEL_WEIGHTS, lines, league) if l) / wout
        proj[k] = (count + MARCEL_OUTS * lg) / (wout + MARCEL_OUTS)
    age = age_last + 1
    factor = 1 + (29 - age) * (0.006 if age < 29 else 0.003)
    aged = dict(proj)
    aged['so'] = proj['so'] * factor
    for k in MARCEL_BAD:
        aged[k] = proj[k] / factor
    # Batters faced per out move with the times on base (the outs stay).
    aged['bf'] = proj['bf'] + sum(aged[k] - proj[k] for k in ('bb', 'hbp', 'hr', 'hip'))
    return aged


def base_runs(c, mult):
    """BaseRuns per out (Smyth: A B / (B + C) + D, C = outs) from per-out counts."""
    h = c['hip'] + c['hr']
    tb = c['hip'] + c['d2'] + 2 * c['d3'] + 4 * c['hr']
    a = c['hip'] + c['bb'] + c['hbp']
    b = mult * (1.4 * tb - 0.6 * h - 3 * c['hr'] + 0.1 * (c['bb'] + c['hbp']))
    return a * b / (b + 1.0) + c['hr']


def main():
    pop_dir, data_dir, aging_path, out_path = sys.argv[1:5]
    T = int(sys.argv[5]) if len(sys.argv) > 5 else 2026
    LAST = T - 1
    lat, obs, size, grade_sq = population(pop_dir)
    model = fit_surrogate(lat, obs, size)
    mu, cov = lat.mean(axis=0), np.cov(lat.T)
    prior_inv = np.linalg.inv(cov)
    print(f'Generated pitchers with 100+ BF: {len(lat)}')
    print('surrogate R^2: ' + ', '.join(f"{k} {m['r2']:.2f}" for k, m in model.items()))
    print('value per BF component: ' + '; '.join(f"{r}: " + ', '.join(f'{k} {v:+.3f}' for k, v in model[r]['value'].items()) for r in RUN_RATES))

    seasons = {y: stats_api(os.path.join(data_dir, f'pit_{y}.json')) for y in (T - 3, T - 2, T - 1, T)}
    league, league_out = {}, {}
    for y, players in seasons.items():
        tot = {k: 0.0 for k in STAT_KEYS}
        for l in players.values():
            if l['pitcher']:
                for k in tot:
                    tot[k] += l[k]
        league[y] = components(tot)
        league_out[y] = per_out(tot)
    savant = {}
    for y in (T - 3, T - 2, T - 1):
        savant[y] = {}
        path = os.path.join(data_dir, f'savant_pit_{y}.csv')
        if os.path.exists(path):
            with open(path, encoding='utf-8-sig') as fh:
                for r in csv.DictReader(fh):
                    savant[y][int(r['player_id'])] = r
    table = json.load(open(aging_path))
    group_of = group_map(table)
    # Yearly random change of each latent: the aging table's noise (grade2: the stuff group's noise,
    # drawn for each pitch, over his usage weights), the measured change of the mix and arm angle.
    yearly_var = np.array([table['groups'][group_of[k]]['noise'] ** 2 * (grade_sq if k == 'grade2' else 1.0) if k in group_of
                           else STYLE_YEARLY_SD.get(k, 0.0) ** 2 for k in LATENT])

    def observe(pid, year):
        """A real pitcher's observables in one season and their sample sizes, or None."""
        line = seasons[year].get(pid)
        if not line or line['battersFaced'] < 1:
            return None
        c = components(line)
        bf = line['battersFaced']
        pitches = line['numberOfPitches'] if line['numberOfPitches'] > 0 else PITCHES_PER_BF * bf
        sv = savant[year].get(pid)
        pct = (lambda k: f(sv.get(k)) / 100.0) if sv else (lambda k: math.nan)
        share = (lambda k: f(sv.get(k)) / 100.0 if sv.get(k) not in (None, '') else 0.0) if sv else (lambda k: math.nan)
        o = dict((k, c[k]) for k in BF_RATES + RUN_RATES)
        o.update({'zone': pct('in_zone_percent'), 'z_swing': pct('z_swing_percent'), 'o_swing': pct('oz_swing_percent'),
                  'whiff': pct('whiff_percent'), 'z_contact': pct('iz_contact_percent'), 'o_contact': pct('oz_contact_percent'),
                  'ev': f((sv or {}).get('exit_velocity_avg')), 'la': f((sv or {}).get('launch_angle_avg')),
                  'sweet': pct('sweet_spot_percent'), 'barrel': pct('barrel_batted_rate'), 'hard': pct('hard_hit_percent'),
                  'gb': pct('groundballs_percent'), 'ld': pct('linedrives_percent'), 'fb': pct('flyballs_percent'),
                  'pu': pct('popups_percent'), 'pull': pct('pull_percent'), 'ff_speed': f((sv or {}).get('ff_avg_speed')),
                  'ff_ivb': f((sv or {}).get('ff_avg_break_z_induced')), 'arm_angle': f((sv or {}).get('arm_angle')),
                  'left': {'L': 1.0, 'R': 0.0}.get((sv or {}).get('pitch_hand'), math.nan),
                  'mix_si': share('n_si_formatted'), 'mix_fc': share('n_fc_formatted'), 'mix_sl': share('n_sl_formatted'),
                  'mix_st': share('n_st_formatted'), 'mix_cu': share('n_cu_formatted') + share('n_sv_formatted'),  # slurves as curves
                  'mix_ch': share('n_ch_formatted'), 'mix_fs': share('n_fs_formatted'),
                  'bf_g': bf / max(line['gamesPitched'], 1.0)})
        zone = (o['zone'] if math.isfinite(o['zone']) else ZONE_SHARE) * pitches
        swing_share = pct('swing_percent') if math.isfinite(pct('swing_percent')) else 0.47
        n = {'pa': bf, 'pitches': pitches, 'zone': zone, 'out': pitches - zone, 'swings': swing_share * pitches,
             'z_swings': (o['z_swing'] if math.isfinite(o['z_swing']) else 0.67) * zone,
             'o_swings': (o['o_swing'] if math.isfinite(o['o_swing']) else 0.29) * (pitches - zone),
             'bip': max(line['atBats'] - line['strikeOuts'] + line['sacFlies'], 1.0),
             'ff_n': (share('n_ff_formatted') if sv else 0.0) * pitches, 'games': line['gamesPitched'], 'one': 1.0}
        return o, n

    # Real league means of the observables, each season (pitchers with 100+ BF, weighted by the
    # sample size of each observable), against the generated pitchers' means weighted the same way.
    shift = {}
    for y in (T - 3, T - 2, T - 1):
        shift[y] = {}
        got_all = [observe(pid, y) for pid, l in seasons[y].items() if l['battersFaced'] >= 100]
        for name in FIT:
            num = den = 0.0
            for got in got_all:
                if not got:
                    continue
                o, n = got
                val = o.get(name, math.nan)
                if math.isfinite(val):
                    num += val * n[kind_of(name)]
                    den += n[kind_of(name)]
            shift[y][name] = num / den - model[name]['wmean'] if den and name not in NO_SHIFT else 0.0
    print(f'real {LAST} minus engine means: ' + ', '.join(f'{k} {v:+.4f}' for k, v in shift[LAST].items()))

    def discrepancy(year, rounds=4):
        """Extra variance of each observable among real pitchers (100+ BF in `year`) beyond the
        surrogate's misfit and sampling. A pitcher's residual e from his own posterior has
        E[e^2] = r - w' S w if r is right (S: the posterior covariance), so the extra is raised by
        the mean of e^2 - r + w' S w until that is zero. An observable that pins its own latent
        (w' S w near r: velocity, arm angle, mix) says nothing and keeps its misfit."""
        data = []
        for pid, l in seasons[year].items():
            if l['battersFaced'] < 100 or not l['pitcher']:
                continue
            o, n = observe(pid, year)
            data.append([(name, model[name]['w'], o[name] - shift[year][name] - model[name]['b'],
                          model[name]['misfit'] + sampling_var(name, model[name]['mean'], n)) for name in OBS if math.isfinite(o.get(name, math.nan))])
        extra = {k: 0.0 for k in OBS}
        for _ in range(rounds):
            acc = {k: [] for k in OBS}
            for terms in data:
                A = prior_inv + sum(np.outer(w, w) / (r + extra[k]) for k, w, _, r in terms)
                S = np.linalg.inv(A)
                lat_ = S @ (prior_inv @ mu + sum(w * y / (r + extra[k]) for k, w, y, r in terms))
                for k, w, y, r in terms:
                    e = y - w @ lat_
                    acc[k].append(e * e - (r + extra[k]) + w @ S @ w)
            extra = {k: max(extra[k] + float(np.mean(v)), 0.0) if v else 0.0 for k, v in acc.items()}
        return extra

    if MISFIT_MODE == 'real':
        extra = discrepancy(LAST)
        for k, x in extra.items():
            model[k]['misfit'] += x
        print(f'misfit raised by the real {LAST} pitchers (sd): ' + ', '.join(f'{k} {math.sqrt(x):.4f}' for k, x in extra.items() if x > 0))

    def infer(pid, years, age_last):
        """Posterior mean of the latents in the last season before the target from the given
        seasons; also, per BF component, the evidence on the pitcher's own deviation from the
        surrogate (sum of e/s, sum of 1/s). `age_last`: his age in that last season."""
        A, v = prior_inv.copy(), prior_inv @ mu
        seen = []
        for y in years:
            got = observe(pid, y)
            if not got:
                continue
            o, n = got
            back = LAST - y
            # Aging from that season to the last one, at the ages he had then.
            delta = sum((aging_delta(table, age_last - back + t) for t in range(back)), np.zeros(len(LATENT)))
            for name in OBS:
                m = model[name]
                val = o.get(name, math.nan)
                if not math.isfinite(val):
                    continue
                r = m['misfit'] + sampling_var(name, m['mean'], n) + back * float(m['w'] ** 2 @ yearly_var)
                adj = val - shift[y][name] + m['w'] @ delta
                A += np.outer(m['w'], m['w']) / r
                v += m['w'] * (adj - m['b']) / r
                if name in BF_RATES:
                    seen.append((name, adj, r - m['misfit']))
        latent = np.linalg.solve(A, v)
        own = {}
        for name, adj, s_var in seen:
            e = adj - model[name]['b'] - model[name]['w'] @ latent
            a, b = own.get(name, (0.0, 0.0))
            own[name] = (a + e / s_var, b + 1.0 / s_var)
        return latent, own

    def predict(latent_last, own, age_last, carry, last):
        latent_next = latent_last + aging_delta(table, age_last)
        pred = {}
        for k in BF_RATES + RUN_RATES:
            m = model[k]
            pred[k] = m['b'] + m['w'] @ latent_next + shift[LAST][k]
        if carry:
            for k in BF_RATES:
                if k in own:
                    es, inv = own[k]
                    d = es / (1.0 / max(model[k]['misfit'], 1e-12) + inv)
                    pred[k] += d
                    for r in RUN_RATES:
                        pred[r] += model[r]['value'][k] * d
        pred['sh'], pred['ci'] = last['sh'], last['ci']
        return pred

    # Marcel for every pitcher with outs in the three seasons, then rebaselined (Tango's step 6);
    # BaseRuns' B multiplier from the last season's league.
    lg = league_out[LAST]
    a0 = lg['hip'] + lg['bb'] + lg['hbp']
    b0 = 1.4 * (lg['hip'] + lg['d2'] + 2 * lg['d3'] + 4 * lg['hr']) - 0.6 * (lg['hip'] + lg['hr']) - 3 * lg['hr'] + 0.1 * (lg['bb'] + lg['hbp'])
    bsr_mult = (lg['runs'] - lg['hr']) / (b0 * (a0 - lg['runs'] + lg['hr']))
    er_per_run = lg['er'] / lg['runs']

    def ip_of(line):
        return line['outs'] / 3 if line else 0.0

    def marcel_all(years):
        """Rebaselined per-out projections of every pitcher who pitched in `years` (newest first)."""
        proj, ip = {}, {}
        lgs = [league_out[y] for y in years]
        pids = {pid for y in years for pid, l in seasons[y].items() if l['pitcher'] and l['outs'] > 0}
        for pid in pids:
            lines = [seasons[y].get(pid) if seasons[y].get(pid, {}).get('outs', 0) > 0 else None for y in years]
            newest = next(i for i, l in enumerate(lines) if l)
            age_last = lines[newest]['age'] + newest
            if not math.isfinite(age_last):
                continue
            pr = marcel_raw(lines + [None] * (3 - len(lines)), age_last, lgs + [None] * (3 - len(lgs)))
            if pr is None:
                continue
            g = sum(l['gamesPitched'] for l in lines if l)
            gs = sum(l['gamesStarted'] for l in lines if l) / max(g, 1.0)
            proj[pid] = pr
            ip[pid] = 0.5 * ip_of(lines[0]) + 0.1 * (ip_of(lines[1]) if len(lines) > 1 else 0.0) + 25 + 35 * gs
        tot = sum(ip.values())
        factor = {k: league_out[LAST][k] / (sum(ip[p] * proj[p][k] for p in proj) / tot) for k in MARCEL}
        print(f'Marcel {len(years)} season(s), {len(proj)} pitchers; rebaselining factors: '
              + ', '.join(f'{k} {v:.3f}' for k, v in factor.items()))
        return {p: {k: v * factor[k] for k, v in c.items()} for p, c in proj.items()}

    def marcel_comp(c):
        """Per-BF components from a per-out Marcel projection; runs and earned runs as Tango's
        columns (runs: Marcel's; earned runs: half Marcel's, half BaseRuns at the league's ratio)."""
        if c is None:
            return None
        bf = c['bf']
        out = {k: c[k] / bf for k in ('so', 'bb', 'hbp', 'hr', 'hip', 'sh', 'ci', 'runs')}
        out['outs'] = 1.0 / bf
        out['er'] = 0.5 * (c['er'] + base_runs(c, bsr_mult) * er_per_run) / bf
        return out

    marcel3_all = marcel_all([T - 1, T - 2, T - 3])
    marcel1_all = marcel_all([T - 1])

    rows = []
    for pid, l_t in seasons[T].items():
        l_last = seasons[LAST].get(pid)
        if not l_last or l_t['battersFaced'] < 100 or l_last['battersFaced'] < 50:
            continue
        age_last = l_last['age']
        c_last = components(l_last)
        one, own1 = infer(pid, [LAST], age_last)
        three, own3 = infer(pid, [T - 3, T - 2, LAST], age_last)
        mar3, mar1 = marcel_comp(marcel3_all.get(pid)), marcel_comp(marcel1_all.get(pid))
        # For the statistical baseline (pitcher_regression_baseline.py): the last season's
        # observables relative to that season's league, the age and Marcel's components.
        o_last = observe(pid, LAST)[0]
        feat = [o_last[k] - shift[LAST][k] - model[k]['wmean'] if math.isfinite(o_last[k]) else 0.0 for k in OBS]
        feat += [1.0 if math.isfinite(o_last[k]) else 0.0 for k in OBS]
        feat += [age_last, math.log(l_last['battersFaced'])] + [mar3[k] for k in BF_RATES + RUN_RATES]
        rows.append({'pid': pid, 'bf': l_t['battersFaced'], 'age_last': age_last,
                     'starter': l_last['gamesStarted'] / max(l_last['gamesPitched'], 1.0), 'actual': rates(components(l_t)),
                     'actual_comp': components(l_t), 'feat': feat, 'last_sh_ci': [c_last['sh'], c_last['ci']], 'marcel3_comp': mar3,
                     'engine1': rates(predict(one, own1, age_last, False, c_last)),
                     'engine3': rates(predict(three, own3, age_last, False, c_last)),
                     'engine3+own': rates(predict(three, own3, age_last, True, c_last)),
                     'engine1+own': rates(predict(one, own1, age_last, True, c_last)),
                     'marcel1': rates(mar1), 'marcel3': rates(mar3),
                     'last': rates(c_last), 'league': rates(league[LAST])})

    methods = ['engine1', 'engine1+own', 'engine3', 'engine3+own', 'marcel1', 'marcel3', 'last', 'league']
    print(f'Target {T}: pitchers with 100+ BF in {T} and 50+ in {LAST}: {len(rows)}; aging mode {AGING_MODE}, misfit {MISFIT_MODE}')
    w = np.array([r['bf'] for r in rows])
    result = {}
    print(f'BF-weighted RMSE (correlation with the actual {T} value); last = the last season as it was')
    print(f"{'stat':6} " + ' '.join(f'{m:>16}' for m in methods))
    for stat in ('k', 'bb', 'hr', 'babip', 'ra9', 'era'):
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
    print('BF-weighted mean bias (prediction - actual)')
    for stat in ('k', 'bb', 'hr', 'babip', 'ra9', 'era'):
        print(f'{stat:6} ' + ' '.join(f"{result[stat][m]['bias']:>+16.4f}" for m in methods))
    json.dump(rows, open(out_path[:-5] + '_rows.json', 'w'))
    json.dump({'target': T, 'aging': AGING_MODE, 'misfit': MISFIT_MODE, 'n': len(rows), 'surrogate_r2': {k: m['r2'] for k, m in model.items()},
               'shift_last': shift[LAST], 'marcel_bsr_mult': bsr_mult, 'result': result}, open(out_path, 'w'), indent=1)


if __name__ == '__main__':
    main()
