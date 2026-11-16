# Player nationality and name-frequency sources (MLB, MiLB, KBO, NPB)

Compiled 2026-11-16. Every figure has a source URL and the season or date it refers to.

**Source labels**
- **[O]** Official: league, government or statistics-office publication, or official league data (MLB Stats API).
- **[P]** Press.
- **[C]** Compilation: academic fact sheet, Wikipedia, Baseball-Reference register, company survey or name site.
- **[Mine]** I computed it myself from roster data. The method is given each time.
- **(snippet)** The page blocked direct access, so the figure comes from a search-engine summary of it. It is unverified.

"Born outside the U.S." follows MLB's convention: born outside the 50 states, so Puerto Rico counts as outside. Counts are by **birthplace**, not citizenship, unless noted.

---

## 1. MLB Opening Day rosters: birthplace

Scope of MLB's counts: active 26-man rosters plus injured, restricted and suspended lists on Opening Day.

| Birthplace | 2025 [O] | 2026 corrected [O] | 2026 first release [O] |
|---|---|---|---|
| Dominican Republic | 100 | 93 | 93 |
| Venezuela | 63 | 60 | 60 |
| Cuba | 26 | 20 | 20 |
| Canada | 13 | **19** | 17 |
| Japan | 12 | 14 | 14 |
| Puerto Rico | 16 | 14 | 14 |
| Mexico | 11 | 7 | 7 |
| Curaçao | 4 | 4 | 4 |
| Panama | 4 | 4 | 4 |
| Colombia | 2 | 3 | 3 |
| South Korea | 3 | 3 | 3 |
| Aruba | 2 | 1 | 1 |
| Bahamas | 1 | 1 | 1 |
| Honduras | 1 | 1 | 1 |
| Nicaragua | 1 | 1 | 1 |
| Taiwan | – | 1 | 1 |
| Australia | 2 | – | – |
| Germany | 1 | – | – |
| South Africa | 1 | – | – |
| "Miscellaneous" (MLB lists these separately from the countries) | 2 (born in Japan 1, Peru 1) | 3 (born in Japan 1, Peru 1, United Kingdom 1)\* | 3 (Japan 1, Peru 1, UK 1) |
| **Total born outside the 50 states** | **265** | **249** | 247 |
| Player pool | 954 | 948 (780 active + 168 inactive) | 948 |
| Share born outside the 50 states | 27.8% | 26.3% | 26.1% |
| Born in the 50 states (pool minus total, my arithmetic) | 689 | 699 | 701 |
| Countries and territories counted by MLB | 18 | 16 | 16 |

\* The corrected 2026 web release does not reprint the player list. Its 16 country counts sum to 246, so 3 are missing from the stated 249. That matches the 3 "Miscellaneous" entries in the original 2026 PDF. **By pure birthplace**, Japan is therefore 13 in 2025 and 15 in 2026, and Peru is 1 in both years. The UK is 1 in 2026.

**Sources**
- 2025: MLB press release, "Opening Day Rosters Feature 265 Internationally Born Players," Opening Day rosters of 2025-03-27. https://www.mlb.com/press-release/press-release-opening-day-rosters-feature-265-internationally-born-players [O]. I re-counted the release's own player list by country, and it matches the stated counts.
- 2026, corrected: MLB press release, "Opening Day rosters feature 249 internationally born players." https://www.mlb.com/press-release/press-release-opening-day-rosters-feature-249-internationally-born-players [O]
- 2026, first release (dated 2026-03-26, PDF with full player list): https://img.mlbstatic.com/mlb-images/image/upload/mlb/rdlkdgzbavfdjmo6ahko.pdf [O]. Web version: https://www.mlb.com/news/press-release-opening-day-rosters-feature-247-internationally-born-players [O]
- AP via The Canadian Press/TSN, 2026-03-26. https://www.tsn.ca/mlb/article/foreign-born-players-in-mlb-drops-to-lowest-percentage-since-2002/ [P]. It reports:
  - The correction: MLB's first announcement left out 2 Canadian-born players.
  - 26.3% is the lowest share since 2014 (also 26.3%). The headline's "since 2002" was corrected.
  - The 2017 high was 29.8%.
  - The 2026 pool was 780 active + 162 IL + 6 restricted.

**Context from the same releases [O]**
- The Dominican Republic has led every year since MLB began releasing this data in 1995.
- Previous totals: 2024 = 264, 2023 = 270, 2022 = 275 (28-man rosters), 2020 = 291 (30-man rosters).
- Cuba's 26 in 2025 was a record.

**Cross-check [Mine]:** I pulled MLB Stats API active rosters for 2026-03-26 (`/api/v1/teams/{id}/roster?rosterType=active&date=2026-03-26&hydrate=person`, field `birthCountry`).
- Result: 775 active players, 27.1% born outside the USA.
- The API birth-country data differs from MLB's official list by a few players. It shows 1 Germany-born and 1 Italy-born player that the release does not list.
- Use the official release for MLB. The API is still useful for the minor-league levels below, where nothing official is published.

---

## 2. US minor leagues and the Dominican Summer League

### 2a. Share by level, 2026 and 2025 [Mine, from MLB Stats API, official data]

**Method**
- All MLB-affiliated teams by level: `sportId` 11 = AAA, 12 = AA, 13 = High-A, 14 = Single-A, 16 = Rookie, split into ACL, FCL and DSL by league.
- Active roster on **July 1** of each season, `birthCountry` of each player. Each player is counted once per level.
- July 1 is before the July draft, so new US draftees are not yet in the complex leagues. Table 2b shows the effect.
- API endpoint: `https://statsapi.mlb.com/api/v1/teams/{teamId}/roster?rosterType=active&season=2026&date=2026-07-01&hydrate=person`

**2026 (active rosters, 2026-07-01)**

