# MLB / MiLB structure notes (state on 2026-11-16)

Research date: 2026-11-16. Every item carries its source. "Stats API" means MLB's own data service
(`statsapi.mlb.com`), queried directly; where a number was *computed* from that data it says so.
Nothing here is from memory alone; where I could not confirm a value it is marked **not confirmed**.

Companion files: `mlb_clubs.json` (30 records), `milb_clubs.json` (120 records).

Abbreviations used for documents cited repeatedly:

- **MLR 2026** = Major League Rules, 2026 edition ("The Official Professional Baseball Rules Book", Office of the Commissioner).
  PDF: https://0359b39c-af98-4f91-8753-051c201c8092.filesusr.com/ugd/638123_e8a18872c61d4c0093ae389e26bc62f7.pdf
  (found via https://wikileague.pages.dev/documents/2026-03-17_rules_major-league-rules/ , which flags the copy as
  "needs review"). I cross-checked the passages used here against the 2024 edition hosted by the MLBPA
  (https://agent.mlbplayers.com/_files/ugd/4d23dc_168483d9e6a7481ab3da1f5f0ea90f41.pdf): the numbers are identical
  except where noted.
- **OBR 2026** = Official Baseball Rules, 2026 edition: https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf
- **BA 2022-26** = MLB-MLBPA Basic Agreement 2022-2026: https://agent.mlbplayers.com/_files/ugd/4d23dc_fa8575fb2a2c4f0ca117bc181aeb6206.pdf
- **MiLB BA** = 2023-2027 Minor League Basic Agreement: https://agent.mlbplayers.com/_files/ugd/4d23dc_ebe9aae5da0b452b9f6dbfe5316df884.pdf

---

## a. MLB regular season, key dates, postseason

### a.1 Schedule format

| Item | Value | Source |
|---|---|---|
| Games per club | 162, scheduled over 182 to 187 days | BA 2022-26, Art. V(A) |
| Leagues / divisions | AL and NL, 15 clubs each, three 5-club divisions (East, Central, West) | Stats API https://statsapi.mlb.com/api/v1/teams?sportId=1&season=2026 |
| Format 2023-2024 | 52 division games (13 v each of 4 rivals); 64 v the 10 other same-league clubs (6 games v 6 clubs, 7 games v 4 clubs); 46 interleague (4 v the "prime"/natural rival, 3 v each of the other 14) | Computed from the Stats API schedule for 2023 and 2024: all 30 clubs = 52/64/46. https://statsapi.mlb.com/api/v1/schedule?sportId=1&season=2023&gameType=R&startDate=2023-03-01&endDate=2023-10-15 |
| Change in 2025 | Prime interleague rivals went from 4 games to 6 (two 3-game series); "each of the 30 Clubs will play two fewer games against non-division league opponents" | https://www.mlb.com/press-release/press-release-mlb-announces-2025-regular-season-schedule |
| Format 2025, 2026 and 2027 | **52 division (13 x 4); 62 same-league non-division (6 games v 8 clubs, 7 games v 2 clubs); 48 interleague (6 v the prime rival home-and-home, one 3-game series v each of the other 14)** | Computed from the Stats API schedule: all 30 clubs = 52/62/48 in 2025, in 2026 (played) and in the published 2027 schedule. https://statsapi.mlb.com/api/v1/schedule?sportId=1&season=2026&gameType=R&startDate=2026-03-01&endDate=2026-10-15 (same with season=2027) |
| Did it change for 2026? | No. 2026 keeps two 3-game series v the prime rival; clubs host the same interleague opponents as in 2024 (alternating years) | https://www.mlb.com/press-release/press-release-mlb-announces-2026-regular-season-schedule |
| Did it change for 2027? | No format change announced; the published 2027 schedule has the same 52/62/48 split. The schedule is "subject to change" and depends on a new CBA (see e.) | https://www.mlb.com/news/mlb-2027-schedule-released ; computed as above |
| Rivalry Weekend | 2026: May 15-17, 11 prime-interleague-rival series plus 4 regional series (PIT-PHI, COL-AZ, DET-TOR, HOU-TEX). 2027: opens the second half on July 15 | 2026 press release above; https://www.mlb.com/news/mlb-2027-schedule-released |
| Before 2023 (for reference) | 2022: 76 division (19 x 4), 66 league, 20 interleague | Computed from Stats API schedule, season=2022 |

Other 2026 scheduling facts: Athletics home series at Las Vegas Ballpark June 8-10 (MIL) and June 12-14 (COL); Mexico City Series
AZ v SD April 25-26; Field of Dreams game MIN v PHI Aug 13; Little League Classic MIL v ATL Aug 23; FIFA World Cup
accommodations (off days in Arlington, Philadelphia, Kansas City, Seattle).
Sources: https://www.mlb.com/press-release/press-release-mlb-announces-2026-regular-season-schedule ; https://www.mlb.com/news/abs-challenge-system-mlb-2026

### a.2 Key dates

| Event | 2026 (played) | Source | 2027 (announced) | Source |
|---|---|---|---|---|
| Pitchers and catchers | First workouts Feb 10-13 depending on club (Feb 10: AZ, ATL, BOS, CWS, SF, TEX; Feb 13: LAD). WBC participants had to report by Feb 11 (P/C) / Feb 12 (position players) | https://www.mlb.com/press-release/press-release-first-workout-dates-and-photo-days-announced-for-2026-spring-training (table is an image in the release) | **Not announced** (null). Spring Training Online only shows projections | https://www.springtrainingonline.com/2027-spring-training-reporting-dates/ |
| Full-squad first workouts | Feb 15-17 | same release | Not announced (null) | - |
| First spring training games | Fri Feb 20 (full slate Sat Feb 21) | https://www.mlb.com/news/press-release-mlb-announces-2026-spring-training-schedule | **Fri Feb 19, 2027**, all 30 clubs | https://www.mlb.com/press-release/press-release-mlb-announces-2027-spring-training-schedule |
| Opening Night | Wed Mar 25 (NYY at SF) | https://www.mlb.com/press-release/press-release-mlb-announces-2026-regular-season-schedule | **Wed Mar 24, 2027** (matchup TBD, Netflix) | https://www.mlb.com/news/mlb-2027-schedule-released |
| Traditional Opening Day | Thu Mar 26 (14 games) | same | **Thu Mar 25, 2027** (14 games); first full slate Sat Mar 27 | same |
| All-Star Game | Tue Jul 14, Citizens Bank Park, Philadelphia (96th) | same | **Tue Jul 13, 2027**, Wrigley Field, Chicago (97th); Home Run Derby Jul 12 | same |
| Second half starts | Thu Jul 16 | Stats API https://statsapi.mlb.com/api/v1/seasons?sportId=1&season=2026 | Thu Jul 15, 2027 (one game), rest Jul 16 | https://www.mlb.com/news/mlb-2027-schedule-released |
| Trade deadline | Mon Aug 3, 6 p.m. ET. The CBA lets the Commissioner set it between Jul 28 and Aug 3 | https://www.mlb.com/news/mlb-trade-deadline-2026-date-time | **Not announced** (null); the rule that allows the date to float is in the expiring CBA | - |
| Postseason eligibility cut-off | Player must be in the organisation by noon ET Sept 1 (see rule text) | MLR 2026 Rule 41(a); https://www.mlb.com/news/mlb-trade-deadline-2026-date-time | unchanged unless a new CBA changes it | - |
| Roster expansion to 28 | Sept 1 | MLR 2026 Rule 2(b)(2)(B) | same rule | - |
| Last day of regular season | Sun Sep 27 | 2026 schedule press release; Stats API | **Sun Sep 26, 2027** | https://www.mlb.com/news/mlb-2027-schedule-released |
| Postseason | Wild Card Series Sep 29-Oct 1; Division Series Oct 3-10; LCS Oct 11-19; World Series Oct 23-28 (ended in 5 games) | Stats API https://statsapi.mlb.com/api/v1/schedule?sportId=1&season=2026&gameType=F,D,L,W&startDate=2026-09-25&endDate=2026-11-10 | Not announced | - |

### a.3 Postseason format (2022 onward, in force 2026)

| Item | Value | Source |
|---|---|---|
| Teams | 12: three division champions and three Wild Cards per league | MLR 2026 Rule 35(a); BA 2022-26 Art. V(A); https://www.mlb.com/news/mlb-playoff-format-faq |
| Seeding | Division champions are seeds 1-3 by winning percentage; Wild Cards are seeds 4-6 by winning percentage. A Wild Card with a better record than a division champion still seeds below it | MLR 2026 Rule 35(b); MLB.com FAQ |
| Byes | Seeds 1 and 2 skip the Wild Card Series | MLR 2026 Rule 35(c)(1) |
| Wild Card Series | Best of 3: 3 v 6 and 4 v 5; **all games at the higher seed's park** | MLR 2026 Rules 35(c)(2), 38(a)(1) |
| Division Series | Best of 5, 2-2-1 (games 1, 2, 5 at higher seed). No reseeding: seed 1 plays the 4/5 winner, seed 2 plays the 3/6 winner | MLR 2026 Rules 35(d), 38(a)(2) |
| League Championship Series | Best of 7, 2-3-2 (games 1, 2, 6, 7 at higher seed) | MLR 2026 Rules 35(e), 38(a)(3) |
| World Series | Best of 7, 2-3-2; home field to the club with the better regular-season winning percentage regardless of seed; tie broken by Rule 34(c) | MLR 2026 Rules 35(f), 38(a)(4) |
| Ties | No tiebreaker games. Two-club tie: (1) head-to-head winning percentage, (2) intradivision record, (3) intraleague record, (4) record in the last half of intraleague games (= after the All-Star break), (5) last half plus one more intraleague game, working backwards until broken. Division ties are resolved before Wild Card ties | MLR 2026 Rule 34(c), 34(d) |
| Postseason rules differences | No automatic runner in extra innings; managers get 2 replay challenges per game instead of 1; ABS challenge works as in the regular season | OBR 2026 Rule 7.01(g); MLB.com FAQ |
| Postseason active roster | 26 (25 minimum) | MLR 2026 Rule 41(a)(3) |

---

## b. Minor-league season format (2026)

Structure in 2026: 120 full-season affiliates, 30 per level, one per MLB club per level; 11 leagues.
Source: Stats API team lists, e.g. https://statsapi.mlb.com/api/v1/teams?sportId=11&season=2026 (sportId 11 = Triple-A, 12 = Double-A, 13 = High-A, 14 = Single-A).

| Level | Leagues (clubs) | Games per club | First game 2026 | Last regular-season game 2026 | Halves | Source |
|---|---|---|---|---|---|---|
| Triple-A | International League (20: East 10, West 10); Pacific Coast League (10: East 5, West 5) | 150 | Fri Mar 27 | Sun Sep 20 | 1st half ends Jun 21, 2nd half starts Jun 23 | https://www.mlb.com/milb/events/playoffs/procedures ; games and dates computed from Stats API schedule (sportId=11) |
| Double-A | Eastern (12: Northeast 6, Southwest 6); Southern (8: North 4, South 4); Texas (10: North 5, South 5) | 138 | Thu Apr 2 (Southern League Fri Apr 3) | Sun Sep 13 | 1st half ends Jun 21, 2nd starts Jun 23 | Computed from Stats API schedule (sportId=12) and https://statsapi.mlb.com/api/v1/seasons?sportId=12&season=2026 ; https://www.mlb.com/news/minor-league-opening-day-guide-2026 |
| High-A | Midwest (12: East 6, West 6); South Atlantic (12: North 6, South 6); Northwest (6, no divisions) | 132 | Thu Apr 2 (Northwest League Fri Apr 3) | Sun Sep 6 | 1st half ends Jun 18, 2nd starts Jun 19 | Computed from Stats API schedule (sportId=13); seasons endpoint sportId=13 |
| Single-A | California (8: North 4, South 4); Carolina (12: North 6, South 6); Florida State (10: East 4, West 6) | 132 | Thu Apr 2 | Sun Sep 6 | 1st half ends Jun 18, 2nd starts Jun 19 | Computed from Stats API schedule (sportId=14); seasons endpoint sportId=14 |

Series structure (computed from the 2026 Stats API schedules; consistent with club releases such as
https://www.milb.com/news/bats-2027-schedule-release ):

- Standard unit is a **six-game series, Tuesday to Sunday, with every Monday off** (league-wide). In 2026 about 88 % of
  all series were six-gamers and 8 % three-gamers (the remainder are split or make-up series); only a handful of games were played
  on Mondays (e.g. Memorial Day May 25, Labor Day Sep 7).
- Each season opens with a **three-game series** (Triple-A Fri-Sun Mar 27-29; lower levels Thu-Sat or Fri-Sun Apr 2-5).
- **All-Star break**: no games at any of the four levels Mon Jul 13-Thu Jul 16, 2026, followed by a three-game series Fri-Sun Jul 17-19.
- Triple-A played **interleague series** between the International League and the PCL in 2026 (each league hosted four
  six-game series against clubs of the other league; e.g. Oklahoma City at Charlotte Jun 9-14).
  Sources: computed from Stats API; https://www.milb.com/charlotte-knights/news/knights-unveil-2026-full-schedule-highlighted-by-a-first-time-visitor
- Example of opponent mix (Double-A Biloxi, 2027): 84 of 138 games inside its four-team division. https://www.mlb.com/milb/biloxi/news/minor-league-baseball-announces-shuckers-2027-schedule

Playoffs 2026 (official procedures: https://www.mlb.com/milb/events/playoffs/procedures ; series actually played checked in the Stats API):

| Level / league | Qualifiers | Format | Dates 2026 |
|---|---|---|---|
| Triple-A (IL and PCL separately) | First-half winner v second-half winner of each league | Best-of-3 League Championship Series, **all games hosted by the first-half winner**, after an off day | Sep 22-24 |
| Triple-A National Championship | IL champion v PCL champion | Single game at Las Vegas Ballpark | Sat Sep 26 |
| Double-A, High-A (Midwest, South Atlantic), Single-A: every league with two divisions | 4 clubs: the winner of each division in each half (best winning percentage). "In the event the same Club wins both halves, the Club with the next best winning percentage in the second half will advance" | Best-of-3 Division Series, then best-of-3 Championship Series. In 2026 every series was played 1-2 (game 1 at one park, games 2-3 at the other) | Double-A: Sep 15-18 and Sep 20-23. High-A and Single-A: Sep 8-11 and Sep 13-16 |
| High-A Northwest League (6 clubs) | 2 clubs: first-half winner and second-half winner | Single best-of-5 Championship Series | from Sep 8 |
| Tie-breakers (all levels) | (1) head-to-head in that half, (2) winning percentage over the last 20 games, (3) add one game at a time (last 21, 22, ...) | | |

Home-field detail (older source, 2022, not restated in the 2026 procedures): in the Division Series the first-half winner hosts
games 2 and 3; in the Championship Series hosting alternates by division by year. https://www.milb.com/news/minor-league-postseason-procedures-guide

2027 minor-league schedule (announced August 2026; loaded in the Stats API): Triple-A Mar 26-Sep 19 (150 games); Double-A Apr 2-Sep 12 (138);
High-A and Single-A Apr 2-Sep 5 (132). Same six-game-series/Monday-off structure.
Sources: https://www.milb.com/news/bats-2027-schedule-release ; https://www.mlb.com/milb/biloxi/news/minor-league-baseball-announces-shuckers-2027-schedule ; Stats API schedule season=2027.

---

## c. Roster-size rules (2026)

### Major League

| Rule | Value | Source |
|---|---|---|
| Active roster, Opening Day to Aug 31 and postseason | Maximum 26, minimum 25 | MLR 2026 Rule 2(b)(2)(A) |
| Pitcher limit | Maximum 13 pitchers (two-way players exempt) | MLR 2026 Rule 2(b)(2)(A), (D); OBR 2026 Rule 4.03(c) |
| September (Sept 1 to end of regular season) | Clubs **must carry 28**; maximum 14 pitchers | MLR 2026 Rule 2(b)(2)(B) |
| 27th player | Allowed for doubleheaders before Sept 1; does not count against the 13-pitcher limit | MLR 2026 Rule 2(b)(2)(A)(i)-(viii) |
| Pitcher / position-player designation | Fixed for the whole season the first time the player is on the active roster | MLR 2026 Rule 2(b)(2)(C) |
| Two-way player | Qualifies with at least 20 MLB innings pitched **and** at least 20 MLB starts as position player or DH with 3+ plate appearances in each, in the current season or either of the two previous seasons | MLR 2026 Rule 2(b)(2)(D)(ii) |
| 40-man roster (Major League Reserve List) | Maximum 40 | MLR 2026 Rule 2(a)(2)(A) |
| Minimum stay in the minors after being optioned | 10 days for position players, 15 days for pitchers and two-way players (exceptions: 27th man, injury replacement). A player may be optioned at most 5 times per season | https://www.mlb.com/glossary/transactions/26-man-roster ; https://www.mlb.com/glossary/transactions/40-man-roster |
| Injured lists | 7-day (acute concussion only), 10-day (position players only), 15-day (pitchers and two-way players), 60-day (anyone). The number is the minimum stay | MLR 2026 Rule 2(c)(1)(A) |
| IL and roster counts | 7/10/15-day IL players count against the 40-man but not the active roster; 60-day IL players count against neither. A 60-day placement requires a full 40-man roster. No limit on the number of players on the IL | MLR 2026 Rule 2(c)(1)(F) |
| Backdating | 10- and 15-day IL stints can be backdated at most 3 days (to the day after the player's last game) | https://www.mlb.com/glossary/injuries/10-day-injured-list |
| Transfers | Time on the 7/10/15-day IL counts toward the 60 days; a player put on the 60-day IL before the season cannot return until 60 days of the season have elapsed | MLR 2026 Rule 2(c)(1)(B), (F)(vii) |
| Bereavement / Family Medical Emergency List | Minimum 3, maximum 7 consecutive days; counts against the 40-man, not the active roster | MLR 2026 Rule 2(c)(2)(C)-(D) |
| Paternity Leave List | Minimum 1, maximum 3 consecutive days | MLR 2026 Rule 2(c)(3)(C) |

### Minor leagues

| Rule | Value | Source |
|---|---|---|
| Active roster, Triple-A | Maximum 28, minimum 24 | MLR 2026 Rule 2(b)(3)(A) |
| Active roster, Double-A | Maximum 28, minimum 24 | MLR 2026 Rule 2(b)(3)(B) |
| Active roster, High-A | Maximum 30, minimum 24 | MLR 2026 Rule 2(b)(3)(C) |
| Active roster, Single-A | Maximum 30, minimum 24 | MLR 2026 Rule 2(b)(3)(D) |
| Rookie | Dominican Summer League clubs 35; no maximum at the US complex leagues | MLR 2026 Rule 2(b)(3)(E)-(F) |
| Experience limits (secondary source only) | High-A: at most 2 players and 1 player-coach with 6+ years of prior service; Single-A: at most 2 players with 5+ years. No pitcher limit in the minors | https://www.thecubreporter.com/minor-league-contracts-roster-rules (Arizona Phil's roster-rules guide, updated Nov 2026) - not checked against the MLR text |
| **Domestic Reserve List, in season** | **165 players** per organisation, "from Opening Day of the AA championship season until 5 P.M. ET on the fifth day following the day that the last game of the World Series starts" | MLR 2026 Rule 2(a)(2)(B). (The 2024 edition says "from the earliest Opening Day of the Minor League season".) |
| **Domestic Reserve List, off-season** | **175 players**, from that moment until the next Double-A Opening Day. A club that goes over has 48 hours to comply | MLR 2026 Rule 2(a)(2)(C) |
| Background | The MiLB BA kept 180/190 through 2023 and allowed MLB to cut by at most 15 (to 165/175) afterwards; MLB did so from 2024 | MiLB BA Art. IX |
| Who counts | All players on domestic minor-league rosters including those on the 7-day IL, Development List and Temporarily Inactive List. Not counted: players on the minor-league 60-day or Full-Season IL, MLB players on rehab assignment, and first-year signees for their first 15 days (or until they play) | MLR 2026 Rules 2(a)(3), 2(c)(1)(F), 2(c)(11)(B), 2(c)(12)(B) |
| Optioned 40-man players (secondary source only) | The in-season 165 includes players on optional assignment from the 40-man roster; the off-season 175 does not | https://www.thecubreporter.com/minor-league-contracts-roster-rules |
| Other reserve lists | 38 players on the Triple-A reserve list for Rule 5 purposes; 35 on the International Rookie (DSL) list per DSL club | MLR 2026 Rule 2(a)(2)(D)-(E) |
| Minor-league Regular IL | Minimum 7 consecutive days; counts against the reserve list, not the active list | MLR 2026 Rule 2(c)(11)(A)(i), (B) |
| Minor-league 60-Day IL | Minimum 60 consecutive days of the season; needs Commissioner's Office approval; **maximum 15 players per organisation at a time** (DSL players excluded); does not count against the Domestic Reserve List | MLR 2026 Rule 2(c)(11)(A)(ii), (B) |
| Minor-league Full-Season IL | For injuries that normally cost a whole season (e.g. Tommy John surgery); player stays on it for the rest of the season including playoffs; does not count against the Domestic Reserve List; no numerical cap stated | MLR 2026 Rule 2(c)(11)(A)(iii), (B) |
| Development List | Minimum 7 consecutive days; not for injured players or players on optional assignment; counts against the Domestic Reserve List but not the active list | MLR 2026 Rule 2(c)(12) |
| Temporarily Inactive List | Minimum 3 consecutive days; in-season only | MLR 2026 Rule 2(c)(13) |

---

## d. Playing rules in force in 2026

### Major League

| Rule | Exact parameters | Source |
|---|---|---|
| Pitch timer | **15 s with the bases empty, 18 s with runners on** (18 since 2024; 20 in 2023). **30 s between batters.** Pitcher must start his delivery before the timer expires; batter must be in the box and alert by the **8-second mark**. Pitcher violation = automatic ball; batter violation = automatic strike. Batter gets **one timeout per plate appearance**. After a dead ball the timer restarts as soon as the pitcher has the ball and play is ready | https://www.mlb.com/glossary/rules/pitch-timer ; https://www.mlb.com/press-release/press-release-mlb-announces-rules-modifications-for-2024-season ; https://www.mlb.com/news/mlb-rules-changes-refresher-2023-season |
| Disengagements (pickoffs / step-offs) | **2 per plate appearance** with runners on; the count resets if a runner advances. A third disengagement must retire a runner, otherwise it is a balk (runners advance one base). Mound visits, injury time-outs and offensive time-outs do not count. With runners on, the timer resets on a disengagement | https://www.mlb.com/glossary/rules/pitch-timer ; https://www.mlb.com/news/mlb-rules-changes-refresher-2023-season |
| Mound visits | 4 per team per nine innings; one extra in the 9th if a team has none left; one extra per extra inning (not carried over); 30 s each | OBR 2026 Rule 5.10(m)(1); https://www.mlb.com/glossary/rules/mound-visit |
| Inning breaks | MLB's Dec 2023 release refers to a 2:15 inning-break clock, reset to 2:00 if a reliever reaches the warning track with less than 2:00 left. Full 2026 Pace of Game Regulations **not found** | https://www.mlb.com/press-release/press-release-mlb-announces-rules-modifications-for-2024-season |
| Defensive shift restriction | When the pitcher is on the rubber and starts his motion, at least 4 fielders besides pitcher and catcher must have both feet in front of the outer boundary of the infield dirt; when the ball is released at least **2 infielders must be entirely on each side of second base**; infielders may not switch sides during an inning except at a substitution. Penalty: if the violating infielder is the first to touch the ball, batter gets first base and runners advance one base; otherwise the pitch is a ball; the offence may decline and take the play | OBR 2026 Rule 5.02(c) and Penalty; https://www.mlb.com/glossary/rules/defensive-shift-limits |
| Base size | First, second and third base are **18 inches square** (15 before 2023); this shortens 1B-2B and 2B-3B by 4.5 inches and home-1B / 3B-home by 3 inches | OBR 2026 Rule 2.03; https://www.mlb.com/glossary/rules/base-sizes ; https://www.mlb.com/news/mlb-rules-changes-refresher-2023-season |
| Extra-inning automatic runner | Every half-inning after the 9th starts with a runner on second: the player preceding that half-inning's lead-off hitter in the batting order (or a pinch-runner; if that player is the pitcher, the one before him may be used). The run is unearned for the pitcher (scored as if he reached on an error, no error charged). **Regular season only**; not used in the postseason | OBR 2026 Rule 7.01(b)(2), 7.01(g); https://www.mlb.com/glossary/rules/designated-runner |
| Three-batter minimum | A starting or relief pitcher must face at least three consecutive batters (to the completion of their plate appearances) or pitch to the end of the half-inning, unless injured or ill. If he ends an inning before facing three and comes back out, he must complete the three. An intentional walk counts; a pickoff does not | OBR 2026 Rule 5.10(g)(1) and Comment |
| Warm-up rule | A pitcher who is sent out to warm up for an inning must face at least one batter | https://www.mlb.com/press-release/press-release-mlb-announces-rules-modifications-for-2024-season |
| Position players pitching | Only players designated pitcher or two-way player may pitch, except that any player may pitch (A) after the 9th inning of an extra-inning game, (B) when his team is **losing by 8 or more runs**, or (C) when his team is **winning by 10 or more runs in the 9th inning** | OBR 2026 Rule 4.03(c)(4); MLR 2026 Rule 2(b)(2)(D)(iv) |
| Designated hitter | Universal DH (both leagues since 2022). The DH is locked to his lineup spot; the DH role is lost for the game if the DH takes the field, the pitcher moves to another position, etc. | OBR 2026 Rule 5.11(a); https://www.mlb.com/glossary/rules/designated-hitter-rule |
| Starting pitcher as DH ("two separate people") | A starting pitcher who bats for himself is listed twice (P and DH): if he is relieved as pitcher he stays in as DH; if he is replaced as DH he may keep pitching. Usable only once, on the initial lineup card | OBR 2026 Rule 5.11(b) |
| Two-way player | Does not count toward the 13-pitcher limit, may pitch in any situation, goes on the 15-day IL as a pitcher; qualification as in section c | MLR 2026 Rule 2(b)(2)(D), 2(c)(1)(A)(i) |
| **ABS Challenge System (new in MLB for 2026)** | Human umpire calls every pitch. **Each team starts with 2 challenges; a successful challenge is retained**, a failed one is lost. **Extra innings:** a team that starts an extra inning with zero challenges gets one for that inning (again in each later inning if it is out). **Only the batter, the pitcher or the catcher** may challenge, by tapping cap or helmet immediately after the call (roughly within 2 seconds), with no help from the dugout or other players; the umpire can refuse a late or assisted challenge. No challenges when a position player is pitching, and none after a replay review. Used in every regular-season and postseason game played in an MLB park (not at the Mexico City Series, Field of Dreams game or Little League Classic in 2026). Calls unaffected by the reversed ball/strike call stand (e.g. a stolen base) | https://www.mlb.com/news/abs-challenge-system-mlb-2026 ; https://www.mlb.com/press-release/press-release-mlb-announces-abs-challenge-system-coming-to-the-major-leagues-beginning-in-the-2026-season |
| ABS strike zone | **Width 17 inches** (the plate). **Top at 53.5 % of the batter's height, bottom at 27 % of his height** (height measured standing, without cleats). The zone is a **two-dimensional plane at the middle of the plate**, 8.5 inches from the front and from the back. (For comparison the human-called zone averaged about 55.6 % / 24.2 %.) | https://www.mlb.com/news/abs-challenge-system-mlb-2026 |
| ABS usage data (for calibration) | Spring Training 2025: 4.1 challenges per game, 13.8 s each, 2.6 % of called pitches challenged, 52.2 % overturned (defence 54.4 %, hitters 50.0 %); Triple-A 2025 overturn rate about 50 % | same |
| Replay review | One manager challenge per game in the regular season, two in the postseason | https://www.mlb.com/news/mlb-playoff-format-faq |
| Other 2026 changes | Base coaches must stay inside the coach's box while the pitcher is on the rubber (warning, then ejection). A runner who initiates contact with a fielder to draw an obstruction call is out and other runners return. OBR 2026 otherwise contains clarifications (lineup-card errors, pitching-change signals, retouch, catcher set-up on plays at the plate) | https://www.mlb.com/news/base-coach-box-set-for-2026-season ; OBR 2026 "Summary of Rule Changes for 2026" |

Note: the pitch-timer regulations are not printed in the OBR (Rule 5.07(c) still shows the legacy 12-second rule); the parameters above come from MLB.com.

### Minor leagues in 2026 (experimental rules by level)

Sources: MLB.com summary https://www.mlb.com/news/new-rule-changes-coming-to-minor-leagues-in-2026 (2026-03-17) and the league memo
published by FanGraphs https://blogs.fangraphs.com/exclusive-mlb-to-implement-experimental-minor-league-rule-changes-for-2026 (2026-03-16).

| Level / league | ABS | Other 2026 experiments |
|---|---|---|
| Triple-A, Pacific Coast League | ABS Challenge, same rules as MLB (2 challenges, retained if successful, extra one in extra innings) | From May 5: **Check-Swing Challenge** (a swing = bat head more than 45 degrees past the handle, measured by bat tracking); uses the same 2 challenges (ball/strike or swing/no-swing, not both on one pitch). PitchCom stoppage costs a mound visit (automatic ball if none left). Batter time-out: clock resets immediately, batter must be ready at 8 s |
| Triple-A, International League | ABS Challenge, same rules as MLB | No check-swing challenges, but from May 5 umpires call check swings by the 45-degree standard. **Second half: second base moved fully inside the diamond**, shortening 1B-2B and 2B-3B by about 9 inches. PitchCom and batter-time-out rules as in the PCL |
| Double-A (all leagues) | None listed (umpire-called) | **Disengagement limit reduced from 2 to 1** (a second disengagement with all runners safe = runners advance one base; resets when a runner advances). Batter time-out: clock resets immediately |
| High-A (all leagues) | None listed | Batter may request time only with runners on base |
| Single-A, Florida State League | **Combined ABS and Check-Swing Challenge**, same format as the PCL | Batter may not request time at all (except special circumstances) |
| Single-A, California and Carolina Leagues | None listed | Batter may not request time at all |
| All levels | - | Clock no longer stops when the catcher leaves the box to give defensive signals (must be back with 9 s left, else automatic ball); mound visits must be over before the visit clock hits zero (else automatic ball); base-coach box and obstruction guidance as in MLB |
| Complex leagues, DSL (and AFL per MLB.com) | - | Starting pitcher may re-enter at the start of the next inning if he threw 25+ pitches in the inning he left |

**Not confirmed:** the pitch-timer values used in the minors in 2026. The last official figures I found are for 2022
(14 s empty; 18 s with runners at Double-A and below, 19 s at Triple-A): https://www.mlb.com/news/mlb-2023-rule-changes-pitch-timer-larger-bases-shifts .
The 2026 memo only says the Disengagement Limit "at other levels" is 2 (with reset on runner advance).

---

## e. Announced for 2027, and status of the CBA

### Officially announced

| Topic | What is announced | Source |
|---|---|---|
| 2027 MLB schedule | Released 2026-07-16: Opening Night Mar 24, Opening Day Mar 25 (earliest ever), All-Star Game Jul 13 at Wrigley Field, season ends Sep 26; same 162-game 52/62/48 format | https://www.mlb.com/news/mlb-2027-schedule-released ; Stats API |
| 2027 Spring Training | Games begin Feb 19, 2027; exhibition games at MLB parks Mar 20-23 | https://www.mlb.com/press-release/press-release-mlb-announces-2027-spring-training-schedule |
| Playing rules for 2027 | **Nothing announced** that I could find (no Competition Committee decision, no MLB release). The 2026 minor-league experiments (check-swing challenge, one-disengagement limit, moved second base, batter time-out limits) are described by MLB as "simply experimental, with no imminent arrival in the Majors" | https://www.mlb.com/news/new-rule-changes-coming-to-minor-leagues-in-2026 |
| Expansion / realignment | **Nothing official.** No expansion cities, no committee decision, no realignment announced; only public remarks by the Commissioner about wanting to choose two markets before his term ends in early 2029. Do not model for 2027 | search of MLB.com and news on 2026-11-16; e.g. https://www.cbssports.com/mlb/news/mlb-expansion-what-to-know-about-plans-fees-possible-locations-more-as-league-looks-to-add-two-new-teams |
| Athletics | Stay at Sutter Health Park, West Sacramento, through 2027; move to a new Las Vegas Strip ballpark planned for 2028 (reported on schedule in May 2026) | https://www.espn.com/mlb/story/_/id/39873638/athletics-play-minor-league-park-vegas-residency (three seasons in West Sacramento, 2025-27) ; https://www.espn.com/mlb/story/_/id/48840195/a-2b-las-vegas-stadium-remains-schedule-open-2028 ; https://www.mlb.com/news/mlb-2026-schedule-released |
| Rays | Back at Tropicana Field since 2026-04-06. A $2.36 B ballpark on Hillsborough College's Dale Mabry campus in Tampa was approved by Tampa City Council (4-3, Aug 27, 2026) and Hillsborough County (5-2, Aug 28, 2026); target opening 2029 per local reporting. No change for 2027 | https://www.mlb.com/news/tampa-city-council-advances-rays-stadium-deal-to-final-vote ; https://www.wtsp.com/article/news/local/hillsboroughcounty/tampa-bay-rays-new-stadium-approved-hillsborough-vote/67-ad20a9b1-7979-49a8-9793-f864fb51124d ; https://www.mlb.com/news/rays-set-to-return-to-repaired-tropicana-field |
| Royals | New downtown ballpark at Crown Center announced 2026-04-22 (about 34,000 seats, open-air; groundbreaking expected Q1 2027; planned opening 2030). Kauffman Stadium fences were moved in for 2026. No change for 2027 | https://www.ky3.com/2026/10/14/royals-share-additional-downtown-stadium-details-special-reveal/ ; https://www.mlb.com/news/royals-moving-outfield-walls-at-kauffman-stadium |
| MiLB rebrands for 2027 | Scranton/Wilkes-Barre RailRiders -> **Scranton/Wilkes-Barre Highlanders**; Springfield Cardinals -> **Ozarks Springbirds**; Wichita Wind Surge -> **Wichita Turbo Tubs**. Same cities, parks and parent clubs | https://www.mlb.com/news/yankees-triple-a-affiliate-rebrands-as-highlanders ; https://www.mlb.com/milb/springfield/news/springfield-cardinals-rebrand-to-ozarks-springbirds ; https://www.mlb.com/milb/wichita/news/the-wind-surge-to-officially-rebrand-as-the-turbo-tubs |
| MiLB ballpark renames | Lehigh Valley: Coca-Cola Park -> Jefferson Park from 2027; Columbia: Segra Park -> Prisma Health Park (announced 2026-10-28) | https://www.mlb.com/milb/lehigh-valley/news/lehigh-valley-ironpigs-announce-expanded-community-outreach-efforts-as-home-ballpark-renamed-jefferson-park ; https://www.mlb.com/milb/columbia/news/261028-fireflies-home-to-become-prisma-health-park-under-15-year-naming-rights-agreement |
| MiLB relocations / affiliation changes for 2027 | **None found.** The Stats API team lists for season=2027 show the same 120 clubs, parents and leagues as 2026 (differences: the SWB and Springfield names and the two park names above). A tracker of yearly changes lists only the three rebrands | Stats API (season=2027); https://minorleaguesource.com/bblinkstate.html (updated 2026-11-08) |
| Eugene Emeralds | Staying at PK Park through at least 2027; the Medford stadium plan was abandoned in November 2026 | https://ballparkdigest.com/2026/11/15/ems-abandon-new-medford-ballpark-planning/ ; https://www.klcc.org/news-briefs/2025-08-07/eugenes-baseball-team-will-stay-in-town-through-2027-season |
| 2027 MiLB schedule | Triple-A Mar 26-Sep 19; Double-A Apr 2-Sep 12; High-A/Single-A Apr 2-Sep 5 (see b.) | see b. |

### Collective bargaining agreement (expires 11:59 p.m. ET on 2026-12-01)

Facts as reported up to about 2026-11-15 (no agreement has been announced):

- Formal talks began on 2026-05-12 in New York. https://wtop.com/sports/2026/05/mlb-players-owners-start-collective-bargaining-7-1-2-months-ahead-of-contracts-expiration/ (AP)
- MLB's economic proposal (May 28) is a salary cap and floor; revised on Nov 3 to a $258.2 M cap and $180.2 M floor starting in **2028**,
  keeping the current luxury-tax and revenue-sharing system for 2027. The MLBPA rejected it and opposes any cap.
  https://www.nbcsportsphiladelphia.com/mlb/philadelphia-phillies/mlb-lockout-tracker-cba-talks-phillies/764481/ (tracker, last updated Nov 4);
  https://frontofficesports.com/article/mlb-labor-talks-players-owners/ (Nov 7: both sides still far apart; Meyer and Manfred each say they expect a deal "eventually")
- **Proposals on the table that would change items in this file - none agreed:**
  - MLB (Aug 27): weekly roster locks (26-man roster set each Monday), 14 pitchers allowed, IL minimums of one week (position players) and two weeks (pitchers), the 60-day IL becoming a 12-week list.
  - MLB (Oct 8): regular season cut from 162 to 154 games from 2029 with a weekly off day; Division Series best-of-7; from 2027 the No. 3 seed would pick its Wild Card opponent (5 or 6) and the No. 1 seed would pick its Division Series opponent.
  - MLB (Nov 3): optional assignments cut from 5 to 4 per season; Rule 5 eligibility one year earlier.
  - MLBPA (Jul 1): 28-man active rosters for the first 15 days of the season with a 14-pitcher maximum; options cut from 5 to 3.
  Source for all four: the NBC Sports Philadelphia tracker above.
- Owners were reported to be meeting Nov 17-19 (https://lastwordonsports.com/baseball/2026/11/09/will-cba-cause-lockout/). Press reports say that
  without a deal by Dec 1 the owners are expected to lock the players out, as in Dec 2021; MLB itself has not announced that.
  AP headlined its report on the 2027 schedule "MLB to have earliest opening day on March 24 in 2027 - if there is a season".
  https://www.nbcsports.com/mlb/news/mlb-to-have-earliest-opening-day-on-march-24-in-2027-if-there-is-a-season
- The Minor League Basic Agreement is separate and runs through 2027. MiLB BA (title page: "2023-2027").

Practical reading for the simulation: use the 2026 rules for 2027 unless a new agreement says otherwise; none of the proposals above is in force.

---

## Notes on the two JSON files

### mlb_clubs.json (30 records)

- League/division/venue: Stats API `teams?sportId=1&season=2026&hydrate=venue(location,fieldInfo)`.
- `elevation_ft`: the Stats API venue elevation (the figure MLB/Statcast uses). Cross-checked against the USGS 3DEP point query
  (https://epqs.nationalmap.gov/v1/json) at the venue coordinates: all within 55 ft; largest gaps PIT 780 v 726, CIN 535 v 490, NYY 55 v 14, ATL 1001 v 965, KC 856 v 888, TEX 545 v 580.
- `capacity`: Stats API 2026 value where MLB updates it yearly (ATL, BAL, KC, MIL, STL, ATH); otherwise the figure on which the MLB.com ballpark
  guide (https://www.mlb.com/news/featured/<park>-guide-capacity-seating-chart-parking-and-more , most updated March 2026) and Wikipedia agree.
  Capacities that three sources give three ways and that I could **not** settle: Great American Ball Park (43,500 / 42,319 / 45,814), Kauffman (38,179 / 38,053 / 37,903),
  T-Mobile Park (47,378 / 47,943 / 47,368), Chase Field (48,263 / 48,330 / 48,405), Oracle Park (41,331 / 41,311 / ~42,300). Differences are under 2 % except Cincinnati.
- `dims_ft`: posted/official figures. The Stats API field data is stale for several parks (T-Mobile, Comerica, loanDepot, Rogers Centre), so each park was
  checked against the club announcement, the MLB.com guide, Wikipedia and Seamheads; choices and alternatives are in each record's `notes`.
  Two deliberate deviations from "posted": Angel Stadium and Kauffman Stadium use the true 330-ft foul-line distance rather than the 347/350 and 347/344 markers
  that are measured away from the poles.
- `wall_ft`: official figures where found (BAL left field, CLE, DET center/right, KC, NYY, TOR, LAA right, SF right, MIA left/right); otherwise the
  2025 row of the Seamheads Ballparks Database (https://www.seamheads.com/ballparks/) cross-checked with Clem's Baseball
  (http://www.andrewclem.com/Baseball/Stadium_statistics.html). **null**: Sutter Health Park (all three), loanDepot park center field.
  Flagged as uncertain in `notes`: Comerica left field, Nationals Park right field, Fenway center field, Petco Park, American Family Field right field, Citi Field.
- Only one park changed its fences for 2026 (Kauffman). Camden Yards changed in 2025.

### milb_clubs.json (120 records)

- **Count check: 30 per level (AAA 30, AA 30, A+ 30, A 30), and every MLB club has exactly one affiliate at each level.** Nothing fails to add up.
  Source: Stats API team lists for sportId 11/12/13/14, season 2026 (league, division, parent, venue, city).
- The file has one extra reference-only field, `ballpark` (2026 venue name).
- Reflected 2025 changes: Columbus Clingstones (ex-Mississippi Braves), Hub City Spartanburgers (ex-Down East Wood Ducks, now High-A), Hickory to the Single-A Carolina League,
  Knoxville Smokies (new park and name), Chesapeake Baysox (name), Oklahoma City Comets (name), Salt Lake Bees in South Jordan.
  https://www.mlb.com/news/minor-league-teams-new-logos-for-2025 ; https://mlb.com/news/carolina-south-atlantic-leagues-to-realign-in-2025
- Reflected 2026 changes: Wilson Warbirds (ex-Carolina Mudcats), Ontario Tower Buzzers (new; Modesto Nuts gone), Inland Empire -> Mariners, Rancho Cucamonga -> Angels,
  Frederick Keys back as Orioles High-A (Aberdeen out), Hill City Howlers (ex-Lynchburg Hillcats), Salem RidgeYaks (ex-Salem Red Sox), new parks in Richmond, Chattanooga and Hillsboro.
  https://www.mlb.com/milb/news/new-minor-league-team-names-and-logos-for-2026 ; https://www.mlb.com/news/mariners-announce-sale-of-single-a-modesto-nuts
- `elevation_ft`: USGS 3DEP elevation at the venue coordinates from the Stats API (https://epqs.nationalmap.gov/v1/json?x=<lon>&y=<lat>&units=Feet);
  Vancouver from the Open-Meteo/Copernicus DEM (https://api.open-meteo.com/v1/elevation). Ground level at the coordinate, not a surveyed field level
  (median difference between the two DEMs: 5 ft). No nulls.
- `capacity`: three sources compared - Stats API, Wikipedia infobox, and the MLB.com ballpark guides (index: https://www.mlb.com/milb/fans/ballpark-guides ; about 80 parks checked).
  The Stats API figure is often rounded or out of date (e.g. Omaha 3,500, Visalia 1,647), so the Wikipedia/MLB.com figure was preferred when they were more specific;
  every deviation is written in `notes`. Definitions vary (fixed seats v total with berm/standing). No nulls, but treat capacities as +/- 10 %.
- `dims_ft` (lf_line, cf, rf_line): Stats API by default, checked against the Wikipedia infobox for all 120 parks and against the MLB.com ballpark guide
  for about 80. The Stats API field data turned out to be wrong or stale for many minor-league parks, so it was overridden wherever at least two
  other sources (Wikipedia, MLB.com guide, club facts page) agreed against it: 34 parks, each documented in `notes`.
  Still open (value left on the Stats API figure, alternative in `notes`): Kannapolis, Durham, Pensacola and Dunedin (three sources, three answers),
  Ontario (new park; MLB.com guide says 325/405/325), El Paso center field (400/403/406). Fayetteville and Columbia carry identical figures in every source, which looks suspicious.
  Hillsboro's new park is from a single non-official source (baseballparks.com).
