"""Tables from `bm rollout --json FILE`: each tactic against swinging away, by class, and split
by the player the tactic depends on (the hitter for bunts, the lead runner for steals, holds
and the hit and run, the hitter's speed for the bunt for a hit).

    python3 tools/rl/rollout_report.py rollout.json > rollout.md

Differences are means over plate appearances of (tactic − swing away), each from the same
state and the same random numbers; ± is the standard error over plate appearances.
"""

import json
import math
import sys

KO = {
    "first_0out_early_close": "1루, 무사, 1–6회, 점수차 1 이하",
    "first_0out_late_close": "1루, 무사, 7회 이후, 점수차 1 이하",
    "second_0out_late_close": "2루, 무사, 7회 이후, 점수차 1 이하",
    "first_second_0out_late_close": "1·2루, 무사, 7회 이후, 점수차 1 이하",
    "first_1out_late_close": "1루, 1사, 7회 이후, 점수차 1 이하",
    "empty_0out": "주자 없음, 무사, 모든 이닝",
    "first_2out": "1루, 2사, 모든 이닝",
}
TACTIC_KO = {"sac_bunt": "희생번트", "bunt_for_hit": "기습번트", "steal": "도루(첫 투구)", "hit_and_run": "히트 앤드 런(첫 투구)", "hold": "주자 묶기"}
SPLIT = {"sac_bunt": ("batter_quality", "타자 공격 가치"), "bunt_for_hit": ("batter_speed", "타자 주력"),
         "steal": ("lead_runner_speed", "선두 주자 주력"), "hold": ("lead_runner_speed", "선두 주자 주력"),
         "hit_and_run": ("lead_runner_speed", "선두 주자 주력")}


def mean_se(xs):
    n = len(xs)
    if n == 0:
        return float("nan"), float("nan")
    m = sum(xs) / n
    v = sum((x - m) ** 2 for x in xs) / (n - 1) if n > 1 else 0.0
    return m, math.sqrt(v / n)


def fmt(m, se, digits=3):
    return f"{m:+.{digits}f} ± {se:.{digits}f}"


def main(path):
    rows = json.load(open(path))
    by = {}
    for r in rows:
        by.setdefault(r["class_name"], []).append(r)
    rollouts = None
    print(f"# 작전 롤아웃 ({len(rows)} 타석)\n")
    for name, rs in by.items():
        tactics = rs[0]["tactics"]
        base_w = mean_se([r["win"][0] for r in rs])[0]
        base_r = mean_se([r["runs"][0] for r in rs])[0]
        we = mean_se([r["win_expectancy"] for r in rs])[0]
        print(f"## {KO.get(name, name)} ({len(rs)} 타석)\n")
        print(f"강공: 승리 {base_w:.3f}(표의 승리 기대값 {we:.3f}), 그 이닝 남은 득점 {base_r:.3f}\n")
        print("| 작전 | 승리 확률 변화 | 그 이닝 득점 변화 |")
        print("|---|---|---|")
        for t, tn in enumerate(tactics):
            if t == 0:
                continue
            dw = mean_se([r["win"][t] - r["win"][0] for r in rs])
            dr = mean_se([r["runs"][t] - r["runs"][0] for r in rs])
            print(f"| {TACTIC_KO.get(tn, tn)} | {fmt(*dw)} | {fmt(*dr)} |")
        print()
        for t, tn in enumerate(tactics):
            if t == 0 or tn not in SPLIT:
                continue
            key, label = SPLIT[tn]
            xs = sorted(r[key] for r in rs)
            lo, hi = xs[len(xs) // 3], xs[2 * len(xs) // 3]
            parts = [("하위 1/3", [r for r in rs if r[key] <= lo]), ("가운데", [r for r in rs if lo < r[key] <= hi]), ("상위 1/3", [r for r in rs if r[key] > hi])]
            cells = []
            for lab, sub in parts:
                dw = mean_se([r["win"][t] - r["win"][0] for r in sub])
                dr = mean_se([r["runs"][t] - r["runs"][0] for r in sub])
                cells.append(f"{lab}(n={len(sub)}): 승리 {fmt(*dw)}, 득점 {fmt(*dr)}")
            print(f"- {TACTIC_KO.get(tn, tn)}, {label}별: " + "; ".join(cells))
        print()


if __name__ == "__main__":
    main(sys.argv[1])