| Level | Players | USA | DOM | VEN | PUR | CUB | MEX | CAN | PAN | COL | Other | Born outside the USA |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| AAA (30 teams) | 849 | 641 | 70 | 58 | 14 | 17 | 11 | 11 | 4 | 5 | 18 | 24.5% |
| AA (30) | 822 | 622 | 68 | 64 | 9 | 15 | 12 | 13 | 5 | 4 | 10 | 24.3% |
| High-A (30) | 870 | 583 | 126 | 98 | 7 | 12 | 7 | 15 | 4 | 4 | 14 | 33.0% |
| Single-A (30) | 878 | 430 | 183 | 156 | 10 | 19 | 26 | 11 | 4 | 2 | 37 | 51.0% |
| Arizona Complex League (15) | 509 | 115 | 169 | 147 | 3 | 15 | 16 | 7 | 7 | 13 | 17 | 77.4% |
| Florida Complex League (15) | 463 | 97 | 167 | 129 | 5 | 8 | 14 | 7 | 10 | 5 | 21 | 79.0% |
| Dominican Summer League (51) | 1,669 | 7 | 795 | 618 | 0 | 45 | 70 | 0 | 38 | 36 | 60 | 99.6% |
| **US-based, AAA to FCL** | 4,391 | 2,488 | 783 | 652 | 48 | 86 | 86 | 64 | 34 | 33 | 117 | **43.3%** |
| **All levels including DSL** | 6,060 | 2,495 | 1,578 | 1,270 | 48 | 131 | 156 | 64 | 72 | 69 | 177 | **58.8%** |

MLB active rosters on the same date: 771 players, 27.1% born outside the USA.

**"Other" in the DSL in 2026** (60 players): Nicaragua 13, Curaçao 10, Aruba 7, Brazil 4, Germany 3, Italy 3, Bahamas 3, Japan 3, Costa Rica 2, Spain 2, and 1 each from China, Australia, Sint Maarten, Netherlands, Haiti, Uganda, Russia, Sudan, France and the U.S. Virgin Islands.

**Small countries in the US-based levels:** Taiwan, Australia, South Korea, Japan, Bahamas, Nicaragua and Curaçao each have about 5 to 13 players.

**2025 (active rosters, 2025-07-01)**

| Level | Players | USA | DOM | VEN | PUR | CUB | MEX | CAN | PAN | COL | Other | Born outside the USA |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| AAA | 850 | 636 | 78 | 60 | 14 | 19 | 7 | 11 | 1 | 5 | 19 | 25.2% |
| AA | 828 | 616 | 75 | 58 | 13 | 14 | 13 | 11 | 4 | 3 | 21 | 25.6% |
| High-A | 854 | 573 | 107 | 90 | 10 | 13 | 16 | 18 | 6 | 5 | 16 | 32.9% |
| Single-A | 863 | 408 | 190 | 164 | 12 | 18 | 15 | 19 | 4 | 6 | 27 | 52.7% |
| ACL | 486 | 94 | 177 | 137 | 4 | 8 | 17 | 3 | 12 | 8 | 26 | 80.7% |
| FCL | 478 | 97 | 192 | 117 | 6 | 18 | 18 | 4 | 6 | 4 | 16 | 79.7% |
| DSL (52 teams) | 1,699 | 14 | 772 | 656 | 0 | 49 | 75 | 0 | 34 | 41 | 58 | 99.2% |
| **US-based, AAA to FCL** | 4,359 | 2,424 | 819 | 626 | 59 | 90 | 86 | 66 | 33 | 31 | 125 | **44.4%** |
| **All levels including DSL** | 6,058 | 2,438 | 1,591 | 1,282 | 59 | 139 | 161 | 66 | 67 | 72 | 183 | **59.8%** |

### 2b. Everyone who appeared during the 2026 season [Mine, MLB Stats API, `rosterType=fullSeason`]

This table includes post-draft signings and rehab assignments. Each player is unique within a level; a player who moved between levels appears in each.

| Level | Players | Born outside the USA | DOM | VEN | MEX | CUB | CAN | PUR |
|---|---|---|---|---|---|---|---|---|
| AAA | 2,092 | 24.7% | 171 | 146 | 27 | 34 | 31 | 35 |
| AA | 1,808 | 27.0% | 175 | 148 | 26 | 29 | 32 | 19 |
| High-A | 1,865 | 33.5% | 248 | 211 | 25 | 28 | 42 | 22 |
| Single-A | 2,228 | 43.4% | 388 | 332 | 48 | 41 | 38 | 24 |
| ACL | 859 | 63.8% | 224 | 192 | 26 | 20 | 13 | 8 |
| FCL | 883 | 63.3% | 254 | 189 | 22 | 21 | 14 | 9 |
| DSL | 1,809 | 99.6% | 858 | 675 | 77 | 45 | 0 | 0 |

**Unique players across levels, 2026**
- US-based minors (AAA to FCL): 6,141 players, 38.3% born outside the USA.
  - Dominican Republic 15.6%, Venezuela 12.4%, Mexico 1.8%, Cuba 1.7%, Canada 1.6%, Puerto Rico 1.3%.
- All levels including DSL: 7,896 players, 52.0% born outside the USA.
  - Dominican Republic 22.7%, Venezuela 17.9%, Mexico 2.4%, Cuba 1.9%.

The complex leagues drop from about 78% born outside the USA on July 1 to about 63% over the full season, because US draftees join after the July draft.

### 2c. Independent check: George Mason University fact sheets [C, academic]

Institute for Immigration Research, George Mason University. Rosters come from Baseball-Reference, accessed in October 2025 and August 2026. The Dominican Summer League is excluded. "Foreign-born" excludes US territories and people born abroad to US-citizen parents.
- Page: https://iir.gmu.edu/sports-and-civic-engagement/in_sports/minor-league-baseball-milb
- 2026 fact sheet: https://d101vc9winf8ln.cloudfront.net/documents/56954/original/2026_MiLB_Fact_Sheet.pdf
- 2025 fact sheet: https://d101vc9winf8ln.cloudfront.net/documents/54521/original/2025_MiLB_Fact_Sheet.pdf

