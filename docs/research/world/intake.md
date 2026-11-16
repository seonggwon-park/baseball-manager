# New players each year: drafts, amateur signings, players used

Research date: 2026-11-16. These are the numbers behind `data/people/careers.json`
(the classes of new players, the drafts and the signings) and two constants of
`crates/bm-world` (`career::USED_PER_SEASON`, `ability::STARTER_SHARE`).

Source tags as in `aging.md`: [O] official or league data, [P] press, [AN] analysis.

## 1. MLB draft, 2021-2025 (counted from the MLB Stats API)

All 3,072 picks of the five drafts, from `https://statsapi.mlb.com/api/v1/draft/<year>` [O],
retrieved 2026-11-16. School type is the pick's `school.schoolClass` (HS = high school,
4YR = four-year college, JC = junior college); signed = a signing bonus is recorded; age is
counted to the day of the draft's first round (2021-07-11, 2022-07-17, 2023-07-09,
2024-07-14, 2025-07-13).

| Rounds | Picks | Four-year college | High school | Junior college | Signed |
|---|---|---|---|---|---|
| 1 | 149 | 55.0 % | 44.3 % | 0.7 % | 99.3 % |
| Competitive-balance and supplemental | 109 | 66.1 % | 33.9 % | 0 | 98.2 % |
| 2-5 | 564 | 73.9 % | 23.8 % | 2.1 % | 98.9 % |
| 6-10 | 750 | 88.1 % | 8.4 % | 3.1 % | 99.5 % |
| 11-20 | 1,500 | 69.7 % | 19.6 % | 10.2 % | 85.3 % |
| All | 3,072 | 74.1 % | 19.3 % | 6.2 % | 92.4 % |

About 0.4 % of picks have another or no school class.

| School type | Picks with a birth date | Age on draft day: mean | SD | 5th / 50th / 95th percentile | Pitchers |
|---|---|---|---|---|---|
| Four-year college | 2,277 | 21.95 | 0.77 | 20.96 / 21.80 / 23.45 | 60.6 % |
| High school | 594 | 18.51 | 0.44 | 17.84 / 18.51 / 19.14 | 43.9 % |
| Junior college | 189 | 20.59 | 1.06 | 19.09 / 20.50 / 21.90 | 72.0 % |

- Primary positions of all picks: pitchers 58.0 %, OF 12.9 %, SS 7.2 %, C 6.5 %, 1B 4.2 %,
  3B 4.1 %, 2B 3.4 %, CF 1.3 %, LF 1.0 %, RF 0.9 %, DH 0.3 %, two-way 0.1 %.
- Birth countries: USA 2,901, Canada 68, Puerto Rico 48, Dominican Republic 12,
  Venezuela 5, Australia 4, Netherlands 4, United Kingdom 3, Panama 3, Cuba 3, Korea 3,
  New Zealand 2, Italy 2, Taiwan 2, Mexico 2, Japan 2, one each from Nicaragua, U.S. Virgin
  Islands, Bahamas, Iceland, France, Russia, Thailand and Spain.
- 2025 alone: 615 picks over 20 rounds (MLB Pipeline via CBS Sports [P]); 43 first-round
  picks, 19 of them high-school players (ON3 [P]).

## 2. International amateurs

- Rules for 2026 [O, league media]: signing period 15 January to 15 December; a player must
  be 16 before he signs and 17 before 1 September of the next year; hard-capped bonus
  pools ($5.44 M to $8.03 M per club). MLB.com, 2026-01-22,
  https://www.mlb.com/news/mlb-international-prospects-signing-day-2026
- Signings in the 2025 period [P]: 1,011 players from 23 countries by 2025-12-02 (Dominican
  Republic 441, Venezuela 386, Mexico 62, Cuba 39, Colombia 25, Taiwan 5, Italy 4, Brazil 2;
  the list is cut off there). Count by Francys Romero, World Baseball Network, 2025-12-03,
  https://worldbaseball.com/the-rapid-growth-of-international-born-players-in-major-league-baseball-in-2025 .
  Not an official MLB figure. About 34 per organisation.
- Reach rates: Dominican signees 2004-2012, 5.4 % reached MLB (Pérez Neró 2019, `aging.md` 4E).

## 3. KBO draft

- 2026 draft (held 2025-09-17): 11 rounds, 1,261 applicants (930 high-school seniors, 216
  college seniors, 51 early-draft applicants, 19 from abroad), 110 selected (8.7 %).
  Kyeongin Ilbo [P], https://www.kyeongin.com/article/1753530
- 2027 draft (held 2026-09-21): 110 picks over 11 rounds; 87 high-school players, 22
  college players (4 of them early-draft juniors), 1 from abroad; 62 pitchers. isplus,
  wikitree, Xports News [P], e.g. https://www.wikitree.co.kr/articles/1161207
- High-school seniors are born in one calendar year (Korean school cohorts), so they are
  17.7-18.7 years old on draft day.

## 4. NPB draft

2025 draft (held 2025-10-23): 116 players, 73 in the main draft (本指名) and 43 in the
development draft (育成), counted from the twelve teams' lists on spread-sports.jp [P],
https://spread-sports.jp/?p=363493 (the page gives no totals; the counts are mine):

| | Picks | High school | University | Corporate | Independent league | Other | Pitchers |
|---|---|---|---|---|---|---|---|
| Main draft | 73 | 19 | 40 | 12 | 1 | 1 | 38 |
| Development draft | 43 | 14 | 14 | 0 | 12 | 3 | 20 |

Per club: main 5-7 (Hanshin 5, DeNA 5, Giants 6, Chunichi 6, Hiroshima 7, Yakult 7,
SoftBank 5, Nippon-Ham 5, Orix 7, Rakuten 7, Lotte 7, Seibu 6); development 1-8.
"Other" includes the two farm-only clubs (Oisix Niigata counted there).

## 5. Players a major-league club uses in a season

1,470 different players appeared in 2025 MLB regular-season games: the union of the
players with hitting (765), pitching (873) and fielding (1,470) season lines in
`https://statsapi.mlb.com/api/v1/stats?stats=season&group=<group>&season=2025&sportId=1&playerPool=All&gameType=R` [O],
retrieved 2026-11-16. That is 49 per club (players traded during the season count once).
The world uses it as the bar for "played in the majors": being among the 49 best of a
major-league organisation at the start of a season.

## 6. Innings of starters and relievers

"The starters-relievers workload split this year is 57%-43%. Fifteen years ago, it was
65%-35%." Tom Verducci, Sports Illustrated, 2026-08-10 [P],
https://www.si.com/mlb/pitching-longevity-hits-all-time-low-verduccis-view . MLB.com gave
66.3 % in 1995, 66.5 % in 2014 and 62.6 % in 2017 (search excerpt; not checked).

## 7. Positional adjustment

Runs per 162 defensive games against an average position, FanGraphs WAR: C +12.5,
SS +7.5, 2B +2.5, 3B +2.5, CF +2.5, LF -7.5, RF -7.5, 1B -12.5, DH -17.5. FanGraphs
Sabermetrics Library, "Positional Adjustment" [AN],
https://library.fangraphs.com/misc/war/positional-adjustment/

## 8. Not found

- A league-wide high-school/college split of the whole MLB draft from a published source
  (counted here instead).
- How many undrafted free agents and minor-league free agents an organisation signs a year.
- KBO developmental-player (육성선수) signings per club and year.
- Minor-league sprint speed or other physical tracking averages by level (Baseball Savant
  shows minor-league Statcast only game by game).
