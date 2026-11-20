"""Tabular Q-learning on the bm_gym environment, and a paired comparison with two baselines.

The state is coarse on purpose: outs, base state, count, whether the game is late and close
(7th inning on, lead of at most one run either way) and the hitter (weak, average, strong by
the observation's ``batter_quality``). The reward is the change of win expectancy, so the
returns of an episode add up to the result minus the starting win expectancy.

Three learners (``--method``):

* ``mc`` (default): Monte Carlo control. The value of an action is the mean return that
  followed it; the policy leaves swinging away only where another action is better by more
  than two standard errors. No bootstrapping, so no maximisation bias.
* ``plain``: one-table Q-learning with a constant step size. Here it learns to take pitch
  after pitch: the max over noisy estimates is biased upwards, and the bias piles up along
  actions that make the plate appearance longer (Sutton & Barto 2018, section 6.7).
* ``double``: double Q-learning (Hasselt 2010) with step sizes 1/n. The early targets, made
  when the next state's values were still zero, never wash out, so rarely tried actions keep
  optimistic values and the greedy policy picks them.

    python examples/q_learning.py --train 40000 --eval 20000 --workers 4

Baselines, played from the same seeds (same clubs, park and side): ``hit``, always swing away
with the runners going on their own; ``ai``, the engine's AI manager runs the offence.
"""

from __future__ import annotations

import argparse
import json
import multiprocessing as mp
import time

import numpy as np

import bm_gym

N_A = bm_gym.N_ACTIONS
OBS = {n: i for i, n in enumerate(bm_gym.OBSERVATION_NAMES)}


def state_of(obs: np.ndarray, info: dict) -> int:
    outs = min(info["outs"], 2)
    bases = info["bases"]
    count = info["balls"] * 3 + info["strikes"]  # 0..11
    agent, opp = info["score"]
    late_close = int(info["inning"] >= 7 and abs(agent - opp) <= 1)
    q = obs[OBS["batter_quality"]]
    hitter = 0 if q < -0.4 else (2 if q > 0.4 else 1)
    return ((((outs * 8 + bases) * 12 + count) * 2 + late_close) * 3) + hitter


N_S = 3 * 8 * 12 * 2 * 3


def describe(s: int) -> str:
    hitter = s % 3
    s //= 3
    late = s % 2
    s //= 2
    count = s % 12
    s //= 12
    bases = s % 8
    outs = s // 8
    b = "".join(c for i, c in enumerate("123") if bases & (1 << i)) or "-"
    return f"outs {outs} bases {b:>3} count {count // 3}-{count % 3} {'late&close' if late else 'other':>10} hitter {['weak', 'avg', 'strong'][hitter]}"


def sensible(s: int) -> np.ndarray:
    """The action mask of a state of the table (it depends only on outs and bases)."""
    s //= 3 * 2 * 12
    bases, outs = s % 8, s // 8
    r1, r2, r3 = bases & 1, bases & 2, bases & 4
    runner = (r1 and not r2) or (r2 and not r3)
    sac = (r1 or r2) and outs < 2
    m = np.zeros(N_A, dtype=bool)
    for a in range(N_A):
        approach, steal = divmod(a, 3)
        steal_ok = steal == 0 or runner
        ok = {1: sac, 3: runner and steal == 0}.get(approach, True)
        m[a] = steal_ok and ok
    return m


