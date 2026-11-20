"""Vertical short (1080x1920, 30 fps): "is the sacrifice bunt really a loss?", from the tactic
rollouts and the tabular RL demo of docs/rl.md, with fictional players.

The bunt numbers, the one example moment and the 300 dots are computed from the rollout output
(`bm rollout --states 300 --rollouts 200 --seed 2025 --json rollout.json`); the steal and RL
numbers are the published ones in docs/rl.md (sections 6-7) and docs/tactics.md (section 2).
The opening shot is a folder of rendered frames (tools/render, e.g. the CMU bunt on the body).

usage:
  python3 -I tactics_short.py ROLLOUT.json HOOK_FRAMES_DIR OUT.mp4
  python3 -I tactics_short.py ROLLOUT.json HOOK_FRAMES_DIR OUT_DIR --stills 2.5,9,18   # PNG stills
env: HOOK_FIRST, HOOK_LAST (1-based frame numbers of the opening shot; the last one is held)
needs: Pillow, ffmpeg, the Noto Sans CJK fonts (fonts-noto-cjk)
"""
import json
import math
import os
import random
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFont

W, H, SS, FPS = 1080, 1920, 2, 30
RIGHT = 900  # right edge for numbers: the apps put their buttons along the right side

# Colours: validated dark surface and series steps (dataviz reference palette, dark mode);
# both series pass the CVD and contrast checks against the surface.
BG = (26, 26, 25)
INK = (240, 239, 236)
INK2 = (195, 194, 183)
MUTED = (154, 153, 143)
NEUTRAL = (107, 106, 100)
BLUE = (57, 135, 229)
RED = (230, 103, 103)
FONT_R = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"
FONT_B = "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc"
KR = 1  # face index of Noto Sans CJK KR in the collections

# ---- data ----------------------------------------------------------------------------------
# Filled from the rollout output by load_rollouts(): one typical moment (wins of 200 rollouts
# each way), the bunt rows (situation, mean change in win probability vs hitting away in %p,
# standard error across moments) and the per-moment differences of one situation.
EXAMPLE = {}
BUNT_ROWS = []
BUNT_CLASSES = [("first_0out_early_close", "무사 1루 · 1~6회"), ("first_0out_late_close", "무사 1루 · 7회 이후"),
                ("first_1out_late_close", "1사 1루 · 7회 이후"), ("second_0out_late_close", "무사 2루 · 7회 이후"),
                ("first_second_0out_late_close", "무사 1·2루 · 7회 이후")]
DOTS_CLASS = "first_0out_late_close"
# Published in docs/rl.md section 6 (41 % and 65 % are inverted from run changes with the
# engine's run-expectancy table) and docs/tactics.md section 2 (76.9 %, 68.4 %).
STEAL_ROWS = [("무사 1루 · 첫 공에 무조건 (주자 전체)", 41, "약 41%"), ("무사 1루 · 첫 공에 무조건 (빠른 1/3)", 65, "약 65%"),
              ("주자가 스스로 판단한 2루 도루 (전체)", 76.9, "76.9%")]
BREAK_EVEN = 68.4  # engine, steal of second with no outs (docs/tactics.md section 2)
# docs/rl.md section 7: win rates over the same 20,000 games.
RL_ROWS = [("항상 강공 (기준)", 0.5029, "", NEUTRAL), ("Q-학습", 0.3689, "거의 모든 볼카운트에서 '기다리기'를 배움", BLUE),
           ("이중 Q-학습", 0.2544, "덜 해 본 작전(도루·번트)을 남발", BLUE),
           ("몬테카를로 + 신중한 규칙", 0.4980, "강공과 거의 같음 (−0.5%p ± 0.2)", BLUE)]


