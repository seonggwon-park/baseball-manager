"""Scenario: the temporary ballpark of the two Seoul clubs that leave Jamsil (2027-2031).

Plays the KBO level's clubs (levels.json talent, foreign players included) in one park at a
time with `bm park` - the game's own engine and park geometry - for Jamsil (2026), the
temporary park with a range of wall heights and alley distances (neither is published), and
every KBO home park of the 2027 world data (data/leagues/kr.json) for the league average.
Writes OUTDIR/<run>.json (one per park, kept and reused), OUTDIR/results.json, a markdown
table on stdout and the chart OUTDIR/jamsil-hr-by-wall.svg.

With --balls FILE (statcast_balls.py) the same parks are also checked with measured balls
instead of the engine's batters: every ball of the file flown through each park with
`bm park --balls`, once as measured (major-league hitters) and once with the exit speed
changed by --ev-shift mph (default -3.7: the engine's KBO-level batters hit 85.6 mph on
average at Jamsil, its major-league batters 89.3 mph; bm park, seed 7, 3,000 games each).

usage: jamsil_park.py BM OUTDIR [--games N] [--seed S] [--threads T] [--balls FILE [--ev-shift MPH]]"""
import argparse
import json
import os
import subprocess

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
JAMSIL = ([100, 120, 125, 120, 100], [2.6, 2.6, 2.6])  # docs/research/world/kbo_clubs.json
TEMP_LINE, TEMP_CF, TEMP_ALLEY = 95, 124, 113.4  # lines and centre reported; alleys filled in as the world data does
WALLS = [2.6, 4.0, 6.0, 7.0, 8.0, 9.0]
ALLEYS = [111.4, 115.4]
ELEVATION = 22


def runs():
    out = [('jamsil-2026', 'Jamsil (2026)', JAMSIL[0], JAMSIL[1], ELEVATION, False)]
    for w in WALLS:
        out.append((f'temp-wall{w}', f'temporary park, wall {w} m', [TEMP_LINE, TEMP_ALLEY, TEMP_CF, TEMP_ALLEY, TEMP_LINE], [w] * 3, ELEVATION, False))
    for a in ALLEYS:
        out.append((f'temp-alley{a}', f'temporary park, alleys {a} m, wall 4 m', [TEMP_LINE, a, TEMP_CF, a, TEMP_LINE], [4.0] * 3, ELEVATION, False))
    seen = set()
    for c in json.load(open(os.path.join(REPO, 'data/leagues/kr.json')))['clubs']:
        if c['league'] != 'kr-top':
            continue
        p = c['park']
        key = (tuple(p['fence_m']), tuple(p['wall_m']))
        if key in seen:
            continue
        seen.add(key)
        out.append((c['id'], p['name']['en'], p['fence_m'], p['wall_m'], p.get('elevation_m') or 0, p['surface'] == 'turf'))
    return out


def park_args(run):
    _, _, fence, wall, elev, turf = run
    cmd = ['--level', 'kbo', '--fence', ','.join(str(x) for x in fence), '--wall', ','.join(str(x) for x in wall), '--elevation', str(elev)]
    return cmd + (['--turf'] if turf else [])


def play(bm, outdir, run, games, seed, threads):
    path = os.path.join(outdir, run[0] + '.json')
    if not os.path.exists(path):
        cmd = [bm, 'park', *park_args(run), '--games', str(games), '--seed', str(seed), '--threads', str(threads), '--json', path]
        subprocess.run(cmd, check=True, capture_output=True)
    d = json.load(open(path))
    tg = d['team_games']
    return {'hr': 2 * d['home_runs'] / tg, 'r': 2 * d['runs'] / tg, 'doubles': 2 * d['doubles'] / tg, 'triples': 2 * d['triples'] / tg,
            'avg': d['avg'], 'slg': d['slg'], 'games': d['games']}


def balls(bm, outdir, run, path, shift):
    """Home runs of the measured balls flown through the park, exit speed changed by `shift` mph."""
    out = os.path.join(outdir, f'{run[0]}-balls{shift:+.1f}.json')
    subprocess.run([bm, 'park', *park_args(run), '--balls', path, '--ev-shift', str(shift), '--json', out], check=True, capture_output=True)
    return json.load(open(out))['home_runs']


def equal_wall(pts, jam):
    """Wall height that gives Jamsil's value, by linear interpolation between the runs around it."""
    for (a, ha), (b, hb) in zip(pts, pts[1:]):
        if ha >= jam >= hb:
            return a + (ha - jam) / (ha - hb) * (b - a)
    return None


