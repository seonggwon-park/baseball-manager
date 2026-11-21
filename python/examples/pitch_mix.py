"""Pitch calling on the engine: how much a pitch-caller gains, and what adaptive batters leave of it.

Three experiments on bm_gym.PitchingEnv (docs/rl.md, section 9):

  collect   Call pitches at random (every pitch type the pitcher has, every intent) against
            batters who do not read the pitcher (adapt 0), and average each pitch's run value by
            count, batter side, pitch type and intent: a table of what each call is worth.
  evaluate  Play the same games (same seeds: same clubs, parks and sides) with several callers and
            compare the runs allowed per game, paired game by game:
              ai      the engine's own choice (each pitch type at its usual share, the count's usual aim)
              greedy  the call with the best value in the table, every time
              greedy_kind  the pitch type with the best value thrown with the count's usual aim
              learned a policy trained with `train`
            with the batters reading the pitcher (adapt 1) or not (adapt 0).
  train     A softmax policy over (pitch type, intent) that starts from the engine's mix: logits
            log(usual share) + theta[count, side, type] + phi[count, side, intent]. REINFORCE with
            the discounted run values that follow each call (gamma 0.95, about 20 pitches, the
            batters' memory is 25), a per-count baseline, and the batters' perception constants
            shaken every game (perturb 1), so that the policy cannot live off one set of constants.

usage:
  python python/examples/pitch_mix.py collect --games 20000 --out table.npz
  python python/examples/pitch_mix.py train --iters 150 --batch 96 --out policy.npz
  python python/examples/pitch_mix.py evaluate --games 4000 --table table.npz --policy policy.npz
"""

from __future__ import annotations

import argparse
import json
import math
import multiprocessing as mp
import time

import numpy as np

import bm_gym

N_KINDS, N_INTENTS, N_COUNTS = 8, 3, 12
KIND_NAMES = ["four_seam", "sinker", "cutter", "slider", "sweeper", "curve", "changeup", "splitter"]
INTENT_NAMES = ["attack", "normal", "chase"]
FASTBALLS = [0, 1, 2]  # four-seam, sinker, cutter


def parse(obs):
    o = np.asarray(obs, dtype=np.float64)
    count = int(np.argmax(o[:12]))
    side = int(o[21] > 0.5)
    avail = o[28:68:5] > 0.5
    usual = o[29:69:5]
    return count, side, avail, usual


# ---- policies -----------------------------------------------------------------------------

def ai_probs(avail, usual):
    """The engine's own choice: pitch types at their usual shares, the count's usual aim."""
    p = np.zeros((N_KINDS, N_INTENTS))
    w = np.where(avail, usual, 0.0)
    w = w / w.sum() if w.sum() > 0 else avail / avail.sum()
    p[:, 1] = w
    return p


def softmax_probs(theta, phi, count, side, avail, usual):
    logits = np.log(np.maximum(usual, 1e-4))[:, None] + theta[count, side][:, None] + phi[count, side][None, :]
    logits = np.where(avail[:, None], logits, -np.inf)
    m = logits.max()
    p = np.exp(logits - m)
    return p / p.sum()


def greedy_probs(table, count, side, avail):
    v = np.where(avail[:, None], table[count, side], -np.inf)
    p = np.zeros_like(v)
    p[np.unravel_index(np.argmax(v), v.shape)] = 1.0
    return p


def greedy_kind_probs(table, count, side, avail):
    """The best pitch type by the table, thrown with the count's usual aim."""
    v = np.where(avail, table[count, side, :, 1], -np.inf)
    p = np.zeros((N_KINDS, N_INTENTS))
    p[int(np.argmax(v)), 1] = 1.0
    return p


# ---- workers ------------------------------------------------------------------------------

_ENV = {}


def env_for(adapt, perturb):
    key = (adapt, perturb)
    if key not in _ENV:
        _ENV[key] = bm_gym.PitchingEnv(seed=0, side=None, adapt=adapt, perturb=perturb)
    return _ENV[key]