def train_mc(games: int, seed: int, eps: float = 0.15):
    """Monte Carlo control with a cautious policy: leave swinging away only on evidence."""
    env = bm_gym.ManagerEnv(seed=seed)
    n = np.zeros((N_S, N_A))
    total = np.zeros((N_S, N_A))
    total2 = np.zeros((N_S, N_A))
    visits = np.zeros(N_S, dtype=np.int64)
    policy = np.zeros(N_S, dtype=np.int64)
    rng = np.random.default_rng(seed)
    t0 = time.perf_counter()
    for g in range(games):
        obs, info = env.reset(seed=seed * 1_000_003 + g)
        traj, rewards = [], []
        while True:
            s = state_of(np.asarray(obs, dtype=np.float32), info)
            valid = np.flatnonzero(env.action_mask())
            a = int(rng.choice(valid)) if rng.random() < eps else int(policy[s] if policy[s] in valid else 0)
            obs, r, done, _, info = env.step(a)
            traj.append((s, a))
            rewards.append(r)
            visits[s] += 1
            if done:
                break
        G = np.cumsum(rewards[::-1])[::-1]  # return from each decision to the end
        for (s, a), ret in zip(traj, G):
            n[s, a] += 1
            total[s, a] += ret
            total2[s, a] += ret * ret
        if g % 500 == 499:
            policy = cautious_policy(n, total, total2)
    return (n, total, total2), visits, time.perf_counter() - t0


def cautious_policy(n, total, total2, z: float = 2.0):
    mean = np.divide(total, n, out=np.zeros_like(total), where=n > 0)
    var = np.divide(total2, n, out=np.zeros_like(total), where=n > 0) - mean**2
    se = np.sqrt(np.divide(np.maximum(var, 0), n, out=np.full_like(total, np.inf), where=n > 1))
    policy = np.zeros(N_S, dtype=np.int64)
    for s in range(N_S):
        m = sensible(s)
        gain = mean[s] - mean[s, 0]
        margin = z * np.sqrt(se[s] ** 2 + se[s, 0] ** 2)
        ok = m & (n[s] >= 200) & (gain > margin)
        ok[0] = False
        if ok.any():
            policy[s] = int(np.argmax(np.where(ok, gain - margin, -np.inf)))
    return policy


def train(games: int, seed: int, method: str = "double", alpha: float = 0.05):
    env = bm_gym.ManagerEnv(seed=seed)
    QA = np.zeros((N_S, N_A))
    QB = np.zeros((N_S, N_A))
    NA = np.zeros((N_S, N_A))
    NB = np.zeros((N_S, N_A))
    visits = np.zeros(N_S, dtype=np.int64)
    rng = np.random.default_rng(seed)
    t0 = time.perf_counter()
    for g in range(games):
        eps = max(0.02, 0.3 * (1 - g / (0.7 * games)))
        obs, info = env.reset(seed=seed * 1_000_003 + g)
        obs = np.asarray(obs, dtype=np.float32)
        s = state_of(obs, info)
        mask = np.asarray(env.action_mask(), dtype=bool)
        while True:
            valid = np.flatnonzero(mask)
            if rng.random() < eps:
                a = int(rng.choice(valid))
            else:
                a = int(valid[np.argmax((QA + QB)[s, valid])])
            obs, r, done, _, info = env.step(a)
            obs = np.asarray(obs, dtype=np.float32)
            visits[s] += 1
            if method == "plain":
                if done:
                    QA[s, a] += alpha * (r - QA[s, a])
                    break
                s2 = state_of(obs, info)
                mask = np.asarray(env.action_mask(), dtype=bool)
                QA[s, a] += alpha * (r + QA[s2, np.flatnonzero(mask)].max() - QA[s, a])
                s = s2
                continue
            # Double Q-learning: one table picks the next action, the other values it.
            upd, other, n = (QA, QB, NA) if rng.random() < 0.5 else (QB, QA, NB)
            n[s, a] += 1
            step = 1.0 / n[s, a]
            if done:
                upd[s, a] += step * (r - upd[s, a])
                break
            s2 = state_of(obs, info)
            mask = np.asarray(env.action_mask(), dtype=bool)
            v2 = np.flatnonzero(mask)
            best = v2[np.argmax(upd[s2, v2])]
            upd[s, a] += step * (r + other[s2, best] - upd[s, a])
            s = s2
    Q = QA if method == "plain" else (QA + QB) / 2
    return Q, visits, time.perf_counter() - t0