| Season | Players | Foreign-born | Born in a US territory | Countries |
|---|---|---|---|---|
| 2025 | 6,034 | 2,270 (38%) | 86 (1%) | 36 |
| 2026 | 5,900 | 2,255 (38%) | 76 (1%) | 35 |

**By level: US-born / foreign-born / territory-born.** A player who moved between levels is counted at each.

| Level | 2025 | 2026 |
|---|---|---|
| AAA | 76 / 23 / 2 (n = 2,333) | 76 / 22 / 2 (n = 2,101) |
| AA | 73 / 26 / 1 (n = 1,778) | 74 / 25 / 1 (n = 1,675) |
| High-A | 68 / 31 / 1 (n = 1,798) | 67 / 32 / 1 (n = 1,745) |
| Single-A | 55 / 43 / 1 (n = 2,092) | 55 / 44 / 1 (n = 2,009) |
| Rookie, ACL+FCL only | 36 / 63 / 1 (n printed as "16,62") | 36 / 63 / 1 (n = 1,745) |

**Country of birth among foreign-born players, 2026 (N = 2,255)**
- Dominican Republic 42.3%, Venezuela 33.6%, Mexico 5.0%, Cuba 4.5%, Canada 4.0%.
- Panama 1.8%, Colombia 1.8%, Taiwan 0.9%, Nicaragua 0.8%, Japan 0.7%, Australia 0.7%.
- South Korea 0.6%, Bahamas 0.6%, Curaçao 0.4%, Aruba 0.3%, Netherlands 0.3%.
- Germany, Italy, Spain and Brazil 0.2% each; New Zealand 0.1%; England, South Africa and Uganda 0.09% each; China and Thailand 0.04% each.
- 3 other Caribbean countries 0.2%; 6 other European countries 0.3%.

**Same, 2025 (N = 2,270)**
- Dominican Republic 43.1%, Venezuela 33.1%, Mexico 4.8%, Cuba 4.6%, Canada 3.8%.
- Panama 1.9%, Colombia 1.8%, Nicaragua and El Salvador 0.8%, Japan 0.7%, Taiwan 0.7%, Australia 0.7%.
- Bahamas 0.6%, South Korea 0.5%, Netherlands 0.4%, Germany 0.2%, Brazil 0.2%.

My API computation agrees with GMU's within about 1 to 2 points at each level for comparable dates.

### 2d. Older and press figures

| Figure | Year | Source |
|---|---|---|
| DSL by nationality: Dominican Republic 760, Venezuela 560, Mexico 41, Colombia 32, Panama 31, Cuba 25, Curaçao 14, USA 12, Nicaragua 10, Brazil 8, Aruba 5, Bahamas 5, Netherlands 3, Costa Rica 1, Honduras 1, Spain 1. Total about 1,509 (sum computed by the summarizer). | 2021 | El Día, 2021-07-28. https://eldia.com.do/560-venezolanos-en-liga-de-verano-rd/ [P] |
| "The percentage of foreign-born minor league players was 48.8, up from 47.8 last year." Based on MLB data; covers all minor league contracts, including the DSL. | 2015 | ESPN/ABC30, 2015-04-06. https://abc30.com/post/cuban-born-players-on-mlb-rosters-dip-in-2015-despite-high-profile-signings/633809/ [P] |
| Opening Day minor league rosters: US 2,392 (73.1%), Dominican Republic 381 (11.7%), Venezuela 269 (8.2%) | 2019 | Baseball America, "2019 Minor League Baseball Rosters By The Numbers" (snippet; page returns 403). https://www.baseballamerica.com/stories/2019-minor-league-baseball-rosters-by-the-numbers/ [P] |

### 2e. International amateur signing system (context)

**Rules for 2026 [O, league media]** — MLB.com, 2026-01-22: https://www.mlb.com/news/mlb-international-prospects-signing-day-2026
- **Signing period:** Jan 15 to Dec 15.
- **Age:** a player must turn 16 before signing and be 17 before Sept 1 of the following year. For 2026 that means born Sept 1, 2008 to Aug 31, 2009, or older.
- **Registration:** players must be registered with MLB in advance.
- **Bonus pools:** hard-capped. Teams can acquire at most 60% of their original pool by trade. Bonuses of $10,000 or less do not count against the pool.
- **2026 pool amounts:**
  - $8,034,900: AZ, BAL, CLE, COL, KC, PIT, STL
  - $7,357,100: ATH, CIN, DET, MIA, MIL, MIN, SEA, TB
  - $6,679,200: ATL, CHC, CWS, LAA, LAD, PHI, TEX, WSH
  - $5,940,000: BOS, SD, TOR
  - $5,440,000: HOU, NYY, NYM, SF
- **MLB Pipeline top 50 international prospects, 2026:** 33 from the Dominican Republic, 14 from Venezuela, 3 from Cuba.

**Signings in the 2025 period (Jan 15 to Dec 15, 2025) [P]**
- As of 2025-12-02, 1,011 players from 23 countries had signed: Dominican Republic 441, Venezuela 386, Mexico 62, Cuba 39, Colombia 25, Taiwan 5, Italy 4, Brazil 2. The full list was cut off.
- This count was compiled by journalist Francys Romero and reported by World Baseball Network, 2025-12-03. https://worldbaseball.com/the-rapid-growth-of-international-born-players-in-major-league-baseball-in-2025
- It is not an official MLB figure. I found no official MLB count of signings by country.

---

## 3. KBO (Korea)

### 3a. Foreign-player rules in force for 2026

