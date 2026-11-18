# Contract options, opt-outs and option years (MLB, KBO, NPB): research notes (state on 2026-11-18)

Research date: 2026-11-18. Purpose: sourced, quantitative facts for the contract system: club options, player options, mutual options, opt-outs and option years ("+1", "+2"), plus how often each kind appears and how often it is used.

**Rules.** Every number comes from a source I read. The URL and a tag sit next to it. **NOT CONFIRMED** means I looked and found nothing usable. Nothing is estimated or filled in from memory. Where sources disagree, both versions are given and the conflict is listed again in section 6.

**Tags.**
- [CBA]: agreement text.
- [official]: league or league-owned sources: MLB.com, KBO releases, NPB.
- [analysis]: MLB Trade Rumors (MLBTR), FanGraphs.
- [press]: news outlets: AP, ESPN, Korean and Japanese papers.
- [data]: a count I made myself from a public list. The method is given each time.
- [peer-reviewed]: none found on these questions.

**Not repeated here.**
- The CBA rules on options (CBT treatment, Quiet Period dates, the ban on voidable years) are in `docs/research/world/contracts_mlb.md` §7.
- The KBO 2024-25 and 2025-26 FA signing tables are in `contracts_kbo.md` §9.2. The 김재환 release clause and the 홍건희 opt-out are in §4.7.
- NPB multi-year basics and the 千賀/武田 variable deals are in `contracts_npb.md` §3E.
- Market totals are in `contracts_markets.md`.
- This file only adds counts and cases on top of those files.

**Main sources (short names used below)**

| Short name | What it is | URL |
|---|---|---|
| MLB.com 2022 | Preview of 2022 options. Thomas Harrigan, 16 Oct 2021. It has **no** decision updates. | https://www.mlb.com/news/players-with-2022-options-and-opt-outs |
| MLB.com 2023 | "Here's how the 2023 option decisions played out". Harrigan, 11 Nov 2022 | https://www.mlb.com/news/mlb-players-with-2023-options-opt-outs |
| MLB.com 2024 | "How each team and player option decision played out". Harrigan, 7 Nov 2023 | https://www.mlb.com/news/mlb-players-with-2024-options-opt-outs |
| MLB.com 2025 | Same title. Harrigan, 5 Nov 2024 | https://www.mlb.com/news/mlb-players-with-2025-options-opt-outs |
| MLB.com 2026 | "Who opted out? Every 2026 contract option decision". Harrigan, 7 Nov 2025. Its option figures come from Cot's Baseball Contracts. | https://www.mlb.com/news/mlb-players-with-2026-options-opt-outs |
| MLB.com 2027 | "Who opted out? Every 2027 option decision". Harrigan, 3 Nov 2026 | https://www.mlb.com/news/mlb-players-with-2027-options-and-opt-outs |
| MLBTR OIR | MLBTR "Offseason In Review", one post per club per offseason. Each lists every **Major League free-agent signing** with terms in brackets (options, buyouts, opt-outs) and usually an "Option Decisions" list. 150 posts, 2021-22 to 2025-26, found from the category pages. | e.g. https://www.mlbtraderumors.com/2025-26-offseason-in-review ; https://www.mlbtraderumors.com/2026/02/offseason-in-review-detroit-tigers-16.html |
| MLBTR Nov 2021 | MLBTR posts of November 2021 on single option decisions. I read monthly archive pages 30-48 at https://www.mlbtraderumors.com/2021/11/page/30/ … /48/ . Single posts are cited by URL below. | |
| AP Nov 2021 | AP reports on 2021 option decisions, as carried by ABC stations. | https://abc7ny.com/post/joc-pederson-kyle-schwarber-among-10-mlb-players-to-become-free-ag/11200279/ ; https://abc7.com/post/trevor-bauer-nolan-arenado-charlie-blackmon-decline-opt-outs;-mic/11208568/ |
| Jaffe 2020 | Jay Jaffe, "Declined Options Reflect Chilly Market and Short Season Struggles", FanGraphs, 5 Nov 2020 | https://blogs.fangraphs.com/declined-options-reflect-chilly-market-and-short-season-struggles/ |

---

## 0. Key numbers at a glance

| Item | Value | Source |
|---|---|---|
| MLB option and opt-out decisions per offseason | 52 / 58 / 63 / 68 / 69 / 65 (2021-22 to 2026-27) | [data] §1.2 |
| MLB club options exercised | **39%** (75 of 194 decided, 2021-22 to 2026-27). Each season: 34-42% | [data] §1.2-1.3 |
| Club options by age (no arbitration years left) | Exercised 54% at ages 29-31; 25% at 32-34; 31% at 35-37 | [data] §1.4 |
| MLB player options exercised (player stays) | **50%** (17 of 34) | [data] §1.3 |
| MLB opt-outs used | **41%** (23 of 56) | [data] §1.3 |
| MLB mutual options exercised by both sides | **0 of 84** (2021-22 to 2026-27) | [data] §3 |
| Who declines mutual options | The club in 30 of 53 cases where the side was reported (57%); the player in 22 (42%); both in 1 | [data] §3 |
| Mutual options exercised by both, earlier | 4 of 100 over about two decades (ESPN 2017); four since 2008 (FanGraphs 2012) | [press], [analysis] §3 |
| Club-option buyout | No buyout in 29% (57 of 196). Where there is one, median **12%** of the option salary (IQR 7-19%) | [data] §2.5 |
| Mutual-option buyout | Median 19% of the option price (n=41) | [data] §2.5 |
| New MLB free-agent contracts, multi-year deals (n=228) | Opt-out ≥31%; club option 17%; mutual option 7%; player option 4%; any of these ≥54% | [data] §2.2 |
| New MLB free-agent contracts, one-year deals (n=520) | Club option 11%; mutual option 9%; any option 21% | [data] §2.2 |
| Where MLB opt-outs fall | First opt-out after year 1 in 46 of 71 deals (65%); after year 2 in 13 (18%). Every 2-year deal with an opt-out has it after year 1 | [data] §2.4 |
| KBO FA deals with option years ("+N") | 5 of 19 (2024-25); 2 of 20 (2025-26) | [data] from contracts_kbo §9.2, §4.2 |
| KBO FA deals with incentive money stated | At least 9 of 19 and 11 of 20. Where the split is reported, option money is 18% and 12% of the maximum totals | [data] §4.2 |
| How KBO option years trigger | Mostly performance conditions that extend the deal automatically. Also player options and mutual options | [press] §4.3 |
| NPB | Multi-year and 変動制 (variable-salary) deals are documented case by case. **How often they are used is NOT CONFIRMED** | §5 |

