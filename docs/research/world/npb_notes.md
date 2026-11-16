# NPB structure and rules: 2026 season, with changes announced for 2027

Research date: 2026-11-16. Companion data file: `npb_clubs.json` (12 first-team clubs, 14 farm teams).

Conventions used here:

- **[official]** = npb.jp, the 野球協約 text, a club or ballpark operator's own page.
- **[press]** = Japanese news report (Kyodo, Jiji, Nikkei, Daily Sports, Chunichi Sports, etc.).
- **[secondary]** = explainer site, blog quoting a newspaper, or Japanese Wikipedia.
- **[derived]** = computed by me from official pages (game counts from the NPB schedule pages, base states from NPB play-by-play pages, elevations from the GSI elevation API). The method is stated each time.
- **NOT CONFIRMED** = I could not find a source good enough; do not treat as fact.

Things that will surprise anyone working from pre-2026 knowledge are collected in section G. Everything I could not confirm is listed in section H.

---

## A. Regular-season format

### A1. Games per club and opponents

- 143 games per club in both 2026 and 2027: 125 inside the league (25 against each of the 5 league opponents) plus 18 interleague. [official] https://npb.jp/games/2026/schedule_note_cl.html , https://npb.jp/games/2026/schedule_note_pl.html , https://npb.jp/games/2027/schedule_note_cl.html , https://npb.jp/games/2027/schedule_note_pl.html
- The 25 games against one opponent split 13 home / 12 away or 12 / 13. A club therefore has 63 or 62 league home games, and the two groups swap each year.
  - 2026, 63 league home games: Hanshin, DeNA, Giants (CL); Nippon-Ham, Seibu, SoftBank (PL). 2027: Yakult, Chunichi, Hiroshima (CL); Rakuten, Lotte, Orix (PL). Same four URLs as above; the full home-count matrices are on those pages.
