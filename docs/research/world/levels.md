# How each level compares with MLB: tracking, outcome rates, translation factors, ages

Research date: 2026-11-16. The 2026 seasons of all nine levels are complete, so both 2025 and 2026 are given wherever possible.

Levels: MLB; Triple-A (International League + Pacific Coast League); Double-A (Eastern, Southern, Texas); High-A (Midwest, Northwest, South Atlantic); Single-A (California, Carolina, Florida State); KBO League; KBO Futures League; NPB (Central + Pacific); NPB farm (2025: Eastern + Western Leagues; 2026: one Farm League with East, Central and West divisions).

## Source labels

- **[official]**: data published by the league or its data provider (MLB Stats API, Baseball Savant / Statcast, npb.jp, koreabaseball.com).
- **[official, my aggregation]**: official tables that I summed or turned into rates. The inputs are shown (section 2.2 and appendix A).
- **[reference DB]**: Baseball-Reference (B-Ref). Where it overlaps with official totals, it matches them (section 2.4).
- **[analysis]**: published analysis (FanGraphs, Clay Davenport, research papers).
- **[press]**: news reports.
- **[fan]**: fan compilations on social media or blogs. The method is unknown; treat these as low confidence.
- **not found**: I found no published value, or the source could not be opened. Nothing in this file is estimated to fill such a cell.

Unit conversion used throughout: 1 mph = 1.609344 km/h.

---

## 0. Main findings

1. **Four-seam velocity [official].** MLB 94.5 mph (2025) and 94.8 mph (2026). Triple-A 93.4 and 93.6. Florida State League (Single-A; the only Single-A league with public tracking) 92.8 and 93.3. The spread between pitchers (SD of pitcher averages, pitchers with 100 or more four-seamers) is similar at each tracked level: MLB 2.25–2.30 mph, Triple-A 2.15–2.24, FSL 1.85–2.06.
   - KBO 2025 is about 146.0 to 146.2 km/h (90.7 to 90.8 mph) [press].
   - NPB 2025 is about 147 to 148 km/h (91.4 to 92.0 mph) [fan/press]. For 2026 the only figure is a fan compilation of 147.7 km/h.
   - Double-A, High-A, the KBO Futures League and the NPB farm have no public league averages.
2. **Batted balls [official].** Average exit velocity:
   - MLB: 88.8 mph in 2025, 88.0 in 2026.
   - Triple-A: 87.5 and 87.2.
   - FSL: 86.4 and 86.5.

   Hard-hit rates (95 mph or more) are 41.0/38.7% (MLB), 37.1/36.2% (Triple-A) and 32.9/34.5% (FSL). There are no public KBO or NPB league averages.
3. **Outcome rates (2026).**

   | Level | R per team-game | Slash line | K% | BB% |
   |---|---|---|---|---|
   | MLB | 4.48 | .244/.318/.400 | 22.1 | 8.9 |
   | Triple-A | 5.31 | .258/.349/.422 | 22.3 | 11.2 |
   | KBO | 5.07 | .268/.346/.404 | 19.5 | 9.4 |
   | NPB | 3.59 | .240/.305/.363 | 21.0 | 7.6 |

   Below Triple-A, offense rose sharply in 2026. Double-A HR/PA went from 2.01% to 3.08%, High-A from 1.90% to 2.91% and Single-A from 1.42% to 2.32%. For calibration, prefer 2025 or a 2025–26 average for those levels (section 2.3).
4. **Translation factors [analysis].**
   - FanGraphs (Rosenblum, 2024) gives wRC+ deductions to reach MLB level: Triple-A −35, Double-A −48, High-A −60, Single-A −70, complex −90.
   - Davenport (2025) studied matched Triple-A/MLB player-seasons for 2001–2025. In 2021–25 the gap was −40 BA, −46 OBP, −99 SLG and −43 EqA points. From his totals I compute that the strikeout rate rises ×1.24 in MLB, the walk rate falls ×0.84 and HR per PA falls ×0.69.
   - ZiPS (Szymborski) places NPB at "AAA½" (2024) or "between AAA and MLB" (2026). It places the KBO at "AA½" (2023, 2024) or "between AA and AAA" (2026).
   - NPB→MLB hitters lose about 49 to 53 wRC+ points on average; MLB→NPB hitters gain about 32 (Rosenblum, 2025).
5. **Average age [reference DB].**

   | Level | Batters (2026) | Pitchers (2026) |
   |---|---|---|
   | MLB | 27.8 | 29.1 |
   | Triple-A | 26.3 | 27.3 |
   | Double-A | 23.7 | 24.6 |
   | High-A | 22.1 | 23.0 |
   | Single-A | 20.7 | 22.0 |
   | KBO | 28.7 | 27.4 |
   | NPB | 27.65 | 27.65 |
   | NPB farm | 25.1 | 26.2 |
   | KBO Futures | not found | not found |

---

## 1. Tracking quantities

### 1.1 Four-seam fastball velocity: league average, pitch-weighted

| Level | 2025 | 2026 | Spread between pitchers (SD of pitcher means, min 100 FF) | Source | Type |
|---|---|---|---|---|---|
| MLB | **94.5 mph** (226,410 FF) | **94.8 mph** (219,174 FF) | 2.30 (2025), 2.25 (2026) | Savant Statcast search, regular season, pitch type FF, grouped by league-year (query Q1 in appendix B). Per-pitcher spread from grouping by name (Q4). AP, 29 Sep 2026: "Four-seam fastballs averaged 94.8 mph this year, according to MLB Statcast, up from 94.5 mph last season." https://ny1.com/nyc/all-boroughs/ap-top-news/2026/09/29/the-fast-get-faster-mlb-pitchers-again-set-fastball-velocity-record-by-averaging-948-mph | [official] + [press] confirms |
| Triple-A | **93.4 mph** (216,775 FF) | **93.6 mph** (216,949 FF) | 2.24 (2025), 2.15 (2026) | Savant Minor League Statcast search, grouped by level-year (Q2), and by name with level AAA (Q5). AP (same article): "Four-seam velocity at Triple-A averaged 93.6 mph, up from 92.7 when tracking started in 2022." | [official] + [press] confirms |
| Double-A | not found | not found | not found | Savant has no Double-A data. No published league average was found. | — |
| High-A | not found | not found | not found | Same as Double-A. | — |
| Single-A | **FSL only: 92.8 mph** (54,464 FF) | **FSL only: 93.3 mph** (57,429 FF) | 2.06 (2025), 1.85 (2026) | Savant minors search, level "A". On Savant, level "A" is the Florida State League; the page says "Florida State League (Single-A) games starting with the 2021 season". AP: "It was 93.3 mph at the eight Single-A ballparks Statcast tracks, an increase from 92 mph in 2021." The California and Carolina Leagues are not tracked publicly. | [official] + [press] |
| KBO | **146.2 km/h (90.8 mph)**, Statiz figure quoted by MyDaily, 24 Dec 2025: "야구 통계 사이트 '스탯티즈' 기준 2025년 KBO리그 평균 포심 패스트볼 구속은 146.2km/h다." https://v.daum.net/v/20251224074236594 · **Conflict:** Chosun Ilbo, 1 Apr 2026: "지난해 KBO 리그의 직구(포심 패스트볼) 평균 구속은 시속 146.0㎞였다. 2024년(142.6㎞)보다 훨씬 빨라져…" https://m.news.nate.com/view/20260401n00442 | not found | not found | Press quoting Statiz, and a newspaper that names no source | [press] |
| KBO Futures | not found | not found | not found | — | — |
| NPB | **147.1 km/h (91.4 mph)**, NPB-wide "ストレート" average. Fan compilation, X account @aozora__nico2: full-season 2014–2025 series posted 5 Oct 2025, https://x.com/aozora__nico2/status/1974793206660489404 . The same account's split as of 31 Aug 2025: starters 146.1, relievers 148.6, total 147.1, https://x.com/aozora__nico2/status/1962149675253051641 . Press, approximate: Chosun Ilbo (2026-04-01, above) gives NPB "평균 시속 148㎞" for 2025. | **147.7 km/h (91.8 mph)**, same fan source, 2014–2026 series posted 8 Oct 2026: https://x.com/aozora__nico2/status/2108167935789609104 (reproduced at https://dnomotoke.com/archives/20261008232547/) | not found | Fan compilation (method and data source unstated) plus one approximate press figure | [fan] / [press] |
| NPB farm | not found | not found | not found | — | — |