---

## 1. MLB: option decisions each offseason (question 1)

### 1.1 Method [data]

- **Unit.** One decision on one contract provision in the five days after the World Series.
- **2022-23 to 2026-27.** All entries come from the MLB.com post-decision lists (MLB.com 2023 … 2027) [official]. Each list is sorted into opt-outs, player options, club options and mutual options. Each entry shows the option salary, the buyout and an "UPDATE" with the outcome.
  - I parsed every entry: 274 players.
  - **Age** = age on 30 June of the season the option covers. Birth dates come from the MLB Stats API, `https://statsapi.mlb.com/api/v1/people/search?names=…` [official]. Six names had two matches (e.g. Max Muncy, Josh Bell, Eduardo Rodriguez); each was resolved by debut year and position.
- **2021-22.** MLB.com 2022 is only a preview, so outcomes were taken player by player from these sources:
  - MLBTR Nov 2021 posts [analysis], for example
    - https://www.mlbtraderumors.com/2021/11/nick-castellanos-reds-opts-out-contract-free-agent.html
    - https://www.mlbtraderumors.com/2021/11/guardians-exercise-option-jose-ramirez-decline-roberto-perez.html
    - https://www.mlbtraderumors.com/2021/11/giants-exercise-club-options-on-wilmer-flores-jose-alvarez-jay-jackson.html
    - https://www.mlbtraderumors.com/2021/11/yusei-kikuchi-to-decline-player-option-test-free-agency.html
    - https://www.mlbtraderumors.com/2021/11/cardinals-decline-options-matt-carpenter-carlos-martinez.html
    - https://www.mlbtraderumors.com/2021/11/rays-pick-up-option-on-mike-zunino.html
  - AP Nov 2021 [press].
  - MLBTR OIR 2021-22 "Option Decisions" lists [analysis].
  - Kyle Seager: the only source is MLBTR's "with the club set to buy out longtime third baseman Kyle Seager" (Nov 2021 archive).
- **Decisions added from MLBTR OIR that are missing from the MLB.com lists:**
  - 2021-22: Jay Jackson, club option, exercised.
  - 2022-23: Jackie Bradley Jr., $12MM mutual option, declined by the club.
  - 2023-24: Miguel Cabrera, $30MM club option, declined; Jon Berti, $3.625MM club option, exercised.
  - 2025-26: Randy Dobnak, $6MM club option, declined.
- **Classification.**
  - "Exercised" includes options picked up by a club that had just acquired the player: Barnhart (Tigers) and Miley (Cubs) in 2021-22.
  - "Other" covers a new contract that replaced the option (Muncy, Wacha, Bummer, Suter, Rea, Webb) and Posey (retired before the decision).
  - Kikuchi (2021-22) and Imanaga (2025-26): the club declined a multi-year club option and the player then declined the player option that followed. Each counts as one club option declined.
  - Multi-year club options (Kikuchi, Green, N. Martinez, Wacha, Imanaga) are converted to an annual value for the salary bands.
- **Context, 2021 [press].** "There have been 170 players who became free agents following the World Series, and about 40 players await decisions on options by Sunday" (AP Nov 2021, abc7ny link above).

### 1.2 Decisions by offseason [data]

| Offseason | Club options | exercised | declined | other | Player options | exercised (stayed) | declined (left) | other | Opt-outs | opted out | stayed | Mutual options | exercised by both |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2021-22 | 32 | 13 | 18 | 1 | 7 | 4 | 3 | 0 | 4 | 1 | 3 | 9 | 0 |
| 2022-23 | 26 | 10 | 16 | 0 | 8 | 3 | 5 | 0 | 11 | 7 | 4 | 13 | 0 |
| 2023-24 | 39 | 13 | 25 | 1 | 6 | 2 | 4 | 0 | 10 | 4 | 6 | 8 | 0 |
| 2024-25 | 36 | 13 | 21 | 2 | 7 | 4 | 2 | 1 | 11 | 4 | 7 | 14 | 0 |
| 2025-26 | 35 | 14 | 20 | 1 | 2 | 1 | 1 | 0 | 14 | 6 | 8 | 18 | 0 |
| 2026-27 | 32 | 12 | 19 | 1 | 5 | 3 | 2 | 0 | 6 | 1 | 5 | 22 | 0 |
| **2021-22 to 2025-26** | **168** | **63** | **100** | 5 | **30** | **14** | **15** | 1 | **50** | **22** | **28** | **62** | **0** |
| **2021-22 to 2026-27** | **200** | **75** | **119** | 6 | **35** | **17** | **17** | 1 | **56** | **23** | **33** | **84** | **0** |