def load_rollouts(path):
    """The bunt rows, the dots and the example moment from `bm rollout --json` output."""
    rows = json.load(open(path))

    def cls(name):
        return [r for r in rows if r["class_name"] == name]

    def bunt_diffs(states):
        i = states[0]["tactics"].index("sac_bunt")
        return [100 * (r["win"][i] - r["win"][0]) for r in states]

    for name, label in BUNT_CLASSES:
        d = bunt_diffs(cls(name))
        m = sum(d) / len(d)
        se = (sum((x - m) ** 2 for x in d) / (len(d) - 1) / len(d)) ** 0.5
        BUNT_ROWS.append((label, round(m, 2), round(se, 2)))
    states = cls(DOTS_CLASS)
    diffs = [round(x * 2) / 2 for x in bunt_diffs(states)]  # one rollout of 200 = 0.5 %p
    # A typical moment: the one whose hitting-away win rate is closest to the situation's mean.
    # (`lead` is the batting side's lead.)
    mean_hit = sum(r["win"][0] for r in states) / len(states)
    ex = min(states, key=lambda r: abs(r["win"][0] - mean_hit))
    i = ex["tactics"].index("sac_bunt")
    score = "동점" if ex["lead"] == 0 else (f"{abs(ex['lead'])}점 {'앞섬' if ex['lead'] > 0 else '뒤짐'}")
    EXAMPLE.update(label=f"{ex['inning']}회{'말' if ex['home'] else '초'} · {score} · 무사 1루",
                   hit=round(ex["win"][0] * 200), bunt=round(ex["win"][i] * 200))
    print("example", EXAMPLE, "rows", BUNT_ROWS)
    return diffs


def font(size, bold=False):
    key = (size, bold)
    if key not in font.cache:
        font.cache[key] = ImageFont.truetype(FONT_B if bold else FONT_R, size * SS, index=KR)
    return font.cache[key]


font.cache = {}


def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))


def ease(x):
    x = clamp(x)
    return 1 - (1 - x) ** 3


def smooth(x):
    x = clamp(x)
    return x * x * (3 - 2 * x)


class Canvas:
    def __init__(self, alpha=1.0):
        self.img = Image.new("RGB", (W * SS, H * SS), BG)
        self.d = ImageDraw.Draw(self.img, "RGBA")
        self.alpha = alpha

    def c(self, rgb, a=1.0):
        return (*rgb, int(255 * clamp(a * self.alpha)))

    def text(self, x, y, s, size, rgb=INK, a=1.0, bold=False, anchor="la"):
        self.d.text((x * SS, y * SS), s, font=font(size, bold), fill=self.c(rgb, a), anchor=anchor)

    def width(self, s, size, bold=False):
        return self.d.textlength(s, font=font(size, bold)) / SS

    def rect(self, x0, y0, x1, y1, rgb, a=1.0, r=0):
        if x1 < x0:
            x0, x1 = x1, x0
        if x1 - x0 < 0.5:
            return
        box = [x0 * SS, y0 * SS, x1 * SS, y1 * SS]
        if r:
            self.d.rounded_rectangle(box, radius=min(r * SS, (x1 - x0) * SS / 2), fill=self.c(rgb, a))
        else:
            self.d.rectangle(box, fill=self.c(rgb, a))

    def line(self, pts, rgb, a=1.0, w=2):
        self.d.line([(x * SS, y * SS) for x, y in pts], fill=self.c(rgb, a), width=max(1, int(w * SS)), joint="curve")

    def dashed(self, x0, y0, x1, y1, rgb, a=1.0, w=2, dash=10, gap=8):
        n = math.hypot(x1 - x0, y1 - y0)
        k = 0.0
        while k < n:
            e = min(n, k + dash)
            self.line([(x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n), (x0 + (x1 - x0) * e / n, y0 + (y1 - y0) * e / n)], rgb, a, w)
            k = e + gap

    def dot(self, x, y, r, rgb, a=1.0):
        self.d.ellipse([(x - r) * SS, (y - r) * SS, (x + r) * SS, (y + r) * SS], fill=self.c(rgb, a))

    def frame(self):
        return self.img.resize((W, H), Image.LANCZOS)


def header(cv, t, chapter, title, sub=None):
    a = smooth(t / 0.4)
    cv.text(80, 236, chapter, 32, MUTED, a, bold=True)
    for i, line in enumerate(title.split("\n")):
        cv.text(80, 290 + i * 84, line, 64, INK, a, bold=True)
    if sub:
        n = len(title.split("\n"))
        cv.text(80, 290 + n * 84 + 14, sub, 34, INK2, a)


def footer(cv):
    cv.text(80, 1478, "가상 선수 · 가상 리그 시뮬레이션", 28, MUTED, 1.0)


def caption(cv, t, t0, lines, y=1330, size=44, bold=True):
    a = smooth((t - t0) / 0.4)
    if a <= 0:
        return
    for i, line in enumerate(lines):
        cv.text(80, y + i * (size + 18) + 12 * (1 - a), line, size, INK, a, bold=bold)