def play(job):
    """One game. job: dict(seed, adapt, perturb, policy, params, record)."""
    env = env_for(job["adapt"], job["perturb"])
    rng = np.random.default_rng(job["seed"] + 7_000_003)
    obs, info = env.reset(seed=job["seed"])
    kind = job["policy"]
    rec = [] if job.get("record") else None
    mix = np.zeros((N_COUNTS, 2, N_KINDS, N_INTENTS))
    while True:
        count, side, avail, usual = parse(obs)
        if kind == "ai":
            p = ai_probs(avail, usual)
        elif kind == "random":
            p = np.repeat(avail[:, None], N_INTENTS, axis=1).astype(float)
            p /= p.sum()
        elif kind == "greedy":
            p = greedy_probs(job["params"]["table"], count, side, avail)
        elif kind == "greedy_kind":
            p = greedy_kind_probs(job["params"]["table"], count, side, avail)
        else:
            p = softmax_probs(job["params"]["theta"], job["params"]["phi"], count, side, avail, usual)
        a = int(rng.choice(N_KINDS * N_INTENTS, p=p.ravel()))
        obs, reward, terminated, _, info = env.step(a)
        k, i = divmod(a, N_INTENTS)
        mix[count, side, k, i] += 1
        if rec is not None:
            rec.append((count, side, k, i, reward, avail.astype(np.int8), usual.astype(np.float32)))
        if terminated:
            return {"allowed": info["score"][1], "calls": info["decisions"], "mix": mix, "rec": rec}