def chart(res, path, shift):
    """Static SVG: home runs at the temporary park against its wall height, Jamsil = 100, for the
    engine's games and (when run) the measured balls."""
    series = [('hr', '엔진 경기 (KBO 수준 선수)', '#2a78d6', 'circle')]
    if 'balls_hr' in res['jamsil-2026']:
        series += [('balls_hr', '실측 타구 (MLB 2025)', '#eb6834', 'square'),
                   ('balls_hr_shift', f'실측 타구, 타구 속도 {shift:+.1f} mph'.replace('-', '−'), '#1baf7a', 'triangle')]
    W, H = 860, 540
    L, R, T, B = 64, 250, 118, 64
    x0, x1, y0, y1 = 2.0, 9.5, 60.0, 220.0
    X = lambda v: L + (v - x0) / (x1 - x0) * (W - L - R)
    Y = lambda v: H - B - (v - y0) / (y1 - y0) * (H - T - B)
    lines = {k: [(w, 100 * res[f'temp-wall{w}'][k] / res['jamsil-2026'][k]) for w in WALLS] for k, *_ in series}
    eqs = [e for e in (equal_wall(lines[k], 100.0) for k, *_ in series) if e is not None]
    font = "font-family=\"'Noto Sans CJK KR','Apple SD Gothic Neo','Malgun Gothic',sans-serif\""
    s = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" {font}>',
         f'<rect width="{W}" height="{H}" fill="#fcfcfb"/>',
         '<text x="24" y="34" font-size="20" font-weight="700" fill="#0b0b0b">임시 구장 담장 높이별 홈런 (잠실 = 100, 모델)</text>',
         '<text x="24" y="57" font-size="13" fill="#52514e">좌우 95 m, 좌우중간 113.4 m(추정), 중앙 124 m. 모든 구장 같은 날씨(24 ℃, 바람 없음), 같은 타자·타구.</text>']
    # Legend, one row.
    lx = 24
    for k, label, color, shape in series:
        s.append(marker(shape, lx + 6, 84, color, 4.5))
        s.append(f'<text x="{lx + 16}" y="88" font-size="12" fill="#52514e">{label}</text>')
        lx += 34 + sum(12.5 if '\uac00' <= ch <= '\ud7a3' else 6.8 for ch in label)
    for v in (60, 80, 100, 120, 140, 160, 180, 200, 220):
        s.append(f'<line x1="{L}" x2="{W - R}" y1="{Y(v):.1f}" y2="{Y(v):.1f}" stroke="#e4e3df" stroke-width="1"/>')
        s.append(f'<text x="{L - 10}" y="{Y(v) + 4:.1f}" font-size="12" fill="#52514e" text-anchor="end">{v}</text>')
    for v in (2, 3, 4, 5, 6, 7, 8, 9):
        s.append(f'<text x="{X(v):.1f}" y="{H - B + 20}" font-size="12" fill="#52514e" text-anchor="middle">{v}</text>')
    s.append(f'<text x="{(L + W - R) / 2:.1f}" y="{H - 22}" font-size="13" fill="#52514e" text-anchor="middle">임시 구장 담장 높이 (m)</text>')
    if eqs:
        a, b = min(eqs), max(eqs)
        s.append(f'<rect x="{X(a) - 1:.1f}" y="{T:.1f}" width="{X(b) - X(a) + 2:.1f}" height="{H - B - T:.1f}" fill="#e4e3df" opacity="0.6"/>')
        txt = f'잠실과 같아지는 높이 {a:.1f}–{b:.1f} m' if b - a >= 0.05 else f'잠실과 같아지는 높이 {a:.1f} m'
        s.append(f'<text x="{X(b) + 6:.1f}" y="{T + 16:.1f}" font-size="12" fill="#52514e">{txt}</text>')
    s.append(f'<line x1="{L}" x2="{W - R}" y1="{Y(100):.1f}" y2="{Y(100):.1f}" stroke="#52514e" stroke-width="1.5" stroke-dasharray="6 4"/>')
    s.append(f'<text x="{L + 6}" y="{Y(100) + 16:.1f}" font-size="12" fill="#52514e">잠실(2026) = 100</text>')
    s.append(f'<line x1="{X(4):.1f}" x2="{X(4):.1f}" y1="{T:.1f}" y2="{H - B}" stroke="#8a8984" stroke-width="1" stroke-dasharray="2 3"/>')
    s.append(f'<text x="{X(4) + 6:.1f}" y="{H - B - 8}" font-size="12" fill="#52514e">게임 자료의 가정 4 m</text>')
    ends = []
    for k, label, color, shape in series:
        pts = lines[k]
        d = ' '.join(('M' if i == 0 else 'L') + f'{X(w):.1f},{Y(h):.1f}' for i, (w, h) in enumerate(pts))
        s.append(f'<path d="{d}" fill="none" stroke="{color}" stroke-width="2" stroke-linejoin="round"/>')
        for w, h in pts:
            s.append(marker(shape, X(w), Y(h), color, 5))
        ends.append([Y(pts[-1][1]), label, color, pts[-1][1], pts[0][1]])
    # Direct labels at the right end, spread so they do not overlap.
    ends.sort()
    for i in range(1, len(ends)):
        ends[i][0] = max(ends[i][0], ends[i - 1][0] + 30)
    xe = X(WALLS[-1])
    for y, label, color, last, first in ends:
        s.append(f'<text x="{xe + 14:.1f}" y="{y + 4:.1f}" font-size="12" fill="#0b0b0b">{label}</text>')
        s.append(f'<text x="{xe + 14:.1f}" y="{y + 19:.1f}" font-size="11" fill="#52514e">2.6 m {first:.0f} · 9 m {last:.0f}</text>')
    s.append(f'<text x="24" y="{H - 4}" font-size="11" fill="#8a8984">bm park. 담장 높이·좌우중간 거리는 발표되지 않았다. 이 구장 모델은 MLB 구장별 홈런 차이를 잘 맞히지 못한다(본문 검증).</text>')
    s.append('</svg>')
    open(path, 'w').write('\n'.join(s) + '\n')
    return eqs