# ---- scenes ------------------------------------------------------------------------------
def scene_hook(cv, t, dur, clip):
    # Portrait close-up of the delivery (stride to follow-through), then a hold on the last frame.
    k = min(len(clip) - 1, int(t * FPS))
    im = Image.open(clip[k]).convert("RGB")
    im = im.resize((W * SS, int(W * SS * im.height / im.width)), Image.LANCZOS)
    top = HOOK_TOP
    cv.img.paste(im, (0, top * SS))
    for i in range(40):  # fade the picture's top edge into the background
        cv.rect(0, top + i * 4, W, top + i * 4 + 4, BG, (1 - i / 40) ** 1.5)
    a = smooth((t - 0.15) / 0.35)
    cv.text(80, 236, "번트, 정말", 88, INK, a, bold=True)
    cv.text(80, 340, "손해일까?", 88, INK, a, bold=True)
    b = smooth((t - 0.8) / 0.4)
    cv.text(80, 470, "작전만 바꿔 150만 번 시뮬레이션해 봤다", 40, INK2, b)
    cv.text(80, 1478, "3D 시뮬레이션 · 가상 선수", 28, INK2, 1.0)


def bezier(p0, p1, p2, p3, n=28, f=1.0):
    out = []
    m = max(2, int(n * f) + 1)
    for i in range(m):
        s = f * i / (m - 1)
        u = 1 - s
        out.append((u ** 3 * p0[0] + 3 * u * u * s * p1[0] + 3 * u * s * s * p2[0] + s ** 3 * p3[0],
                    u ** 3 * p0[1] + 3 * u * u * s * p1[1] + 3 * u * s * s * p2[1] + s ** 3 * p3[1]))
    return out