def run(jobs, workers):
    if workers <= 1:
        return [play(j) for j in jobs]
    with mp.get_context("fork").Pool(workers) as pool:
        return pool.map(play, jobs, chunksize=max(1, len(jobs) // (workers * 8)))


# ---- experiments --------------------------------------------------------------------------

def collect(args):
    t = time.time()
    jobs = [dict(seed=100_000 + g, adapt=0.0, perturb=0.0, policy="random", params=None, record=True) for g in range(args.games)]
    res = run(jobs, args.workers)
    s = np.zeros((N_COUNTS, 2, N_KINDS, N_INTENTS))
    ss = np.zeros_like(s)
    n = np.zeros_like(s)
    for r in res:
        for count, side, k, i, reward, _, _ in r["rec"]:
            s[count, side, k, i] += reward
            ss[count, side, k, i] += reward * reward
            n[count, side, k, i] += 1
    mean = np.where(n > 0, s / np.maximum(n, 1), -9.0)
    se = np.sqrt(np.maximum(ss / np.maximum(n, 1) - mean ** 2, 0) / np.maximum(n, 1))
    np.savez(args.out, table=mean, se=se, n=n)
    print(f"{args.games} games at random, {int(n.sum())} calls, {time.time() - t:.0f} s -> {args.out}")
    # The best call by count (batter on the pitcher's side), against the value of hitting the usual aim.
    for c in range(N_COUNTS):
        b, st = divmod(c, 3)
        for side in (1, 0):
            v = mean[c, side]
            k, i = np.unravel_index(np.argmax(v), v.shape)
            print(f"  {b}-{st} {'same' if side else 'opp '}: best {KIND_NAMES[k]}+{INTENT_NAMES[i]} {v[k, i]:+.4f} ± {se[c, side, k, i]:.4f}"
                  f" (n {int(n[c, side, k, i])}); four-seam normal {v[0, 1]:+.4f}")


def paired(a, b):
    d = np.asarray(a, float) - np.asarray(b, float)
    return d.mean(), d.std(ddof=1) / math.sqrt(len(d))


def fastball_share(mix):
    tot = mix.sum(axis=(2, 3))
    fb = mix[:, :, FASTBALLS, :].sum(axis=(2, 3))
    return fb.sum() / tot.sum(), fb / np.maximum(tot, 1)


def evaluate(args):
    table = np.load(args.table)["table"] if args.table else None
    pol = np.load(args.policy) if args.policy else None
    callers = ["ai"] + (["greedy", "greedy_kind"] if table is not None else []) + (["learned"] if pol is not None else [])
    out = {}
    for adapt in args.adapt:
        for perturb in args.perturb:
            results = {}
            for c in callers:
                params = {"table": table} if c.startswith("greedy") else ({"theta": pol["theta"], "phi": pol["phi"]} if c == "learned" else None)
                jobs = [dict(seed=500_000 + g, adapt=adapt, perturb=perturb, policy="softmax" if c == "learned" else c, params=params) for g in range(args.games)]
                results[c] = run(jobs, args.workers)
            base = [r["allowed"] for r in results["ai"]]
            print(f"batters read the pitcher: {adapt:.0f}, constants shaken: {perturb:.0f} ({args.games} games, same seeds)")
            for c in callers:
                allowed = [r["allowed"] for r in results[c]]
                mix = sum(r["mix"] for r in results[c])
                fb, _ = fastball_share(mix)
                intents = mix.sum(axis=(0, 1, 2))
                line = f"  {c:<8} runs allowed per game {np.mean(allowed):.3f}"
                if c != "ai":
                    d, se = paired(allowed, base)
                    line += f"  vs ai {d:+.3f} ± {se:.3f}"
                line += f"  fastballs {fb:.1%}  intents attack/normal/chase {intents[0] / intents.sum():.1%}/{intents[1] / intents.sum():.1%}/{intents[2] / intents.sum():.1%}"
                print(line)
                out[f"{c}_adapt{adapt:.0f}_perturb{perturb:.0f}"] = {"allowed": float(np.mean(allowed)), "mix": mix.tolist()}
    if args.json:
        json.dump(out, open(args.json, "w"))


def train(args):
    rng = np.random.default_rng(args.seed)
    theta = np.zeros((N_COUNTS, 2, N_KINDS))
    phi = np.zeros((N_COUNTS, 2, N_INTENTS))
    phi[:, :, 0] = phi[:, :, 2] = args.intent_start  # start close to the usual aim
    baseline = np.zeros(N_COUNTS)
    m_t, v_t, m_p, v_p = (np.zeros_like(theta), np.zeros_like(theta), np.zeros_like(phi), np.zeros_like(phi))
    b1, b2, eps = 0.9, 0.999, 1e-8
    t0 = time.time()
    seed = 1_000_000
    hist = []
    for it in range(1, args.iters + 1):
        jobs = [dict(seed=seed + g, adapt=1.0, perturb=args.perturb, policy="softmax", params={"theta": theta, "phi": phi}, record=True) for g in range(args.batch)]
        seed += args.batch
        res = run(jobs, args.workers)
        g_t = np.zeros_like(theta)
        g_p = np.zeros_like(phi)
        n = 0
        allowed = []
        new_base = np.zeros(N_COUNTS)
        new_n = np.zeros(N_COUNTS)
        for r in res:
            allowed.append(r["allowed"])
            rec = r["rec"]
            rewards = np.array([x[4] for x in rec])
            ret = np.zeros_like(rewards)
            acc = 0.0
            for t in range(len(rewards) - 1, -1, -1):
                acc = rewards[t] + args.gamma * acc
                ret[t] = acc
            for (count, side, k, i, _, avail, usual), G in zip(rec, ret):
                p = softmax_probs(theta, phi, count, side, avail.astype(bool), usual.astype(np.float64))
                adv = G - baseline[count]
                new_base[count] += G
                new_n[count] += 1
                # d log pi / d theta[count, side, :] = onehot(k) - p over kinds; likewise for intents.
                g_t[count, side] += adv * ((np.arange(N_KINDS) == k) - p.sum(axis=1))
                g_p[count, side] += adv * ((np.arange(N_INTENTS) == i) - p.sum(axis=0))
                n += 1
        baseline = np.where(new_n > 0, 0.8 * baseline + 0.2 * new_base / np.maximum(new_n, 1), baseline)
        g_t /= n
        g_p /= n
        # Adam ascent on the expected run value.
        for g, m, v, w in ((g_t, m_t, v_t, theta), (g_p, m_p, v_p, phi)):
            m[:] = b1 * m + (1 - b1) * g
            v[:] = b2 * v + (1 - b2) * g * g
            mh = m / (1 - b1 ** it)
            vh = v / (1 - b2 ** it)
            w += args.lr * mh / (np.sqrt(vh) + eps)
        hist.append(float(np.mean(allowed)))
        if it % 10 == 0 or it == 1:
            recent = np.mean(hist[-10:])
            print(f"  iter {it:4d}  runs allowed per game (last 10 batches) {recent:.3f}  {time.time() - t0:.0f} s", flush=True)
    np.savez(args.out, theta=theta, phi=phi, history=np.array(hist))
    print(f"saved {args.out}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("collect")
    c.add_argument("--games", type=int, default=20000)
    c.add_argument("--out", default="table.npz")
    e = sub.add_parser("evaluate")
    e.add_argument("--games", type=int, default=4000)
    e.add_argument("--table")
    e.add_argument("--policy")
    e.add_argument("--perturb", type=float, nargs="+", default=[0.0])
    e.add_argument("--adapt", type=float, nargs="+", default=[0.0, 1.0])
    e.add_argument("--json")
    t = sub.add_parser("train")
    t.add_argument("--iters", type=int, default=150)
    t.add_argument("--batch", type=int, default=96)
    t.add_argument("--lr", type=float, default=0.05)
    t.add_argument("--gamma", type=float, default=0.95)
    t.add_argument("--perturb", type=float, default=1.0)
    t.add_argument("--intent-start", type=float, default=-2.0)
    t.add_argument("--seed", type=int, default=1)
    t.add_argument("--out", default="policy.npz")
    for p in (c, e, t):
        p.add_argument("--workers", type=int, default=4)
    args = ap.parse_args()
    {"collect": collect, "evaluate": evaluate, "train": train}[args.cmd](args)


if __name__ == "__main__":
    main()