Earlier seasons, for trend context:

- **MLB** [official, Savant Q1]: 94.2 mph (2023), 94.3 (2024).
- **Triple-A**: 93.0 (2023), 93.0 (2024).
- **FSL**: 92.2 (2023), 92.8 (2024).
- **KBO 2024, two conflicting values**:
  - Chosun Ilbo: 142.6 km/h (above).
  - Newsis, 16 Mar 2025: "지난 시즌 KBO 직구 평균 구속은 144.2㎞였다." https://sports.news.nate.com/view/20250316n02095
  - The 144.2 km/h figure was earlier published for **2022** (Statiz, season to 3 Oct 2022) by Sports Chunchu (THE GATE): https://www.spochoo.com/news/articleView.html?idxno=103319. Newsis may have reused it.
- **NPB** [fan, same aozora series]: 2022 146.0, 2023 146.5, 2024 146.7 km/h.

Other pitch quantities from Savant [official]:

| Level | FF spin (rpm), 2025 / 2026 | FF extension (ft), 2025 / 2026 |
|---|---|---|
| MLB | 2,323 / 2,314 | 6.5 / 6.5 |
| Triple-A | 2,264 / 2,264 | 6.4 / 6.4 |
| FSL | 2,321 / 2,331 | 6.3 / 6.3 |

AP gives MLB FF spin as 2,322 for 2025 and 2,314 for 2026.

### 1.2 Spread of pitcher fastball velocity: detail

My calculation from Savant per-pitcher averages (one row per pitcher, regular season, four-seamers; queries Q4/Q5).

| Level | Season | Pitchers with ≥100 FF | Mean of pitcher means | SD | 10th pct | Median | 90th pct | SD weighted by pitch count, all pitchers |
|---|---|---|---|---|---|---|---|---|
| MLB | 2025 | 499 | 94.46 | 2.30 | 91.6 | 94.5 | 97.4 | 2.26 (759 pitchers) |
| MLB | 2026 | 493 | 94.67 | 2.25 | 91.9 | 94.6 | 97.5 | 2.25 (775) |
| Triple-A | 2025 | 653 | 93.49 | 2.24 | 90.6 | 93.5 | 96.2 | 2.19 (1,137) |
| Triple-A | 2026 | 634 | 93.64 | 2.15 | 90.9 | 93.7 | 96.3 | 2.17 (1,146) |
| FSL (Single-A) | 2025 | 177 | 92.83 | 2.06 | 90.4 | 92.8 | 95.4 | 1.92 (444) |
| FSL (Single-A) | 2026 | 181 | 93.16 | 1.85 | 90.8 | 93.1 | 95.7 | 1.95 (485) |

The Triple-A rows include MLB players on rehab assignments; I did not remove them. KBO, NPB, Double-A, High-A, the Futures League and the NPB farm: not found.

### 1.3 Exit velocity, hard-hit rate, barrels, launch angle (all batted balls)

[official] Savant search, regular season, grouped by league-year (MLB) or level-year (minors), with the minors limited to tracked pitches (queries Q3, Q6). Hard-hit means exit velocity of 95 mph or more.

| Level | Season | Batted balls | Avg EV (mph) | Hard-hit % | Barrels per batted ball % | Avg launch angle |
|---|---|---|---|---|---|---|
| MLB | 2025 | 124,888 | **88.8** | **41.0** | 8.6 | 13° |
| MLB | 2026 | 124,611 | **88.0** | **38.7** | 7.7 | 13° |
| Triple-A | 2025 | 111,509 | **87.5** | **37.1** | 6.6 | 12° |
| Triple-A | 2026 | 111,805 | **87.2** | **36.2** | 6.3 | 13° |
| FSL (Single-A) | 2025 | 27,632 | **86.4** | **32.9** | 4.9 | 12° |
| FSL (Single-A) | 2026 | 26,549 | **86.5** | **34.5** | 5.4 | 11° |
| Double-A, High-A, California and Carolina Leagues | 2025–26 | — | not found | not found | — | — |
| KBO | 2025–26 | — | not found | not found | — | — |
| KBO Futures | — | — | not found | not found | — | — |
| NPB, NPB farm | — | — | not found | not found | — | — |

Notes:

- About 89% of FSL pitches are tracked (169,567 of 189,273 in 2025; 171,705 of 192,573 in 2026), which matches AP's "eight Single-A ballparks". Triple-A is 99.9% tracked.
- MLB exit velocity fell 0.8 mph from 2025 to 2026. I did not find a published explanation.
- KBO: Sports2i, the KBO's official records company, publishes counts of hitters with high average exit velocity, but no league average. For example, Sports Kyunghyang (1 Aug 2024), citing Sports2i: hitters with an average exit velocity of 140 km/h or more numbered 11 in 2023 and 18 in 2024 up to that date. https://sports.khan.co.kr/article/202408011432003

### 1.4 Bat speed

| Level | 2025 | 2026 | Spread | Source | Type |
|---|---|---|---|---|---|
| MLB | **71.8 mph** (297,284 competitive swings) | **72.1 mph** (297,750) | Qualified batters: mean 72.0, SD 2.64, 10th–90th pct 68.9–75.2 (226 batters, 2025). 2026: mean 72.2, SD 2.81, 68.6–75.7 (209 batters). | League value: Savant search with the "competitive swing" flag (Q7). Spread: my calculation from the Savant bat-tracking leaderboard CSV (Q9). 2024 value for reference: 71.5. | [official] |
| Triple-A, FSL | not found | not found | — | The Savant minors search returns "--" for bat speed, swing length and attack angle at both levels (Q8). | — |
| All other levels | not found | not found | — | — | — |

### 1.5 Sprint speed

| Level | 2025 | 2026 | Source |
|---|---|---|---|
| MLB | Mean over players **27.32 ft/s**, SD 1.38 (617 players); weighted by competitive runs 27.42 | **27.34 ft/s**, SD 1.37 (612 players); weighted 27.44 | My calculation from the Savant sprint-speed leaderboard CSV (Q10) [official data]. Savant's glossary defines a "Bolt" as a run of 30 ft/s or more: https://www.mlb.com/glossary/statcast/sprint-speed |
| Every other level | not found | not found | No public minor-league, KBO or NPB sprint-speed data was found. |

### 1.6 Plate-discipline rates from tracking data

My calculation from Savant grouped counts (Q11–Q13): swings, whiffs and pitches, split into in-zone (zones 1–9) and out-of-zone (zones 11–14).

| Level | Season | Swing% | Whiffs per swing % | Zone% | Swing% in zone | Swing% out of zone (chase) | Contact% in zone | Contact% out of zone |
|---|---|---|---|---|---|---|---|---|
| MLB | 2025 | 47.8 | 25.3 | 50.6 | 66.9 | 28.2 | 82.7 | 55.3 |
| MLB | 2026 | 47.7 | 25.1 | 47.5 | 66.7 | 30.5 | 83.9 | 57.1 |
| Triple-A | 2025 | 46.2 | 26.8 | 46.2 | 67.0 | 28.2 | 82.2 | 54.5 |
| Triple-A | 2026 | 46.1 | 26.1 | 47.0 | 66.9 | 27.7 | 82.6 | 55.2 |
| FSL | 2025 | 45.4 | 27.4 | 48.5 | 66.7 | 25.3 | 81.5 | 50.4 |
| FSL | 2026 | 45.2 | 29.3 | 47.7 | 66.7 | 25.7 | 79.9 | 49.0 |

