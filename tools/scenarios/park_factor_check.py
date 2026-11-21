"""Check of the park geometry behind `bm park`: the same measured balls (every ball of the
Statcast sample, statcast_balls.py) flown through each major-league park of the game data,
and the parks' home-run indexes compared with Baseball Savant's Statcast park factors (three
seasons, same batters home and away: a far larger and steadier sample than the 30 days).

Two versions of the flights: the game-average weather (24 C, no wind) at the park's
elevation, so only the fences, walls and altitude differ; and each park's mean game-time
temperature over the sample's games (mlb_weather.py), still without wind.

The index of a park is its home runs over the mean of the parks compared, x 100. Savant's
page: https://baseballsavant.mlb.com/leaderboard/statcast-park-factors?type=year&year=2025&batSide=&stat=index_wOBA&condition=All&rolling=3
(saved as HTML; the table is the page's `var data = [...]`).

usage: park_factor_check.py BM BALLS.csv WEATHER.json SAVANT.html OUTDIR"""
import json
import math
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from park_balls_check import CLUBS, REPO, fly, weather  # noqa: E402

# Statcast home-team code -> Savant venue name (parks with three seasons in 2023-2025).
VENUES = {
    'BAL': 'Oriole Park at Camden Yards', 'BOS': 'Fenway Park', 'NYY': 'Yankee Stadium', 'TOR': 'Rogers Centre', 'CWS': 'Rate Field',
    'CLE': 'Progressive Field', 'DET': 'Comerica Park', 'KC': 'Kauffman Stadium', 'MIN': 'Target Field', 'HOU': 'Daikin Park',
    'LAA': 'Angel Stadium', 'SEA': 'T-Mobile Park', 'TEX': 'Globe Life Field', 'ATL': 'Truist Park', 'MIA': 'loanDepot park',
    'NYM': 'Citi Field', 'PHI': 'Citizens Bank Park', 'WSH': 'Nationals Park', 'CHC': 'Wrigley Field', 'CIN': 'Great American Ball Park',
    'MIL': 'American Family Field', 'PIT': 'PNC Park', 'STL': 'Busch Stadium', 'AZ': 'Chase Field', 'COL': 'Coors Field',
    'LAD': 'UNIQLO Field at Dodger Stadium', 'SD': 'Petco Park', 'SF': 'Oracle Park',
}


def corr(xs, ys):
    mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
    return sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / math.sqrt(sum((x - mx) ** 2 for x in xs) * sum((y - my) ** 2 for y in ys))


def main():
    bm, balls, weather_file, savant_html, outdir = sys.argv[1:6]
    os.makedirs(outdir, exist_ok=True)
    savant = {d['venue_name']: int(d['index_hr']) for d in json.loads(re.search(r'var data = (\[.*?\]);\s*\n', open(savant_html).read(), re.S).group(1))}
    parks = {c['id']: c['park'] for c in json.load(open(os.path.join(REPO, 'data/leagues/us.json')))['clubs']}
    games = json.load(open(weather_file))
    lines = [x.rstrip('\n').split(',') for x in open(balls)]
    # Mean game-time temperature of each park over the sample's balls.
    temp = {}
    for x in lines:
        w = weather(games.get(x[7]))
        if w is not None:
            temp.setdefault(x[6], []).append(w[0])
    plain = os.path.join(outdir, 'balls.csv')
    with open(plain, 'w') as f:
        f.writelines(','.join(x[:4]) + '\n' for x in lines)
    rows = []
    for code, venue in sorted(VENUES.items()):
        p = parks[CLUBS[code]]
        elev = p.get('elevation_m') or 0
        hr = fly(bm, plain, p['fence_m'], p['wall_m'], elev, os.path.join(outdir, f'factor-{code}.json'), p.get('batted_drag', 1.0))
        t = sum(temp[code]) / len(temp[code])
        warm = os.path.join(outdir, f'balls-{code}-temp.csv')
        with open(warm, 'w') as f:
            f.writelines(','.join(x[:4]) + f',{t:.1f},0,0\n' for x in lines)
        hr_t = fly(bm, warm, p['fence_m'], p['wall_m'], elev, os.path.join(outdir, f'factor-temp-{code}.json'), p.get('batted_drag', 1.0))
        rows.append([code, venue, savant[venue], hr, hr_t, t])
    for k in (3, 4):
        mean = sum(r[k] for r in rows) / len(rows)
        for r in rows:
            r.append(100 * r[k] / mean)
    print(f'{len(lines)} balls; parks {len(rows)}')
    print('| park | Savant HR index (2023-25) | flown, game-average weather | flown, park mean temperature | mean temperature C |')
    print('|---|---|---|---|---|')
    for code, venue, sv, hr, hr_t, t, idx, idx_t in sorted(rows, key=lambda r: -r[2]):
        print(f'| {venue} ({code}) | {sv} | {idx:.0f} | {idx_t:.0f} | {t:.1f} |')
    out = {}
    for k, name in ((6, 'game-average weather'), (7, 'park mean temperature')):
        xs, ys = [r[k] for r in rows], [r[2] for r in rows]
        r = corr(xs, ys)
        rmse = math.sqrt(sum((x - y) ** 2 for x, y in zip(xs, ys)) / len(xs))
        sd_s = math.sqrt(sum((y - sum(ys) / len(ys)) ** 2 for y in ys) / len(ys))
        sd_m = math.sqrt(sum((x - sum(xs) / len(xs)) ** 2 for x in xs) / len(xs))
        out[name] = {'r': r, 'rmse': rmse, 'sd_savant': sd_s, 'sd_model': sd_m}
        print(f'{name}: correlation {r:.2f}, RMS difference {rmse:.1f} points (spread of Savant indexes {sd_s:.1f}, of the model {sd_m:.1f})')
    json.dump({'rows': rows, 'stats': out}, open(os.path.join(outdir, 'factor_check.json'), 'w'), indent=1)


if __name__ == '__main__':
    main()
