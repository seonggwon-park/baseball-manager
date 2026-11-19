"""Scenario: the temporary ballpark of the two Seoul clubs that leave Jamsil (2027-2031).

Plays the KBO level's clubs (levels.json talent, foreign players included) in one park at a
time with `bm park` - the game's own engine and park geometry - for Jamsil (2026), the
temporary park with a range of wall heights and alley distances (neither is published), and
every KBO home park of the 2027 world data (data/leagues/kr.json) for the league average.
Writes OUTDIR/<run>.json (one per park, kept and reused), OUTDIR/results.json, a markdown
table on stdout and the chart OUTDIR/jamsil-hr-by-wall.svg.

usage: jamsil_park.py BM OUTDIR [--games N] [--seed S] [--threads T]"""
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


def play(bm, outdir, run, games, seed, threads):
    rid, _, fence, wall, elev, turf = run
    path = os.path.join(outdir, rid + '.json')
    if not os.path.exists(path):
        cmd = [bm, 'park', '--level', 'kbo', '--fence', ','.join(str(x) for x in fence), '--wall', ','.join(str(x) for x in wall),
               '--elevation', str(elev), '--games', str(games), '--seed', str(seed), '--threads', str(threads), '--json', path]
        if turf:
            cmd.append('--turf')
        subprocess.run(cmd, check=True, capture_output=True)
    d = json.load(open(path))
    tg = d['team_games']
    return {'hr': 2 * d['home_runs'] / tg, 'r': 2 * d['runs'] / tg, 'doubles': 2 * d['doubles'] / tg, 'triples': 2 * d['triples'] / tg,
            'avg': d['avg'], 'slg': d['slg'], 'games': d['games']}


