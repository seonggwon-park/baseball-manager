"""Fit the pitching value model (data/people/pitch_value.json; decision 2026-11-18: a linear
model of the arsenal) to the engine's results.

usage: fit_pitching.py OUT.json DIR SEED [SEED ...]

DIR holds, for each seed, players_SEED.csv and players_SEED.jsonl from
`bm players --clubs 60 --games 400 --seed SEED --json players_SEED.jsonl > players_SEED.csv`.
The target is each starter's runs allowed per nine innings (50 or more starts over the 400
games); the features are his attributes, his delivery and, pitch type by pitch type, the share
of his pitches it makes up and that share times its grade and its own shape (the deviations
from the league's average pitch of the type), the same as `ability::pitching_rate` computes.
Ridge regression on standardised features, its strength chosen by five-fold cross-validation;
the weights are written in each feature's own units with the mean it is measured from.

`per_sd`, the pitchers' unit of `ability::per_sd`, keeps the earlier model's unit (0.551 runs a
game for a staff: one standard deviation on every pitching attribute and pitch grade) in
proportion to the spread: it is widened by the new model's spread of predictions among these
starters over the earlier model's (OLD_SD, its spread of 0.197 runs a game on the same kind of
league), so that a pitcher's value in standard deviations means about what it did.
"""
import csv, json, sys
import numpy as np

KINDS = ['FF', 'SI', 'FC', 'SL', 'ST', 'CU', 'CH', 'FS']
OLD_UNIT, OLD_SD = 0.551, 0.197
ATTRS = ['velocity', 'stuff', 'control', 'command', 'deception', 'hold_runners', 'concentration', 'composure', 'stamina']
DELIVERY = ['arm_angle', 'extension', 'speed_sd', 'spin_sd', 'tilt_sd']
FIELDS = ['grade', 'speed', 'spin', 'active', 'tilt', 'height']

def features(p):
    a, pit = p['attrs'], p['pitching']
    out = {k: a[k] for k in ATTRS}
    out['left'] = 1.0 if p['throws'] == 'left' else 0.0
    for k in DELIVERY:
        out[k] = pit[k]
    ps = pit['pitches']
    total = sum(x['usage'] for x in ps)
    out['n_pitches'] = len(ps)
    for kind in KINDS:
        q = [x for x in ps if x['kind'] == kind]
        u = sum(x['usage'] for x in q) / total if q and total > 0 else 0.0
        out['use_' + kind] = u
        for f in FIELDS:
            out[kind + '_' + f] = u * q[0][f] if q else 0.0
    return out

def load(d, seed):
    players = {}
    for line in open(f'{d}/players_{seed}.jsonl'):
        x = json.loads(line)
        players[(x['club'], x['slot'])] = x['player']
    rows = []
    for r in csv.DictReader(open(f'{d}/players_{seed}.csv')):
        if r['kind'] == 'P' and float(r['starts']) >= 50:
            rows.append((features(players[(int(r['club']), int(r['slot']))]), float(r['runs']) / float(r['outs']) * 27))
    return rows

def ridge_cv(X, y, alphas, folds=5, seed=0):
    rng = np.random.default_rng(seed)
    idx = rng.permutation(len(y))
    parts = np.array_split(idx, folds)
    best = None
    for a in alphas:
        pred = np.zeros(len(y))
        for k in range(folds):
            test = parts[k]
            train = np.concatenate([parts[j] for j in range(folds) if j != k])
            Xt, yt = X[train], y[train]
            beta = np.linalg.solve(Xt.T @ Xt + a * np.eye(X.shape[1]), Xt.T @ (yt - yt.mean()))
            pred[test] = yt.mean() + X[test] @ beta
        r2 = 1 - ((y - pred) ** 2).sum() / ((y - y.mean()) ** 2).sum()
        if best is None or r2 > best[1]:
            best = (a, r2)
    return best

def main():
    out, d, seeds = sys.argv[1], sys.argv[2], sys.argv[3:]
    rows = [r for s in seeds for r in load(d, s)]
    names = list(rows[0][0].keys())
    X = np.array([[f[n] for n in names] for f, _ in rows])
    y = np.array([t for _, t in rows])
    mean, sd = X.mean(0), X.std(0)
    sd[sd == 0] = 1.0
    Z = (X - mean) / sd
    alpha, r2 = ridge_cv(Z, y, [1, 3, 10, 30, 100, 300, 1000])
    beta = np.linalg.solve(Z.T @ Z + alpha * np.eye(Z.shape[1]), Z.T @ (y - y.mean()))
    weights = beta / sd
    pred = (X - mean) @ weights
    doc = {
        '_comment': 'Pitching value model (crates/bm-world/src/ability.rs, pitching_rate): runs allowed per nine innings = sum over features of weight x (value - mean). Fitted by tools/value/fit_pitching.py to the engine\'s starters; do not edit by hand.',
        'fit': {'starters': len(y), 'seeds': seeds, 'ridge_alpha': alpha, 'cross_validated_r2': round(r2, 4), 'ra9_sd': round(float(y.std()), 4), 'prediction_sd': round(float(pred.std()), 4)},
        'per_sd': round(OLD_UNIT * float(pred.std()) / OLD_SD, 4),
        'features': {n: [round(float(m), 6), round(float(w), 6)] for n, m, w in zip(names, mean, weights)},
    }
    json.dump(doc, open(out, 'w'), indent=1)
    print(f'{len(y)} starters, ridge alpha {alpha}, cross-validated R2 {r2:.3f}, RA9 sd {y.std():.3f}, prediction sd {pred.std():.3f}')

if __name__ == '__main__':
    main()