- In MLB.com's lists, "player options" and "opt-outs" are the same right with different wording. MLB.com merges them into one section from 2026 on. MLBTR says the same: "sometimes referred to as opt-outs and sometimes as player options. Despite the different terminology…" (D. McDonald, 10 Apr 2026, https://www.mlbtraderumors.com/2026/04/2026-27-player-option-opt-out-preview-april-edition.html) [analysis].
- Combined, a player holding either kind of exit left in **40 of 90** decisions (44%) [data].
- **Who opted out** (age in brackets) [data from the official lists]:
  - 2021-22: Castellanos (30).
  - 2022-23: Bogaerts (30), Correa (28), deGrom (35), Rizzo (33), Rodón (30), N. Martinez (32), Profar (30).
  - 2023-24: Manaea (32), E. Rodriguez (31), Soler (32), Stroman (33).
  - 2024-25: Cole (34; came back on his existing deal), Manaea (33), N. Martinez (34), Snell (32).
  - 2025-26: Alonso (31), Bellinger (30), Bregman (32), E. Díaz (32), H.-S. Kim (30), R. Suarez (35).
  - 2026-27: M. King (32).

### 1.3 Exercise rates by type [data]

| Type | 2021-22 to 2025-26 | 2021-22 to 2026-27 | By season |
|---|---|---|---|
| Club option exercised (of exercised + declined) | 63/163 = **39%** | 75/194 = **39%** | 42%, 38%, 34%, 38%, 41%, 39% |
| Club option exercised, players with no arbitration years left | 58/150 = 39% | 67/176 = 38% | - |
| Club option exercised, player still arbitration-eligible (as noted by MLB.com) | 5/13 | 8/18 | - |
| Player option exercised (player stays) | 14/29 = **48%** | 17/34 = **50%** | - |
| Opt-out used | 22/50 = **44%** | 23/56 = **41%** | 25%, 64%, 40%, 36%, 43%, 17% |
| Mutual option exercised by both sides | 0/62 | 0/84 | 0 every season |

- MLB.com marks "arbitration eligible through …" only from its 2025 list on. Earlier arbitration-eligible club options are mixed into the "no arbitration years left" row.
  - When a club declines such an option the player is not a free agent. Example: Duran, Contreras (2025-26).
- **Earlier baseline** (Jaffe 2020 [analysis]). Players with ≥6 years of service, club and mutual options together:
  - 2019-20: 10 of 43 exercised. 2020-21: 6 of 40.
  - By size, 2019: >$10M 5 of 18; $5-9.9M 5 of 20; <$5M 0 of 8. 2020: 3 of 21, 1 of 7, 2 of 12.
  - Options of $5M or more were exercised at 26.3% (2019) and 14.3% (2020).
  - My closest comparable figure: club plus mutual options, no arbitration years left, 2021-22 to 2025-26 = **58/211 = 27%** [data].
  - It is not fully comparable: Jaffe left out mutual options declined by players.

### 1.4 Club options by size, buyout and age [data]

Pooled 2021-22 to 2026-27; club options decided either way, players with no arbitration years left (n=176).

| Option salary (annual) | Exercised |
|---|---|
| under $5M | 17/39 (44%) |
| $5M to under $10M | 23/65 (35%) |
| $10M to under $15M | 14/35 (40%) |
| $15M or more | 12/36 (33%) |

| Buyout as a share of the option salary | Exercised |
|---|---|
| no buyout | 18/48 (38%) |
| under 10% | 20/47 (43%) |
| 10% to under 20% | 17/50 (34%) |
| 20% or more | 10/29 (34%) |

| Age on 30 June of the option season | Exercised |
|---|---|
| 28 or younger | 3/5 |
| 29-31 | 31/57 (**54%**) |
| 32-34 | 14/55 (**25%**) |
| 35-37 | 15/48 (31%) |
| 38+ | 4/11 |

- Option size and buyout share barely separate exercised from declined options.
  - Median buyout share for players with no arbitration years left: 8.0% exercised (n=65), 9.1% declined (n=109).
  - All club options with a known buyout: 7.1% (n=72) and 8.3% (n=118).
- Age separates them clearly: after 31 the rate roughly halves.
- Largest decisions, for scale: Cabrera $30M (declined, $8M buyout); Moncada $25M (declined, $5M); Cueto $22M (declined, $5M); Alcantara $21M (exercised); Rasmussen $20M (exercised; it had escalated from $8M); Robert Jr. $20M (exercised 2025-26, declined 2026-27). All from the MLB.com lists [official], Cabrera from MLBTR OIR.

### 1.5 Player options [data]

| Group | Exercised (player stays) |
|---|---|
| Age 31 or younger | 2/3 |
| Age 32-34 | 11/17 (65%) |
| Age 35+ | 4/14 (29%) |
| Option salary under $10M | 10/20 |
| $10M to under $20M | 4/9 |
| $20M or more | 3/5 |

- Older players decline more often. At 35+ several declined in order to retire or to re-sign for more years: Verlander 2022-23, Eovaldi and Kershaw 2024-25 (MLB.com 2023 and 2025 [official]).

### 1.6 Opt-outs [data]

| Group | Opted out |
|---|---|
| Age 29 or younger | 1/4 |
| Age 30-32 | 15/30 (50%) |
| Age 33+ | 7/22 (32%) |
| 1 year left when deciding | 13/28 (46%) |
| 2 years left | 6/18 (33%) |
| 3 years left | 3/5 |
| 4-6 years left | 1/5 |

- Some opt-outs can be overridden by the club:
  - Cole 2024-25: the Yankees could void his opt-out by adding a year at $36M (MLB.com 2025).
  - Pederson 2025-26 and Story 2025-26: the club could void the opt-out with a club option (MLB.com 2026).

### 1.7 What the decision lists do not capture

- **Options of players released during the season.** A player released in-season loses the option (club options are bought out), so his option never reaches a decision list.
  - Test: 109 options (club, mutual, player or vesting) were attached to one-year free-agent deals signed 2021-22 to 2025-26. **83 (76%)** appear in the MLB.com list the following November [data, MLBTR OIR against MLB.com; for 2021-22 signings, MLB.com 2023].
  - By signing offseason: 13/15, 11/16, 17/25, 22/25, 20/28.
  - The other 26 include Kimbrel, Maldonado, Myers, Ureña, Kinley, Lorenzen, Ozuna and others. Why each one is missing is **NOT CONFIRMED** player by player.