Caveat: MLB Zone% drops from 50.6 to 47.5 and chase rises from 28.2 to 30.5 between 2025 and 2026. That looks like a change in how Savant classifies the zone, possibly the batter-height zone used with the 2026 challenge system, rather than a change in behaviour. I did not verify this. Swing% and whiffs per swing do not depend on the zone and are the safer numbers.

**KBO.** One hand-charted sample from 2020 (about 29,736 pitches, focused on the better teams and pitchers). Ben Howell, "Modeling Plate Discipline from the KBO to MLB", Table 1, https://benhowell71.com/wp-content/uploads/2021/03/Modeling-Plate-Discipline-from-the-KBO-to-MLB.pdf

| Metric | KBO (charted, 2020) | MLB comparison value given by Howell |
|---|---|---|
| Swing% | 47.1 | 46.6 |
| Whiff% | 22.2 | 24.5 |
| First-pitch swing% | 29 | 28.3 |
| Ground-ball % | 51.2 | 45.3 |
| K% | 17.4 | 21.8 |
| BB% | 9.4 | 8.3 |

This is [analysis] with a non-random sample. NPB, NPB farm, KBO Futures, Double-A and High-A: not found.

### 1.7 Measurement notes that matter when comparing leagues

- **KBO measurement change.** Trackman became the KBO's single official velocity system from 2025. Before that, parks used different systems; Trackman was shown at 7 of 10 parks in 2024. A column in Ilgan Sports (18 Mar 2025) says Trackman readings ran faster than scouts' radar guns and calls this "구속 인플레이션". https://v.daum.net/v/20250318085716375 . SPOTV (25 Mar 2025) describes Trackman as "올해부터 KBO리그 공식 구속 측정 업체로 선정된 '트랙맨'": https://www.spotvnews.co.kr/news/articleView.html?idxno=737154 . Part of the jump from 142.6 km/h (2024) to 146.0 km/h (2025) reported by Chosun may therefore be a measurement change. This is not quantified anywhere I found.
- **NPB.** No official or press league-average velocity was found. The "ストレート" figures are fan compilations, with an unknown pitch-classification and data source (probably broadcast or stadium readings). Treat them as about ±1 km/h at best.
- **Minor-league coverage.** Savant: "All Triple-A games starting with the 2023 season, as well as Pacific Coast League games and Charlotte home games for the 2022 season; Florida State League (Single-A) games starting with the 2021 season." https://baseballsavant.mlb.com/statcast-search-minors

---

## 2. League outcome rates

### 2.1 Summary (latest two seasons)

Rates are my computation from official totals. PA is plate appearances. OBP = (H + BB + HBP) / (AB + BB + HBP + SF). K% and BB% are per PA, and BB includes intentional walks. "Team-game" counts each team's game separately (one game = 2 team-games).

| Level | Season | R per team-game | AVG | OBP | SLG | K% | BB% | HR/PA | HR per team-game | Source |
|---|---|---|---|---|---|---|---|---|---|---|
| MLB | 2026 | 4.48 | .244 | .318 | .400 | 22.1 | 8.9 | 3.03% | 1.15 | MLB Stats API, sum of 30 teams [official] |
| Triple-A | 2026 | 5.31 | .258 | .349 | .422 | 22.3 | 11.2 | 2.92% | 1.13 | MLB Stats API, sportId 11 [official] |
| Double-A | 2026 | 5.24 | .249 | .340 | .415 | 24.4 | 10.9 | 3.08% | 1.17 | MLB Stats API, sportId 12 [official] |
| High-A | 2026 | 5.54 | .249 | .351 | .413 | 24.2 | 12.0 | 2.91% | 1.13 | MLB Stats API, sportId 13 [official] |
| Single-A | 2026 | 5.57 | .248 | .354 | .395 | 24.4 | 12.5 | 2.32% | 0.90 | MLB Stats API, sportId 14 [official] |
| KBO | 2026 | 5.07 | .268 | .346 | .404 | 19.5 | 9.4 | 2.41% | 0.95 | koreabaseball.com team stats [official] |
| KBO Futures | 2026 | 5.44 | .270 | .362* | .383 | 16.9 | 11.1 | 1.51% | 0.59 | Sum of all player lines on koreabaseball.com Futures pages [official, my aggregation] |
| NPB (CL+PL) | 2026 | 3.59 | .240 | .305 | .363 | 21.0 | 7.6 | 2.23% | 0.83 | npb.jp team batting [official] |
| NPB farm | 2026 | 4.11 | .253 | .327 | .364 | 19.1 | 8.8 | 1.55% | 0.58 | npb.jp farm team batting, 3 divisions [official] |
| MLB | 2025 | 4.45 | .245 | .315 | .404 | 22.2 | 8.4 | 3.09% | 1.16 | MLB Stats API [official] |
| Triple-A | 2025 | 5.24 | .258 | .347 | .421 | 22.6 | 10.9 | 2.87% | 1.10 | MLB Stats API [official] |
| Double-A | 2025 | 4.29 | .235 | .323 | .360 | 23.5 | 10.3 | 2.01% | 0.75 | MLB Stats API [official] |
| High-A | 2025 | 4.51 | .233 | .330 | .358 | 23.7 | 11.0 | 1.90% | 0.71 | MLB Stats API [official] |
| Single-A | 2025 | 4.82 | .237 | .340 | .345 | 23.5 | 11.9 | 1.42% | 0.54 | MLB Stats API [official] |
| KBO | 2025 | 4.73 | .262 | .338 | .389 | 19.7 | 9.1 | 2.13% | 0.83 | koreabaseball.com team batting and team pitching [official] |
| KBO Futures | 2025 | 6.22 | .286 | .381* | .427 | 17.8 | 12.0 | 2.15% | 0.85 | Same as 2026 [official, my aggregation] |
| NPB (CL+PL) | 2025 | 3.29 | .244 | .305 | .351 | 19.4 | 7.1 | 1.71% | 0.64 | npb.jp [official] |
| NPB farm | 2025 | 3.80 | .250 | .321 | .343 | 18.2 | 8.5 | 1.07% | 0.40 | npb.jp Eastern + Western [official] |

\* The Futures League player lists have no sacrifice-fly column. I estimated each player's sacrifice flies from his displayed OBP: SF = round((H + BB + HBP) / OBP) − AB − BB − HBP, kept between 0 and the PA residual. That gives 453 SF in 2025 and 518 in 2026. The OBP may be off by about ±.001. All other Futures rates use only counted totals.

Per-game rates depend on each league's game-length rules (extra-inning limits, ties, doubleheader rules), which differ between MLB, MiLB, the KBO and NPB. Per-PA rates are the better basis for comparing strength.

### 2.2 Inputs (league totals) used for 2.1