| Rule | Detail | Source |
|---|---|---|
| Regular foreign players | 3 per club. With the Asia quota, a club holds 4 in total, and **all 4 may play in the same game**. | KBO, 2026-03-26 [O] |
| Positions in practice | Every club signed 2 pitchers and 1 position player for 2026. I did not find the formal position rule in the regulations. | Yonhap via Korea Times, 2026-01-02 [P] |
| Salary cap for new foreign players | $1M per player, counting salary, signing bonus, options and transfer fee. | OSEN, 2022-10-18 [P]; StarNews, 2026-10-22 [P] |
| Team cap for the 3 foreign players | $4M combined (in force since 2023). The cap rises by $100k for each re-signing year of each re-signed player; the article's example reaches $4.6M with three re-signed players. Incentives count as actually paid. | OSEN, 2022-10-18 [P] |
| Injury replacements (since 2024) | If a foreign player needs at least 6 weeks of treatment, the club may sign a replacement without using a replacement count. The replacement costs at most $100k per month. When the injured player returns, the club must either swap the replacement (using one count) or release him. Any nationality is possible: Japanese and Australian replacements were used in 2026. | KBO, 2023-11-01 [O] |
| **Asia quota (new in 2026)** | 1 player per club. | KBO, 2025-01-22 and 2026-03-26 [O] |
| Asia quota: who is eligible | Nationals of countries in the Baseball Federation of Asia, plus Australia. Dual nationals who hold a non-Asian nationality are excluded. The player must have played in an Asian league in the previous or current year. Any position. | same [O] |
| Asia quota: cost | At most **$200,000** for a new signing: salary, signing bonus, options actually paid and transfer fee, excluding tax, with at most $20,000 per month. On re-signing, salary may rise by $100,000 per year. | same [O] |
| Asia quota: replacement | Once per season. | KBO, 2025-01-22 [O] |
| Roster sizes | Game roster grows from 28 registered / 26 eligible to 29 / 27. Club roster limit rises from 65 to 68. For the second half of 2026 only, 30 / 28 to cover players sent to the Asian Games. | KBO, 2025-01-22 and 2026-03-26 [O]; Newsseoul on the 2026-06-02 executive committee [P] |
| Rookie of the Year | Asia-quota players are not eligible. | OSEN headline (snippet) [P] |
| Futures League community clubs | Their foreign-player limit rose from 4 to 6. This applies to the independent clubs in Korea's second-tier Futures League, not to KBO teams. | Newsseoul, 2026-06-08 [P] |

**Source URLs**
- KBO, 2025-01-22 (board decision): https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11353
- KBO, 2026-03-26 (2026 changes): https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11879
- KBO, 2023-11-01 (injury replacements): https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9862
- OSEN, 2022-10-18: https://osen.co.kr/article/G1111968536
- StarNews, 2026-10-22: https://www.starnewskorea.com/en/sports/2026/10/22/2026101811073891063
- Newsseoul (2026-06-02 executive committee): https://newsseoul.co.kr/news/view/1065600624087265
- Yonhap via Korea Times, 2026-01-02: https://www.koreatimes.co.kr/sports/20260102/kbo-clubs-complete-foreign-player-signings-for-new-season

### 3b. Rules announced for 2027: none confirmed

- **Proposal under review:** raise the holding limit from 3 to 5 foreign players, with first-team registration and play still capped at 3. Adding the Asia quota player gives up to 6 per club. The Players' Association opposes it.
  - Segye Ilbo, 2026-08-21: https://www.segye.com/newsView/20260821508050 [P]
  - KBO said it would decide the direction "within this year," after talking with the Players' Association (June 2026 report, snippet) [P]
- **KBO board, 2026-10-07:** easing the holding limit was only "placed under review." The size and timing are not set.
  - Korea Daily: https://www.koreadaily.com/article/20261007234049854 [P]
  - Xportsnews: https://www.xportsnews.com/article/2205770 [P]
- **The only 2027 foreign-player item decided** at that board meeting: foreign players' contracts follow the new participation period of Jan 25 to Nov 23, from 2027-01-01 [P, same articles].
- The KBO's official press-release list from 2026-10-24 to 2026-11-16 has no foreign-player rule decision. https://www.koreabaseball.com/MediaNews/Notice/List.aspx [O]
- A StarNews columnist proposed letting the Asia quota include players from non-Asian leagues such as the US minors. This is opinion, not a rule [P, 2026-10-22].

### 3c. Nationalities of 2026 foreign players

**(i) Contracted at the start of 2026: 30 regular foreign players [Mine]**
- Method: the club-by-club signings reported by Yonhap/Korea Times (2026-01-02) and Dailian (2026-01-02), plus Kyunghyang (2025-12-11) for NC and press reports on Samsung's third player. Birth country comes from Baseball-Reference.
- Sources:
  - https://www.koreatimes.co.kr/sports/20260102/kbo-clubs-complete-foreign-player-signings-for-new-season [P]
  - https://www.dailian.co.kr/news/view/1593342/ [P]
  - https://sports.khan.co.kr/article/202512111431003 [P, snippet]

| Birth country | Players (of 30) |
|---|---|
| USA | 19 |
| Venezuela | 5 |
| Dominican Republic | 3 |
| Cuba | 1 |
| Panama | 1 |
| Canada | 1 |

Positions: 20 pitchers and 10 position players.

At least 2 of the 30 were gone before Opening Day:
- One club's US pitcher had his deal voided after a medical in January (OSEN, snippet).
- Another club's US pitcher was released before the season with an elbow injury. That club then signed an Australian pitcher on a 6-week temporary contract (OSEN/Daum, 2026-03, snippet).

**(ii) Asia-quota players [P]**

| Point in time | Japan | Australia | Taiwan | Source |
|---|---|---|---|---|
| Signed, as of 2025-12-24 | 7 | 2 | 1 | Yonhap via Korea Times, 2026-01-02 |
| End of regular season | 8 | 1 | 1 | StarNews, 2026-10-22. Consistent with the 4 swaps below. |