def marker(shape, x, y, color, r):
    ring = 'stroke="#fcfcfb" stroke-width="2"'
    if shape == 'square':
        return f'<rect x="{x - r:.1f}" y="{y - r:.1f}" width="{2 * r:.1f}" height="{2 * r:.1f}" rx="1" fill="{color}" {ring}/>'
    if shape == 'triangle':
        return f'<path d="M{x:.1f},{y - r * 1.2:.1f} L{x + r * 1.1:.1f},{y + r * 0.8:.1f} L{x - r * 1.1:.1f},{y + r * 0.8:.1f} Z" fill="{color}" {ring}/>'
    return f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r}" fill="{color}" {ring}/>'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('bm')
    ap.add_argument('outdir')
    ap.add_argument('--games', type=int, default=20000)
    ap.add_argument('--seed', type=int, default=7)
    ap.add_argument('--threads', type=int, default=4)
    ap.add_argument('--balls')
    ap.add_argument('--ev-shift', type=float, default=-3.7)
    a = ap.parse_args()
    os.makedirs(a.outdir, exist_ok=True)
    res, names = {}, {}
    for run in runs():
        res[run[0]] = play(a.bm, a.outdir, run, a.games, a.seed, a.threads)
        names[run[0]] = run[1]
        if a.balls:
            res[run[0]]['balls_hr'] = balls(a.bm, a.outdir, run, a.balls, 0.0)
            res[run[0]]['balls_hr_shift'] = balls(a.bm, a.outdir, run, a.balls, a.ev_shift)
    # League average of the 2027 world: every club's home park once (the shared Seoul park twice).
    clubs = [c for c in json.load(open(os.path.join(REPO, 'data/leagues/kr.json')))['clubs'] if c['league'] == 'kr-top']
    homes = []
    for c in clubs:
        for rid, *_ in runs():
            if rid in names and not rid.startswith(('jamsil', 'temp')):
                p = c['park']
                if [x for x in json.load(open(os.path.join(a.outdir, rid + '.json')))['fence_m']] == p['fence_m'] and json.load(open(os.path.join(a.outdir, rid + '.json')))['wall_m'] == p['wall_m']:
                    homes.append(rid)
                    break
    league_hr = sum(res[h]['hr'] for h in homes) / len(homes)
    league_r = sum(res[h]['r'] for h in homes) / len(homes)
    eqs = chart(res, os.path.join(a.outdir, 'jamsil-hr-by-wall.svg'), a.ev_shift)
    eq = equal_wall([(w, res[f'temp-wall{w}']['hr']) for w in WALLS], res['jamsil-2026']['hr'])
    json.dump({'results': res, 'names': names, 'league_hr': league_hr, 'league_r': league_r, 'wall_equal_to_jamsil': eq, 'wall_equal_all': eqs},
              open(os.path.join(a.outdir, 'results.json'), 'w'), indent=1)
    print(f'League average of the 2027 home parks ({len(homes)} clubs): {league_hr:.3f} home runs and {league_r:.2f} runs a game')
    print('| 구장 | 경기당 홈런 | 홈런 팩터 | 경기당 득점 | 득점 팩터 | 경기당 2루타 | 경기당 3루타 |')
    print('|---|---|---|---|---|---|---|')
    for rid in sorted(res, key=lambda k: -res[k]['hr']):
        r = res[rid]
        print(f"| {names[rid]} | {r['hr']:.2f} | {100 * r['hr'] / league_hr:.0f} | {r['r']:.2f} | {100 * r['r'] / league_r:.0f} | {r['doubles']:.2f} | {r['triples']:.2f} |")
    if eq is not None:
        print(f'Wall height that gives Jamsil\'s home runs: {eq:.1f} m')
    if a.balls:
        jam = res['jamsil-2026']
        print(f'\n| 구장 | 엔진 경기 (잠실 = 100) | 실측 타구 그대로 (잠실 = 100) | 실측 타구, 타구 속도 {a.ev_shift:+.1f} mph (잠실 = 100) |')
        print('|---|---|---|---|')
        for rid in sorted(res, key=lambda k: -res[k]['hr']):
            r = res[rid]
            print(f"| {names[rid]} | {100 * r['hr'] / jam['hr']:.0f} | {100 * r['balls_hr'] / jam['balls_hr']:.0f} | {100 * r['balls_hr_shift'] / jam['balls_hr_shift']:.0f} |")
        print(f"Jamsil: {jam['balls_hr']} and {jam['balls_hr_shift']} home runs of the measured balls")


if __name__ == '__main__':
    main()