| Level | Season | Team-games | R | PA | AB | H | 2B | 3B | HR | BB | IBB | HBP | SO | SF |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| MLB | 2026 | 4858 | 21769 | 183849 | 163329 | 39849 | 7493 | 671 | 5575 | 16337 | 545 | 2118 | 40700 | 1298 |
| Triple-A | 2026 | 4446 | 23628 | 171799 | 148157 | 38237 | 7610 | 811 | 5014 | 19231 | 195 | 2232 | 38307 | 1416 |
| Double-A | 2026 | 4096 | 21455 | 156270 | 135059 | 33614 | 6534 | 719 | 4812 | 17031 | 165 | 2291 | 38112 | 1244 |
| High-A | 2026 | 3904 | 21645 | 151429 | 128833 | 32105 | 6382 | 740 | 4411 | 18205 | 79 | 2650 | 36687 | 1225 |
| Single-A | 2026 | 3908 | 21752 | 151705 | 128124 | 31830 | 6223 | 963 | 3524 | 18963 | 55 | 2791 | 36942 | 1354 |
| KBO | 2026 | 1440 | 7301 | 56732 | 49548 | 13277 | 2259 | 202 | 1366 | 5319 | 137 | 838 | 11054 | 437 |
| KBO Futures | 2026 | 1198 | 6518 | 47161 | 40138 | 10827 | 1918 | 243 | 711 | 5232 | n/a | 861 | 7962 | 518 (est.) |
| NPB (CL+PL) | 2026 | 1716 | 6168 | 63898 | 57180 | 13742 | 2335 | 209 | 1426 | 4841 | 236 | 632 | 13430 | 374 |
| NPB farm | 2026 | 1676 | 6893 | 62904 | 55695 | 14115 | 2481 | 373 | 972 | 5562 | 36 | 717 | 11997 | 462 |
| MLB | 2025 | 4860 | 21614 | 182926 | 163664 | 40138 | 7745 | 628 | 5650 | 15379 | 556 | 1928 | 40645 | 1307 |
| Triple-A | 2025 | 4464 | 23381 | 171483 | 148484 | 38290 | 7677 | 881 | 4922 | 18747 | 193 | 2254 | 38803 | 1366 |
| Double-A | 2025 | 4112 | 17625 | 152649 | 132903 | 31263 | 5948 | 739 | 3064 | 15782 | 124 | 2075 | 35830 | 1242 |
| High-A | 2025 | 3916 | 17651 | 147225 | 126691 | 29532 | 5874 | 798 | 2790 | 16235 | 132 | 2590 | 34846 | 1229 |
| Single-A | 2025 | 3896 | 18773 | 149076 | 126691 | 29985 | 5532 | 920 | 2112 | 17702 | 71 | 2806 | 35029 | 1327 |
| KBO | 2025 | 1440 | 6813 | 55996 | 49021 | 12824 | 2243 | 208 | 1191 | 5123 | n/a | 806 | 11024 | 451 |
| KBO Futures | 2025 | 1078 | 6709 | 42905 | 36349 | 10379 | 1937 | 223 | 921 | 5148 | n/a | 709 | 7622 | 453 (est.) |
| NPB (CL+PL) | 2025 | 1716 | 5654 | 64178 | 57532 | 14053 | 2315 | 263 | 1096 | 4559 | 290 | 595 | 12475 | 379 |
| NPB farm | 2025 | 1742 | 6628 | 65474 | 57919 | 14508 | 2452 | 390 | 702 | 5562 | 44 | 726 | 11890 | 480 |

Notes on the inputs:

- **KBO Futures team-games** are summed from the official standings ("경기" column):
  - 2025, North: 94 + 93 + 94 + 99 + 98. South: 102 + 99 + 98 + 96 + 102 + 103. Total 1,078.
  - 2026, North: 96 + 101 + 102 + 104 + 102 + 101. South: 96 + 94 + 102 + 99 + 98 + 103. Total 1,198.
  - Standings pages: https://www.koreabaseball.com/Futures/TeamRank/North.aspx?gameDate=2025-11-16 , .../South.aspx?gameDate=2025-11-16 , and the same with gameDate=2026-11-16.
- **KBO Futures completeness check.** The pitching totals summed from the official Futures pitcher pages agree with the batting totals:
  - 2025: H 10,359 vs 10,379; HR 918 vs 921; BB 5,132 vs 5,148; SO 7,616 vs 7,622; R 6,688 vs 6,709.
  - 2026: H 10,819 vs 10,827; HR 711 vs 711; BB 5,230 vs 5,232; SO 7,961 vs 7,962; R 6,516 vs 6,518.
- **Futures League teams.**
  - 2025: 11 teams, the 10 KBO clubs' second teams plus Sangmu (the armed-forces team).
  - 2026: 12 teams. A 12th team, listed as 울산 (Ulsan), appears in the 2026 standings and player lists. It is not an affiliate of any KBO club.
- **KBO 2025.** BB, HBP and SO come from the official team pitching table (league totals: BB 5,123, HBP 806, SO 11,024). The official team batting page (Basic2) with walks failed to load (server error). B-Ref's KBO 2025 page gives the same BB, HBP and SO.

### 2.3 Per-league detail and a 2026 break in the series

**MiLB by league** [official, MLB Stats API, my aggregation by league]

| League (level) | 2025: R/G, AVG/OBP/SLG, K%, BB%, HR/PA | 2026: same columns |
|---|---|---|
| International (AAA) | 4.94, .251/.340/.410, 23.1, 10.8, 2.85% | 5.10, .252/.343/.416, 23.1, 11.1, 3.03% |
| Pacific Coast (AAA) | 5.83, .271/.361/.442, 21.7, 11.3, 2.92% | 5.73, .270/.360/.433, 20.7, 11.3, 2.71% |
| Eastern (AA) | 4.19, .232/.318/.362, 23.9, 10.1, 2.05% | 5.14, .248/.336/.413, 24.7, 10.6, 3.02% |
| Southern (AA) | 4.01, .230/.321/.340, 23.5, 10.6, 1.75% | 5.13, .245/.336/.411, 24.1, 10.8, 3.13% |
| Texas (AA) | 4.62, .243/.331/.373, 23.0, 10.4, 2.15% | 5.44, .254/.348/.420, 24.3, 11.2, 3.11% |
| Midwest (A+) | 4.53, .237/.334/.359, 22.5, 11.3, 1.69% | 5.60, .249/.355/.411, 24.0, 12.5, 2.81% |
| Northwest (A+) | 4.86, .241/.338/.387, 23.7, 11.1, 2.43% | 5.40, .254/.348/.416, 23.8, 11.2, 2.85% |
| South Atlantic (A+) | 4.30, .225/.320/.342, 24.8, 10.7, 1.82% | 5.56, .247/.349/.413, 24.6, 11.9, 3.05% |
| California (A) | 5.24, .247/.347/.358, 23.4, 11.8, 1.40% | 6.13, .262/.359/.424, 24.4, 11.6, 2.55% |
| Carolina (A) | 4.49, .231/.335/.330, 23.8, 11.8, 1.24% | 5.27, .241/.351/.376, 23.9, 13.0, 2.08% |
| Florida State (A) | 4.87, .234/.340/.351, 23.3, 12.1, 1.64% | 5.46, .245/.354/.391, 24.8, 12.6, 2.42% |

**NPB by league** [official, npb.jp, my aggregation]

| League | 2025 | 2026 |
|---|---|---|
| Central | 3.21, .242/.302/.350, 19.5, 7.0, 1.70% | 3.40, .236/.301/.350, 21.7, 7.7, 2.03% |
| Pacific | 3.38, .246/.307/.352, 19.4, 7.2, 1.72% | 3.79, .245/.308/.377, 20.3, 7.5, 2.43% |
| Farm, Eastern League (2025) / East division (2026) | 3.86, .252/.319/.351, 18.3, 8.1, 1.26% | 4.33, .255/.329/.378, 19.6, 9.2, 1.86% |
| Farm, Western League (2025) / Central division (2026) | 3.73, .249/.325/.331, 18.0, 9.0, 0.82% | 4.27, .259/.332/.368, 18.2, 8.7, 1.45% |
| Farm, West division (2026) | — | 3.67, .244/.317/.342, 19.7, 8.6, 1.29% |

NPB notes:

