"""A statistical baseline for pitchers.py with the same information and no engine: ridge
regressions of next season's per-BF components on the last season's observables (relative to
the league; Statcast metrics, pitch mix and arm angle included), the age and Marcel's components,
fitted on the validation year (TRAIN rows, e.g. target 2025) and applied to the test year (TEST
rows, e.g. target 2026). Also paired bootstrap intervals (pitchers resampled) for the differences
in RMSE between the methods, and the mean biases.

usage: pitcher_regression_baseline.py TRAIN_rows.json TEST_rows.json [METHOD]   (METHOD default engine3+own)"""
import json
import math
import sys

import numpy as np

COMP = ['so', 'bb', 'hbp', 'hr', 'hip', 'runs', 'er', 'outs']
STATS = ('k', 'bb', 'hr', 'babip', 'ra9', 'era')
CV_STATS = ('k', 'bb', 'hr', 'babip', 'ra9')  # earned runs would count the runs twice


def rates(c):
    bip = max(1 - c['so'] - c['bb'] - c['hbp'] - c['hr'] - c['sh'] - c['ci'], 1e-6)
    outs = max(c['outs'], 1e-6)
    return {'k': c['so'], 'bb': c['bb'], 'hr': c['hr'], 'babip': c['hip'] / bip, 'ra9': 27 * c['runs'] / outs, 'era': 27 * c['er'] / outs}


def fit(rows, lam):
    X = np.array([r['feat'] for r in rows])
    mu, sd = X.mean(axis=0), X.std(axis=0) + 1e-9
    Z = np.hstack([np.ones((len(X), 1)), (X - mu) / sd])
    w = np.array([r['bf'] for r in rows])
    coefs = {}
    for c in COMP:
        y = np.array([r['actual_comp'][c] for r in rows])
        P = np.eye(Z.shape[1]) * lam
        P[0, 0] = 0.0
        coefs[c] = np.linalg.solve(Z.T @ (Z * w[:, None]) + P * w.mean(), Z.T @ (w * y))
    return mu, sd, coefs


def predict(model, r):
    mu, sd, coefs = model
    z = np.concatenate([[1.0], (np.array(r['feat']) - mu) / sd])
    c = {k: float(z @ coefs[k]) for k in COMP}
    c['sh'], c['ci'] = r['last_sh_ci']
    return rates(c)


def main():
    train, test = json.load(open(sys.argv[1])), json.load(open(sys.argv[2]))
    method = sys.argv[3] if len(sys.argv) > 3 else 'engine3+own'
    # The ridge penalty is chosen by 5-fold cross-validation on the training year only.
    rng = np.random.default_rng(1)
    folds = rng.integers(0, 5, len(train))
    var = {s: np.var([q['actual'][s] for q in train]) for s in CV_STATS}
    best = None
    for lam in (0.1, 1.0, 3.0, 10.0, 30.0, 100.0, 300.0, 1000.0, 3000.0, 10000.0):
        err = 0.0
        for k in range(5):
            m = fit([r for r, f in zip(train, folds) if f != k], lam)
            for r, f in zip(train, folds):
                if f == k:
                    p = predict(m, r)
                    err += r['bf'] * sum((p[s] - r['actual'][s]) ** 2 / var[s] for s in CV_STATS)
        if best is None or err < best[0]:
            best = (err, lam)
    lam = best[1]
    model = fit(train, lam)
    for r in test:
        r['regression'] = predict(model, r)
    methods = [method, 'regression', 'marcel3', 'marcel1']
    w = np.array([r['bf'] for r in test])
    print(f'ridge penalty {lam} (5-fold CV on the training year, {len(train)} pitchers); test pitchers {len(test)}')
    print('BF-weighted RMSE; 90% bootstrap interval of the difference to ' + method + ' (positive: ' + method + ' better)')
    print(f"{'stat':6} " + ' '.join(f'{m:>14}' for m in methods) + '   ' + '   '.join(f'{m} - {method}' for m in methods[1:]))
    boots = rng.integers(0, len(test), size=(2000, len(test)))
    out = {}
    for s in STATS:
        actual = np.array([r['actual'][s] for r in test])
        sq = {m: w * (np.array([r[m][s] for r in test]) - actual) ** 2 for m in methods}
        rmse = {m: math.sqrt(sq[m].sum() / w.sum()) for m in methods}
        diffs = []
        for m in methods[1:]:
            d = [math.sqrt(sq[m][b].sum() / w[b].sum()) - math.sqrt(sq[method][b].sum() / w[b].sum()) for b in boots]
            lo, hi = np.percentile(d, [5, 95])
            diffs.append(f'{rmse[m] - rmse[method]:+.4f} [{lo:+.4f}, {hi:+.4f}]')
        out[s] = {'rmse': rmse}
        print(f'{s:6} ' + ' '.join(f'{rmse[m]:>14.4f}' for m in methods) + '   ' + '   '.join(diffs))
    print('BF-weighted mean bias (prediction - actual); correlation with the actual value')
    for s in STATS:
        actual = np.array([r['actual'][s] for r in test])
        cells = []
        for m in methods:
            p = np.array([r[m][s] for r in test])
            cells.append(f'{np.sum(w * (p - actual)) / w.sum():+.4f} ({np.corrcoef(p, actual)[0, 1]:+.2f})')
        print(f'{s:6} ' + ' '.join(f'{c:>16}' for c in cells))
    return out


if __name__ == '__main__':
    main()