- **Previous leagues:**
  - Japanese independent leagues (e.g., Shikoku Island League), WBSC via WBN, 2025-12-09: https://worldbaseball.com/first-players-signed-under-new-kbo-import-players-quota-rule
  - NPB first and farm teams (Korea Herald, 2025-12-30: https://www.koreaherald.com/article/10646003 ; Kyunghyang, 2026-07-07)
  - An NPB farm system (the Taiwanese player)
  - The Australian league and an NPB farm team (the Australian infielder)
- **In-season swaps:** 4 clubs replaced their Asia-quota player. One swap was Australian to Japanese, and the other three replacements were Japanese.
  - Kyunghyang, 2026-07-07: https://sports.khan.co.kr/article/202607071434003 [P]
  - Dailian, 2026-08-08: https://www.dailian.co.kr/news/view/1676094/ [P]
  - Xportsnews, 2026-09: "4 cases" [P, snippet]

**(iii) Every non-Korean player who appeared in a 2026 KBO regular-season game [Mine]**
- Source: Baseball-Reference register roster tables for all 10 KBO clubs, birthplace field, accessed 2026-11-16. https://www.baseball-reference.com/register/league.cgi?id=163dcec5 [C]
- Each player is counted once. One player played for two clubs.
- One Korean-national domestic player born in China is excluded.
- 9 players had no birthplace on Baseball-Reference. I assigned their nationality from press reports: 8 Japanese and 1 Taiwanese. For 2 of the 8, Japanese nationality is inferred: one from the end-of-season count, one from a signing reported as coming from a Japanese independent league.

| Birth country / nationality | All (65) | Asia quota (14) | Other foreign, including injury replacements (51) |
|---|---|---|---|
| USA | 32 | 0 | 32 |
| Japan | 12 | 11 | 1 |
| Venezuela | 7 | 0 | 7 |
| Dominican Republic | 5 | 0 | 5 |
| Australia | 4 | 2 | 2 |
| Taiwan | 1 | 1 | 0 |
| Cuba | 1 | 0 | 1 |
| Panama | 1 | 0 | 1 |
| Canada | 1 | 0 | 1 |
| Puerto Rico | 1 | 0 | 1 |

---

## 4. NPB (Japan)

### 4a. Foreign-player rules

**Official text: NPB Agreement (日本プロフェッショナル野球協約), 2025 edition [O]**
- Hosted by the players' association (JPBPA): https://jpbpa.net/wp-content/uploads/jpbpa-pdf/ag2025.pdf (listing page: https://jpbpa.net/contract/).
- No 2026 edition is posted there.

| Rule | Text, paraphrased |
|---|---|
| Art. 82-2: number of foreign players | A club may hold **any number** of foreign players under 支配下 (registered) contracts. **Active-roster (出場選手登録) registration is limited to 4**, and at most **3** of them may be registered as pitchers, or at most 3 as position players, at the same time. Allowed mixes at the limit: 2 P + 2 B, 3 P + 1 B, or 1 P + 3 B. |
| Art. 79 | At most 70 支配下 players per club per season (80 under the Art. 57-2 relief rule). |
| Art. 81 | Active roster of at most 29. |
| Art. 82: who counts as foreign | Anyone without Japanese nationality, **except** the cases below. |
| Exception (1) | 3 or more cumulative years at a Japanese junior high school, high school, junior college or technical school, or a school admitted to the high-school baseball federation, before signing. |
| Exception (2) | 4 or more consecutive years at a Japanese university, or a member team of the Japan University Baseball Federation, before signing. |
| Exception (3) | 5 or more years living in Japan plus 3 or more seasons on a Japan Amateur Baseball Association (company or club) team. |
| Exception (4) | The player has qualified for NPB free agency, from the next season on. Per ja.wikipedia (below), three players active in 2026 are exempt this way: born in Cuba (FA from 2023), Cuba (2025) and Puerto Rico (2025). |
| Exception (5) | Draftees who fell short of (1) or (2) become exempt once their school years plus seasons as a 支配下 player reach 5. This applies from the next season and needs approval by the executive committee. |

- **Schooling exception and the draft:** players covered by exceptions (1) to (3) and (5) must enter NPB through the draft. Exception (4) does not need the draft.
  - Source: Japanese Wikipedia "外国人枠 (日本プロ野球)," revision of 2026-08-24, citing the 2025 Agreement. https://ja.wikipedia.org/wiki/外国人枠_(日本プロ野球) [C]
- **Development (育成) contracts:** foreigners may sign them. A foreigner aged 26 or over who newly signs a 育成 contract can be promoted to 支配下 only until March 31 (育成 rules Art. 9-3, per the same Wikipedia page) [C].
- **Special rule allowing 5 active foreign players:** since 2020, under the COVID-era special rules, the limit has been 5 registered, with at most 4 on the game bench, not all pitchers and not all position players. Japanese Wikipedia (2026-08-24) says this applied **through 2025** [C].
  - Blogs say it continues in 2026, e.g., https://teams.one/news/npb-foreign-player-rules/ [C].
  - **I found no official NPB confirmation for 2026.**

### 4b. Nationalities of foreign-born players in 2026 [Mine]

**Method**
- Baseball-Reference register roster tables for all 12 NPB clubs, birthplace field, accessed 2026-11-16.
  - Central League: https://www.baseball-reference.com/register/league.cgi?id=4f61709d [C]
  - Pacific League: https://www.baseball-reference.com/register/league.cgi?id=9f67f305 [C]
- These tables cover **only players who appeared in first-team (1軍) games in 2026.**
- Counted: players with a birthplace outside Japan. Of 743 roster rows, 413 have no birthplace on Baseball-Reference. Almost all of those have Japanese names; 3 have non-Japanese names and are left out because they could not be verified.

| Birth country | Central League | Pacific League | NPB total |
|---|---|---|---|
| USA | 15 | 12 | 27 |
| Dominican Republic | 16 | 11 | 27 |
| Venezuela | 5 | 4 | 9 |
| Taiwan | 0 | 5 | 5 |
| Cuba | 2 | 2 | 4 |
| Panama | 2 | 0 | 2 |
| Mexico | 0 | 2 | 2 |
| Puerto Rico | 0 | 1 | 1 |
| Colombia | 0 | 1 | 1 |
| **Total** | **40** | **38** | **78** |

Per club: 4 to 9 foreign-born players used.

Caveats:
- Not everyone counted here takes a foreign-quota slot. At least 3 of the 78 (2 Cuban-born, 1 Puerto Rican-born) are exempt through the FA rule.
- Foreign players who stayed in the farm or on 育成 contracts are not counted. Several clubs keep many young Dominicans, Cubans, Venezuelans and Taiwanese there.
- NPB publishes no nationality field, and I found no published 2026 count of NPB foreign players by nationality.

Official 2026 Opening Day active-roster notice, with players listed by name (foreign players appear in katakana): https://npb.jp/announcement/roster/roster_0326.html [O]

---

## 5. Name-frequency sources

| Country / dataset | Source and type | Data year | Top entries with counts |
|---|---|---|---|
| **USA surnames** | Census Bureau, "Frequently Occurring Surnames from the 2010 Census." Page: https://www.census.gov/topics/population/genealogy/data/2010_surnames.html ; file: https://www2.census.gov/topics/genealogy/2010surnames/names.zip [O] | 2010 census | 162,253 surnames with at least 100 bearers, covering 265,667,228 people. An "ALL OTHER NAMES" row holds 29,312,001 more. Each name has % White, Black, API, AIAN, two-or-more races and Hispanic, which helps when generating Hispanic-heritage US-born players. See the top 20 below. |
| **USA given names, boys born 1995 to 2009** | SSA national baby-name data (Social Security card applications; every name with 5 or more births per year and sex). File names.zip (yob1880 to the latest year): https://www.ssa.gov/oact/babynames/limits.html [O] | yearly | **Not computed: ssa.gov returns 403 from this environment.** Method: sum `n` for sex = M in yob1995 to yob2009. Decade tables (via search, SSA pages): 2000s (21.28M male births): Jacob 274,064, Michael 250,702, Joshua 232,037, Matthew 221,620, Daniel 203,951 (https://www.ssa.gov/oact/babynames/decades/names2000s.html ; the order after Jacob comes from NameCensus https://namecensus.com/decades/2000s/ [C]). 1990s (20.56M male births): Michael 462,451, Christopher 360,280, Matthew 351,673, Joshua 329,184 (https://www.ssa.gov/oact/babynames/decades/names1990s.html) (snippet). |
| **Dominican Republic given names** | ONE (national statistics office), "Infografía: nombres más comunes registrados 2022 y 2023," from civil-registry birth records [O]. https://www.one.gob.do/publicaciones/2024/infografia-nombres-mas-comunes-registrados-2022-y-2023/ (returns 403 here; figures via Diario Libre, 2024-11-25: https://diariolibre.com/actualidad/nacional/2024/11/25/los-nombres-mas-comunes-en-republica-dominicana/2922804 [P]) | births 2022 to 2023 | Boys born in 2023: Adriel 334, Enmanuel 290, Sebastián 288. 2022: Sebastián 335. Among **fathers'** names (closer to the players' cohort): José first, then Juan and Luis; no counts were reported (El Día, 2024-11-20: https://eldia.com.do/como-te-llamas-sorpresa-asi-cambian-los-nombres-mas-populares-en-rd/ [P]). **Not found:** names by birth cohort for 1995 to 2009. |
| **Dominican Republic surnames** | "Top 15 per the JCE (Junta Central Electoral)": Rodríguez, Pérez, Martínez, García, Reyes, Sánchez, Díaz, Peña, Jiménez, Ramírez, Hernández, Rosario, González, Santana, Núñez. En Segundos, 2021-12-23: https://ensegundos.do/2021/12/23/top-100-apellidos-mas-comunes-en-la-republica-dominicana [P/C] | not stated | No counts. The article credits the JCE but also says the list came from Forebears (conflict). **Not found: official counts.** |
| **Venezuela surnames and given names** | **Official: not found.** Academic: Rodríguez-Larralde A. et al., "Surname frequency and the isonymy structure of Venezuela," *Am J Hum Biol* 12(3):352–362, 2000; surnames of 3.9M adults aged 40+ from the 1991 electoral register [C]. I did not see its tables. Statista via Gestión (2023): González is the most common surname in Venezuela (snippet) https://gestion.pe/mix/tendencias-mix/los-apellidos-mas-comunes-en-america-latina-cuales-son-apellidos-frecuentes-en-latinoamerica-2023-bolivia-peru-colombia-ecuador-chile-brasil-argentina-noticia/ [P] | 1991 / 2023 | Not found. |
| **Cuba surnames and given names** | **Official (ONEI): not found.** Press citing Forebears/Statista: Rodríguez is the most common surname. https://www.larepublica.co/ocio/rodriguez-es-el-apellido-mas-comun-en-colombia-costa-rica-uruguay-cuba-y-panama-3609911 [P] (snippet) | not stated | Not found. |
| **Mexico surnames** | INE electoral roll (padrón electoral), as reported by Xataka, 2018-02-20: https://www.xataka.com.mx/otros-1/cuales-son-los-apellidos-mas-comunes-en-mexico-el-ine-lo-revela-en-su-padron [P]. INE statistics page cited: http://www.ine.mx/credencial/estadisticas-lista-nominal-padron-electoral/ [O, not checked] | about 2018 | **Paternal surname:** Hernández about 3.53M, García about 2.60M, then Martínez, López, González. **Maternal surname:** Hernández about 2.61M, then the same order. Most common full name: Juan Hernández Hernández (2,838). Conflict: El Cronista, citing an INE 2016 report, gives Hernández 4,875,002, García 3,575,861, Martínez 3,385,839 (snippet) https://www.cronista.com/mexico/actualidad-mx/estos-son-los-100-apellidos-100-mexicanos-mas-comunes/ [P] |
| **Mexico given names** | INEGI ("Cuéntame de México"), newborns 2017 to 2020, via Expansión, 2022-04-12: https://politica.expansion.mx/mexico/2022/04/12/cuales-son-los-nombres-mas-comunes-en-mexico [P] | births 2017 to 2020 | Santiago 46,943, Mateo 35,482, Sebastián 25,554, Leonardo 23,687, Matías 21,953, Emiliano 20,863, Diego 20,005. **This cohort is too young.** Not found: names for the 1995 to 2009 cohort. Press says RENAPO's all-population list has Juan first and José Luis second, without counts or a period (snippet). |
| **Japan surnames** | 名字由来net (Myoji-Yurai), estimated by ルーツ製作委員会 from government statistics and national telephone directories, data of 2026-10: https://myoji-yurai.net/prefectureRanking.htm [C]. Official government surname statistics: none found. | 2026-10 | See the top 20 below. |
| **Japan boys' names, born 1995 to 2009** | Meiji Yasuda Life, 「生まれ年別名前ベスト10」 (survey of its policyholders' children, done each year since 1989): https://www.meijiyasuda.co.jp/enjoy/ranking/year_men/boy.html [C] | each birth year | **Top 10 by year, kanji only, no counts on the page.** See the table below. |
| **Taiwan surnames** | Ministry of the Interior, Department of Household Registration, 《全國姓名統計分析》 (8th edition), household registers of 2023-06-30. Press release: https://www.ris.gov.tw/info-liferay/app/channel/pressReleaseDetail/23303697 [O] | 2023-06-30 | Population 23,373,283. Top 10 in order: 陳, 林, 黃, 張, 李, 王, 吳, 劉, 蔡, 楊; together 12,339,778 (52.79%). Top 100 surnames cover 22,582,375 (96.62%). 1,785 surnames in all. Per surname: 陳 2,618,994 (11.21%), 林 1,947,520 (8.33%) (遠見, snippet: https://www.gvm.com.tw/article/107385 [P]). An earlier edition (風傳媒; year reported as 2016): 陳 2,619,560 (11.14%), 林 1,953,760 (8.31%), 黃 1,421,439 (6.05%) (https://www.storm.mg/article/182130, snippet). |
| **Taiwan male given names** | Same MOI release [O] | 2023-06-30 | All ages: 家豪 14,038, 志明 12,719, 建宏 12,196. These are mostly older cohorts. For people born 2001 to 2010, the 2018 edition put 承恩 first (press, snippet: https://newtalk.tw/news/view/2018-11-05/162285). Not found: counts for the 1995 to 2009 cohort. |

**US Census 2010, top 20 surnames (count; % Hispanic) [O]**
1. Smith 2,442,977 (2.4)
2. Johnson 1,932,812 (2.4)
3. Williams 1,625,252 (2.5)
4. Brown 1,437,026 (2.5)
5. Jones 1,425,470 (2.3)
6. Garcia 1,166,120 (92.0)
7. Miller 1,161,437 (2.2)
8. Davis 1,116,357 (2.4)
9. Rodriguez 1,094,924 (93.8)
10. Martinez 1,060,159 (92.9)
11. Hernandez 1,043,281 (94.9)
12. Lopez 874,523 (92.9)
13. Gonzalez 841,025 (95.0)
14. Wilson 801,882 (2.6)
15. Anderson 784,404 (2.4)
16. Thomas 756,142 (2.5)
17. Taylor 751,209 (2.5)
18. Moore 724,374 (2.3)
19. Jackson 708,099 (2.5)
20. Martin 702,625 (5.6)

**名字由来net, top 20 Japanese surnames (approximate bearers) [C]**
1. 佐藤 1,782,000
2. 鈴木 1,732,000
3. 高橋 1,351,000
4. 田中 1,284,000
5. 伊藤 1,029,000
6. 渡辺 1,020,000
7. 山本 1,005,000
8. 中村 1,003,000
9. 小林 989,000
10. 加藤 855,000
11. 吉田 795,000
12. 山田 781,000
13. 佐々木 638,000
14. 山口 617,000
15. 松本 602,000
16. 井上 591,000
17. 木村 552,000
18. 林 524,000
19. 斎藤 520,000
20. 清水 513,000

**Meiji Yasuda, boys' top 10 by birth year [C]** ("=" marks a tie for that rank)

| Year | 1 | 2 | 3 | 4 | 5 | 6–10 |
|---|---|---|---|---|---|---|
| 1995 | 拓也 | 健太 | 翔太 | 翼 | 大樹 | 大貴, 翔, 亮太, 拓哉, 雄大 |
| 1996 | 翔太 | 健太 | 大輝 | 翼 | 大樹 | 拓海; 7= 直人/達也; 9 翔; 10= 康平/雄大/亮太 |
| 1997 | 翔太 | 翔 | 健太 | 大輝 | 陸 | 拓海, 大地, 大樹; 9= 翼/駿 |
| 1998 | 大輝 | 海斗 | 翔 | 翔太 | 5= 大地/大樹/拓海 | 8 一輝, 9 涼太, 10= 匠/智也 |
| 1999 | 大輝 | 拓海 | 海斗 | 4= 大輔/陸/翔 | – | 7 大樹, 8 翔太, 9= 健太/蓮 |
| 2000 | 翔 | 翔太 | 大輝 | 4= 優斗/拓海 | – | 6 海斗, 7 竜也, 8= 陸/蓮, 10= 一輝/健太/竜 |
| 2001 | 大輝 | 翔 | 海斗 | 陸 | 5= 蓮/翼 | 7= 健太/拓海, 9= 優太/翔太 |
| 2002 | 駿 | 2= 拓海/翔 | – | 蓮 | 5= 翔太/颯太 | 7 海斗, 8 健太, 9 大輝, 10= 大樹/優 |
| 2003 | 大輝 | 翔 | 3= 大翔/翔太 | – | 匠 | 6= 太陽/拓海, 8 蓮, 9 悠斗, 10= 海斗/翼 |
| 2004 | 蓮 | 颯太 | 3= 翔太/拓海 | – | 大翔 | 6 颯, 7= 翔/優斗/陸, 10 翼 |
| 2005 | 1= 翔/大翔 | – | 拓海 | 翔太 | 颯太 | 6 翼, 7= 海斗/輝, 9= 太陽/大和 |
| 2006 | 陸 | 大翔 | 3= 大輝/蓮 | – | 翼 | 6 悠斗, 7 翔太, 8= 海斗/空/優太/陽斗 |
| 2007 | 大翔 | 蓮 | 大輝 | 翔太 | 5= 悠斗/陸 | 7= 優太/優斗, 9 大和, 10= 健太/悠希/翔 |
| 2008 | 大翔 | 悠斗 | 陽向 | 翔太 | 5= 悠人/颯太 | 7= 悠太/翔, 9 蓮, 10= 駿/陸 |
| 2009 | 大翔 | 翔 | 3= 瑛太/大和 | – | 蓮 | 6= 悠真/陽斗, 8 悠斗, 9= 颯真/颯太 |

Meiji Yasuda ranks kanji spellings only. A spelling such as 大翔 has several readings (Hiroto, Yamato, Taiga…), so these lists do not give romanizations.

---

## 6. Conflicts between sources

1. **MLB 2026 total:** the first release said 247 (26.1%, Canada 17). The corrected release says 249 (26.3%, Canada 19) after two Canadian-born players were added. AP's headline said "lowest since 2002"; the corrected text says "lowest since 2014."
2. **MLB 2025 pool:** 954 total, but the stated parts (779 active + 173 inactive) add up to 952.
3. **MLB country lists versus totals:** MLB's country lists leave out "Miscellaneous" birthplaces, so they sum to 2 or 3 less than the total in both years. By pure birthplace, Japan is 13 (2025) and 15 (2026).
4. **MLB Stats API versus MLB's list:** the API's `birthCountry` disagrees with MLB's official list by a few players (for example, a Germany-born and an Italy-born player not in MLB's 2026 list).
5. **GMU fact sheets:** they say "81% of MiLB players were born in the DR, Venezuela or Mexico." It should read "of foreign-born MiLB players" (42.3 + 33.6 + 5.0). The 2025 rookie-level n is misprinted as "16,62."
6. **Baseball America (snippets):** the same 2,392 US-born count appears in both the 2017 and 2019 breakdowns, which is probably a copying error in one of them.
7. **Mexico surname counts:** the INE-based figures differ between the 2016 and 2018 press reports (Hernández 4.88M versus about 3.53M paternal), probably because of different bases.
8. **Dominican surnames:** the En Segundos list is credited to the JCE but also said to come from Forebears.
9. **SSA decade tables:** SSA revises its counts, so versions differ slightly (e.g., Jacob, 2000s: 274,064 versus 274,077).
10. **NPB 2026 five-player rule:** blogs say it continues; Japanese Wikipedia documents it only through 2025; I found no official NPB statement.