- The Central League did not use the DH in 2026; it adopts the DH from 2027. https://npb.jp/news/detail/20250804_05.html . CL batting lines therefore include pitchers batting.
- The 2026 farm division tables are each team's full-season totals, so they include games against other divisions.
- The 2026 farm is a single Farm League with three divisions and 14 teams. Two of them have no first team (Oisix Niigata, Hayate Shizuoka). https://npb.jp/bis/2026/stats/std_2c.html

**Breaks in the 2025→2026 series**

- **MiLB, Double-A and below.** HR/PA rose by 53% at Double-A, 54% at High-A and 64% at Single-A, but by only 2% at Triple-A (my ratios of the totals in 2.2). By league the rise ranged from +17% (Northwest) to +82% (California).
  - Baseball America has covered this: "Explaining Minor League Baseball's Historic Home Run Surge In 2026" https://www.baseballamerica.com/stories/explaining-minor-league-baseballs-historic-home-run-surge-in-2026/ and "Just How Much Did MiLB Offense Increase In 2026?" https://www.baseballamerica.com/stories/just-how-much-did-milb-offense-increase-in-2026/ . Both pages returned HTTP 403 to my tools, so their content is known to me only from search-engine summaries.
  - According to those summaries: MiLB teams averaged 1.09 HR per game, up from 0.79 the year before; Triple-A uses the MLB ball while the lower levels use a different MiLB ball; and BA did not find a confirmed ball change. My own totals give 1.086 in 2026 (17,761 HR in 16,354 team-games) and 0.786 in 2025 (12,888 in 16,388).
  - **Cause unconfirmed.** For calibrating "true" Double-A to Single-A strength, the 2025 rows are the less disturbed baseline.
- **NPB.** HR/PA rose from 1.71% to 2.23% and R per team-game from 3.29 to 3.59 (my totals above). No official explanation was found.
- **KBO.** R per team-game rose from 4.73 to 5.07 and HR/PA from 2.13% to 2.41%. In 2025 KBO offense had fallen from 2024: Segye Ilbo reports a league AVG of .277 in 2024 vs .260 to 1 Sep 2025. https://www.segye.com/newsView/20250902515520

### 2.4 Cross-checks between sources

- **MiLB.** B-Ref's register pages give the same league totals as the MLB Stats API. Example: International League 2026, R/G 5.10, .252/.343/.416, PA 112,452. https://www.baseball-reference.com/register/league.cgi?id=24986ebc
- **KBO.** B-Ref's KBO pages (2025: https://www.baseball-reference.com/register/league.cgi?id=4e296e3b ; 2026: …?id=163dcec5) match the official PA, AB, R, H, HR and SF exactly for 2026. For 2025, PA differs by 1 (55,995 vs 55,996).
- **NPB.** B-Ref differs from npb.jp by 1 to 2 PA per league. I used npb.jp.
- **MLB.** The B-Ref "League Average" rows agree: R/G 4.45 (2025) and 4.48 (2026). https://www.baseball-reference.com/leagues/majors/2026.shtml

### 2.5 Ratios to MLB in the same season

My computation: the level's rate divided by MLB's rate.

| Level | Season | R/G | AVG | OBP | SLG | K% | BB% | HR/PA |
|---|---|---|---|---|---|---|---|---|
| Triple-A | 2026 | 1.19 | 1.06 | 1.10 | 1.05 | 1.01 | 1.26 | 0.96 |
| Double-A | 2026 | 1.17 | 1.02 | 1.07 | 1.04 | 1.10 | 1.23 | 1.02 |
| High-A | 2026 | 1.24 | 1.02 | 1.10 | 1.03 | 1.09 | 1.35 | 0.96 |
| Single-A | 2026 | 1.24 | 1.02 | 1.11 | 0.99 | 1.10 | 1.41 | 0.77 |
| KBO | 2026 | 1.13 | 1.10 | 1.09 | 1.01 | 0.88 | 1.06 | 0.79 |
| KBO Futures | 2026 | 1.21 | 1.11 | 1.14 | 0.96 | 0.76 | 1.25 | 0.50 |
| NPB | 2026 | 0.80 | 0.99 | 0.96 | 0.91 | 0.95 | 0.85 | 0.74 |
| NPB farm | 2026 | 0.92 | 1.04 | 1.03 | 0.91 | 0.86 | 1.00 | 0.51 |
| Triple-A | 2025 | 1.18 | 1.05 | 1.10 | 1.04 | 1.02 | 1.30 | 0.93 |
| Double-A | 2025 | 0.96 | 0.96 | 1.03 | 0.89 | 1.06 | 1.23 | 0.65 |
| High-A | 2025 | 1.01 | 0.95 | 1.05 | 0.89 | 1.07 | 1.31 | 0.61 |
| Single-A | 2025 | 1.08 | 0.97 | 1.08 | 0.85 | 1.06 | 1.41 | 0.46 |
| KBO | 2025 | 1.06 | 1.07 | 1.07 | 0.96 | 0.89 | 1.09 | 0.69 |
| KBO Futures | 2025 | 1.40 | 1.16 | 1.21 | 1.06 | 0.80 | 1.43 | 0.69 |
| NPB | 2025 | 0.74 | 1.00 | 0.97 | 0.87 | 0.87 | 0.84 | 0.55 |
| NPB farm | 2025 | 0.86 | 1.02 | 1.02 | 0.85 | 0.82 | 1.01 | 0.35 |

Interpretation caution: league averages mix the hitters' and pitchers' strength, the ball and the parks. A league that is weaker on both sides can still produce MLB-like rates. The minors, for example, have a K% similar to MLB but a much higher BB%. These ratios are targets for the simulated league environment, not strength factors. Strength factors are in section 3.

---

## 3. Translation factors to MLB (major-league equivalencies)

### 3.1 Minor leagues → MLB

**(a) FanGraphs, Jordan Rosenblum, "The Top 30 Projected Hitting Prospects Entering 2024" (8 Apr 2024)** [analysis]

https://fantasy.fangraphs.com/the-top-30-projected-hitting-prospects-entering-2024 ; translation table (Figure 1): https://fantasy.fangraphs.com/wp-content/uploads/sites/3/2024/04/fig1-4-6.png

| Level | wRC+ points to subtract to get the MLB-equivalent wRC+ |
|---|---|
| AAA | −35 |
| AA | −48 |
| A+ | −60 |
| A | −70 |
| CPX (complex leagues) | −90 |

- **Method.** "The translations are based on 2021 to 2023 transitions across leagues" (the post-2021 minor-league structure). Leagues within a level are pooled because "there are small translation differences across leagues within a given level".
- **Example given.** "a 170 wRC+ at Class A is equivalent to a 100 wRC+ at the major league level".
- **Triple-A baseline.** The author gives "close to a 70 wRC+, the Triple-A mean on an MLB scale" (used as a regression target).
- **Unpublished parts.** The text says translations also exist for K%, BB% and HR/BBE, but gives no values. The exact weighting of transitions is not described.

**(b) Clay Davenport, "Players moving between AAA and Majors" (2 Sep 2025)** [analysis]

https://claydavenport.com/2025/09/02/players-moving-between-aaa-and-majors/

- **Method.** All players from 2001–2025 with at least 50 PA at both Triple-A and MLB in the same season: 3,631 matched pairs.
  - Each pair is weighted by the smaller of the two PA totals.
  - Stats are his "nodif" translations, which adjust only for park and league offense, with no league-quality or age adjustment.
- **Selection bias.** Davenport warns that the gap is "almost certainly an overstatement". Lucky Triple-A hitters get promoted, and unlucky MLB hitters get sent down.
- **Totals:**

  | | AB | H | 2B | 3B | HR | BB | SO | BA | OBA | SLG | EqA |
  |---|---|---|---|---|---|---|---|---|---|---|---|
  | AAA | 374,279 | 105,499 | 19,758 | 2,844 | 14,751 | 38,666 | 72,995 | .282 | .349 | .468 | .285 |
  | MLB | 380,494 | 92,268 | 15,874 | 2,354 | 10,157 | 32,451 | 90,412 | .242 | .302 | .377 | .243 |

