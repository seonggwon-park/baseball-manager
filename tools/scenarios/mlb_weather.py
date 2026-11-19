"""Game-time weather of the Statcast sample's games from the MLB Stats API schedule (one
request per date): temperature (F), wind speed (mph) and direction as the club reports it
("Out To CF", "In From LF", "L To R", ...) and the condition ("Roof Closed", "Dome", ...).

usage: mlb_weather.py STATCAST_DIR OUT.json   (dates from STATCAST_DIR/YYYY-MM-DD.csv)"""
import glob
import json
import os
import subprocess
import sys
import time


def main():
    src, out = sys.argv[1], sys.argv[2]
    dates = sorted(os.path.basename(f)[:10] for f in glob.glob(os.path.join(src, '????-??-??.csv')))
    games = {}
    for d in dates:
        url = f'https://statsapi.mlb.com/api/v1/schedule?sportId=1&date={d}&hydrate=weather'
        for attempt in range(3):
            r = subprocess.run(['curl', '-sS', '-m', '60', url], capture_output=True, text=True)
            if r.returncode == 0 and r.stdout.startswith('{'):
                break
            time.sleep(5)
        for day in json.loads(r.stdout).get('dates', []):
            for g in day['games']:
                w = g.get('weather') or {}
                games[str(g['gamePk'])] = {'date': d, 'venue': g.get('venue', {}).get('name'), 'home': g['teams']['home']['team']['name'],
                                           'condition': w.get('condition'), 'temp_f': w.get('temp'), 'wind': w.get('wind')}
        time.sleep(1)
    json.dump(games, open(out, 'w'), indent=1)
    print(f'{len(games)} games on {len(dates)} dates; with weather: {sum(1 for g in games.values() if g["wind"])}')


if __name__ == '__main__':
    main()