## 7. Open gaps

- **SSA boys born 1995 to 2009:** yearly sums not computed because ssa.gov blocks this environment. Download names.zip and sum yob1995 to yob2009 for sex = M.
- **Official frequency data:**
  - Dominican Republic, Venezuela and Cuba: none found for surnames or given names.
  - Mexico: none found for the 1995 to 2009 cohort's given names.
  - Forebears (a compilation) was unreachable (HTTP 502), so its estimates were not used.
- **Japan:** Meiji Yasuda gives ranks only, without counts or readings, for 1995 to 2009.
- **Taiwan:** per-surname counts beyond 陳 and 林 and counts by birth cohort are only in the MOI book, which was not accessed.
- **Venezuela:** the academic surname study's frequency tables were not seen.
- **Minor leagues:** no recent official MLB or Baseball America percentage of foreign-born minor leaguers was found; the BA pages were blocked. My Stats API counts use official data, but they are my computation.
- **MLB signings:** no official count of international signings by country.
- **KBO:**
  - The exact Opening Day (2026-03-28) composition of the 30 regular slots is unverified for 2 clubs that changed players before the season.
  - The 2027 foreign-player rules are undecided as of 2026-11-16.
  - The formal position rule for the 3 foreign players was not found.
- **NPB:**
  - Farm and 育成 foreign players are not covered.
  - 3 players with non-Japanese names and no Baseball-Reference birthplace were not classified.
  - The 2026 edition of the NPB Agreement and NPB's 2026 special rules were not found online.

## 8. Files used for the computations

Scratchpad folder: `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad`

Scripts:
- `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad/scripts/milb_birth.py` (MLB Stats API counts)
- `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad/scripts/unique_full.py` (unique full-season counts)
- `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad/scripts/kbo_count.py` (KBO)
- `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad/scripts/npb_count.py` (NPB)
- `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad/scripts/br_roster.py` (Baseball-Reference roster parser)

Data:
- Raw API and roster downloads: `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad/research/dl/`
- Aggregate JSON: `/tmp/claude-0/-home-user-baseball-manager/0e8ba671-bc14-5a22-88e9-9f31be84e757/scratchpad/research/dl/statsapi/summary_*.json`

Only counts by country are reported here; no player names.