- **Change from AAA to MLB by period** (points of BA / OBA / SLG / EqA): 2001–05: −41/−51/−95/−46. 2006–10: −40/−49/−90/−43. 2011–15: −39/−46/−86/−41. 2016–19: −37/−43/−86/−39. **2021–25: −40/−46/−99/−43.**
- **By age.** The drop varies little with age; for example −42 EqA at age 22 and −45 at age 25 (full table in the source).
- **Component ratios, MLB ÷ AAA (my computation from his totals).** PA is approximated by AB + BB, since HBP and SF are not given. BABIP = (H − HR) / (AB − SO − HR).

  | Component | AAA | MLB | Ratio |
  |---|---|---|---|
  | K per PA | .1768 | .2189 | **×1.24** |
  | BB per PA | .0936 | .0786 | **×0.84** |
  | HR per PA | .0357 | .0246 | **×0.69** |
  | ISO | .186 | .134 | **×0.72** |
  | BABIP | .317 | .293 | **×0.93** |
  | AVG | — | — | ×0.86 |

**(c) Szymborski (ZiPS), FanGraphs chat, 1 Aug 2024** [analysis, qualitative]

On whether the Triple-A–MLB gap is growing: "I've been tinkering with the data … and I'm not actually finding an increasing gap." https://blogs.fangraphs.com/dan-szymborski-fangraphs-chat-8-1-24/

**Physical gap implied by section 1** [official data, my comparison]

| | Triple-A vs MLB | FSL vs MLB |
|---|---|---|
| Four-seam velocity | −1.1 (2025) and −1.2 (2026) mph | −1.7 and −1.5 mph |
| Average exit velocity | −1.3 and −0.8 mph | −2.4 and −1.5 mph |
| Hard-hit rate | −3.9 and −2.5 points | −8.1 and −4.2 points |

### 3.2 NPB ↔ MLB

| Source | Date | Finding | Method |
|---|---|---|---|
| Szymborski (ZiPS), FanGraphs chats [analysis] | 1 Aug 2024; 12 Mar 2026 | 2024: "I still get NPB as AAA1/2 and KBO as AA1/2." 2026: "ZiPS sees NPB as between AAA and MLB"; most of the NPB→MLB/MiLB data covers "the last 25 years". | ZiPS league translations (not published numerically). https://blogs.fangraphs.com/dan-szymborski-fangraphs-chat-8-1-24/ ; https://blogs.fangraphs.com/dan-szymborski-fangraphs-chat-3-12-26/ |
| Jordan Rosenblum, "Creating a Projection for Munetaka Murakami" (FanGraphs) [analysis] | 22 Dec 2025 | **NPB→MLB: wRC+ fell by 49 points** (weighted average of 8 hitters, FanGraphs NPB data since 2019). **53 points** for 19 hitters since 2000 with no prior MLB time (Baseball-Reference data). **MLB→NPB: wRC+ rose by 32 points** (52 hitters). He reads 32 as "comparable to Triple-A" and 53 as "closer to Double-A" (compare his 2024 level table: AAA −35, AA −48). He says Davenport's transition data suggests NPB is "a bit stronger than Triple-A". He used the steeper (AA-like) value for OOPSY 2026. | Delta method: wRC+ in adjacent seasons (last NPB season, first MLB season, or the reverse), weighted by the smaller PA. No aging adjustment ("trivial"). No component (K%, BB%, ISO) factors published. https://blogs.fangraphs.com/creating-a-projection-for-munetaka-murakami/ |
| Clay Davenport, "2019 Japan Statistics" [analysis] | 12 Nov 2019 | "The difficulty rating for the leagues are between the values for our AA and AAA leagues – perhaps a little closer to AA, but definitely in between." Translated EqA examples (untranslated → translated): Seiya Suzuki .347 → .303; Tomoya Mori .328 → .297; Masataka Yoshida .332 → .293. That is −.031 to −.044 EqA; the league average EqA is .260 by definition. | Davenport Translations: league difficulty estimated from players who appear in both leagues; power, strikeouts and average are translated at different rates. https://claydavenport.com/2019/11/12/2019-japan-statistics/ |
| Bradley Woodrum, "Japanese and Korean Prospects in Context" (FanGraphs) [analysis] | 24 Dec 2014 | Hitters lose "about 20 points of wOBA+" moving to MLB (wOBA+ = wRC+ without park factors, using MLB linear weights). Pitchers lose about 10 points of FIP−. | Rule of thumb from his Hardball Times Annual 2014 work. The method is not described in the article. https://blogs.fangraphs.com/japanese-and-korean-prospects-in-context/ |

### 3.3 KBO ↔ MLB

| Source | Date | Finding | Method |
|---|---|---|---|
| Szymborski (ZiPS) [analysis] | 16 Feb 2023; 1 Aug 2024; 12 Mar 2026 | 2023: "ZiPS thinks of the KBO as an AA 1/2 league, so there's a translation hit." 2024: "KBO as AA1/2." 2026: the KBO is "treated like a league between AA and AAA … the depth of talent isn't the same." | ZiPS translations. https://blogs.fangraphs.com/dan-szymborski-fangraphs-chat-2-16-23/ , chats above |
| Szymborski, "Ha-seong Kim Has Big MLB Aspirations and Projections" (FanGraphs) [analysis] | 12 Oct 2020 | Places the KBO "somewhere between Double- and Triple-A". | Based on historical KBO→MLB and KBO→NPB moves; the translations account for park and league offense. https://blogs.fangraphs.com/ha-seong-kim-has-big-mlb-aspirations-and-projections/ |
| Clay Davenport, "Coming to America (from Korea)" [analysis] | 15 Jan 2016 | Regression of US performance on KBO performance gives, for example: KBO EqA .323 → US .283 (Jung-ho Kang); .333 → .293 (Byung-ho Park); .294 → .257 (Hyun-soo Kim). That is −.037 to −.040 EqA at the top of the KBO. The calibration uses mainly Triple-A→KBO movers. Translated aggregate: year 1 (AAA) .232/.306/.390, EqA .247 vs year 2 (KBO) .238/.309/.400, EqA .251, "the translation works very well". He measured KBO league strength up almost 5% from 2013 to 2014 after the foreign-player limit rose. | Davenport Translations: players who played in both leagues, using their US translated values over the previous three years. https://claydavenport.com/2016/01/15/coming-to-america-from-korea/ |
| Ben Howell, "Modeling Plate Discipline from the KBO to MLB" (2021) [analysis] | 2021 | Table 2 (FanGraphs data) for six KBO→MLB hitters in adjacent seasons: Kang, H.S. Kim, B.H. Park, Thames, Hwang, Ruf. **My unweighted means of his table:** K% 17.3 → 24.6 (×1.42); BB% 13.2 → 10.1 (×0.76); wRC+ 161.5 → 103.3 (−58). Every one of the six lost wRC+ and BB/K. | Last KBO season vs first MLB season. Tiny sample with selection bias. https://benhowell71.com/wp-content/uploads/2021/03/Modeling-Plate-Discipline-from-the-KBO-to-MLB.pdf |

Howell's Table 2 (KBO season → MLB season; BB% / K% / wRC+):