- Total: 858 games (429 hosted by each league's clubs). [official] https://npb.jp/games/2026/schedule_note_pl.html
- Opening-series hosts are set by old standings: CL = top three of two seasons earlier, PL = top three of three seasons earlier (2026: CL used 2024, PL used 2023; 2027: CL uses 2025, PL uses 2024). In 2026 Hanshin gave up its opener because its park was unavailable, so fourth-placed Hiroshima hosted. [official] https://npb.jp/news/detail/20251105_02.html , https://npb.jp/news/detail/20251106_01.html , https://npb.jp/news/detail/20261104_03.html , https://npb.jp/news/detail/20261110_01.html

### A2. Interleague (交流戦)

- 18 games per club: one 3-game series against each of the six clubs of the other league, 9 home and 9 away, 108 games in total. They count in the league standings. [official] https://npb.jp/interleague/2026/schedule_note_il.html
- Home and away alternate by year: a pairing hosted by one club in 2026 is hosted by the other in 2027. [official] https://npb.jp/news/detail/20251105_01.html
- 2026 period: 26 May (Tue) to 14 June (Sun), reserve days to 18 June. All 108 games were completed by 17 June [derived from the schedule pages]. [official] https://npb.jp/interleague/2026/information.html
- 2026 interleague rules: DH only in games hosted by Pacific League clubs; probable starters announced; no suspended games; interleague "champion" = best winning percentage, with its own tie-break list (wins, head-to-head, TQB, ER-TQB, team batting average, previous year's interleague rank). [official] https://npb.jp/interleague/2026/information.html
- 2027 period: 1 June (Tue) to 20 June, reserve period 21 to 24 June. [official] https://npb.jp/interleague/2027/schedule_note_il.html

### A3. Weekly pattern (2026, counted from the official schedule)

[derived] from https://npb.jp/games/2026/schedule_04_detail.html and the other monthly pages (858 games played, postponed rows excluded):

| Mon | Tue | Wed | Thu | Fri | Sat | Sun |
|---|---|---|---|---|---|---|
| 25 | 144 | 133 | 103 | 138 | 157 | 158 |

- So the normal week is two 3-game series, Tuesday to Thursday and Friday to Sunday, with Monday as the travel/off day. Of the six playing days, Thursday has the fewest games.
- No doubleheaders were played in 2026 (the CL Climax Series outline also says no doubleheaders).
- Start times: 18:00 in 553 of 858 games; day games mostly 14:00 (166) or 13:00 (57), mainly at weekends.
- 41 postponements appear in the 2026 schedule pages; they were made up later (unlike the farm league, see C).

### A4. Extra innings and ties

- Regular season: a game still tied after 12 innings is a tie; ties are not replayed. I found no 2026 rule text for the regular season, but the evidence is solid:
  - [derived] All 16 first-team tie games of 2026 are on the NPB schedule pages. 15 of them were 12-inning games; the other (4 Sep, Yakult v Chunichi, 10 innings) was called for field conditions. Example: https://npb.jp/scores/2026/0904/s-d-20/ ; schedule pages: https://npb.jp/games/2026/schedule_08_detail.html
  - [official for CS] NPB's 2026 Climax Series outline uses the same 12-inning wording: https://npb.jp/games/2026/info_cscl.html
  - [press] 12 innings restored from 2022: https://www.daily.co.jp/baseball/2022/01/19/0014995361.shtml ; [secondary] 2026 unchanged: https://teams.one/news/baseball-tie-breaker-system/
- A regulation game can also end as a tie when it is called (the 4 Sep 2026 game above).
- No tie-break runner in first-team games in 2026. A tie-break is listed only as something to study in NPB's business plan for the year to September 2027. [official] https://npb.jp/organization/docpb_09_2026.pdf

### A5. Standings

- Ranked by winning percentage = wins / (wins + losses); ties are left out of the calculation. [official, PL agreement excerpt, 2022 edition] https://npb.jp/games/2022/info_cspl.html
- Central League tie-breakers (2026 text): equal percentage, then more wins; if still level, (1) head-to-head winning percentage, (2) winning percentage in league games only, (3) previous year's rank. [official] https://npb.jp/games/2026/info_cscl.html
- Pacific League tie-breakers: (1) head-to-head winning percentage, (2) winning percentage in the 125 league games, (3) previous year's rank. The only public text is the 2022 edition of the agreement excerpt; **2026 wording NOT CONFIRMED**. [official, 2022] https://npb.jp/games/2022/info_cspl.html
- Final positions for the Climax Series are fixed at the end of the schedule two days before the series starts. [official] https://npb.jp/games/2026/info_cs.html

### A6. Key dates

2026 [official] https://npb.jp/event/2026/ unless another URL is given:

| Item | Date |
|---|---|
| Spring camp starts | 1 Feb (camp guide: https://npb.jp/camp/2026/spr_t.html). Team activity is banned 1 Dec to 31 Jan by 野球協約 Art. 173 |
| Exhibition games (オープン戦) | 21 Feb to 22 Mar, 101 games planned (https://npb.jp/news/detail/20251219_01.html) |
| World Baseball Classic | 5 to 17 Mar (Tokyo pool 5 to 10 Mar) |
| Farm season opens | 14 Mar |
| Opening Day, both leagues | 27 Mar (Fri) |
| Interleague | 26 May to 14 Jun (reserve to 18 Jun) |
| Fresh All-Star (farm) | 27 Jul, Niigata |
| All-Star Games | 28 Jul Tokyo Dome, 29 Jul Toyama (no regular-season games 27 to 30 Jul [derived]) |
| Trade / new-contract deadline | 31 Jul (https://npb.jp/news/detail/20260731_02.html) |
| Regular season ended | PL 6 Oct, CL 8 Oct (https://npb.jp/news/detail/20261006_06.html , https://npb.jp/news/detail/20261008_02.html) |
| Farm Japan Championship | 3 to 4 Oct, Shizuoka |
| Climax Series | First Stage from 10 Oct, Final Stage from 14 Oct |
| Draft | 22 Oct |
| Japan Series | 24 Oct to 1 Nov scheduled; ended 28 Oct (SoftBank 4-0 DeNA) (https://npb.jp/nippons/2026/) |

2027 [official] https://npb.jp/event/2027/ :

| Item | Date |
|---|---|
| Spring camp starts | Not yet published. 1 Feb is the earliest allowed (Art. 173) and is the usual date |
| Exhibition games | Not yet announced (the 2026 list came out on 19 Dec 2025) |
| Opening Day, both leagues | 26 Mar (Fri) (decided 1 Jun 2026: https://npb.jp/news/detail/20260601_02.html) |
| Interleague | 1 Jun to 24 Jun including the reserve period |
| All-Star Games | 21 Jul Mizuho PayPay Dome Fukuoka, 22 Jul ZOZO Marine Stadium, reserve 23 Jul |
| End of regular season | Schedule as announced runs to 30 Sep (PL) and 2 to 3 Oct (CL), before make-ups; 3 CL games are still unassigned [derived from https://npb.jp/games/2027/schedule_10_detail.html and https://npb.jp/games/2027/schedule_note_cl.html] |
| Climax Series | First Stage from 9 Oct, Final Stage from 13 Oct |
| Japan Series | 23 Oct to 31 Oct; Games 1, 2, 6, 7 at the Pacific League club's park |
| NPB Awards | 25 Nov |

Further out: the 2030 All-Star Game will be in Kurashiki (Muscat Stadium). [official] https://npb.jp/news/detail/20260729_04.html

---

## B. Postseason

### B1. Climax Series (rules as published by NPB on 24 Aug 2026)

[official] https://npb.jp/games/2026/info_cs.html (both leagues) and https://npb.jp/games/2026/info_cscl.html (CL outline)

- Teams: regular-season 1st, 2nd and 3rd in each league.
- **First Stage**: 2nd v 3rd, best of three (first to 2 wins), every game at the 2nd-place club's park. If neither side reaches 2 wins because of ties or lost dates, more wins advances; equal wins sends the 2nd-place club through. The stage stops as soon as the 2nd-place club is certain to have at least as many wins.
- **Final Stage, normal case**: league champion v First Stage winner, up to 6 games, every game at the champion's park. The champion starts with a 1-win advantage; first to 4 wins including the advantage.
- **Final Stage, new from 2026**: the stage becomes up to **7 games with a 2-win advantage (first to 5)** when either
  1. the champion finished 10 or more games ahead of the First Stage winner, or
  2. the First Stage winner had a regular-season winning percentage below .500.
- Equal wins at the end of either stage: the higher-seeded club advances. The stage ends once the higher seed is certain to have at least as many wins as the lower seed.
- Extra innings: 12-inning limit, a tie stands and is not replayed. Special called-game rule: when the home (higher-seeded) club needs only a tie to clinch, the game ends after a scoreless top of the 12th, or the moment the home side ties it in the bottom of the 12th.
- No doubleheaders; probable starters are announced; six umpires.
- Eligible players: on the club's 支配下 list on 31 August and continuously since. Registration and deregistration work as in the regular season. Statistics are kept separately.
- DH: follows league rules (none in the Central League in 2026; used in the Pacific League). The CL outline does not mention the DH.
- 2026 application: the CL Final Stage was a 6-game stage; the PL Final Stage was a 7-game stage because SoftBank finished 11.5 and 13 games ahead of Seibu and Nippon-Ham. SoftBank won it in three games (2 + 3 = 5). [official] https://npb.jp/news/detail/20261006_04.html , https://npb.jp/news/detail/20261006_05.html , https://npb.jp/games/2026/
- 2026 dates: First Stage 10 to 12 Oct (reserve 13 Oct); Final Stage 14 to 19 Oct, 20 Oct as Game 7 or a reserve day, 21 Oct reserve.

### B2. Japan Series (2026 outline)

[official] https://npb.jp/nippons/2026/information.html

- Best of seven; the series ends when one team has 4 wins.
- Format 2-3-2 with travel days. 2026: Games 1, 2, 6, 7 at the Central League club's park. 2027: Games 1, 2, 6, 7 at the Pacific League club's park (https://npb.jp/event/2027/).
- Extra innings: Games 1 to 7 end as a tie after 12 innings. From Game 8 on there is no inning limit. No suspended games.
- If ties leave the series undecided after Game 7, Game 8 is played the next day at the Game 7 park; a Game 9 would follow after a travel day at the other park.
- DH: used only in games at the Pacific League club's park (2026 rule). For 2027 both leagues use the DH, so every game should have it, but **no 2027 Japan Series outline exists yet: NOT CONFIRMED**.
- Probable starters announced for every game; six umpires.
- Eligibility: 40 players, chosen from those on the 支配下 list on 31 August and continuously since. 26 players on the bench, 8 coaches. Two bullpen catchers may be listed but may not sit on the bench.
- 2026 result: SoftBank beat DeNA 4-0; the first three games went to extra innings. https://npb.jp/nippons/2026/

---

## C. Farm league in 2026

### C1. Structure (changed in 2026)

- From 2026 the farm is one league, the **ファーム・リーグ (Farm League)**, with three regional divisions. The Eastern League and Western League no longer exist as competitions. [official] https://npb.jp/news/detail/20260122_01.html , https://npb.jp/farm/2026/schedule_note_2.html , https://npb.jp/campaign/2026/farm/
  - **East Division (東地区, 5)**: Rakuten, Oisix Niigata, Yakult, Lotte, Nippon-Ham.
  - **Central Division (中地区, 5)**: Seibu, Giants, DeNA, Hayate Shizuoka, Chunichi.
  - **West Division (西地区, 4)**: Orix, Hanshin, Hiroshima, SoftBank.
- For comparison, 2025: Eastern League 8 (Giants, Seibu, Lotte, Nippon-Ham, Rakuten, DeNA, Oisix Niigata, Yakult); Western League 6 (Chunichi, SoftBank, Hanshin, Orix, Hiroshima, Kufu Hayate Shizuoka). Chunichi and Hayate therefore changed grouping. [official] https://npb.jp/farm/2025/
- Two clubs have no first team: **オイシックス新潟アルビレックスBC** and **ハヤテベンチャーズ静岡** (called くふうハヤテベンチャーズ静岡 in 2024 and 2025). Both were admitted in the 2023 expansion review. [official] https://npb.jp/news/detail/20230929_03.html , https://npb.jp/bis/2026/stats/std_2c.html
- Games against other divisions are called "ファーム・リーグ 交流戦". [official] https://npb.jp/farm/2026/schedule_note_2.html

### C2. Schedule

[official] https://npb.jp/farm/2026/schedule_note.html (planned games by club and by pairing)

- Pattern: six series inside the division, then two series against other divisions, repeated. East and Central have five clubs each, so one East club and one Central club also meet during division weeks.
- Planned games differ by club: **135 to 146**.

| Division | Club | In division | Interdivision | Planned | Played 2026 | W-L-T |
|---|---|---|---|---|---|---|
| East | Nippon-Ham | 74 | 62 | 136 | 111 | 62-42-7 |
| East | Rakuten | 86 | 53 | 139 | 117 | 57-55-5 |
| East | Lotte | 78 | 57 | 135 | 114 | 51-56-7 |
| East | Yakult | 73 | 62 | 135 | 112 | 48-56-8 |
| East | Oisix Niigata | 85 | 55 | 140 | 119 | 46-69-4 |
| Central | DeNA | 81 | 58 | 139 | 122 | 74-40-8 |
| Central | Giants | 82 | 62 | 144 | 124 | 75-47-2 |
| Central | Seibu | 79 | 64 | 143 | 121 | 69-51-1 |
| Central | Chunichi | 86 | 56 | 142 | 123 | 54-64-5 |
| Central | Hayate Shizuoka | 84 | 56 | 140 | 122 | 37-81-4 |
| West | SoftBank | 104 | 37 | 141 | 125 | 70-49-6 |
| West | Orix | 100 | 37 | 137 | 119 | 54-57-8 |
| West | Hanshin | 107 | 39 | 146 | 126 | 49-63-14 |
| West | Hiroshima | 99 | 38 | 137 | 121 | 47-63-11 |

Played and W-L-T: [official] https://npb.jp/farm/2026/

- **Cancelled games are not made up.** Division rank is by winning percentage over the games actually played. 139 of the 977 scheduled games (14 percent) were cancelled in 2026, leaving 838 played [derived from the schedule pages and the planned totals above]. [official] https://npb.jp/news/detail/20260122_01.html
- Season: opened 14 March, finished 27 September. [official] https://npb.jp/news/detail/20260927_01.html
- Individual titles are awarded per division. [official] https://npb.jp/campaign/2026/farm/
- Other farm events: spring educational league 3 to 12 March (40 games); Fresh All-Star 27 July in Niigata, played as "Central League clubs plus Oisix Niigata" against "Pacific League clubs plus Hayate Shizuoka" (format for 2027 undecided); autumn Miyazaki Phoenix League 7 to 28 October. [official] https://npb.jp/news/detail/20260122_01.html , https://npb.jp/campaign/2026/farm/ , https://npb.jp/event/2026/

### C3. Farm Japan Championship 2026

[official] https://npb.jp/farmchamp/2026/information.html , https://npb.jp/farmchamp/2026/

- Four teams, single elimination, neutral site (Shizutetsu Stadium Kusanagi, Shizuoka): the three division winners plus one wild card, the second-place club with the best winning percentage.
- 3 October: First Stage (semi-finals). Game 1 is the 2nd-best v 3rd-best division winner; Game 2 is the best division winner v the wild card. 4 October: Final Stage.
- Tie rules: semi-finals use a tie-break from the 10th inning (runners on first and second, nobody out, batting order continues), no inning limit. The final is played to a decision with no tie-break.
- DH used. 26 players on the bench. At most 4 foreign players in the line-up at once (1 pitcher and 3 fielders at most). In principle up to 5 育成 players may appear.
- Eligibility: registered (支配下 or 育成) on 31 August; players who spent time on the first-team roster must meet minimum farm playing time.
- 2026: Nippon-Ham (East winner) beat SoftBank 7-6, the Giants (wild card) beat DeNA 3-1, and Nippon-Ham beat the Giants 5-2 in the final.

### C4. Farm playing rules seen in 2026 (inferred from NPB game pages, not from a rule document)

I could not find the 2026 Farm League agreement text. The following is **[derived]** from NPB box scores and play-by-play pages; treat as observed practice.

- **Extra innings stop after the 10th.** Of the 45 tie games in 2026, 13 were 10-inning games and 32 ended after 9; none went longer. Source pages: https://npb.jp/farm/2026/schedule_04_detail.html and the game pages linked from it, e.g. https://npb.jp/scores_farm/2026/0324/s-m-01/
- **The 10th inning is a tie-break inning.** It started with a runner on second and nobody out in games on 24 Mar, 25 Apr, 4 May, 29 May, 6 Jun and 9 Jun, and with runners on first and second in games on 22 Jul, 31 Jul and 21 Sep. So the format switched in mid-season, as in the 2025 trial. Example: https://npb.jp/scores_farm/2026/0324/s-m-01/playbyplay.html ; background [press] https://www.daily.co.jp/baseball/2024/11/21/0018363453.shtml
- **Many games do not go to extra innings at all.** 16 of the 32 nine-inning ties were the last game of a series, and most of the rest were in July to early September. The exact condition (travel day, heat, time limit) is **NOT CONFIRMED**.
- **DH**: a DH appeared in all 56 box scores I sampled (4 home games for each of the 14 clubs), including games hosted by Central League clubs.
- **育成 players**: at most 5 per club per farm official game (規約 Art. 8, see D7).
- **Heat rules**: from 21 Aug 2026 the home club may delay or cancel a farm game when WBGT is forecast at 31 degrees C or higher. [official] https://npb.jp/npb/heatillness_guideline_farm_20260820.pdf
- **PitchCom** was to be trialled in farm games from the second half of 2026. [press] https://www.daily.co.jp/baseball/2026/07/06/0020559272.shtml

### C5. Third and fourth teams (三軍・四軍)

Which clubs have them, from NPB's official 2026 staff lists (https://npb.jp/announcement/2026/managers_g.html and the same page for each club code: t, db, c, s, d, h, f, b, e, l, m):

| Club | Third team | Fourth team | Evidence in the 2026 staff list |
|---|---|---|---|
| SoftBank | Yes | Yes | 三軍監督 and coaches; 四軍チーフ兼守備走塁コーチ plus two coaches (no fourth-team manager in 2026) |
| Giants | Yes | No | 三軍監督 and six third-team coaches |
| Seibu | Yes | No | 三軍チーフコーチ and four 三軍野手コーチ |
| Hiroshima | Title only | No | one 三軍統括兼大野寮長; no third-team schedule found |
| Orix, Rakuten, Lotte | No "三軍" titles | No | 育成 (development) coaches only |
| Hanshin, DeNA, Yakult, Chunichi, Nippon-Ham | No | No | none |

How they play: **they are in no league and their games are unofficial** (非公式試合・練習試合・交流戦). Opponents in 2026:

- SoftBank third team: Shikoku Island League plus clubs (Ehime, Kagawa, Tokushima, Kochi), Kyushu Asia League clubs (Hinokuni, Kitakyushu Shimonoseki, Oita), university and company teams, Korean clubs' reserve sides (Samsung, Lotte), a BC League select team, and the Giants' third team; plus a pre-season event in Kagoshima. [official] https://www.softbankhawks.co.jp/farm/informal_game/schedule_third/2026/
  - The club site lists only a third-team schedule for 2026. Whether the fourth squad plays its own games in 2026 is **NOT CONFIRMED** (a search summary of a Sponichi report said the reorganised fourth squad concentrates on training; I could not open the article).
- Giants third team: BC League clubs (Fukushima, Shinano, Tochigi, Yamanashi, Ibaraki, Kanagawa), company and university teams, a Korean club, and other clubs' third teams. [official] https://www.giants.jp/schedule/third-team/?month=202606
- Seibu third team: BC League clubs, a Nihonkai League select team, universities, company teams, and the Giants' third team. The club warns that these games are unofficial and may be shortened to 5 or 7 innings. [official] https://www.seibulions.jp/farm/schedule_third/
- 育成 players may also be loaned to independent-league clubs (規約 Art. 8).
- Possible change: NPB's plan for the year to September 2027 lists "considering participation of NPB third teams in IPBL (independent league) official games". [official] https://npb.jp/organization/docpb_09_2026.pdf

Player counts on 31 July 2026 (支配下 / 育成): Giants 70/35, Yakult 69/10, DeNA 70/11, Chunichi 70/11, Hanshin 70/10, Hiroshima 68/10, Nippon-Ham 69/13, Rakuten 70/10, Seibu 70/32, Lotte 70/18, Orix 70/21, SoftBank 70/48. [official] https://npb.jp/news/detail/20260731_02.html

---

## D. Roster-size rules (contracts excluded)

Main text: 日本プロフェッショナル野球協約 2025 edition, hosted by the players' association (the 2026 edition is not posted): [official text] https://jpbpa.net/wp-content/uploads/jpbpa-pdf/ag2025.pdf

### D1. 支配下登録 (players under control)

- Maximum **70** per club in a year (80 only under the emergency relief clause). (Art. 79)
- No new player contracts may be approved from 1 August until the day after the season ends, and trades are allowed only until **31 July** (waiver claims excepted). So 31 July is the deadline for adding players and for promoting 育成 players. (Art. 64, Art. 108; 2026 notice [official] https://npb.jp/news/detail/20260731_02.html)

### D2. 出場選手登録 (active roster) and ベンチ入り

- 野球協約 Art. 81 still says 29. In practice the limit is **31 registered, 26 in uniform for a game**. This began as a 2020 COVID measure and has been renewed every year. [official, 2020] https://npb.jp/news/detail/20200617_03.html ; [press, renewal for 2026, announced 11 Nov 2025] https://www.nikkei.com/article/DGXZQOKC1165D0R11C25A1000000/
- NPB's own 2026 notice confirms the 31 figure: a retiring player's special registration lets a club "exceed 31" for one day. [official] https://npb.jp/announcement/roster/roster_0925.html
- The 26 figure is confirmed by NPB's "ベンチ入り選手" page for each game: 26 players per side in the four 2026 games I checked (1 Apr, 26 May, 4 Sep, 1 Oct). [derived] e.g. https://npb.jp/scores/2026/0904/s-d-20/roster.html
- The players registered but not in uniform (up to 5) are in practice mostly starting pitchers who are not due to pitch; that is common knowledge, not something I sourced.
- Opening-day rosters must be filed 2 days before the opener; a player registered in season can play from the day of the notice. (Art. 84)
- 2027: unchanged (31 / 26, foreign 5 / 4), announced 14 Oct 2026. [secondary, two blogs quoting Nikkan Sports; original URL not retrieved] https://dnomotoke.com/archives/20261014143042/ , https://matomelotte.com/archives/60103011.html

### D3. Re-registration wait

- A deregistered player cannot be re-registered until **10 days have passed, counting the day of the deregistration notice**. Example: deregistered 1 October 2026, eligible again from 11 October. (Art. 85) [official] https://npb.jp/announcement/roster/roster_1001.html
- A player who withdraws from the All-Star Game is deregistered automatically and cannot return until his club has played 10 games after the All-Star Game (exceptions for concussion and prior injury). (Art. 86)

### D4. Special deregistration rules, all seen in force in 2026

There is **no injured list** in the 野球協約; an injured player is simply deregistered and still counts toward the 70. The exceptions to the 10-day wait are:

- **脳振盪特例措置 (concussion)**: the player may return inside 10 days after clearing the return protocol; his designated replacement (代替指名選手) may be deregistered and re-registered without the 10-day wait. In force: 19 and 25 September 2026. [official] https://npb.jp/announcement/roster/roster_0919.html , https://npb.jp/announcement/roster/roster_0925.html
- **NPB感染症特例 (infectious disease)**: same mechanism. Still in force in 2026 (12 August, 1 October). [official] https://npb.jp/announcement/roster/roster_0812.html , https://npb.jp/announcement/roster/roster_1001.html . It grew out of the 2020 "感染拡大防止特例". The 2026 Japan Series outline also lets a club replace a listed player who falls ill with COVID-19 or influenza. https://npb.jp/nippons/2026/information.html
- **慶弔休暇特例 (family leave)**: same mechanism. In force 12 August 2026. [official] https://npb.jp/announcement/roster/roster_0812.html
- **引退選手・特例登録 (retirement game)**: a retiring player may be registered for one day above the 31 limit and is removed automatically the next day. In force 25 September 2026. [official] https://npb.jp/announcement/roster/roster_0925.html
- Note: the roster notice URLs carry only the month and day, so they will be overwritten in 2027.

### D5. Foreign players

- Any number may be held as 支配下 players. (Art. 82-2)
- Written rule: at most 4 on the active roster, and no more than 3 pitchers or 3 fielders at once. (Art. 82-2)
- Practice since 2020, renewed for 2026 and 2027: **5 on the active roster, 4 on the bench**. [official, 2020] https://npb.jp/news/detail/20200617_03.html ; [press] https://www.nikkei.com/article/DGXZQOKC1165D0R11C25A1000000/
- Pitcher / fielder split under the 5-man version: the five may be 4+1, 3+2, 2+3 or 1+4 but not all pitchers or all fielders; the four on the bench may be 3+1, 2+2 or 1+3. [secondary; I did not find the agreement text] https://teams.one/news/npb-foreign-player-rules/ , https://baseball-tokidoki.com/nippon-professional-baseball-gaikokujinwaku/
- Farm championship: at most 1 foreign pitcher and 3 foreign fielders in the line-up at once. [official] https://npb.jp/farmchamp/2026/information.html

### D6. When a foreign national is treated as a domestic player (Art. 82)

A player without Japanese nationality is not counted as foreign if any of these applies:

1. before signing, 3 or more years in total at a Japanese junior high school, high school or junior college;
2. before signing, 4 or more consecutive years at a Japanese university;
3. before signing, 5 or more years living in Japan and 3 or more seasons with a team of the Japan Amateur Baseball Association (company/club baseball);
4. after signing, he earns NPB free-agent rights; he is treated as domestic from the season after the commissioner's notice;
5. he entered through the draft without meeting 1 or 2, and his school years in Japan plus his seasons as a 支配下 player reach 5; from the next season, with approval of the executive committee.

### D7. 育成選手 (developmental players)

Text: 日本プロ野球育成選手に関する規約, 2014 edition (the newest one posted; **later amendments NOT CHECKED**). [official text] https://jpbpa.net/wp-content/uploads/2021/12/tp2014.pdf

- They sit outside the 70. There is no numerical limit on how many a club may hold. (Art. 1, Art. 5)
- A club may hold them only if it has **65 or more 支配下 players at the end of July**. A club below 65 at the end of June must explain by the end of July whether it will promote 育成 players or sign others; the executive committee can grant an exception. (Art. 3)
- **Uniform numbers must have three digits.** (Art. 8-4)
- They may play in farm official games, at most **5 per club per game**, and in unofficial first- and second-team games; they cannot play in first-team official games. (Art. 8)
- Promotion to 支配下 needs a new standard contract inside the 70 limit, so the deadline is **31 July** (野球協約 Art. 64). A newly signed foreign 育成 player aged 26 or older must be promoted by the **end of March**. (Art. 9)
- After 3 seasons without promotion a 育成 player becomes a free agent automatically on 30 November, and may then re-sign for one year at a time. A player signed on or after 1 July does not have that year counted. (Art. 10, Art. 4-10)
- A 支配下 player can only be moved to 育成 status after being released, and not during the same season in which he was released in mid-season. (Art. 9-2)
- 育成 contracts can be traded until 31 July. (Art. 11)

---

## E. Playing rules in force in 2026 that a simulation must model

### E1. Designated hitter

- Pacific League: DH (since 1975). Central League: **no DH in 2026**, the last season in which Central League pitchers bat.
- Interleague 2026: DH only in games hosted by Pacific League clubs. Japan Series 2026: DH only at the Pacific League park. [official] https://npb.jp/interleague/2026/information.html , https://npb.jp/nippons/2026/information.html
- **Confirmed: the Central League adopts the DH from the 2027 season.** Decided by the league board on 4 Aug 2025, with 2026 as a grace year. [official] https://npb.jp/news/detail/20250804_05.html
- The "Ohtani rule" (a starting pitcher who is also the DH may stay in as DH after leaving the mound) is expected to apply. [press] https://www.nikkei.com/article/DGXZQODH044KD0U5A800C2000000/ , https://www.tokyo-np.co.jp/article/426284

### E2. Extra innings

- First team: 12 innings, then a tie; no tie-break runner (see A4).
- Japan Series: 12 innings through Game 7, unlimited from Game 8 (B2). Farm: see C4.

### E3. Pitch timing

- Existing rule since 2009: with the bases empty the pitcher must deliver within **15 seconds** of receiving the ball, or a ball is called. NPB's own pace-of-play document describes "15 seconds, bases empty" (the rule book itself says 12). **I found no 2026 text confirming how it is enforced now.** [official, 2008] https://npb.jp/gbp/2008/tanshuku.html ; [press, 2009] https://www.j-cast.com/2009/02/13035928.html?p=all
- **No pitch clock in 2026**, no limit on pick-off throws. [secondary] https://spaia.jp/column/baseball/npb/34766
- Pitch clock plan: on 3 Aug 2026 the executive committee decided unanimously to introduce one; **the start date and the number of seconds are undecided** (the MLB values of 15 seconds empty and 18 with runners are the reference). The players' association had voted on 28 July to accept it and asked for 2027. At the 16 Sept owners' meeting it was still "under study"; NPB wants all parks to start together. [press] https://kumanichi.com/articles/2021447 , https://www.daily.co.jp/baseball/2026/08/04/0020666991.shtml , https://full-count.jp/2026/09/18/post2017562/
- PitchCom: see F.

### E4. Video review (リクエスト)

- Each manager has **2 requests through nine innings**; a request that overturns the call is not used up. **In extra innings the count resets to 1 per team.** Only the manager may ask. Balls and strikes and check swings are not reviewable. [press, 2017 decision: "2 through nine innings, 1 in extra innings"] https://baseballking.jp/ns/138188/ ; [secondary, 2026 wording including the reset and the kept request] https://teams.one/news/baseball-request-rule-guide/ , https://num-11235.hateblo.jp/entry/2026/03/26/211610
- **New in 2026: all reviews are done at a Replay Center** in NPB's Tokyo office by two first-team umpires and an operator who are not working the game; the result goes to the crew chief by headset. Target time 5 minutes. The ruling is final, and arguing it means ejection. About 500 requests a season; 179 by the end of interleague play in 2026. [press] https://www.daily.co.jp/baseball/2026/08/25/0020742929.shtml , https://www.daily.co.jp/baseball/2026/08/18/0020717378.shtml

### E5. Strike zone

- Rule-book definition, unchanged: the space over home plate whose upper limit is a horizontal line at the midpoint between the top of the batter's shoulders and the top of his uniform trousers, and whose lower limit is the line at the bottom of the kneecap, judged from the batter's stance as he prepares to swing. Called by the plate umpire. [secondary quoting 公認野球規則] https://ja.wikipedia.org/wiki/ストライクゾーン
- Automated ball-strike calling: **not used and no decision taken.** NPB's plan for the year to September 2027 lists the pitch clock, PitchCom, the ABS system and a tie-break as items to study. The players' association has said it will ask for ABS. [official] https://npb.jp/organization/docpb_09_2026.pdf ; [press] https://www.daily.co.jp/baseball/2026/08/03/0020666150.shtml

### E6. Ball

- Unified ball (統一試合球) rules, in force since 1 Feb 2015: coefficient of restitution **target 0.4134** (before 2015 the standard was a range, 0.4034 to 0.4234); circumference 22.9 to 23.5 cm; weight 141.7 to 148.8 g; 108 stitches. [official] https://npb.jp/npb/20150203release.html
- Context, computed from NPB team batting tables: home runs rose from 1,096 in 2025 to 1,426 in 2026 (CL 543 to 648, PL 553 to 778) and runs per team-game from 3.29 to 3.59. I found no official statement about the ball. [derived] https://npb.jp/bis/2025/stats/tmb_c.html , https://npb.jp/bis/2025/stats/tmb_p.html , https://npb.jp/bis/2026/stats/tmb_c.html , https://npb.jp/bis/2026/stats/tmb_p.html

### E7. Bases

- **New in 2026: first, second and third base are 18 inches square (45.7 cm), up from 15 inches (38.1 cm).** NPB calls it the 統一ベース. The executive committee confirmed it on 19 Jan 2026 for camp and exhibition games, with regular-season use "if there are no problems". Home plate is unchanged. [press] https://www.jiji.com/jc/article?k=2026011901055&g=spo , https://www.chunichi.co.jp/article/1195739
- That it was then used in first-team and farm official games from Opening Day comes from a secondary site only. [secondary] https://baseball-village.com/bigger_bases/
- NPB's 2024 rule note still gave 15 inches for Japan, so this is a change. [official] https://npb.jp/npb/2024rules.html
- Stolen bases did not rise: 921 in 2025, 814 in 2026 (CL 417 to 447, PL 504 to 367). [derived, same tables as E6]

### E8. MLB rules that do NOT apply in NPB

- Infield shift limits (rule 5.02(c)): the Japanese rule book says they are not applied in Japan. [official] https://npb.jp/npb/2024rules.html
- Three-batter minimum (rule 5.10(g)): not applied; a reliever must only finish the current batter. [official] https://npb.jp/npb/2025rules.html
- 2026 rule-book amendments are mostly wording (pitching motion, balks, runner hit by a batted ball). [official] https://npb.jp/npb/2026rules.html

### E9. Dangerous swing penalty (new during 2026)

- From 12 May 2026, first team and farm: a batter who lets go of the bat during a swing gets a warning; a second time in the same game means ejection; immediate ejection if the whole bat hits someone or goes into a dugout, camera pit or the stands. Bunts are excluded. [official] https://npb.jp/npb/2026rules_2.html ; [press] https://www.daily.co.jp/baseball/2026/05/11/0020342925.shtml

### E10. Other

- Probable starters are announced in both leagues (stated in the interleague, Climax Series and Japan Series outlines).
- Mound visits: the 2025 rule-book amendment limits visits without a pitching change to 4 per team per nine innings plus 1 per extra inning. Whether NPB applies this as written is **NOT CONFIRMED**. [official] https://npb.jp/npb/2025rules.html

---

## F. Announced for 2027 and later

| Change | Status | Source |
|---|---|---|
| Central League DH from 2027 | Decided | https://npb.jp/news/detail/20250804_05.html |
| Schedule: 143 games, opening 26 Mar, interleague 1 to 20 Jun, All-Star 21 and 22 Jul (Fukuoka, Chiba), CS from 9 Oct, Japan Series from 23 Oct at the PL park | Announced | https://npb.jp/event/2027/ , https://npb.jp/games/2027/ |
| Roster sizes 31 / 26, foreign 5 / 4 | Unchanged for 2027 | https://dnomotoke.com/archives/20261014143042/ (blog quoting Nikkan Sports, 14 Oct 2026) |
| PitchCom in first-team games from 2027, optional for clubs | Reported on 31 Aug 2026 (wire story carried by Akita Sakigake Shimpo) from league sources: "decided and already notified to the 12 clubs"; **no NPB announcement found** | https://www.sakigake.jp/news/article/20260831CO0139/ |
| Pitch clock | Decided in principle on 3 Aug 2026; **start date undecided**, a 2028 start was suggested in the press | https://kumanichi.com/articles/2021447 , https://full-count.jp/2026/09/18/post2017562/ |
| ABS, tie-break, expanding the Replay Center, more games, postseason format | Listed only as study items for the year to September 2027 | https://npb.jp/organization/docpb_09_2026.pdf |
| Farm League: three-division format continues; NPB will evaluate the two farm-only clubs and consider a second public call for new farm clubs | Plan | https://npb.jp/organization/docpb_09_2026.pdf |
| 2027 farm schedule and division membership | **Not yet announced** (the 2026 schedule came out on 22 Jan 2026) | https://npb.jp/news/detail/20260122_01.html |
| Yakult farm team moves from Toda (Saitama) to Moriya (Ibaraki): "Swallows Wings Square", opening March 2027; 100 m lines, 122 m centre, artificial turf, 3,000 seats | Decided | https://npb.jp/news/detail/20260109_03.html , https://yakult-swallows.co.jp/swallows_wings_square |
| Nippon-Ham farm team moves from Kamagaya (Chiba) to Eniwa (Hokkaido), target about 2030 | Announced 2 Jul 2026 | https://www.daily.co.jp/baseball/2026/07/02/0020542515.shtml |
| Lotte farm team moves from Urawa (Saitama) to Kimitsu (Chiba) in 2030 | Decided | https://npb.jp/news/detail/20260109_03.html , https://www.hokkaido-np.co.jp/article/1315096/ |
| Chunichi farm team to leave Nagoya Stadium for another Tokai municipality; target early 2030s, public call to municipalities in fiscal 2026 | Planned; site not chosen as far as I could confirm | https://npb.jp/news/detail/20260109_03.html , https://dime.jp/genre/2114047/ |
| Lotte first-team park: replacement stadium in Makuhari; basic plan being drawn up by Chiba City, the club and AEON Mall | Planning; opening year and roof type not confirmed | https://www.city.chiba.jp/sogoseisaku/miraitoshi/marinesaiseibi/marine_kihonkeikaku_three_party_agreement.html |
| Yakult first-team park: replacement stadium in the Jingu Gaien redevelopment | Planned; timetable not verified | https://ja.wikipedia.org/wiki/明治神宮野球場 |
| First-team expansion | None announced. I found no announcement of new outfield dimensions for any park for 2027 | (search result, no source) |

---

## G. Facts that contradict common assumptions

1. **The Eastern and Western Leagues are gone.** Since 2026 there is one Farm League with East, Central and West divisions, 14 teams, unequal schedules (135 to 146 planned, 111 to 126 played), no make-up games, and a four-team championship with a wild card.
2. **The Climax Series Final Stage is not always "6 games, 1-win advantage".** Since 2026 it is 7 games with a 2-win advantage when the champion won by 10 or more games or the challenger finished below .500. It was triggered in the Pacific League in its first year.
3. **2026 was the last Central League season without the DH.**
4. **Two "pitcher's parks" changed in 2026.** Vantelin Dome Nagoya's alleys went from 116 m to 110 m and its fence from 4.8 m to 3.6 m. Rakuten's park brought its fence in by up to 6 m (alleys 110 m and 112 m) and was renamed.
5. **Bases are 18 inches since 2026**, replay review moved to a central Replay Center, and a dangerous-swing ejection rule was added in May 2026.
6. **A pitch clock is decided but has no start date**; there was none in 2026.
7. **The 31-man roster, 26-man bench and 5 foreign players are still "special" extensions** of the written 29 / 25 / 4, renewed yearly and already renewed for 2027.
8. **Infectious-disease, concussion, family-leave and retirement registration exceptions were all in use in 2026.**
9. **Offence jumped in 2026** (home runs up 30 percent) while stolen bases fell despite the larger bases.
10. **ES CON Field's published dimensions disagree**: 97 m / 99 m lines in club-era reports, 98 / 100 in NPB's stadium database; centre field 121 or 122.

---

## H. Not confirmed

Ballparks (details per club in `npb_clubs.json`, field `unconfirmed`):

- `elevation_m` for all 12 parks is derived from the GSI elevation model at NPB's published coordinates. No park publishes an official elevation. ES CON Field's value is from the coarse 10 m model.
- ES CON Field: foul-line distances (97/99 v 98/100), centre field (121 v 122), alley distances, and the height of the high wall section near the left-field pole.
- Alley distances (`lcf`, `rcf`) are official only for Rakuten's park and Vantelin Dome. For the other ten they come from Japanese Wikipedia and one secondary list that agree with each other.
- Wall heights: official only for Jingu, Vantelin Dome, Rakuten and (4 m, rubber fence only) Tokyo Dome. Sources disagree for Koshien (3.0 v 3.2 m), Kyocera Dome (4.2 v 4.0 m), Yokohama (5.0 to 5.3 m) and Tokyo Dome (4.0 v 4.24 m).
- Capacities for Yokohama, Vantelin Dome, Kyocera Dome, Rakuten (2022 figure), ZOZO Marine (2025 figure) and Mazda are not from an operator page I could read.

Rules and structure:

- Pacific League standings tie-breakers: only the 2022 agreement excerpt is public.
- Regular-season 12-inning limit: no 2026 rule text found, but confirmed from NPB box scores (A4). The 15-second rule: no 2026 text and no evidence of how it is enforced.
- Request counts (2, plus 1 in extra innings): from 2017 press and 2026 explainers, not from an NPB page.
- Pitcher / fielder split for five foreign players: secondary sources only.
- 育成 rules: the 2014 edition is the newest text I could read.
- Farm extra-inning and tie-break rules, and farm DH use: inferred from box scores and play-by-play, not from the Farm League agreement.
- Regular-season use of the 18-inch bases from Opening Day 2026: press reported the plan; confirmation is from a secondary site.
- Whether SoftBank's fourth squad plays games in 2026.
- 2027: PitchCom (press report only), roster sizes (blog quoting Nikkan Sports), DH in interleague games and the Japan Series (implied only), spring camp and exhibition dates, farm schedule and divisions.
- Timetables for the Jingu and Makuhari replacement stadiums and the Chunichi farm move.
