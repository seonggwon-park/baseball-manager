"""Kinematic checks of a CMU throw or swing at the full capture rate (120 fps).

Throw: peak hand speed (about the ball speed at release), foot contact to release, stride as a
share of standing height, and how far the pelvis and the shoulder line have turned towards the
target at leg lift, foot contact and release (0 = side-on, 90 = square to the target).
Swing: per swing, the peak speed of the hands and of a sweet spot on the line through the two
hands (0.50-0.65 m beyond their midpoint; Statcast measures bat speed at the sweet spot).

usage: python3 -I kin_check.py throw SKELETON.asf MOTION.amc
       python3 -I kin_check.py swing SKELETON.asf MOTION.amc
"""
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cmu_fk  # noqa: E402

FPS = 120.0


def tracks(asf, amc):
    bones, hier = cmu_fk.read_asf(asf)
    frames = cmu_fk.read_amc(amc)
    out = []
    for f in frames:
        pos, _ = cmu_fk.fk(bones, hier, f)
        out.append({k: np.array(cmu_fk.to_blender(v)) for k, v in pos.items()})
    return out


def speed(x):
    v = np.zeros(len(x))
    v[1:-1] = np.linalg.norm(x[2:] - x[:-2], axis=1) * FPS / 2  # central difference
    return v


def throw(P):
    T = len(P)

    def track(j):
        return np.array([p[j] for p in P])

    rh = speed(track("rhand"))
    rel = int(np.argmax(rh))
    print("frames", T, "duration s", round(T / FPS, 2))
    print("peak right-hand speed m/s", round(rh.max(), 1), "at frame", rel)
    lt, lf = track("ltoes"), track("lfoot")
    lift = int(np.argmax(lf[:rel, 2]))
    land = next(k for k in range(lift, T) if lt[k, 2] < 0.04)
    print("leg-lift peak frame", lift, "foot (toe base) above its start m", round(lf[lift, 2] - lf[0, 2], 2))
    print("foot contact frame", land, "-> release", rel, ":", round((rel - land) / FPS * 1000), "ms")
    stride = lt[land] - track("rtoes")[0]
    stride[2] = 0
    height = track("head")[0, 2] - min(lt[0, 2], track("rtoes")[0, 2])  # the head bone ends near the crown
    print("standing height m", round(height, 3), "stride m", round(np.linalg.norm(stride), 3),
          "=", round(100 * np.linalg.norm(stride) / height), "% of height")
    target = np.degrees(np.arctan2(stride[1], stride[0]))

    def opened(a, b, k):
        d = track(a)[k] - track(b)[k]
        return (np.degrees(np.arctan2(d[1], d[0])) - target + 180) % 360 - 180

    # Hip joints end the hipjoint bones and the shoulders end the clavicles in the ASF.
    for name, k in (("lift", lift), ("contact", land), ("release", rel)):
        h, s = opened("lhipjoint", "rhipjoint", k), opened("lclavicle", "rclavicle", k)
        print(f"{name}: pelvis turned {h:.0f} deg, shoulders turned {s:.0f} deg, separation {h - s:.0f} deg")
    knee, hip = track("lfemur"), track("lhipjoint")
    print("at lift: knee above hip m", round(knee[lift, 2] - hip[lift, 2], 2))


def swing(P):
    def smooth(x, w=5):
        k = np.ones(w) / w
        return np.stack([np.convolve(x[:, i], k, mode="same") for i in range(3)], axis=1)

    R = smooth(np.array([p["rhand"] for p in P]))
    L = smooth(np.array([p["lhand"] for p in P]))
    mid = (R + L) / 2
    axis = (R - L) / np.linalg.norm(R - L, axis=1)[:, None]
    hm = speed(mid)
    hm[:5] = hm[-5:] = 0
    fast = hm > 4.0
    starts = [i for i in range(1, len(fast)) if fast[i] and not fast[i - 1]]
    print("frames", len(P), "swings (hands faster than 4 m/s) start at s:", [round(s / FPS, 2) for s in starts])
    for s in starts:
        w = slice(max(0, s - 30), s + 60)
        line = f"swing at {s / FPS:.2f} s: hands peak {hm[w].max():.1f} m/s"
        for reach in (0.50, 0.58, 0.65):
            v = speed(mid + reach * axis)
            v[:5] = v[-5:] = 0
            line += f"; sweet spot {reach:.2f} m out {v[w].max():.1f} m/s ({v[w].max() / 0.44704:.0f} mph)"
        print(line)


def main():
    mode, asf, amc = sys.argv[1:4]
    P = tracks(asf, amc)
    {"throw": throw, "swing": swing}[mode](P)


if __name__ == "__main__":
    main()