| Hitter | KBO | MLB |
|---|---|---|
| Kang | 2014: 13.6 / 21.2 / 189 | 2015: 6.0 / 21.2 / 128 |
| H.S. Kim | 2015: 16.0 / 10.0 / 148 | 2016: 10.4 / 14.7 / 121 |
| B.H. Park | 2015: 12.5 / 25.9 / 184 | 2016: 8.6 / 32.8 / 79 |
| Thames | 2016: 14.0 / 19.5 / 165 | 2017: 13.6 / 29.6 / 125 |
| Hwang | 2016: 8.8 / 11.8 / 134 | 2017: 8.8 / 26.3 / 26 |
| Ruf | 2019: 14.1 / 15.3 / 149 | 2020: 13.0 / 23.0 / 141 |

### 3.4 MLB / Triple-A → KBO and NPB (imported players)

- **MLB→NPB.** Rosenblum (2025): +32 wRC+ for 52 hitters (above).
- **Triple-A/MLB→KBO.** No systematic published study found. The closest is Davenport (2016), whose KBO difficulty factor is calibrated on Triple-A→KBO movers, with year 1 and year 2 translated lines matching (above). Korean press coverage is anecdotal; for example, Sports Kyunghyang (Dec 2023) compares imports' Triple-A and KBO OPS case by case: https://sports.khan.co.kr/article/202312031340003
- **Pitchers.** Apart from Woodrum's "about 10 points of FIP−" (NPB→MLB, 2014), no published pitcher translation factors (K%, BB%, HR rate) were found for either Asian league.

### 3.5 Second teams: KBO Futures → KBO, NPB farm → NPB

Not found. No published translation factors or level placements for the KBO Futures League or the NPB farm were located.

### 3.6 Conflicts between sources

1. **NPB's level.**
   - ZiPS: "AAA½" (2024), then "between AAA and MLB" (2026).
   - Davenport (2019): between AA and AAA, "a little closer to AA".
   - Rosenblum (2025): about AAA from MLB→NPB movers, about AA from NPB→MLB movers; he also cites Davenport as showing NPB "a bit stronger than Triple-A".

   The direction of the move matters, so this reflects selection effects as well as true level.
2. **KBO's level.** Consistent across sources: between AA and AAA (Szymborski/ZiPS 2020, 2023, 2024, 2026). No published source places it outside that range.
3. **Size of the Triple-A→MLB gap.**
   - Rosenblum: −35 wRC+ (2021–23).
   - Davenport: −43 EqA points (2021–25).

   They use different scales, so I did not convert one into the other. Both note or imply the selection-bias problem.
