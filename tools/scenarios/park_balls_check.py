"""Check of `bm park --balls`: the measured balls of each major-league park (the output of
statcast_balls.py) flown through that park as the game data describes it
(data/leagues/us.json), and the home runs counted against the real ones.

The 2025 Rays played in Tampa (Steinbrenner Field) while the game data has their 2026 park,
so their home games are left out. Every park's balls are flown three ways: in the game data's
park and the game-average weather (24 C, no wind), in the same park and each game's reported
weather (mlb_weather.py: temperature, and wind speed and direction as the club reports it),
and in the average park (330/377/404 ft, 8.5 ft wall, 180 m) with each game's weather. The
last one shows how much of the park-to-park difference the park's shape explains.

usage: park_balls_check.py BM BALLS.csv WEATHER.json OUTDIR"""
import json
import math
import os
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# Statcast home-team code -> game-data club whose park it is (2025 parks; real cities).
CLUBS = {
    'BAL': 'baltimore-schooners', 'BOS': 'boston-lanterns', 'NYY': 'new-york-empires', 'TOR': 'toronto-voyageurs',
    'CWS': 'chicago-sentinels', 'CLE': 'cleveland-forgers', 'DET': 'detroit-motormen', 'KC': 'kansas-city-plainsmen',
    'MIN': 'minneapolis-lumberjacks', 'ATH': 'sacramento-prospectors', 'HOU': 'houston-wildcatters', 'LAA': 'anaheim-surfriders',
    'SEA': 'seattle-evergreens', 'TEX': 'arlington-wranglers', 'ATL': 'atlanta-firebirds', 'MIA': 'miami-flamingos',
    'NYM': 'new-york-ferrymen', 'PHI': 'philadelphia-liberties', 'WSH': 'washington-statesmen', 'CHC': 'chicago-gales',
    'CIN': 'cincinnati-steamers', 'MIL': 'milwaukee-harvesters', 'PIT': 'pittsburgh-ironclads', 'STL': 'st-louis-archers',
    'AZ': 'phoenix-scorpions', 'COL': 'denver-summits', 'LAD': 'los-angeles-palms', 'SD': 'san-diego-navigators',
    'SF': 'san-francisco-fogcutters',
}


# Wind direction as the clubs report it -> bearing the air moves toward (rad; 0 = out to
# centre field, + toward right field).
TOWARD = {'Out To CF': 0, 'Out To RF': 45, 'Out To LF': -45, 'In From CF': 180, 'In From LF': 135, 'In From RF': -135,
          'L To R': 90, 'R To L': -90}
AVERAGE_PARK = ([100.58, 114.91, 123.14, 114.91, 100.58], [2.59] * 3, 180)  # 330/377/404 ft, 8.5 ft wall


def weather(game):
    """(temperature C, wind x, wind y m/s) of a game, or None."""
    if not game or game.get('temp_f') in (None, ''):
        return None
    speed, _, label = (game.get('wind') or '0 mph, None').partition(', ')
    mph = float(speed.split()[0]) if speed.split() else 0.0
    b = TOWARD.get(label)
    v = 0.0 if b is None else mph * 0.44704
    rad = math.radians(b or 0)
    return (float(game['temp_f']) - 32) / 1.8, v * math.sin(rad), v * math.cos(rad)


def fly(bm, path, fence, wall, elevation, out):
    cmd = [bm, 'park', '--level', 'mlb', '--balls', path, '--fence', ','.join(map(str, fence)), '--wall', ','.join(map(str, wall)),
           '--elevation', str(elevation), '--json', out]
    subprocess.run(cmd, check=True, capture_output=True)
    return json.load(open(out))['home_runs']


def stats(rows, k):
    """Correlation of home runs per ball with the real ones across parks, sum of squared
    differences over the flown count, mean absolute log ratio, total."""
    xs = [r[k] / r[1] for r in rows]
    ys = [r[2] / r[1] for r in rows]
    mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
    cov = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    r = cov / math.sqrt(sum((x - mx) ** 2 for x in xs) * sum((y - my) ** 2 for y in ys))
    chi = sum((row[k] - row[2]) ** 2 / max(row[k], 1) for row in rows)
    mad = sum(abs(math.log(max(row[k], 1) / max(row[2], 1))) for row in rows) / len(rows)
    return r, chi, mad, sum(row[k] for row in rows)


def main():
    bm, balls, weather_file, outdir = sys.argv[1:5]
    os.makedirs(outdir, exist_ok=True)
    parks = {c['id']: c['park'] for c in json.load(open(os.path.join(REPO, 'data/leagues/us.json')))['clubs']}
    games = json.load(open(weather_file))
    by_team = {}
    for line in open(balls):
        by_team.setdefault(line.rstrip('\n').split(',')[6], []).append(line.rstrip('\n').split(','))
    rows = []
    for code, club in sorted(CLUBS.items()):
        lines = by_team.get(code, [])
        plain, windy = os.path.join(outdir, f'balls-{code}.csv'), os.path.join(outdir, f'balls-{code}-weather.csv')
        with open(plain, 'w') as f, open(windy, 'w') as g:
            for x in lines:
                f.write(','.join(x[:4]) + '\n')
                w = weather(games.get(x[7]))
                g.write(','.join(x[:4]) + ('' if w is None else ',%.1f,%.2f,%.2f' % w) + '\n')
        p = parks[club]
        elev = p.get('elevation_m') or 0
        base = fly(bm, plain, p['fence_m'], p['wall_m'], elev, os.path.join(outdir, f'park-{code}.json'))
        wet = fly(bm, windy, p['fence_m'], p['wall_m'], elev, os.path.join(outdir, f'park-weather-{code}.json'))
        avg = fly(bm, windy, *AVERAGE_PARK, os.path.join(outdir, f'average-weather-{code}.json'))
        real = sum(1 for x in lines if x[5] == 'home_run')
        rows.append((code, len(lines), real, base, wet, avg, 'wall' in p.get('estimated', [])))
    print('| park (Statcast code) | balls in play | home runs, real | flown: park, average weather | park, game weather | average park, game weather |')
    print('|---|---|---|---|---|---|')
    for code, n, real, base, wet, avg, est in rows:
        print(f"| {code}{' (wall estimated)' if est else ''} | {n} | {real} | {base} | {wet} | {avg} |")
    print(f"| all | {sum(r[1] for r in rows)} | {sum(r[2] for r in rows)} | {sum(r[3] for r in rows)} | {sum(r[4] for r in rows)} | {sum(r[5] for r in rows)} |")
    out = {}
    for k, name in ((3, 'park, average weather'), (4, 'park, game weather'), (5, 'average park, game weather')):
        r, chi, mad, tot = stats(rows, k)
        out[name] = {'r': r, 'chi2': chi, 'mean_abs_log_ratio': mad, 'home_runs': tot}
        print(f'{name}: correlation with the real home runs per ball across {len(rows)} parks {r:.2f}, '
              f'sum (flown - real)^2 / flown {chi:.1f}, mean |log(flown / real)| {mad:.3f}, home runs {tot}')
    json.dump({'rows': rows, 'stats': out}, open(os.path.join(outdir, 'check.json'), 'w'), indent=1)


if __name__ == '__main__':
    main()