def play(args):
    """Win (1/0) of each seed for one policy: 'hit', 'ai' or a greedy Q table."""
    policy, seeds, Q = args
    env = bm_gym.ManagerEnv(seed=0)
    wins = []
    for sd in seeds:
        obs, info = env.reset(seed=int(sd))
        if policy == "ai":
            wins.append(int(env.finish_with_ai()))
            continue
        while True:
            if policy == "hit":
                a = 0
            elif policy == "table":
                s = state_of(np.asarray(obs, dtype=np.float32), info)
                a = int(Q[s]) if env.action_mask()[int(Q[s])] else 0
            else:
                s = state_of(np.asarray(obs, dtype=np.float32), info)
                valid = np.flatnonzero(env.action_mask())
                a = int(valid[np.argmax(Q[s, valid])])
            obs, r, done, _, info = env.step(a)
            if done:
                wins.append(int(info["won"]))
                break
    return wins


def evaluate(policies: dict, n: int, workers: int, seed: int):
    seeds = np.arange(n) + 10**9 + seed
    chunks = np.array_split(seeds, workers)
    out = {}
    with mp.Pool(workers) as pool:
        for name, (kind, Q) in policies.items():
            parts = pool.map(play, [(kind, c, Q) for c in chunks])
            out[name] = np.concatenate([np.asarray(p) for p in parts])
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--train", type=int, default=40000)
    ap.add_argument("--eval", type=int, default=20000)
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--method", choices=["mc", "double", "plain"], default="mc")
    ap.add_argument("--json", default=None)
    a = ap.parse_args()

    if a.method == "mc":
        stats, visits, secs = train_mc(a.train, a.seed)
        table = cautious_policy(*stats)
        learned = ("table", table)
        n, total, _ = stats
        Q = np.divide(total, n, out=np.zeros_like(total), where=n > 0)
    else:
        Q, visits, secs = train(a.train, a.seed, a.method)
        table = np.array([int(np.flatnonzero(sensible(s))[np.argmax(Q[s, sensible(s)])]) for s in range(N_S)])
        learned = ("q", Q)
    print(f"{a.method}: trained on {a.train} games in {secs:.0f} s ({visits.sum()} decisions); states visited: {(visits > 0).sum()} of {N_S}")
    print(f"states where the learned policy leaves swinging away: {(table != 0).sum()} of {N_S}")
    res = evaluate({"hit": ("hit", None), "ai": ("ai", None), "q": learned}, a.eval, a.workers, a.seed)
    print(f"\nwin rate over {a.eval} games from the same seeds (paired differences ± standard error):")
    for k, w in res.items():
        print(f"  {k:4s} {w.mean():.4f}")
    for x, y in [("q", "hit"), ("q", "ai"), ("ai", "hit")]:
        d = res[x].astype(float) - res[y]
        print(f"  {x} − {y}: {d.mean():+.4f} ± {d.std(ddof=1) / np.sqrt(len(d)):.4f}")

    # What the learned policy does where it was trained most, against swinging away.
    print("\nlearned choices that differ from swinging away (sensible actions only), most visited first:")
    names = bm_gym.ACTION_NAMES
    rows = []
    for s in np.argsort(-visits):
        if visits[s] == 0:
            break
        best = int(table[s])
        if best != 0:
            rows.append((describe(s), names[best], float(Q[s, best] - Q[s, 0]), int(visits[s])))
    for d, n, gain, v in rows[:40]:
        print(f"  {d}: {n:<18} +{gain:.4f} (visits {v})")
    if a.json:
        with open(a.json, "w") as f:
            json.dump({"win": {k: float(w.mean()) for k, w in res.items()}, "n": a.eval, "train": a.train,
                       "differs": [{"state": d, "action": n, "gain": g, "visits": v} for d, n, g, v in rows]}, f, indent=1)


if __name__ == "__main__":
    main()