4. **KBO velocity.**
   - 2025: 146.2 (Statiz, via MyDaily) vs 146.0 km/h (Chosun).
   - 2024: 142.6 (Chosun) vs 144.2 (Newsis; the same number as Statiz's 2022 figure).
   - Measurement systems changed in 2025.
5. **MLB 2025 velocity in Korean press.** MyDaily cites Savant as giving an MLB average of 152.4 km/h (94.7 mph) for 2025. Savant and AP give 94.5 mph for four-seamers. MyDaily may have used a different pitch set or per-pitcher averaging.

---

## 4. Average age of batters and pitchers

Source: Baseball-Reference [reference DB].

- **Definitions** (B-Ref column notes): "Batters' average age, weighted by AB + Games Played". "Pitchers' average age, weighted by 3*GS + G + SV".
- **Level rows** marked "computed" are my weighted averages of the league values: batting weighted by league PA, pitching by league batters faced. B-Ref's own weights (AB + G, 3GS + G + SV) are not available at league level, so this is an approximation.

| Level | 2025 batters | 2025 pitchers | 2026 batters | 2026 pitchers | League inputs (batters / pitchers) | Source |
|---|---|---|---|---|---|---|
| MLB | 27.9 | 29.2 | 27.8 | 29.1 | League Average row | https://www.baseball-reference.com/leagues/majors/2025.shtml , .../2026.shtml |
| Triple-A | 26.54 (computed) | 27.23 (computed) | 26.34 (computed) | 27.27 (computed) | 2025: IL 26.4/27.3, PCL 26.8/27.1. 2026: IL 26.1/27.2, PCL 26.8/27.4 | B-Ref register league pages (appendix C) |
| Double-A | 23.79 (computed) | 24.63 (computed) | 23.69 (computed) | 24.65 (computed) | 2025: EL 23.6/24.7, SL 23.8/24.7, TL 24.0/24.5. 2026: EL 23.6/24.8, SL 23.7/24.6, TL 23.8/24.5 | same |
| High-A | 22.24 (computed) | 23.16 (computed) | 22.12 (computed) | 23.02 (computed) | 2025: MWL 22.3/23.1, NWL 22.4/23.2, SAL 22.1/23.2. 2026: MWL 22.1/22.8, NWL 22.4/23.1, SAL 22.0/23.2 | same |
| Single-A | 20.89 (computed) | 21.76 (computed) | 20.69 (computed) | 21.96 (computed) | 2025: CAL 21.3/21.7, CAR 20.5/21.6, FSL 21.0/22.0. 2026: CAL 21.0/22.0, CAR 20.3/21.9, FSL 20.9/22.0 | same |
| KBO | 28.7 | 27.2 | 28.7 | 27.4 | — | https://www.baseball-reference.com/register/league.cgi?id=4e296e3b (2025), ?id=163dcec5 (2026) |
| KBO Futures | not found | not found | not found | not found | B-Ref's "Futures" league is the US Futures Collegiate League, not the KBO Futures League. The KBO's Futures pages list no ages. The KBO's 2026 roster release gives no average age: https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11831 | — |
| NPB (CL+PL) | 27.65 (computed) | 27.80 (computed) | 27.65 (computed) | 27.65 (computed) | 2025: CL 27.9/27.9, PL 27.4/27.7. 2026: CL 27.8/27.8, PL 27.5/27.5 | B-Ref Japan Central/Pacific pages |
| NPB farm | 24.83 (computed) | 26.20 (computed) | 25.1 | 26.2 | 2025: Eastern 25.0/26.5, Western 24.6/25.8. 2026: Farm League 25.1/26.2 | B-Ref Japan Eastern/Western (2025) and Japan Farm (2026) pages |

KBO pitchers are younger than KBO batters (27.2–27.4 vs 28.7). Every other level shows the reverse.

---

## 5. Open gaps

1. **Double-A and High-A tracking.** No public pitch velocity, exit velocity, hard-hit, bat speed or sprint data, and no published league averages. Savant covers only Triple-A and the FSL.
2. **Single-A outside the FSL.** California and Carolina Leagues: no public tracking.
3. **Minor-league bat speed and sprint speed.** Not public. The Savant minors search returns "--" for bat-tracking metrics and has no sprint data.
4. **KBO tracking.**
   - 2026 league-average fastball velocity: not found.
   - The 2025 value conflicts between sources (146.0 vs 146.2 km/h).
   - Exit velocity, hard-hit rate, bat speed, sprint speed and the spread of velocity between pitchers: not found. Statiz, the most likely source, blocked automated access (HTTP 502 through my proxy).
5. **KBO Futures.**
   - Tracking data and average ages: not found.
   - No official league or team totals are published; I summed the player lines, with SF estimated.
   - No translation factor to the KBO first team.
6. **NPB tracking.**
   - No official or press league-average velocity; only a fan compilation (147.1 / 147.7 km/h) and an approximate press figure (about 148 km/h).
   - Exit velocity, bat speed, sprint speed and the spread between pitchers: not found.
7. **NPB farm.** Tracking data and translation factors: not found.
8. **Component translation factors.**
   - The only per-component (K%, BB%, HR, ISO, BABIP) factors are my derivation from Davenport's Triple-A/MLB totals and Howell's six-hitter KBO table.
   - Rosenblum's K%, BB% and HR/BBE level translations exist but are unpublished.
   - Pitcher-side factors (K%, BB%, HR allowed) by level are essentially unpublished. Woodrum's 2014 FIP− rule of thumb for NPB is the only one found.
9. **MLB/AAA→KBO for imported hitters.** No systematic published study. It could be computed from B-Ref register data (Triple-A year t vs KBO year t+1) if needed.
10. **Cause of the 2026 minor-league HR surge** (Double-A and below). Not confirmed; Baseball America's articles could not be opened directly.
11. **Savant zone definition from 2026.** The apparent change in Zone% and chase rate between MLB 2025 and 2026 is unverified. Use Swing% and whiffs per swing for calibration until it is confirmed.
12. **Articles found but not readable.**
    - Beyond the Box Score, "Projecting the NPB to MLB Transition for Batters" (2015; has K%/BB%/ISO results) could not be fetched (connection reset; Wayback copy blocked). https://www.beyondtheboxscore.com/2015/4/1/8322487/projecting-npb-to-mlb-transition-batters-japan-plate-discipline-iso
    - Namuwiki's velocity page (cited by search engines for "2025: MLB 151.7, NPB 148.0, KBO 146.2 km/h") returned HTTP 403, so it is not used. https://namu.wiki/w/%EA%B5%AC%EC%86%8D(%EC%95%BC%EA%B5%AC)
13. **Baseball Prospectus.** No published DRC+ level adjustment for minor leagues, KBO or NPB was found.

---

## Appendix A. Official outcome-data URLs

- **MLB / MiLB**, MLB Stats API team season hitting. Sum all teams for each sportId (1 = MLB, 11 = AAA, 12 = AA, 13 = High-A, 14 = Single-A):
  - `https://statsapi.mlb.com/api/v1/teams/stats?season=2026&sportIds=11&group=hitting&stats=season&gameType=R` (change season and sportIds).
  - League membership: `https://statsapi.mlb.com/api/v1/teams?sportId=11&season=2026`.
- **NPB**, team batting:
  - 2025: https://npb.jp/bis/2025/stats/tmb_c.html , https://npb.jp/bis/2025/stats/tmb_p.html ; farm https://npb.jp/bis/2025/stats/tmb_e.html , https://npb.jp/bis/2025/stats/tmb_w.html
  - 2026: https://npb.jp/bis/2026/stats/tmb_c.html , https://npb.jp/bis/2026/stats/tmb_p.html ; farm https://npb.jp/bis/2026/stats/tmb_2e.html , https://npb.jp/bis/2026/stats/tmb_2c.html , https://npb.jp/bis/2026/stats/tmb_2w.html
- **KBO**:
  - Team batting: https://www.koreabaseball.com/Record/Team/Hitter/Basic1.aspx (season and series chosen with the page's form: 2025 or 2026, "KBO 정규시즌").
  - Team pitching: https://www.koreabaseball.com/Record/Team/Pitcher/Basic1.aspx (2025).
  - 2026 batting with BB/HBP/SO: https://eng.koreabaseball.com/Stats/TeamStats.aspx
- **KBO Futures**:
  - Player batting: https://www.koreabaseball.com/Futures/Player/Hitter.aspx ; player pitching: https://www.koreabaseball.com/Futures/Player/Pitcher.aspx . Each was filtered by season and team through the page form, all pages read: 498 batter lines in 2026 and 479 in 2025.
  - Standings: https://www.koreabaseball.com/Futures/TeamRank/North.aspx?gameDate=2026-11-16 and South.aspx (and gameDate=2025-11-16).

## Appendix B. Baseball Savant queries used

Prefix for MLB: `https://baseballsavant.mlb.com/statcast_search?all=true&hfGT=R%7C&min_pitches=0&min_results=0&min_pas=0&sort_col=pitches&sort_order=desc`

Prefix for minors: `https://baseballsavant.mlb.com/statcast-search-minors?all=true&hfGT=R%7C&min_pitches=0&min_results=0&min_pas=0&sort_col=pitches&sort_order=desc&minors=true`

For minors, `hfFlag=is%5C.%5C.tracked%7C` limits results to tracked pitches. It does not change the velocity or exit-velocity averages, because untracked pitches have no values. `hfLevel=AAA%7C` or `hfLevel=A%7C` selects a level.

- **Q1.** MLB four-seam: `&hfPT=FF%7C&hfSea=2026%7C2025%7C2024%7C2023%7C&player_type=pitcher&group_by=league-year&chk_stats_velocity=on&chk_stats_spin_rate=on&chk_stats_release_extension=on`
- **Q2.** Minors four-seam: same parameters with `group_by=level-year` (plus the tracked flag).
- **Q3.** MLB batted balls: `&hfSea=2026%7C2025%7C2024%7C2023%7C&player_type=batter&group_by=league-year&chk_stats_pa=on&chk_stats_bip=on&chk_stats_launch_speed=on&chk_stats_hardhit_percent=on&chk_stats_launch_angle=on&chk_stats_barrels_per_bbe_percent=on`
- **Q6.** Minors batted balls: same as Q3 with `group_by=level-year` and the tracked flag.
- **Q4 / Q5.** Per-pitcher four-seam averages: `&hfPT=FF%7C&hfSea=2025%7C&player_type=pitcher&group_by=name&chk_stats_velocity=on` (MLB), plus `hfLevel=AAA%7C` or `hfLevel=A%7C` for the minors. The same with 2026.
- **Q7.** MLB bat speed: `&hfSea=2026%7C2025%7C2024%7C&hfFlag=is%5C.%5C.competitive%7C&player_type=batter&group_by=league-year&chk_stats_swings=on&chk_stats_sweetspot_speed_mph=on&chk_stats_swing_length=on&chk_stats_attack_angle=on`
- **Q8.** Minors bat speed: same as Q7 with `group_by=level-year`. Returns "--".
- **Q9.** Bat-tracking leaderboard (qualified batters): `https://baseballsavant.mlb.com/leaderboard/bat-tracking?type=batter&gameType=Regular&minSwings=q&seasonStart=2025&seasonEnd=2025&dateStart=2025-03-01&dateEnd=2025-11-30&csv=true` (and 2026).
- **Q10.** Sprint speed: `https://baseballsavant.mlb.com/leaderboard/sprint_speed?min_season=2025&max_season=2025&position=&team=&min=0&csv=true` (and 2026).
- **Q11.** Swing and whiff counts: `&player_type=batter&chk_stats_pa=on&chk_stats_swings=on&chk_stats_whiffs=on&chk_stats_k_percent=on&chk_stats_bb_percent=on`, grouped by league-year (MLB) or level-year (minors).
- **Q12 / Q13.** As Q11 with `hfZ=1%7C2%7C3%7C4%7C5%7C6%7C7%7C8%7C9%7C` (in zone) or `hfZ=11%7C12%7C13%7C14%7C` (out of zone).

## Appendix C. Baseball-Reference register league pages used

`https://www.baseball-reference.com/register/league.cgi?id=<id>`

| League | 2025 id | 2026 id |
|---|---|---|
| International | 6eda1325 | 24986ebc |
| Pacific Coast | e379cb03 | 4cb49f78 |
| Eastern | 6ca1523b | 3cb9afef |
| Southern | 0a0f5955 | 6d155a4f |
| Texas | 8b9fc809 | f7e94388 |
| Midwest | 1de37e23 | 58ec9e82 |
| Northwest | 257fa48d | 00b8bbe3 |
| South Atlantic | a0ffbab5 | ba63c1e4 |
| California | ffd2dae5 | 3dc8ac04 |
| Carolina | 71af6568 | cdadfd83 |
| Florida State | b04525c8 | 5d28341c |
| Japan Central | aa6efe96 | 4f61709d |
| Japan Pacific | ab01bc4f | 9f67f305 |
| Japan Eastern (farm) | ae6eefd5 | — |
| Japan Western (farm) | 96ad99c9 | — |
| Japan Farm | — | 896ff3ee |
| KBO | 4e296e3b | 163dcec5 |