def scene_branch(cv, t, dur):
    header(cv, t, "방법", "같은 순간을 복제해\n작전만 바꿔 본다")
    a = smooth((t - 0.3) / 0.4)
    # The moment.
    w = cv.width(EXAMPLE["label"], 40, True) + 64
    cv.rect(540 - w / 2, 520, 540 + w / 2, 600, (44, 44, 42), a, r=18)
    cv.text(540, 560, EXAMPLE["label"], 40, INK, a, bold=True, anchor="mm")
    branches = [("강공", 300, EXAMPLE["hit"]), ("희생번트", 780, EXAMPLE["bunt"])]
    b = ease((t - 0.8) / 0.6)
    for name, cx, wins in branches:
        if b > 0:
            cv.line(bezier((540, 600), (540, 650), (cx, 640), (cx, 690), f=b), INK2, 1.0, 3)
        if b >= 1:
            pw = cv.width(name, 36, True) + 44
            cv.rect(cx - pw / 2, 690, cx + pw / 2, 750, (44, 44, 42), 1.0, r=30)
            cv.text(cx, 720, name, 36, INK, 1.0, bold=True, anchor="mm")
    # 200 rollouts per branch as a unit chart: they finish in random order, then line up
    # wins (blue) first.
    cols, cell, gy = 20, 18, 800
    for bi, (name, cx, wins) in enumerate(branches):
        gx = cx - cols * cell / 2
        rng = random.Random(11 + bi)
        start_slot = list(range(200))
        rng.shuffle(start_slot)
        shown = list(range(200))
        rng.shuffle(shown)
        when = {j: 1.4 + bi * 0.2 + 2.0 * k / 200 for k, j in enumerate(shown)}
        if b >= 1:
            cv.line([(cx, 752), (cx, gy - 16)], INK2, smooth((t - 1.4) / 0.3), 3)
        m = ease((t - 3.9 - bi * 0.1) / 0.8)
        for j in range(200):
            s0 = smooth((t - when[j]) / 0.12)
            if s0 <= 0:
                continue
            a0, a1 = start_slot[j], j
            x = gx + cell / 2 + ((a0 % cols) * (1 - m) + (a1 % cols) * m) * cell
            y = gy + cell / 2 + ((a0 // cols) * (1 - m) + (a1 // cols) * m) * cell
            cv.dot(x, y, 7 * s0, BLUE if j < wins else NEUTRAL, 1.0)
        g = smooth((t - 4.7 - bi * 0.2) / 0.4)
        if g > 0:
            cv.text(cx, gy + 10 * cell + 26, f"승 {wins} · 패 {200 - wins}", 34, INK2, g, anchor="ma")
            cv.text(cx, gy + 10 * cell + 78, f"승률 {wins / 2:.1f}%", 42, INK, g, bold=True, anchor="ma")
    k = smooth((t - 1.2) / 0.4)
    cv.dot(96, 1186, 8, BLUE, k)
    cv.text(114, 1186, "이긴 시뮬레이션", 30, INK2, k, anchor="lm")
    cv.dot(366, 1186, 8, NEUTRAL, k)
    cv.text(384, 1186, "진 시뮬레이션 (점 하나 = 1번)", 30, INK2, k, anchor="lm")
    caption(cv, t, 5.3, ["이 장면만 보면 번트가 3%p 더 이겼다.", "진짜 이득일까?"], y=1300)
    footer(cv)


def scene_dots(cv, t, dur, diffs):
    header(cv, t, "같은 상황 300장면 (무사 1루 · 7회 이후 · 1점 차 이내)", "300장면을 모으면")
    base, x_of = 1110, (lambda v: 540 + v * 30)
    # axis
    a = smooth((t - 0.2) / 0.4)
    cv.line([(110, base + 10), (970, base + 10)], NEUTRAL, a, 2)
    for v in (-10, -5, 0, 5, 10):
        cv.line([(x_of(v), base + 10), (x_of(v), base + 22)], NEUTRAL, a, 2)
        lab = "0" if v == 0 else (f"+{v}" if v > 0 else f"−{-v}")
        cv.text(x_of(v), base + 30, lab, 30, INK2, a, anchor="ma")
    cv.text(540, base + 78, "번트 − 강공 · 승리 확률 차이 (%p)", 30, MUTED, a, anchor="ma")
    # dots stacked by value (multiples of 0.5 %p, one rollout = 0.5 %p)
    rng = random.Random(3)
    order = list(range(len(diffs)))
    rng.shuffle(order)
    appear = {idx: 0.5 + 2.4 * k / len(order) for k, idx in enumerate(order)}
    stacks = {}
    for idx in sorted(range(len(diffs)), key=lambda i: appear[i]):
        v = diffs[idx]
        h = stacks.get(v, 0)
        stacks[v] = h + 1
        s = smooth((t - appear[idx]) / 0.15)
        if s <= 0:
            continue
        col = BLUE if v > 0 else (RED if v < 0 else NEUTRAL)
        cv.dot(x_of(v), base - 7 - h * 14, 6 * s, col, 0.9)
    n = len(diffs)
    up = sum(v > 0 for v in diffs) / n
    dn = sum(v < 0 for v in diffs) / n
    eq = 1 - up - dn
    g = smooth((t - 3.0) / 0.4)
    if g > 0:
        cv.dot(100, 610, 9, RED, g)
        cv.text(120, 610, f"강공이 나아 보인 장면 {dn:.0%}", 34, INK, g, anchor="lm")
        cv.dot(100, 664, 9, BLUE, g)
        cv.text(120, 664, f"번트가 나아 보인 장면 {up:.0%}", 34, INK, g, anchor="lm")
        cv.dot(100, 718, 9, NEUTRAL, g)
        cv.text(120, 718, f"같음 {eq:.0%}", 34, INK2, g, anchor="lm")
    m = sum(diffs) / n
    sd = (sum((v - m) ** 2 for v in diffs) / (n - 1)) ** 0.5
    se = sd / n ** 0.5
    k = smooth((t - 4.0) / 0.5)
    if k > 0:
        xm = x_of(m)
        cv.rect(x_of(m - 1.96 * se), base + 2, x_of(m + 1.96 * se), base + 18, INK, k * 0.9)
        cv.line([(xm, 780), (xm, base + 10)], INK, k, 3)
        cv.text(xm - 14, 790, f"평균 {m:+.2f}%p".replace("-", "−"), 38, INK, k, bold=True, anchor="ra")
        cv.text(xm - 14, 840, "흰 띠: 95% 신뢰구간", 28, INK2, k, anchor="ra")
    caption(cv, t, 4.9, ["한 장면의 차이는 대부분 운.", "평균하면 번트가 0.5%p 손해."])
    footer(cv)


def scene_bunt(cv, t, dur):
    header(cv, t, "실험 1 · 희생번트", "희생번트는 강공보다\n얼마나 이길까?", "강공 대비 승리 확률 차이 · 모두 점수 차 1점 이내")
    x0, scale, top, rh = 760, 250, 610, 118
    a = smooth((t - 0.3) / 0.4)
    cv.line([(x0, top - 20), (x0, top + rh * len(BUNT_ROWS) - 20)], INK2, a, 2)
    for i, (name, v, se) in enumerate(BUNT_ROWS):
        y = top + i * rh
        cv.text(80, y, name, 36, INK, a, bold=False)
        g = ease((t - 0.7 - i * 0.25) / 0.8)
        xv = x0 + v * scale * g
        cv.rect(x0, y + 50, xv, y + 88, RED if v < 0 else BLUE, a, r=4)
        w = smooth((t - 1.6 - i * 0.25) / 0.4)
        if w > 0:
            lo, hi = x0 + (v - 1.96 * se) * scale, x0 + (v + 1.96 * se) * scale
            cv.line([(lo, y + 69), (hi, y + 69)], INK, w, 2)
            cv.line([(lo, y + 59), (lo, y + 79)], INK, w, 2)
            cv.line([(hi, y + 59), (hi, y + 79)], INK, w, 2)
            cv.text(RIGHT, y, f"{v:+.2f}%p".replace("-", "−"), 36, INK, w, bold=True, anchor="ra")
    yb = top + rh * len(BUNT_ROWS) - 6
    cv.text(x0 - 12, yb, "← 강공이 낫다", 28, MUTED, a, anchor="ra")
    cv.text(x0 + 12, yb, "번트가 낫다 →", 28, MUTED, a, anchor="la")
    cv.text(80, yb + 42, "가로선: 95% 신뢰구간 · 상황마다 300장면 × 200번", 26, MUTED, a)
    caption(cv, t, 4.2, ["주자가 1루나 2루에만 있으면 손해.", "무사 1·2루만 본전."], y=1322, size=42)
    footer(cv)


def scene_steal(cv, t, dur):
    header(cv, t, "실험 2 · 도루", "첫 공에 무조건\n뛰게 하면?", "2루 도루 성공률 ('약'은 득점 변화에서 거꾸로 푼 값)")
    x_of = lambda p: 100 + p / 100 * 700
    top, rh = 700, 170
    a = smooth((t - 0.3) / 0.4)
    # break-even band
    xb = x_of(BREAK_EVEN)
    for i in range(3):  # only across the bars, so it never runs through a label
        cv.dashed(xb, top + i * rh + 42, xb, top + i * rh + 106, INK, a, 2, dash=8, gap=6)
    cv.text(xb, top + rh * 3 - 30, "손익분기 68.4%", 28, INK, a, bold=True, anchor="ma")
    for i, (name, p, lab) in enumerate(STEAL_ROWS):
        y = top + i * rh
        cv.text(80, y, name, 34, INK, a)
        g = ease((t - 0.8 - i * 0.35) / 0.9)
        cv.rect(x_of(0), y + 52, x_of(p * g), y + 96, RED if p < BREAK_EVEN else BLUE, a, r=4)
        cv.text(RIGHT, y + 74, lab, 36, INK, smooth((t - 1.5 - i * 0.35) / 0.3), bold=True, anchor="rm")
    yt = top + rh * 3 - 30
    cv.text(x_of(0), yt, "0%", 28, INK2, a, anchor="ma")
    cv.text(x_of(100), yt, "100%", 28, INK2, a, anchor="ma")
    caption(cv, t, 4.0, ["억지로 보내면 손해,", "주자 판단에 맡기면 이득."], y=1300, size=42)
    if t > 5.0:
        cv.text(80, 1422, "무사 1루 억지 도루: 승리 확률 −2.8 ~ −3.8%p", 30, INK2, smooth((t - 5.0) / 0.4))
    footer(cv)


def scene_rl(cv, t, dur):
    header(cv, t, "실험 3 · 강화학습", "AI에게 4만 경기를\n학습시켰더니", "같은 2만 경기로 평가한 승률")
    x_of = lambda v: 80 + v / 0.6 * 820
    top, rh = 690, 168
    a = smooth((t - 0.3) / 0.4)
    xr = x_of(RL_ROWS[0][1])
    cv.dashed(xr, top - 10, xr, top + rh * 4 - 40, INK2, a, 2)
    for i, (name, v, note, col) in enumerate(RL_ROWS):
        y = top + i * rh
        cv.text(80, y, name, 36, INK, a, bold=True)
        g = ease((t - 0.8 - i * 0.4) / 0.9)
        cv.rect(x_of(0), y + 50, x_of(v * g), y + 92, col, a, r=4)
        h = smooth((t - 1.6 - i * 0.4) / 0.4)
        cv.text(RIGHT, y, f"{v:.3f}", 36, INK, h, bold=True, anchor="ra")
        if note:
            cv.text(80, y + 104, note, 30, INK2, h)
    caption(cv, t, 4.6, ["작전 사이 차이(0.5~5%p)가", "한 경기의 운보다 훨씬 작아 AI도 착각한다."], y=1352, size=40)
    footer(cv)


def scene_end(cv, t, dur):
    a = smooth(t / 0.4)
    cv.text(80, 300, "결론", 36, MUTED, a, bold=True)
    for i, line in enumerate(["이 엔진 안에서는", "강공보다 나은 작전이", "없었다"]):
        cv.text(80, 360 + i * 100, line, 80, INK, a, bold=True)
    items = ["희생번트: 무사 1·2루만 본전, 나머지는 손해", "억지 도루: 성공률 약 41% < 손익분기 68.4%",
             "강화학습: 강공을 넘는 정책을 못 찾음"]
    for i, s in enumerate(items):
        b = smooth((t - 0.8 - i * 0.35) / 0.4)
        cv.dot(92, 732 + i * 76, 7, INK2, b)
        cv.text(116, 732 + i * 76, s, 36, INK, b, anchor="lm")
    c = smooth((t - 2.4) / 0.4)
    for i, s in enumerate(["가상 선수 · 가상 리그 시뮬레이션 결과입니다.", "상대의 번트 수비 변화와 견제는 반영하지 않았습니다.",
                           "엔진의 번트·도루 모델은 리그 평균 빈도와 결과에만 맞췄습니다."]):
        cv.text(80, 1040 + i * 48, s, 30, INK2, c)
    cv.text(80, 1236, "모션 데이터: CMU Graphics Lab Motion Capture Database (mocap.cs.cmu.edu)", 24, MUTED, c)


SCENES = [(4.2, "hook"), (8.0, "branch"), (7.6, "dots"), (8.6, "bunt"), (8.2, "steal"), (8.6, "rl"), (6.4, "end")]
HOOK_TOP = 430  # top of the opening shot; its upper edge fades into the background under the title


def render(t, diffs, clip):
    acc = 0.0
    for dur, name in SCENES:
        if t < acc + dur or name == SCENES[-1][1]:
            lt = t - acc
            fade = clamp(min((dur - lt) / 0.3, 1.0 if name == "hook" else lt / 0.3))
            cv = Canvas(alpha=fade)
            if name == "hook":
                scene_hook(cv, lt, dur, clip)
                if fade < 1:  # fade the picture too
                    cv.img = Image.blend(Image.new("RGB", cv.img.size, BG), cv.img, fade)
            elif name == "dots":
                scene_dots(cv, lt, dur, diffs)
            else:
                globals()["scene_" + name](cv, lt, dur)
            return cv.frame()
        acc += dur


def main():
    diffs = load_rollouts(sys.argv[1])
    clip_dir = sys.argv[2]
    clip = sorted(os.path.join(clip_dir, f) for f in os.listdir(clip_dir) if f.endswith(".png"))
    # HOOK_FIRST / HOOK_LAST (1-based, inclusive) pick the frames to play; the last one is held.
    first = int(os.environ.get("HOOK_FIRST", "1"))
    last = int(os.environ.get("HOOK_LAST", str(len(clip))))
    clip = clip[first - 1:last]
    total = sum(d for d, _ in SCENES)
    if "--stills" in sys.argv:
        out = sys.argv[3]
        os.makedirs(out, exist_ok=True)
        for s in sys.argv[sys.argv.index("--stills") + 1].split(","):
            render(float(s), diffs, clip).save(os.path.join(out, f"t{float(s):05.1f}.png"))
        return
    n = int(round(total * FPS))
    ff = subprocess.Popen(["ffmpeg", "-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS),
                           "-i", "-", "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p",
                           "-movflags", "+faststart", sys.argv[3]], stdin=subprocess.PIPE)
    for k in range(n):
        ff.stdin.write(render(k / FPS, diffs, clip).tobytes())
        if k % 150 == 0:
            print("frame", k, "of", n, flush=True)
    ff.stdin.close()
    ff.wait()
    print("total seconds", total, "frames", n)


if __name__ == "__main__":
    main()