def chart(res, league_hr, path):
    """Static SVG: home runs per game against the temporary park's wall height."""
    W, H = 760, 460
    L, R, T, B = 70, 40, 92, 70
    x0, x1, y0, y1 = 2.0, 9.5, 0.0, 2.5
    X = lambda v: L + (v - x0) / (x1 - x0) * (W - L - R)
    Y = lambda v: H - B - (v - y0) / (y1 - y0) * (H - T - B)
    pts = [(w, res[f'temp-wall{w}']['hr']) for w in WALLS]
    jam = res['jamsil-2026']['hr']
    # Wall height that gives Jamsil's rate, by linear interpolation between the runs around it.
    eq = None
    for (a, ha), (b, hb) in zip(pts, pts[1:]):
        if ha >= jam >= hb:
            eq = a + (ha - jam) / (ha - hb) * (b - a)
    font = "font-family=\"'Noto Sans CJK KR','Apple SD Gothic Neo','Malgun Gothic',sans-serif\""
    s = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" {font}>',
         f'<rect width="{W}" height="{H}" fill="#fcfcfb"/>',
         '<text x="24" y="34" font-size="20" font-weight="700" fill="#0b0b0b">임시 구장 담장 높이별 경기당 홈런 (모델)</text>',
         f'<text x="24" y="58" font-size="13" fill="#52514e">KBO 수준 선수, 구장마다 {res["jamsil-2026"]["games"]:,}경기, 두 팀 합계. 좌우 95 m, 좌우중간 113.4 m(추정), 중앙 124 m</text>']
    for v in (0.0, 0.5, 1.0, 1.5, 2.0, 2.5):
        s.append(f'<line x1="{L}" x2="{W - R}" y1="{Y(v):.1f}" y2="{Y(v):.1f}" stroke="#e4e3df" stroke-width="1"/>')
        s.append(f'<text x="{L - 10}" y="{Y(v) + 4:.1f}" font-size="12" fill="#52514e" text-anchor="end">{v:.1f}</text>')
    for v in (2, 3, 4, 5, 6, 7, 8, 9):
        s.append(f'<text x="{X(v):.1f}" y="{H - B + 20}" font-size="12" fill="#52514e" text-anchor="middle">{v}</text>')
    s.append(f'<text x="{(L + W - R) / 2:.1f}" y="{H - 24}" font-size="13" fill="#52514e" text-anchor="middle">임시 구장 담장 높이 (m)</text>')
    s.append(f'<text x="18" y="{(T + H - B) / 2:.1f}" font-size="13" fill="#52514e" text-anchor="middle" transform="rotate(-90 18 {(T + H - B) / 2:.1f})">경기당 홈런</text>')
    for val, label, anchor in ((jam, f'잠실(2026) {jam:.2f}', 'end'), (league_hr, f'KBO 평균(2027 구장) {league_hr:.2f}', 'end')):
        s.append(f'<line x1="{L}" x2="{W - R}" y1="{Y(val):.1f}" y2="{Y(val):.1f}" stroke="#8a8984" stroke-width="1.5" stroke-dasharray="6 4"/>')
        s.append(f'<text x="{W - R - 4}" y="{Y(val) - 6:.1f}" font-size="12" fill="#52514e" text-anchor="{anchor}">{label}</text>')
    if eq is not None:
        s.append(f'<line x1="{X(eq):.1f}" x2="{X(eq):.1f}" y1="{Y(jam):.1f}" y2="{H - B}" stroke="#8a8984" stroke-width="1" stroke-dasharray="2 3"/>')
        s.append(f'<text x="{X(eq) + 6:.1f}" y="{H - B - 8}" font-size="12" fill="#52514e">잠실과 같아지는 높이 ≈ {eq:.1f} m</text>')
    path_d = ' '.join(('M' if i == 0 else 'L') + f'{X(w):.1f},{Y(h):.1f}' for i, (w, h) in enumerate(pts))
    s.append(f'<path d="{path_d}" fill="none" stroke="#2a78d6" stroke-width="2.5" stroke-linejoin="round"/>')
    for w, h in pts:
        s.append(f'<circle cx="{X(w):.1f}" cy="{Y(h):.1f}" r="5" fill="#2a78d6" stroke="#fcfcfb" stroke-width="2"/>')
    for w, note in ((2.6, ''), (4.0, ' (게임 자료의 가정)'), (6.0, '')):
        h = res[f'temp-wall{w}']['hr']
        s.append(f'<text x="{X(w) + 8:.1f}" y="{Y(h) - 9:.1f}" font-size="12" fill="#0b0b0b">{h:.2f}{note}</text>')
    s.append(f'<text x="24" y="{H - 6}" font-size="11" fill="#8a8984">bm park, 같은 선수(시드 고정). 담장 높이와 좌우중간 거리는 발표되지 않아 범위로 보였다.</text>')
    s.append('</svg>')
    open(path, 'w').write('\n'.join(s) + '\n')
    return eq


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('bm')
    ap.add_argument('outdir')
    ap.add_argument('--games', type=int, default=20000)
    ap.add_argument('--seed', type=int, default=7)
    ap.add_argument('--threads', type=int, default=4)
    a = ap.parse_args()
    os.makedirs(a.outdir, exist_ok=True)
    res, names = {}, {}
    for run in runs():
        res[run[0]] = play(a.bm, a.outdir, run, a.games, a.seed, a.threads)
        names[run[0]] = run[1]
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
    eq = chart(res, league_hr, os.path.join(a.outdir, 'jamsil-hr-by-wall.svg'))
    json.dump({'results': res, 'names': names, 'league_hr': league_hr, 'league_r': league_r, 'wall_equal_to_jamsil': eq}, open(os.path.join(a.outdir, 'results.json'), 'w'), indent=1)
    print(f'League average of the 2027 home parks ({len(homes)} clubs): {league_hr:.3f} home runs and {league_r:.2f} runs a game')
    print('| 구장 | 경기당 홈런 | 홈런 팩터 | 경기당 득점 | 득점 팩터 | 경기당 2루타 | 경기당 3루타 |')
    print('|---|---|---|---|---|---|---|')
    for rid in sorted(res, key=lambda k: -res[k]['hr']):
        r = res[rid]
        print(f"| {names[rid]} | {r['hr']:.2f} | {100 * r['hr'] / league_hr:.0f} | {r['r']:.2f} | {100 * r['r'] / league_r:.0f} | {r['doubles']:.2f} | {r['triples']:.2f} |")
    if eq is not None:
        print(f'Wall height that gives Jamsil\'s home runs: {eq:.1f} m')


if __name__ == '__main__':
    main()