- **Vesting options that vested.** They are not "decisions". Example: Maldonado's $4.5MM option vested in 2022-23 (MLBTR OIR Astros 2022-23) and is not in the MLB.com list.

---

## 2. MLB: how often new contracts contain options (question 2)

### 2.1 Method [data]

- **Population.** Every item in the "Major League Signings" or "Major League Free Agent Signings" list of all 150 MLBTR OIR posts (30 clubs × 5 offseasons, 2021-22 to 2025-26) [analysis].
- **Size.** 748 MLB contracts after dropping spending-total lines: 520 one-year (split and non-guaranteed deals counted as one year) and 228 multi-year.
- **Included.** Accepted qualifying offers and posted NPB/KBO players.
- **Not included.** Minor-league deals. Deals signed after a post was published (e.g. Montgomery, March 2024).
- **Flags.** Each item's bracketed terms were searched for club or team option, player option, mutual option, vesting, opt-out. Then every one of the 227 items that mention an option or opt-out was read by hand and 11 were corrected.
  - Bellinger 2023-24 "2025 and '26 seasons are player options" counts as an opt-out.
  - Club/vesting and conditional club/player options count as club options.
  - Means 2024-25 ("club has $6MM option") counts as a club option.
- **Known undercount.** OIR does not always mention an opt-out. I added the cases proven by other sources:
  - 2021-22: Correa, Rodón, Soler, Stroman. The MLB.com 2023 and 2024 lists show them as opt-out decisions on those very deals.
  - 2025-26: Kyle Tucker, 4 years / $240MM, "opt-outs after the 2027 and 2028 seasons" (MLBTR, https://www.mlbtraderumors.com/kyle-tucker).
  - Shares below are therefore **lower bounds**, the 2021-22 ones most of all (those OIR lists give the fewest details).

### 2.2 Shares by offseason [data]

| Offseason | One-year deals | with club option | with mutual option | with any option | Multi-year deals | with opt-out | opt-out or player option | with club option | with mutual option | with any option or opt-out |
|---|---|---|---|---|---|---|---|---|---|---|
| 2021-22 | 98 | 12 (12%) | 2 (2%) | 15 (15%) | 51 | 11 (22%)* | 12 (24%)* | 11 (22%) | 0 | 22 (43%)* |
| 2022-23 | 87 | 9 (10%) | 6 (7%) | 16 (18%) | 57 | 17 (30%) | 23 (40%) | 9 (16%) | 1 (2%) | 28 (49%) |
| 2023-24 | 103 | 16 (16%) | 8 (8%) | 25 (24%) | 42 | 19 (45%) | 21 (50%) | 7 (17%) | 5 (12%) | 29 (69%) |
| 2024-25 | 115 | 12 (10%) | 13 (11%) | 25 (22%) | 38 | 15 (39%) | 15 (39%) | 7 (18%) | 4 (11%) | 24 (63%) |
| 2025-26 | 117 | 9 (8%) | 19 (16%) | 28 (24%) | 40 | 9 (23%)* | 9 (23%)* | 5 (12%) | 7 (18%) | 20 (50%)* |
| **All** | **520** | **58 (11%)** | **48 (9%)** | **109 (21%)** | **228** | **71 (31%)** | **80 (35%)** | **39 (17%)** | **17 (7%)** | **123 (54%)** |

\* Includes the corrections in 2.1.

- Other clauses on multi-year deals: player options (not opt-outs) 10 (4%); vesting options 6 (3%).
- One-year deals with a player option: 4 of 520. With a vesting option: 3.
- Mutual options attached to one-year deals rose from 2 of 98 (2021-22) to 19 of 117 (2025-26).

### 2.3 By length and guarantee, multi-year deals, pooled [data]

| Group | n | Opt-out | Club option | Mutual option | Player option | Any |
|---|---|---|---|---|---|---|
| 2 years | 114 | 35 (31%) | 17 (15%) | 12 (11%) | 5 (4%) | 67 (59%) |
| 3 years | 49 | 16 (33%) | 10 (20%) | 2 (4%) | 2 (4%) | 25 (51%) |
| 4-5 years | 43 | 14 (33%) | 11 (26%) | 3 (7%) | 3 (7%) | 24 (56%) |
| 6+ years | 22 | 6 (27%) | 1 (5%) | 0 | 0 | 7 (32%) |
| Guarantee under $20M | 68 | 14 (21%) | 13 (19%) | 10 (15%) | 2 (3%) | 36 (53%) |
| $20M to under $100M | 120 | 43 (36%) | 22 (18%) | 5 (4%) | 7 (6%) | 69 (58%) |
| $100M or more | 38 | 14 (37%) | 4 (11%) | 2 (5%) | 1 (3%) | 18 (47%) |

- Opt-outs are most common in the $20M to $100M+ range. Small multi-year deals more often carry a club or mutual option instead.
- One-year deals by guarantee: any option in 19/163 (12%, under $2M), 37/146 (25%, $2-4.99M), 35/111 (32%, $5-9.99M) and 17/72 (24%, $10M+).

### 2.4 Where opt-outs fall in new contracts [data]

The 71 multi-year deals with an opt-out are counted by contract year of the first opt-out. Years are as written in the OIR item; the first season is the second calendar year of the offseason.

| Deal length | After year 1 | After year 2 | After year 3 | After year 4 | After year 5 | Not stated |
|---|---|---|---|---|---|---|
| 2 years (n=35) | 35 | - | - | - | - | - |
| 3 years (n=16) | 9 | 7 | - | - | - | - |
| 4-5 years (n=14) | 2 | 4 | 7 | 1 | - | - |
| 6+ years (n=6) | - | 2 (Burnes, Báez) | - | 2 (J.H. Lee, Story) | 1 (Soto, 15 years) | 1 (Yamamoto, "two opt-outs") |
| **All (n=71)** | **46 (65%)** | **13 (18%)** | **7 (10%)** | **3 (4%)** | **1** | **1** |

- **More than one opt-out** in 15 of 71 deals. Examples: Bichette, "opt-outs after each season" (3 years); Peralta, after 2024, '25 and '26 (4 years); N. Martínez 2021-22, after each of 2022-24.
- **Conditional opt-outs:**
  - Smyly (100 IP).
  - Mancini and Conforto (350 PA).
  - Matsui ("conditional opt-outs").
  - Pivetta ("conditional injury protection for the team"; MLB.com 2027 shows his 2027 player option turned into a club option after time on the IL).
- **Club can override or void the opt-out:** Story, Soto (+$40MM), Santander, Pederson (mutual option), Y. Rodríguez (club option).

### 2.5 Buyout size relative to the option salary [data]

| Sample | n | No buyout | Median buyout ÷ option salary (where there is a buyout) | Spread |
|---|---|---|---|---|
| Club options in the MLB.com decision lists, 2021-22 to 2026-27 (salary and buyout known) | 196 | 57 (29%) | **12.1%** | IQR 7.1-18.8%; 30 of 139 at ≥20%; 3 at ≥50% (L. Hendriks $15M on $15M; Acuña $10M on $17M; Albies $4M on $7M) |
| Club options with both figures in MLBTR OIR "Option Decisions" | 25 | - | 11.7% | IQR 7.2-17.8% |
| Mutual options with both figures in MLBTR OIR "Option Decisions" | 41 | - | **18.8%** | IQR 9.8-26.8%; max 67% |
| New one-year and multi-year deals in OIR stating "$X buyout on $Y mutual option" | 23 | 0 | 14.3% | 5-29% |
| Same, club options | 12 | 1 | 16.7% | 0-35% |

- **Bands, MLB.com club options** (annual option value). Share with a buyout and median ratio: 30/50 and 10.1% (option under $5M); 47/68 and 13.3% ($5-10M); 32/40 and 10.0% ($10-15M); 30/38 and 12.6% ($15M+).
- **Mean buyout** where one exists: $1.50M.
- **Why mutual-option buyouts are larger** [analysis]: "They're accounting measures, essentially an unofficial deferral within the term of the contract itself. The player agrees to push back a percentage of the guaranteed money to the end of the deal in the form of an option buyout." (A. Franco, MLBTR, 23 Apr 2026, https://www.mlbtraderumors.com/2026/04/2026-27-club-options-al-east.html).

### 2.6 Extensions (non-free-agent multi-year deals) [data]

- MLBTR OIR "Extensions" lists, 2021-22 to 2025-26: **78** multi-year extensions.
  - A club option is mentioned in **37 (47%)**: 38 matches less one false hit (Salvador Perez, whose extension *replaced* a club option).
  - Vesting options 2; opt-outs 2; player option 1; mutual options 0.
- Examples [analysis]: Soderstrom, 7 years / $86MM "includes $2MM buyout of $27MM club option for 2033"; Luzardo, 5 years / $135MM "includes $32.5MM club option for 2032" (OIR 2025-26).

### 2.7 Published figures

- **NOT CONFIRMED**: I found no published share of free-agent or multi-year contracts that carry options or opt-outs. The searches covered FanGraphs, Baseball Prospectus, MLBTR and the press.
- FanGraphs' free-agent rankings note that options, opt-outs and incentives are left out of their estimates (search result only; page not opened).
- Sources I could not use:
  - Spotrac's option tracker returned HTTP 402/403.
  - Cot's Baseball Contracts sits behind an anti-bot check. I did not try to get around it.
  - MLBTR's Contract Tracker, which has an "Options" filter, shows only 5 rows without a paid login. I did not log in.

---

## 3. Mutual options: how often both sides say yes (question 3)

| Finding | Value | Source |
|---|---|---|
| Mutual options decided, 2021-22 to 2026-27 | **84** (9 / 13 / 8 / 14 / 18 / 22) | [data] §1.2 |
| Exercised by both sides | **0** | [official] MLB.com 2023 ("none were exercised"), 2024, 2025 ("were declined, either by the player or the team"), 2026 ("none were exercised"), 2027 (each marked "declined"); 2021-22 from MLBTR Nov 2021 and AP |
| Side that declined (53 cases where reported) | club 30 (57%), player 22 (42%), both 1 (Merrifield 2023-24, "Both sides declined $18MM mutual option") | [data] MLBTR OIR "Option Decisions" 2021-22 to 2025-26, plus MLBTR Nov 2021 for Duvall, Pederson, Schwarber and Marisnick |
| One side exercised, the other declined | Chafin 2021-22: "mutual option was exercised by the team and declined by the player" | [press] AP Nov 2021 |
| ESPN study | "We went through a list of 100 mutual options over the past two decades… Only four of the 100 on our list were exercised by both sides" (96% not exercised). Sam Miller, ESPN, 24 Jan 2017. Read through a fetch tool's extract; the direct download failed. | [press] https://www.espn.com/mlb/story/_/id/18519226/show-mind-games-mutual-options-not-money |
| FanGraphs list | "I've only identified four fully-exercised mutual options dating back to 2008": D. Ward (Cubs, 2008, $1.2M), M. Olivo (Royals, 2009, $2.7M), B. Moehler (Astros, 2010, $3M), J. Giambi (Rockies, 2012, $1M). Wendy Thurm, 19 Dec 2012 | [analysis] https://blogs.fangraphs.com/what-are-mutual-options-anyway/ |
| League wording | "Mutual options become guaranteed only when both the player and the team agree to exercise the option, which rarely happens." | [official] MLB.com 2025, 2026, 2027 |
| MLBTR wording | "The mutual options are almost certain to be declined by either the player or team (usually the latter)." | [analysis] Franco 2026-04-23 (URL in 2.5) |

- **Who declines.**
  - Clubs decline after poor or injured seasons. Examples: Hoskins, Quintana, Garver (2025-26).
  - Players decline after good ones and test the market. Examples: King, Woodruff, Bader (2025-26).
  - Woodruff declined a $20MM mutual option, took the $10MM buyout and then accepted the Brewers' qualifying offer (OIR Brewers 2025-26) [analysis].
- **For the model.** An exercise rate of 0-4% fits every source.
- **CBT treatment.**
  - The 2022-26 Basic Agreement treats a mutual option as a club option (`contracts_mlb.md` §7) [CBA].
  - Thurm (2012) says that under the agreement of that time mutual options were treated like player options for the tax.
  - These are different agreements, so the rule changed rather than the sources conflicting.

---

## 4. KBO: option years and incentives (question 4)

### 4.1 Vocabulary and form

| Fact | Source |
|---|---|
| Options are written into the "특약 기재란" (special-terms box) at the end of the KBO player contract form | [press] 류선규 (former SK GM), StarNews, 22 Dec 2025, https://www.starnewskorea.com/sports/2025/12/22/2025122210234562998 |
| There are two kinds: **기간 옵션** (term option, the "+1" years) and **성적(성과) 옵션** (performance money). Performance options are either **출장 옵션** (games, PA, innings) or **기록 옵션** (hits, HR, RBI, wins, ERA); players prefer appearance options | same |
| "In ordinary salary contracts options are rare, but in FA contracts options are close to routine" (FA 계약에서 옵션은 일상에 가깝다) | same |
| "The details of performance options are mostly undisclosed"; unlike MLB they are almost never published in the KBO | same |
| First term option in the KBO: SK and FA 박경완, **3+1 years**, up to 23억, December 2002 | same |
| KBO uses "옵션" for incentive money as well as for option years. FA signings must announce salary, signing bonus, options and length | [official] KBO N12242 attachment 2, via `contracts_kbo.md` §4.1 |
| Players sometimes prefer guaranteed years. In the 2025-26 market one player took 2 years / 26억 over his old club's **1+1** years / 30억 (estimated). Another took 2 years / 22억 over **2+1** / 30억 (estimated). Players not named | [press] StarNews 22 Dec 2025 |

### 4.2 How common: FA deals [data]

Counted from the two FA signing tables in `contracts_kbo.md` §9.2. Each row cites a press source there; I did not re-check them for this file.

| Market | Contracts signed | With option years ("+N") | Stated incentive/option money | Stated fully guaranteed | No breakdown reported | Option money ÷ max total (deals with a breakdown) |
|---|---|---|---|---|---|---|
| 2024-25 | 19 | **5 (26%)**: 구승민 2+2, 노경은 2+1, 김강률 3+1, 서건창 1+1, 이용찬 2+1 | ≥9 (47%) | 2 (최정, 장현식) | 8 (incl. 김강률 "salary up to 9억") | 51.7 of 281.1억 = **18.4%** over 8 deals (11.7% if the 2 fully guaranteed deals are added) |
| 2025-26 | 20 | **2 (10%)**: 양현종 2+1, 김태훈 3+1 | ≥11 (55%) | 2 (김현수, 손아섭) | 7 | 52 of 437억 = **11.9%** over 10 deals (10.7% with the 2 guaranteed deals) |
| 2026-27 so far (17 Nov 2026) | 2 | 0 | 2 (양의지 5억 incentives; 오태곤 1.6억 options) | - | - | - |

- Per deal, the option share ranged from 2.5% (박찬호, 2 of 80억) to 36% (노경은, 9 of 25억).
- **Official aggregate**: no KBO figure on how many FA or multi-year contracts carry option years or incentives. **NOT CONFIRMED**.

**Non-FA multi-year deals (비FA 다년계약)** [press]

| Player (club, date) | Terms | Option content | Source |
|---|---|---|---|
| 박종훈, 문승원 (SSG, Dec 2021) | 5 years 65억 / 55억 | options 9억 / 8억 | `contracts_kbo.md` §4.3 (Nocut) |
| 김광현 (SSG, 2022) | 4 years 151억 | salary 131억 + options 20억 | Sports Kyunghyang 2025-08-04 https://sports.khan.co.kr/article/202508041657003 |
| 구창모 (NC, 17 Dec 2022) | **6+1 years**, up to 132억 | a 7th option year | fnnews 2022-12-18 https://www.fnnews.com/news/202212181131372673 |
| 구자욱 (Samsung), 고영표 (KT), 구창모 | 5 years 120억; 5 years 107억; "6 years 125억" | "Of these [100억+ deals], only 김광현 and 류현진 pass 100억 on salary alone, without options". So 구자욱, 구창모 and 고영표 rely on option money to reach 100억+ | Sports Kyunghyang 2025-08-04 |
| 최주환 (Kiwoom, 5 Nov 2024) | **2+1+1 years**, up to 12억 | 3억 a year; 2025-26 guaranteed; "옵션 충족 시 다음 시즌 계약이 자동 연장" (each further year extends automatically if the option is met); the condition is described as staying healthy | OSEN 2024-11-05 https://osen.co.kr/article/G1112451407 |
| 송성문 (Kiwoom, 4 Aug 2025) | 6 years 120억, **fully guaranteed**, with a posting clause. Not carried out because he was posted to MLB | none | Sports Kyunghyang 2025-08-04; 2026-07-13 https://sports.khan.co.kr/article/202607131103003 |
| 하영민 (Kiwoom, 13 Jul 2026) | 8 years (2027-34), 80억 "including salary and options"; split not disclosed | options of undisclosed size | same; OSEN 2026-07-14 https://osen.co.kr/article/G1112839052 |

### 4.3 How option years are triggered, with outcomes [press]

| Player and deal | Trigger | Outcome | Source |
|---|---|---|---|
| 김현수, LG, FA Dec 2021: **4+2 years / 115억** (4 years 90억 + 2 years 25억) | The +2 years "extend automatically if the option is met". The test is the **4-year cumulative** record, not the 2025 season | **Not met** (GM 차명석: "채우지 못했다"). Became an FA in 2025-26 and signed with KT (see contracts_kbo §9.2) | OSEN 2025-10-06 https://osen.co.kr/article/G1112670736 |
| 허경민, Doosan, FA 2020-21 | Player option worth 3 years / 20억 | **Declined** after 2024 ("3년 20억 옵션을 거부하고 FA를 신청"); signed with KT for 4 years / up to 40억 | OSEN 2025-10-06 (same) |
| 박민우, NC, FA 23 Nov 2022: **5+3 years / up to 140억** (5 years up to 90억 = signing bonus 35 + salary 35 + options 10; +3 years up to 50억) | "Vesting option" on **years 4 and 5 (2026, 2027)**. The GM chose current form over a contract-long average: "계약이 연장되는 시점에서 얼마나 잘하는지를 보고 싶었다" | Pending (decided after 2027); 2026 result **NOT CONFIRMED** | Ilgan Sports 2022-11-23 https://isplus.com/article/view/isp202211230140 |
| 안치홍, Hanwha, FA 2023-24: **4+2 years / 72억** (4 years 55억 = 47 guaranteed + 8 options; +2 years 17억 = 13 + 4) | The +2 is a **mutual option** ("구단과 선수 모두에게 선택권") | Taken over by Kiwoom (Nov 2025); the mutual option falls after 2027 | OSEN 2025-11-20 https://osen.co.kr/article/G1112699479 |
| 최주환, Kiwoom, non-FA Nov 2024: 2+1+1 | Automatic yearly extension when the option is met | 2027 year: **NOT CONFIRMED** | OSEN 2024-11-05 |
| 홍건희 (Doosan, 2+2) and 양의지 (Doosan, 4+2) | Player opt-out of the option years | Both opted out (홍건희 after 2025; 양의지 Nov 2026, then re-signed for 2 years / 55억) | `contracts_kbo.md` §4.7, §9.2 |

- **Summary.**
  - The common KBO pattern is a **performance condition that extends the contract automatically**: cumulative stats (김현수), late-contract stats (박민우), or health and playing time (최주환).
  - Player options (허경민, 홍건희, 양의지) and mutual options (안치홍) also exist.
  - No KBO case of a pure club decision on a "+1" year turned up in what I read. **NOT CONFIRMED** whether such deals exist.
- **2+1 deals whose option year arrived in 2024-2026, outcomes NOT CONFIRMED**:
  - 서건창 (KIA 1+1, option year 2026), 이재학 (NC 2+1, 2025), 한현희 (Lotte 3+1, 2026), 오선진 (Hanwha 1+1, 2024), 김민성 (2+1, 2026).
  - Option years falling after the 2026 season, no report found by 18 Nov 2026: 노경은 (SSG 2+1), 이용찬 (NC 2+1), 구승민 (Lotte 2+2).
  - A search found 노경은's 2+1 terms (bonus 3 + salary 13 + options 9억) but nothing on the option condition (newspim and Daum results, not opened).
- **Official signal [official].** KBO's FA-eligibility releases list "FA계약 유보선수": players who qualified, including "계약 기간 연장 옵션 발동 등으로" (through extension options being triggered and similar), but whose contract has not ended. There were 11 (2024-25, N10294), 14 (2025-26, N11753) and 9 (2026-27, N12242).
  - The release does not say which players are deferred because of an option. The 2026-27 names in the attachment are mostly ordinary long contracts (e.g. 고영표, 나성범, 박민우).
  - https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11753 (and 12242, 10294)

---

## 5. NPB: options, variable salaries, opt-outs (question 5)

| Fact | Source |
|---|---|
| **Variable salary (年俸変動制)**, SoftBank 柳田悠岐: 7-year deal, Dec 2019. The first 4 years carried on the old contract as "出来高が翌年のベース年俸に加算される減額のない年俸変動制": incentives earned are added to next year's base salary, and there are no cuts. After 4 years the terms of the last 3 years are reviewed | [press] Daily 2019-12-26 https://www.daily.co.jp/baseball/2019/12/26/0012990257.shtml |
| 柳田 signed for ¥610M in year 1 of the "年俸変動制の７年契約". The piece adds that "In recent years SoftBank has signed one registered player after another to multi-year deals": 森 (4 years, ¥460M a year), 中村 and 今宮 (4-year deals), 明石 (2 years) | [press] Nikkan Gendai 2020-12-24 https://www.nikkan-gendai.com/articles/view/sports/283099 |
| SoftBank 周東佑京, Jan 2026: **5 years, ¥2.0bn plus incentives**; ¥400M in year 1; **"４年目以降は変動制"** (variable from year 4) | [press] Daily 2026-01-24 https://www.daily.co.jp/baseball/2026/01/24/0019944407.shtml |
| 千賀滉大 (SoftBank), Dec 2021: 5 years at ¥600M a year (¥3.0bn on paper), "千賀自身が途中で契約を破棄できるオプトアウト条項付き" (with an opt-out he can use part-way through). Seen as "effectively a one-year deal". He then left as an overseas FA and signed with the Mets for 5 years / $75MM (MLBTR OIR 2022-23). This confirms `contracts_npb.md` §3E | [press] Nikkan Gendai 2021-12-07 https://www.nikkan-gendai.com/articles/view/sports/298324 ; [analysis] MLBTR OIR 2022-23 Mets |
| 菅野智之 (Giants), Jan 2021. **Reported offer** of a 4-year contract with an opt-out. One report: "今後３年間は契約を破棄できるオプトアウト条項" (an opt-out in each of the next three years). Another: "シーズン毎のオプトアウト条項" (a per-season opt-out). The terms he actually signed are **NOT CONFIRMED** | [press] Nikkan Gendai 2021-01-04 https://www.nikkan-gendai.com/articles/view/sports/283380 ; BaseballKing https://baseballking.jp/ns/260396/ |
| 村上宗隆 (Yakult): signed a **3-year deal in the 2022 offseason**; 2025 was the last year; posted after 2025. The club president said in June 2025 the club would file the posting if asked. A contractual posting clause is **NOT CONFIRMED** | [press] Daily 2025-09-18 https://www.daily.co.jp/baseball/2025/09/18/0019485957.shtml |

- **How often NPB uses multi-year deals, options or 変動制**: **NOT CONFIRMED**.
  - I found no league, union or press count. Search terms: 複数年契約 人数, 変動制 割合.
  - The JPBPA salary survey publishes averages and medians only (`contracts_npb.md` §6E).
  - Reported NPB terms are estimates (推定) and usually come as "X years ¥Y plus 出来高 (incentives)". In the FA rows of `contracts_npb.md` §6B, 大山 (5 years ¥1.7bn) and 山川 (4 years ¥1.2bn) are reported with incentives.
- **Club and player option years in NPB contracts**: no case was documented in what I read; the forms seen were variable salary, opt-outs and incentives. **NOT CONFIRMED**.

---

## 6. Conflicts between sources

1. **Margot 2024-25.**
   - MLB.com 2025: "$12 million club option ($2 million buyout)", declined.
   - MLBTR OIR Twins 2024-25: "Declined $12MM mutual option on OF Manuel Margot".
   - Counted as a club option in §1. Left out of the mutual "who declined" split.
2. **Melancon 2021-22.** MLB.com 2022: mutual option ($5M, $1M buyout). MLBTR OIR Padres: "declined his end of $5MM mutual option". MLBTR's Padres post: "declined his $5MM player option in favor of a $1MM buyout" (https://www.mlbtraderumors.com/2021/11/padres-announce-several-option-decisions.html). Counted as mutual.
3. **Gardner and O'Day 2021-22.** MLB.com 2022 and MLBTR describe a player option followed by a club option. AP says they "declined mutual options, as did New York". Counted as player options.
4. **2021-22 option amounts.**
   - Carpenter: $18.5M club option, $2M buyout (MLB.com 2022) against "$12 million option" (AP).
   - José Ramírez: $11M (MLB.com) against $12MM (MLBTR, OIR).
   - Zunino: $4M (MLB.com) against $7MM after escalators (MLBTR).
   - R. Pérez: $5.5M (MLB.com) against $7MM (MLBTR).
   - The MLB.com figures are used in §1.4.
5. **Jaffe 2020 internal mismatch.** The text gives 43 options for 2019-20 (10 exercised). The table's size bands add up to 46 (18 + 20 + 8).
6. **MLB.com 2024 list** heads its mutual options "had mutual options for 2023". The page covers 2024 options, so it is read as 2024.
7. **구창모's length and total.** "6+1년 총액 최대 132억" (fnnews 2022-12-18). "6년 125억" (Sports Kyunghyang 2025-08-04). "7년 132억" (OSEN 2026-07-14, Sports Kyunghyang 2026-07-13). These look like the 6-year guarantee against the full 7 years, but that reading is unconfirmed.
8. **菅野's 2021 opt-out.** "Opt-out within the next three years" (Nikkan Gendai) against "per-season opt-out" (BaseballKing). Both describe a reported offer.
9. **Completeness of MLB.com and MLBTR.**
   - MLB.com omits options voided by in-season release and a few others (§1.7).
   - MLBTR OIR omits some opt-outs (Correa, Rodón, Soler, Stroman in 2021-22; Tucker in 2025-26).
   - Both are corrected where another source proves the clause.

---

## 7. Gaps (NOT CONFIRMED)

- **MLB, released players.** How many options each year end through an in-season release. A test suggests about a quarter of one-year-deal options (§1.7).
- **MLB, missing reasons.** Why each of the 26 options missing from the MLB.com lists is missing.
- **MLB, published shares.** Any published share of FA or multi-year contracts with options or opt-outs; only my own counts exist.
- **MLB, unreachable trackers.** Spotrac (HTTP 402/403), Cot's (anti-bot check) and MLBTR's Contract Tracker (paid login) were not used.
- **MLB, late signings.** Signings after the OIR posts (late spring) are missing from §2. Opt-outs in some big deals may still be missing even after the corrections.
- **Mutual options.** Who declined in the 2026-27 cases. The MLB.com 2027 list says "declined" without naming the side, and the 2026-27 OIR posts do not exist yet.
- **Mutual options, last joint exercise.** Of the sources read, the latest named case is Giambi in 2012 (FanGraphs). Nothing later turned up.
- **KBO, results not found.**
  - Outcomes of the option years of 서건창, 이재학, 한현희, 오선진, 김민성 and 주권 (2+2; the option years fall in 2026-27).
  - Outcomes of 노경은, 이용찬, 구승민 and 최주환 after 2026, and 박민우's 2026 vesting progress.
- **KBO, aggregates and splits.**
  - Any official count of option-year deals or of options exercised.
  - 고영표's guaranteed/option split. "95억 guaranteed + 12억 options" appeared only in a search snippet I could not open.
  - Whether any KBO "+1" is a pure club decision.
- **NPB.**
  - How often multi-year, variable-salary and option deals are used.
  - How such deals are written (also open in `contracts_npb.md`).
  - 菅野's signed 2021 terms; 村上's contract wording; 田中将大's 2021-22 Rakuten terms.
  - Any NPB club or player option year.

---

## 8. Working files

Downloaded copies of the sources, the parsed data and the counting scripts were kept outside the repository; every figure above links to its source and gives its counting method.
