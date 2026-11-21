# Free-agent markets, contracts and career exits: measured behaviour and literature

Compiled 2026-11-17 for the free-agent market and contract system. MLB is the focus; Section 7 has a little measured KBO/NPB behaviour.
Rules (eligibility, service time, arbitration procedure, CBT thresholds) are covered elsewhere and are deliberately left out.
This file complements `docs/research/world/aging.md`; it does not repeat its aging curves, draft or prospect numbers. Sources from that file are cited by its IDs (for example "aging.md M8").

**Tags**
- **[peer-reviewed]** journal article (journal, year and sample given in the row).
- **[working-paper]** academic working paper, thesis or conference poster (not refereed).
- **[analysis]** FanGraphs, Baseball Prospectus, The Ringer, MLB Trade Rumors, SABR *Baseball Research Journal* and similar (not refereed).
- **[press]** news outlets, including league-owned MLB.com and university press releases.
- **[data]** a table I counted myself from a public database. The method is described next to it. The counting scripts were kept outside the repository.
- **NOT CONFIRMED** means I looked and could not find or read the number. "Derived" means my own arithmetic on published numbers.

Source IDs (J = journal, W = working paper, A = analysis, P = press, D = data) are listed in Section 10.

---

## 0. Headline numbers

The calibration targets below are restated from the tables further down. See each row for the method and caveats.

- **Price of a win (free agency).** $10.6M per projected WAR on average for 2020-2026 in 2025 dollars, net of the minimum salary. The price is not linear in quality: $7.4M per WAR for 0-1 WAR players, $8.6M for 1-2 WAR, $11.8M for 2+ WAR (A2). The 2025-26 offseason ran $11.2M overall and $12.8M for 2+ WAR players. Earlier series: $4.6M (2006) to $10.5M (2017) per WAR (A4).
- **Arbitration pay vs free agency.** Rule of thumb: 40/60/80% of free-agent value in arbitration years 1/2/3 (A8). Measured for 2013-2021 (derived from A7): batters about 21/33/55%, starters 21/36/51%, relievers 28/61/86%. Super Two first year: 17-24%.
- **The market each winter (2021-22 to 2025-26)** [data] D1:
  - About 260-280 players per free-agent class in the FanGraphs tracker.
  - Of the domestic players: 59-70% got a major-league contract, 16-27% a minor-league contract, and 12-19% signed nothing that season.
  - 66-80% of major-league deals were for one year.
  - Mean AAV $6.6-7.8M; median AAV $2.75-5.9M.
  - Total guarantees $2.9-4.0B per winter.
- **Age discount** [data] D1, pooled over five winters.
  - Share of free agents left without any contract: 6-9% at 28-32, 17% at 34, 22% at 35, 31-36% at 36 and older.
  - Among 2+ WAR free agents, 82-94% got multi-year deals at ages 28-33, against 47-60% at 34-35.
  - Median guarantee per platform-season WAR: $29M at 29 or younger, $26M at 30-31, $12M at 32-33, $6.4M at 34-35, $6.1M at 36 and older.
  - Median AAV per platform WAR falls far less with age: $6.2M to $4.6M. Age cuts years first, then price.
- **Veterans are disappearing** [data] D3, D4.
  - Share of MLB plate appearances by batters 35 and older: 11.5-13.0% in 2003-2007 (peak 12.96% in 2007), 4.3% in 2025 (lowest of 1995-2025).
  - Batters 33 and older: 21-23% in 2003-2007, 11.8% in 2025.
  - PA-weighted batter age: 29.3 (2004) to 27.9 (2025).
  - Pitchers did not get younger. Their age weighted by batters faced was 29.0 in 2025, the highest since 1995. Pitchers 35 and older faced 9.7% of batters in 2025.
  - The SD of batter ages (PA-weighted) was 3.6-3.7 years in 2019-2025, consistent with the 3.7 the team measured.
- **Exit hazard by age (regular hitters, 300+ PA)** [data] D3. The chance that a season of 300+ PA was the player's last:
  - 2000-2009: 0-5% at ages 25-35.
  - 2012-2021: 11% at 33, 14% at 34 and 20% at 35.
  - All hitters at 35: 22% (2000-09) vs 41% (2012-21).
- **Career length.** Position players who debuted 1902-1993 averaged 5.6 years, and 1 in 5 played a single year (J19). In my count, hitters who debuted in the 1990s played a mean of 7.0 seasons (median 6); pitchers 5.7 (median 4) (D3).
- **Released vs retired.** NOT CONFIRMED. No study splitting MLB career endings into release vs voluntary retirement was found (Section 6.6).
- **Shirking and the contract year.** Peer-reviewed estimates are small: about 3-10 OPS points below expectation in the first year of a multi-year deal and +8 OPS points in the walk year (J15). Players unlikely to get another contract underperform more. Fixed-effects estimates of the contract-year boost are +4 to +7% OPS (J17, A1).
- **Payroll 2026.** Opening Day (AP): highest $352.2M (Mets), lowest $62.3M (Guardians), average salary $5,335,966, median salary $1.4M, minimum salary $780,000 (P1).
- **Payroll 2025.**
  - Total MLB payroll $5.32B; Dodgers $345.3M plus $169.4M luxury tax (P2).
  - Early-season 2025 club payrolls: minimum $69.9M, median $158.8M, maximum $390.1M (A12; median derived).
- **League revenue.** $12.1B in 2024, a record (Forbes via P3). 2025: NOT CONFIRMED.
- **Salary share by service-time group, 2019-2025** (A13): pre-arbitration about 14-15%, arbitration about 24-25%, free-agent-eligible about 61%. Pre-arbitration players produced 38-41% of WAR; free-agent-eligible players 28-29%.

---

## 1. Peer-reviewed and academic literature on MLB pay and contracts

### 1A. Pay vs marginal revenue product (MRP) and service class

| Study | Tag / venue | Sample | Method | Finding (numbers as published) | Access |
|---|---|---|---|---|---|
| Scully 1974 (J1) | [peer-reviewed] *American Economic Review* 64(5):915-930 | 1968-69 seasons (as described by secondary sources) | Two stages: wins per player, then revenue per win, giving MRP; compares MRP with salary under the reserve clause | Own numbers NOT CONFIRMED (text not accessed). Secondary summary (W1): Scully's "monopsony exploitation ratios" (MRP minus salary, over MRP) "were often in the 0.5-0.85 range in the reserve clause era". | JSTOR, not read |
| Krautmann 1999 (J2) | [peer-reviewed] *Economic Inquiry* 37(2):369-381 | Not in abstract | Critique of Scully; proposes a "free-market returns" approach that infers MRP from free-agent salaries | Abstract: Scully-style estimates imply even the highest-paid players are "grossly underpaid", which he finds implausible given competitive free-agent bidding. Applied to less mobile players, his method gives "much more reasonable" MRPs. No numbers in abstract. | https://ideas.repec.org/a/oup/ecinqu/v37y1999i2p369-81.html (abstract) |
| Bradbury 2013 (J3) and Krautmann's reply 2013 (J4) | [peer-reviewed] *J. Sports Economics* 14(1):87-96 and 97-105 | - | Debate | Bradbury: Scully's flaws are "not as serious" as Krautmann claims, and the free-market approach is "not necessarily superior". Krautmann: the two methods answer different questions. No numbers in abstracts. | DOIs 10.1177/1527002511418981, 10.1177/1527002511419299 |
| Blair, Humphreys & Pyun 2017 (J5; working paper W4, 2015) | [peer-reviewed] *Managerial and Decision Economics* 38(5):676-688 | MLB position players, 2001-2011 seasons | Scully method with spline revenue functions and fixed-effects stochastic production functions | Exploitation ratio about **0.89 for rookies, 0.75 for arbitration-eligible players, 0.21 for free agents**. Recent CBAs lowered the free-agent ratio only. (Working-paper abstract.) | https://ideas.repec.org/p/wvu/wpaper/15-48.html |
| Krautmann, von Allmen & Berri 2009 (J6) | [peer-reviewed] *Int. J. Sport Finance* 4(3):161-175 | NFL, NBA, MLB | Free-market approach | Abstract: restricted (reserve-clause) players are underpaid; owners capture less surplus as player bargaining power rises; the largest surplus comes from the most valuable players. MLB numbers NOT CONFIRMED (paywalled). | https://ideas.repec.org/a/jsf/intjsf/v4y2009i3p161-175.html |
| Hyman 2020 (W1) | [working-paper] UC Berkeley senior honors thesis | Position players 2012-2019, n=2,807 player-seasons | Scully / Humphreys-Pyun method with WAR | Mean (median) exploitation ratio: all 0.334 (0.709). Rookies 0.866 (0.918), n=1,060. Arbitration 0.412 (0.608), n=765. Free-agency-eligible -0.300 (0.123), n=982; a negative value means paid above MRP. On teams above .550: arbitration 0.782, free agents 0.531. Revenue per win about $870,000 for teams below .550, with a steeper slope above .550. | https://econ.berkeley.edu/sites/default/files/Economics%20Senior%20Honors%20Thesis%20-%20Jake%20Hyman.pdf |
| Hakes & Turner 2011 (J8; working paper 2007) | [peer-reviewed] *J. Productivity Analysis* 35(1):61-74 | Position players, 130+ AB, 1985-2005 (aging.md P3) | Salary paths by experience within ability quintiles; salaries indexed to the geometric mean free-agent salary | "Free agents are paid proportionately with their production at all ability levels, whereas young players' salaries are suppressed by similar amounts." Rookies get about **8%** of the geometric-mean free-agent salary. Top-quintile players at their salary peak get 2.35 times it. Salaries fall near retirement but "the standard errors are so large that the coefficients are not statistically significant." The free-agent geometric mean rose from $682,000 to $4.08M over the sample. | https://mpra.ub.uni-muenchen.de/4326/1/MPRA_paper_4326.pdf |
| Hakes & Sauer 2006 (J7) | [peer-reviewed] *J. Economic Perspectives* 20(3):173-186 | Players 1999-2004 | Win regression, then log-salary regression on OBP and SLG | Abstract only: some skills (on-base) were undervalued early in the period, and "the market corrected" as knowledge spread. Coefficients NOT CONFIRMED: the publisher PDF returned 403, and no readable copy was found. | DOI 10.1257/jep.20.3.173 |
| Congdon-Hohman & Lanning 2017 (W2) | [working-paper] Holy Cross WP 17-02 | Free-agent hitters in 3 cohorts: 1988-90 (n=104), 1998-2000 (n=154), 2008-10 (n=205) | Log first-year salary on prior-year output, age and age squared | Mean age 31.6, 32.9, 32.8. Mean salary after free agency $922,675, $2,438,075, $3,185,636. Age: 1980s cohort +0.835 (age) and -0.012 (age²), both significant, implying a salary peak near 34.8 (derived). Later cohorts: age insignificant (for example -0.137 and +0.001). Holding performance fixed, first-year salary showed no age penalty in 1998-2010. | https://hcapps.holycross.edu/hcs/RePEc/hcx/HC1702-Congdon-Lanning_FreeAgencyRewards.pdf |
| Terry, McGee & Kass 2018 (J13) | [peer-reviewed] *Sport Management Review* 21(2):189-201 | 345 free-agent transactions, 2012-2015 | Premium = pay minus prior-year MRP | Premiums are larger with multiple bidders, when the signing club contended the prior year or has more resources, and when the player's previous club was successful and large-market. No numbers in abstract. | DOI 10.1016/j.smr.2017.06.006 |

### 1B. Contract length

| Study | Tag / venue | Sample | Method | Finding | Access |
|---|---|---|---|---|---|
| Kahn 1993 (J9) | [peer-reviewed] *Review of Economics and Statistics* 75(1):157-164 | MLB panel (years not in abstract) | Fixed effects | Both arbitration and free-agency eligibility raised pay, but **only free agency lengthened contracts**. Consistent with Nash bargaining. The arbitration result suggests a "winner's curse": duration rises only where the team could lose the player. | https://ideas.repec.org/a/tpr/restat/v75y1993i1p157-64.html |
| Krautmann & Oppenheimer 2002 (J10) | [peer-reviewed] *J. Sports Economics* 3(1):6-17 | Free agents (years not in abstract) | Wage regression including contract length | Longer contracts carry higher salaries, but the **return to performance falls with contract length**: a trade-off, not a compensating differential. No numbers in abstract. | DOI 10.1177/1527002502003001002 |
| Link & Yosifov 2012 (J11) | [peer-reviewed] *J. Sports Economics* 13(1):3-19 | All free-agent position players who signed in 1984-1994 and 2003-2006 | Multi-period salary models | Players "appear willing to trade monetary returns to performance" for longer guaranteed deals. Weakest for 1990-94. | DOI 10.1177/1527002510396984 |
| Solow & Krautmann 2020 (J12) | [peer-reviewed] *J. Sports Economics* 21(7):705-722 | Long-term contracts (sample size not in abstract) | Ex ante value: aging-adjusted win forecasts times team-specific revenue per win, discounted | Teams **overpay on average, and more so for longer contracts**; part of the gap may be non-win benefits. No numbers in abstract. | DOI 10.1177/1527002520930259 |
| Cuccolo 2025 (W3) | [working-paper] Ursinus honors paper | 652 free-agent position players on 2+ year deals, 1991-2025 | Contract-length regression | Younger and higher-performing players are more likely to get long deals; a qualifying offer is associated with longer deals; the 2022 CBA with longer deals at the top. Effect sizes NOT CONFIRMED (abstract only). | https://digitalcommons.ursinus.edu/bus_econ_hon/63 |

### 1C. Shirking and the contract-year effect

| Study | Tag / venue | Sample | Method | Finding | Access |
|---|---|---|---|---|---|
| Lehn 1982 (J18) | [peer-reviewed] *J. Law & Economics* 25(2):343-366 | - | - | Classic paper on disability days under guaranteed long-term contracts. NOT CONFIRMED: no abstract available, text not accessed. | DOI 10.1086/467020 |
| Ahlstrom, Si & Kennelly 1999 (J16) | [peer-reviewed] *J. Sport Management* 13(3):181-196 | 172 free agents; two seasons before and one after filing | Equity vs expectancy theory | Performance "tended to decline" after players signed with new teams. | DOI 10.1123/jsm.13.3.181 |
| Maxcy, Fort & Krautmann 2002 (J14) | [peer-reviewed] *J. Sports Economics* 3(3):246-255 | Players before negotiations vs others | Availability and in-game performance | **Rejects strategic pre-contract performance.** Pitchers with nagging injuries are more likely to go on the disabled list while under long-term contracts (strategic behaviour or clubs protecting an investment). O'Neill (A1) adds: no significant change in SLG in contract years, but fewer DL days and more at-bats for players seeking contracts. | DOI 10.1177/1527002502003003002 |
| Krautmann & Solow 2009 (J15), with press summary P8 | [peer-reviewed] *J. Sports Economics* 10(1):6-22 | 527 free-agent hitters on contracts of 2+ years, 1997-2007. These were about **44% of free-agent contracts**; 56% were one-year deals (P8). | Expected OPS from past performance, age and injuries; controls for the chance that a contract is the player's last | **First year of a 3-year deal: about 3 OPS points below expectation; up to 10 points on 5-year deals.** Back to expectation by year 2. Walk year: about **+8 OPS points**. Players unlikely to sign again show a "large and statistically significant" shortfall (size NOT CONFIRMED). | DOI 10.1177/1527002508327382; P8 |
| O'Neill 2013 (J17) and O'Neill 2014 (A1) | [peer-reviewed] *Int. Advances in Economic Research* 19(3):215-232; [analysis] SABR *Baseball Research Journal*, Fall 2014 | 256 free-agent hitters (6+ years of MLB service) under the 2006-2011 CBA; 1,016 player-years | Fixed effects; controls for estimated probability of retirement | Contract year: **+6.11 OPS100 points (+6.7%)**, worth about $470,000 a year. Each point of retirement probability: -1.0034 OPS100. Raw contract-year OPS100 is lower (85.9 vs 97.2) because 23.1% of contract-year observations were retirements (3.2% otherwise). OLS gives a negative, insignificant effect. Cited: Hummel & O'Neill 2011 (2004-08 free agents) **+4.2 to +5.5% OPS**, and retiring players **-11.2 to -13.2%**. | DOI 10.1007/s11294-013-9419-y; https://sabr.org/journal/article/do-hitters-boost-their-performance-during-their-contract-years/ |
| Martin, Eggleston, Seymour & Lecrom 2011 ("One-Hit Wonders", *NINE*); White & Sheldon 2014 (*Motivation and Emotion*) | [peer-reviewed] | - | - | NOT CONFIRMED (Project MUSE / Springer blocked; abstracts not read) | - |

### 1D. Career length and exit

| Study | Tag / venue | Sample | Method | Finding | Access |
|---|---|---|---|---|---|
| Witnauer, Rogers & Saint Onge 2007 (J19); press release P7 | [peer-reviewed] *Population Research and Policy Review* 26(4):371-386 | 5,989 position players who began careers 1902-1993; 33,272 person-years; pitchers excluded | Life tables | Average career **5.6 years**. **1 in 5** have a single-year career. At every point of a career, the chance of it ending is **at least 11%**. Era and position detail NOT CONFIRMED (paywalled). | DOI 10.1007/s11113-007-9038-5; https://www.colorado.edu/today/2007/07/08/average-major-league-baseball-career-56-years-according-new-study |
| Groothuis & Hill 2008 (J20; working paper 2007) | [peer-reviewed] *Southern Economic Journal* 75(2):574-590 | All 3,185 players in MLB 1990-2004 (690 already active in 1990 with mean tenure 6 years, plus all debuts 1990-2004) | Discrete-time logit hazard with a 5th-order tenure polynomial | The hazard by tenure is U-shaped: it falls for about 7 years, then rises (table in 6.5). Hitters: **age +0.196** (t=9.93), games -0.0189, SLG -2.65, runs per game -2.715, strikeouts per game +0.763, catcher -1.034, infield -0.441. Pitchers: **age +0.170** (t=10.14), ERA +0.119, wins -0.148, K/BB -0.351, outs -0.0037, games -0.018, left-handed -0.173. Derived: each extra year of age raises exit odds by about 22% (hitters) or 19% (pitchers) at fixed performance. No race effect for Black hitters. | https://econ.appstate.edu/RePEc/pdf/wp0702.pdf |
| Erickson et al. 2017 (J21) | [peer-reviewed] *Orthopaedic J. Sports Medicine* 5(12) | 153 retired MLB pitchers with elbow ligament reconstruction (1974-2015) vs matched controls | Reasons for retirement from MLB data, team sites and injury reports | Released during the season: **23.6% (34/144) vs 9.7% (14/144) of controls**. Time to retirement 4.4 ± 4.7 years (controls) vs 4.4 ± 3.5 (surgery group). "Performance" as the reason was more common after surgery. Category percentages are not in the abstract. | DOI 10.1177/2325967117745021 |

---

## 2. Free-agent pricing

### 2A. Dollars per WAR, recent markets (Clemens, FanGraphs 2026) [analysis] A2, A3

Method (A2):
- Starting pitchers and position players only; relievers excluded "due to leverage concerns".
- Major-league deals with disclosed terms; minor-league deals and players with negative projections excluded.
- Steamer projection for year 1. Future years assume an aging decline of **0.4 WAR per year** ("the mean decline across the multi-year ZiPS projections"; Smith used 0.5) and **5% annual inflation**.
- The league minimum is subtracted from each salary. 2025 dollars throughout.
- 2025-26 sample: 89 players. The 2+ WAR tier was 19 players and $1.866B of guarantees.

| Offseason (season) | Overall $/WAR | 0-1 WAR | 1-2 WAR | 2+ WAR |
|---|---|---|---|---|
| 2019-20 (2020) | $10.85M | $10.93M | $8.52M | $11.99M |
| 2020-21 (2021) | $6.70M | $4.09M | $6.31M | $7.22M |
| 2021-22 (2022) | $11.24M | $9.46M | $8.77M | $12.15M |
| 2022-23 (2023) | $10.34M | $6.87M | $9.77M | $10.81M |
| 2023-24 (2024) | $12.08M | $8.22M | $8.83M | $14.13M |
| 2024-25 (2025) | $11.92M | $5.96M | $8.95M | $13.91M |
| 2025-26 (2026) | $11.23M | $6.74M | $8.51M | $12.84M |
| Average | $10.62M | $7.37M | $8.60M | $11.76M |

Other results from A2 and A3:
- 95% confidence intervals: 0-1 WAR $6.3-8.6M; 1-2 WAR $7.8-9.4M; 2+ WAR $10.6-12.9M.
- 2020-23, 2+ WAR players got about 40% more per projected win than 0-1 WAR players (about 55% excluding 2020). In the last three years the ratio was "roughly double".
- Aggregates: $9.8M per WAR (2020-23) vs $11.7M (2024-26), inflation-adjusted by league payroll. Stars' $/WAR is "up 30% on a relative basis".
- Tiers for 2020-2026 (A3): 0-1 WAR $7.4M (406 players); 1-2 WAR $8.6M (236); 2-3 WAR $10.5M (83); 3+ WAR $12.3M (62), splitting into 3-4 WAR $11.1M (40) and 4+ WAR $13.2M (22).
- Hitters vs pitchers (A3): 0-1 WAR $6.2M vs $9.9M (the pitcher figure is attributed to pitchers returning from injury); 1-2 WAR $8.2M vs $8.9M; 2+ WAR $12M vs $11.4M.
- Replacement level: the tier gap shrinks but persists when replacement level is raised. Worked example at a 750-WAR league: 0-1 WAR $7.7M, 1-2 WAR $11.1M, 2+ WAR $13.9M. Below-replacement contracts averaged $3.9M per full-time equivalent.
- Bat-first players get "a little more" than their WAR implies and glove-first players "a little less" (A3, qualitative).
- No breakdown by contract length or age (A2, A3).

### 2B. Earlier estimates and linearity

| Source | Tag | Finding |
|---|---|---|
| Swartz 2017 (A4) | [analysis] | Actual (not projected) WAR of all free-agent-eligible players, three-year averages, net of the minimum, including draft-pick cost. **$/WAR: 2006 $4.6M, 2008 $5.6M, 2010 $5.8M, 2012 $6.9M, 2014 $7.7M, 2015 $8.7M, 2016 $9.6M, 2017 $10.5M.** Forecast $11.1M (2018) to $13.9M (2022), with salaries growing about 5.9% a year. |
| Swartz 2017 (A5) | [analysis] | Implied team **discount rate about 10% a year** (tested 5/10/15%; below 5% "unreasonably small"). Free agents earning $2M or more above the minimum. Draft-pick compensation adds about 7% to cost (up to 20% in 2015). |
| Edwards 2020 (A6) | [analysis] | Offseasons 2018-2020, projections cut by 10%. All players: $8.7M/WAR overall, 2+ WAR $8.8M, 0-2 WAR $8.3M (about linear). **Relievers excluded: 0-2 WAR $6.6M, about 25% cheaper per win; more than 30% in 2019-2020.** |

Conflict: Swartz's 2017 series found $/WAR "still linear". Edwards (2020, relievers excluded) and Clemens (2026) find stars cost 25-100% more per win than role players. Part of the difference is method: actual vs projected WAR, and whether relievers are included.

### 2C. Price by age and contract length [data] D1, with literature

The literature: Clemens's model assumes -0.4 WAR a year of aging and 5% inflation (A2). Swartz implies a 10% discount rate (A5). Long contracts are overpaid ex ante (J12) and carry lower returns to performance (J10, J11). No published FanGraphs or Baseball Prospectus table of $/WAR by age or contract length was found (NOT CONFIRMED).

My own count from the FanGraphs RosterResource free-agent tracker (method in 3A):
- Seasons 2022-2026 (offseasons 2021-22 to 2025-26), first major-league contract per player, international signees included.
- Price = AAV as listed (CBT-style present value, so deferrals are discounted) divided by **platform-season WAR** (FanGraphs WAR in the year before free agency). This is not projected WAR: platform WAR includes luck, so these figures run below Clemens's $/projected WAR. Treat them as relative.
- The minimum salary is not subtracted.

**Players with 2+ platform WAR (n=131):**

| Age (June 30 of first contract season) | n | Share on multi-year deals | Median years | Aggregate AAV per WAR | Median AAV per WAR | Median guarantee per WAR | Aggregate guarantee per WAR |
|---|---|---|---|---|---|---|---|
| 29 or younger | 21 | 81% | 4 | $6.50M | $6.20M | $29.1M | $38.2M |
| 30-31 | 40 | 90% | 4 | $5.86M | $6.10M | $26.0M | $28.9M |
| 32-33 | 34 | 88% | 2 | $5.39M | $5.02M | $12.2M | $17.3M |
| 34-35 | 22 | 50% | 1.5 | $4.96M | $4.73M | $6.4M | $11.8M |
| 36 and older | 14 | 50% | 1.5 | $5.32M | $4.58M | $6.1M | $11.2M |

The aggregate for 29 and younger includes Ohtani ($700M nominal) and Soto ($765M); medians are more robust.

| Contract length (2+ WAR players) | n | Aggregate AAV per platform WAR |
|---|---|---|
| 1 year | 30 | $4.77M |
| 2 years | 29 | $5.08M |
| 3-4 years | 39 | $6.36M |
| 5+ years | 33 | $6.03M |

For players with 1.0-1.99 platform WAR (n=158), AAV per WAR is flat by age from 30 on ($5.65-5.96M; $7.94M at 29 or younger, n=9). It rises with length: 1 year $5.09M (n=101), 2 years $6.75M (37), 3-4 years $8.26M (18). Longer deals go to players the market expects to be better, so length and price per *past* WAR are confounded with quality.

Takeaway (descriptive): in 2022-2026 the market discounted age mainly through **term**. From 30-31 to 34-35, median years fell from 4 to 1.5 and the median guarantee per WAR fell about 75%, while the median AAV per WAR fell about 22% (derived from the table).

### 2D. Arbitration pay as a share of free-agent value

| Source | Tag | Arb year 1 | Arb year 2 | Arb year 3 | Notes |
|---|---|---|---|---|---|
| "40/60/80 rule" (Tango, building on Studenmund's 2007 *Hardball Times Annual*), via Nicholson-Smith 2011 (A8) | [analysis] | 40% | 60% | 80% | A rule of thumb for groups. Studenmund: arbitration-eligible players as a group earned **about 55%** of free agents' pay per win. Sample and method not given in A8. |
| Clemens 2022 (A7): $ above minimum per positive WAR, 2013-2021, 2% inflation; free agency "roughly $6.5 million above minimum salary per WAR" | [analysis] | Batters $1.36M (21%); starters $1.38M (21%); relievers $1.79M (28%) | $2.13M (33%); $2.35M (36%); $3.98M (61%) | $3.59M (55%); $3.34M (51%); $5.61M (86%) | Percentages are derived (divided by $6.5M). Super Two first year: $1.08M / $1.11M / $1.57M (17/17/24%). Pre-arbitration: $0 above the minimum. Relievers are paid more per WAR at every stage. |
| Chamberlain & Dolinar 2015 (A9) | [analysis] | - | - | - | Arbitration salary rises about 13-14% per career WAR (batters / pitchers). The best model explains 56% of variance. Defence "carries little to no weight". |
| Hakes & Turner (J8) | [peer-reviewed] | - | - | - | Rookies about 8% of the geometric-mean free-agent salary; ordering by ability is complete by year 5. |

Conflict: the 40/60/80 rule is well above the 2013-2021 measurement for batters and starters (about 21/33-36/51-55%). Relievers are close to it.

---

## 3. Market counts, 2021-22 to 2025-26

### 3A. Method [data] D1

- **Source.** FanGraphs RosterResource Free Agent Tracker, `https://www.fangraphs.com/roster-resource/free-agent-tracker?season=YYYY`, downloaded 2026-11-17. "Season" 2022 means the 2021-22 offseason.
- **What I read.** The page's embedded JSON: one row per contract, with contract type, years, total, AAV, signing team, prior team, age, service time and prior-season WAR.
- **Scope.** The tracker lists 262-283 unique players per class. The page does not define eligibility. The list holds players with 6+ years of service plus non-tendered, released and posted NPB/KBO players. It excludes most minor-league free agents: MLBTR lists roughly 215 names (approximate count) of minor-league free agents with MLB experience in November 2025 alone (A17).
- **Counting unit.** Each player's **first contract** of the cycle (lowest contract ID). Later re-signings after a release are ignored.
  - "MLB contract" = contract type Free Agent, Free Agent/Arb (signed with under 6 years of service), Arbitration, Extension or Pre-Arb.
  - "MiLB" = minor-league contract.
  - "No contract" = no deal recorded as of 2026-11-17. This mixes retirements, unsigned players and moves abroad (signing teams in the data are MLB clubs only).
- **Domestic vs international.** Players whose prior team is NPB or KBO (5-13 a year) are excluded from the outcome percentages. Some of them have contracts the tracker does not link; for example, three posted NPB players in 2025-26 show no contract.
- **Age** is "the player's age on 6/30" of the contract's first season (as stated on the 2026 page; assumed the same for earlier seasons).
- **Platform WAR** is the tracker's previous-season WAR column.
- **Contract years** are as recorded. At least one opt-out deal is stored as 1 year (Correa, 2022, $35.1M), so the multi-year share may be slightly understated.

### 3B. Outcomes by offseason [data] D1

| Offseason | Unique players (international) | Domestic n | MLB contract | MiLB contract | No contract | MLB first contracts incl. international | 1-year share | Multi-year share | Mean years | Mean AAV | Median AAV | Sum of guarantees |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2020-21 (context; COVID) | 318 (7) | 311 | 201 (65%) | 66 (21%) | 44 (14%) | 207 | 170 (82%) | 18% | 1.32 | $4.02M | $2.00M | $1.55B |
| 2021-22 | 262 (8) | 254 | 157 (62%) | 50 (20%) | 47 (19%) | 161 | 109 (68%) | 32% | 1.75 | $7.11M | $4.75M | $3.22B |
| 2022-23 | 270 (8) | 262 | 161 (61%) | 70 (27%) | 31 (12%) | 166 | 109 (66%) | 34% | 1.82 | $7.77M | $5.88M | $3.98B |
| 2023-24 | 283 (9) | 274 | 163 (59%) | 66 (24%) | 45 (16%) | 168 | 124 (74%) | 26% | 1.57 | $6.64M | $4.12M | $3.14B |
| 2024-25 | 264 (5) | 259 | 176 (68%) | 48 (19%) | 35 (14%) | 181 | 137 (76%) | 24% | 1.51 | $6.88M | $3.75M | $3.41B |
| 2025-26 | 280 (13) | 267 | 187 (70%) | 42 (16%) | 38 (14%) | 193 | 154 (80%) | 20% | 1.38 | $6.58M | $2.75M | $2.94B* |

\*Excludes the posted NPB/KBO players' unlinked contracts.

Notes on the table:
- **Contract lengths** (MLB first contracts):
  - 2021-22: 1y 109, 2y 23, 3y 11, 4y 8, 5y 4, 6y 3, 7y 2, 10y 1.
  - 2022-23: 1y 109, 2y 28, 3y 14, 4y 3, 5y 5, 6y 2, 7y 1, 8y 1, 9y 1, 11y 2.
  - 2023-24: 1y 124, 2y 24, 3y 10, 4y 4, 5y 2, 6y 1, 7y 1, 10y 1, 12y 1.
  - 2024-25: 0y 1, 1y 137, 2y 24, 3y 12, 4y 1, 5y 2, 6y 1, 7y 1, 8y 1, 15y 1.
  - 2025-26: 1y 154, 2y 22, 3y 9, 4y 1, 5y 6, 7y 1.
- **AAV distribution**, pooled over the five offseasons (844 domestic MLB deals): p10 $0.85M, p25 $1.5M, median $4.0M, p75 $10.0M, p90 $18.0M, p99 $36.5M, maximum $57.2M. 29% of deals had an AAV of $1.5M or less.
- **Pooled outcome**, 2021-22 to 2025-26 (1,316 domestic players): MLB 64%, MiLB 21%, no contract 15%.
  - Pitchers (n=745): 64% / 20% / 15%; 28% of their MLB deals multi-year.
  - Hitters (n=571): 64% / 22% / 14%; 23% multi-year.
- **Prior WAR of the pool:** below 0 for 453 players; 0-0.9 for 493; 1-1.9 for 170; 2+ for 132; missing for 68 (did not play).
  - Median platform WAR: about +0.5 to +0.7 for MLB signers, -0.02 to -0.23 for MiLB signers, -0.33 to -0.09 for players with no contract.
  - Mean age: MLB signers 32.4-32.9; no contract 33.6-35.3.
- **Pool age:** mean 32.4-33.1 each year. Players 35 or older were 21-32% of the pool.

**Cross-checks** (different counting rules, so not conflicts):
- Becker counted only fully guaranteed MLB free-agent deals signed by early or mid-March:
  - 2024-25: **130 signings, $3,392,675,000, 217 years** ($26,097,500 per free agent). Lengths: 1y 93, 2y 17, 3y 12, 4y 2, 5y 2, 6y 1, 7y 1, 8y 1, 15y 1 (A10).
  - 2025-26: **142 signings** (division table; the team table as read sums to 144) and **$3.034B** (derived from the division table) (A11).
- The tracker's larger counts add in-season signings, non-tenders and posted players.
- ESPN (Spotrac data): teams spent on average **$1.6B a year** on free agents after the 2016-2020 seasons, $3.2B in 2021-22 and $3.5B in 2022-23 as of 2022-12-23 (P5).
- Context from 1997-2007 (J15/P8): 56% of free-agent contracts were one-year deals; the denominator is not stated.

### 3C. Outcome by age, pooled 2021-22 to 2025-26, domestic players [data] D1

| Age | n | MLB contract | MiLB | No contract | Multi-year (share of all) | 3+ years (share of all) | Multi-year among MLB deals | Mean years (MLB) | Median AAV (MLB) | Mean AAV (MLB) |
|---|---|---|---|---|---|---|---|---|---|---|
| 27 or younger | 29 | 62% | 31% | 7% | 7% | 7% | 2/18 | 1.94 | $1.65M | $7.79M |
| 28-29 | 137 | 66% | 26% | 7% | 19% | 12% | 26/91 | 1.87 | $3.00M | $8.79M |
| 30 | 143 | 67% | 27% | 6% | 23% | 14% | 33/96 | 2.07 | $5.00M | $8.89M |
| 31 | 175 | 69% | 22% | 9% | 22% | 13% | 39/121 | 1.77 | $5.62M | $7.79M |
| 32 | 163 | 67% | 24% | 9% | 23% | 12% | 38/110 | 1.61 | $4.75M | $7.18M |
| 33 | 166 | 70% | 19% | 11% | 17% | 5% | 28/116 | 1.34 | $2.75M | $4.96M |
| 34 | 139 | 57% | 27% | 17% | 12% | 6% | 16/79 | 1.30 | $3.50M | $5.91M |
| 35 | 124 | 59% | 19% | 22% | 16% | 4% | 20/73 | 1.37 | $4.00M | $6.20M |
| 36 | 81 | 51% | 14% | 36% | 6% | 1% | 5/41 | 1.15 | $2.53M | $4.66M |
| 37+ | 159 | 62% | 7% | 31% | 7% | 1% | 11/99 | 1.13 | $4.50M | $6.62M |

**Players with 2+ platform WAR only (n=132).** All got MLB deals, except 1 of 5 at age 36.

| Age | n | Multi-year | 3+ years | Mean years | Median AAV |
|---|---|---|---|---|---|
| 28-29 | 19 | 84% | 68% | 4.37 | $23.3M |
| 30 | 20 | 90% | 80% | 5.05 | $21.6M |
| 31 | 20 | 90% | 70% | 4.10 | $22.2M |
| 32 | 17 | 82% | 59% | 2.88 | $16.0M |
| 33 | 17 | 94% | 35% | 2.47 | $13.0M |
| 34 | 17 | 47% | 41% | 1.88 | $15.0M |
| 35 | 5 | 60% | 40% | 2.40 | $20.0M |
| 36 | 5 | 40% | 20% | 1.75 | $11.0M |
| 37+ | 10 | 50% | 20% | 1.70 | $11.4M |

**Age × platform WAR.** Each cell: n / MLB contract / MiLB / no contract / multi-year MLB deal (share of all).

| Age | WAR below 0 | WAR 0-0.9 | WAR 1-1.9 | WAR 2+ | Did not play |
|---|---|---|---|---|---|
| 30 or younger | 109 / 53% / 42% / 5% / 1% | 114 / 68% / 26% / 6% / 11% | 26 / 88% / 4% / 8% / 42% | 41 / 100% / 0 / 0 / 85% | 19 / 32% / 32% / 37% / 5% |
| 31-33 | 167 / 45% / 37% / 18% / 3% | 192 / 74% / 19% / 6% / 12% | 65 / 97% / 2% / 2% / 42% | 54 / 100% / 0 / 0 / 89% | 26 / 46% / 38% / 15% / 8% |
| 34-35 | 86 / 22% / 41% / 37% / 2% | 102 / 70% / 18% / 13% / 9% | 38 / 92% / 5% / 3% / 34% | 22 / 100% / 0 / 0 / 50% | 15 / 33% / 40% / 27% / 7% |
| 36+ | 91 / 23% / 20% / 57% / 0% | 85 / 76% / 5% / 19% / 2% | 41 / 90% / 0 / 10% / 15% | 15 / 93% / 0 / 7% / 47% | 8 / 38% / 0 / 62% / 12% |

**By offseason, players 35 or older** (MLB / MiLB / no contract; multi-year as share of all):

| Offseason | n | MLB | MiLB | No contract | Multi-year |
|---|---|---|---|---|---|
| 2021-22 | 69 | 49% | 13% | 38% | 13% |
| 2022-23 | 56 | 64% | 12% | 23% | 14% |
| 2023-24 | 74 | 58% | 15% | 27% | 5% |
| 2024-25 | 82 | 61% | 12% | 27% | 9% |
| 2025-26 | 83 | 60% | 11% | 29% | 10% |

Ages 30-32 had no contract in 5-11% of cases and got multi-year deals in 19-26%.

### 3D. Long-contract context

- **The Ringer** (A16), MLBTR top-20 free-agent rankings since 2005-06:
  - Average length for the top 20 was **3.3 years** over the 17 offseasons before 2022-23, then **5.5** in 2022-23 (as of December 22, 2022).
  - Top 10: **4.3** vs **7.6** years.
  - Explanations offered: CBT savings from a lower AAV, interest rates, revenue recovery, decision-makers' horizons. Zimmerman's finding that hitters age faster after 30 is the counterpoint.
- **ESPN** (P5):
  - Only four free agents ever had deals longer than 10 years; three of them came in December 2022.
  - 10 of the last 11 deals of 10+ years were signed since 2019.
  - **581 players became free agents before turning 30 over the 2020-2022 offseasons, vs 182 in 2012-2014.**
  - Explanations: luxury-tax AAV, younger free agents, better conditioning, owner spending.

---

## 4. The shrinking veteran market

### 4A. Age of MLB playing time, 1995-2025 [data] D3, cross-checked with D4

Method:
- Lahman database, 2026 edition: R package `Lahman` 14.0-0 on CRAN, published 2026-02-12; Batting and Pitching tables include the 2025 season.
- Age = season minus birth year, minus 1 if born in July or later (age on June 30).
- Batters: PA = AB + BB + HBP + SH + SF, excluding player-seasons where batters faced exceed twice the PA (pitchers batting). Pitchers weighted by batters faced.
- Cross-check: Baseball-Reference's league BatAge is 29.2-29.3 in 2001-2006, 28.9 in 2010, 27.9 in 2019 and 2024-2025, and 27.8 in 2026 (D4); my PA-weighted values match to within 0.1. B-Ref PAge is 28.4 in 2019, 29.0 in 2024, 29.2 in 2025 and 29.1 in 2026. The two use different weights (B-Ref's weighting not read).

| Season | Batter age (PA-weighted) | Pitcher age (BF-weighted) | PA to 33+ | PA to 35+ | PA to 26 or younger | BF by 33+ | BF by 35+ | BF by 26 or younger |
|---|---|---|---|---|---|---|---|---|
| 1995 | 28.70 | 28.14 | 16.0% | 8.2% | 31.3% | 12.9% | 5.6% | 37.4% |
| 2000 | 29.16 | 28.66 | 21.5% | 11.6% | 31.3% | 19.8% | 9.7% | 33.3% |
| 2004 | 29.34 | 28.88 | 22.1% | 12.7% | 26.5% | 18.9% | 12.6% | 35.4% |
| 2005 | 29.26 | 28.98 | 22.7% | 11.5% | 27.2% | 20.7% | 13.8% | 33.5% |
| 2007 | 29.13 | 28.52 | 22.3% | 13.0% | 30.7% | 16.2% | 11.8% | 40.3% |
| 2010 | 28.89 | 28.15 | 20.9% | 9.4% | 32.3% | 16.5% | 7.9% | 40.8% |
| 2015 | 28.35 | 28.28 | 14.6% | 6.6% | 34.6% | 13.1% | 7.3% | 35.8% |
| 2017 | 28.29 | 28.41 | 17.1% | 5.2% | 36.9% | 15.5% | 4.9% | 34.8% |
| 2019 | 27.91 | 28.25 | 12.8% | 6.4% | 40.4% | 13.1% | 6.1% | 36.7% |
| 2021 | 28.34 | 28.59 | 12.0% | 5.0% | 33.7% | 16.3% | 7.2% | 34.2% |
| 2022 | 28.22 | 28.65 | 11.5% | 5.0% | 32.9% | 14.6% | 7.5% | 30.9% |
| 2023 | 27.99 | 28.82 | 12.4% | 5.0% | 38.9% | 16.9% | 8.1% | 29.8% |
| 2024 | 27.94 | 28.87 | 12.0% | 4.4% | 39.2% | 19.5% | 7.3% | 30.0% |
| 2025 | 27.88 | 29.03 | 11.8% | **4.3%** | 39.3% | 18.9% | 9.7% | 30.6% |

The batters' 35+ share peaked at 12.96% in 2007. 2025 is the lowest value in 1995-2025.

**Share of playing time by age band, three-season pools** [data] D3:

| Age band | Batters (PA) 2003-05 | Batters 2013-15 | Batters 2023-25 | Pitchers (BF) 2003-05 | Pitchers 2013-15 | Pitchers 2023-25 |
|---|---|---|---|---|---|---|
| 24 or younger | 11.0% | 16.7% | 18.5% | 16.0% | 16.2% | 9.7% |
| 25-27 | 25.8% | 27.6% | 30.5% | 30.2% | 32.0% | 31.7% |
| 28-30 | 27.8% | 26.3% | 26.5% | 23.1% | 27.0% | 27.1% |
| 31-33 | 18.9% | 18.4% | 16.7% | 15.2% | 13.3% | 18.5% |
| 34-36 | 10.9% | 7.7% | 6.3% | 7.4% | 7.6% | 10.1% |
| 37+ | 5.5% | 3.2% | 1.5% | 8.0% | 3.9% | 2.9% |

**Spread of batter ages** (non-pitchers with at least 1 PA; SD in years, unweighted / PA-weighted) [data] D3:

| Season | Unweighted SD | PA-weighted SD |
|---|---|---|
| 2000 | 4.14 | 3.99 |
| 2005 | 4.46 | 4.16 |
| 2010 | 4.22 | 4.12 |
| 2015 | 3.82 | 3.91 |
| 2019 | 3.72 | 3.67 |
| 2022 | 3.63 | 3.65 |
| 2023 | 3.74 | 3.72 |
| 2024 | 3.62 | 3.63 |
| 2025 | 3.63 | 3.65 |

About 630-690 batters a season.

### 4B. Published analyses

| Source | Tag | Finding |
|---|---|---|
| Clemens, January 2026 (A14) | [analysis] | Playing-time-weighted age by position, 2002-10 vs 2021-25: catchers 29.7 to 28.7, shortstops 28.0 to 26.7, **DH 31.4 to 29.7** (the largest drop). The universal DH did not make sluggers older. Pitchers with 150+ IP: in their 30s 27 (2025) vs 30 (2010) vs about 33 (2000-15 average); in their 20s 43 vs 73 vs about 62. Explanations: workload protection of young arms; good young hitters with long team control arrive earlier and play. Untested: the universal DH, CBA service-time incentives, Rule 5 dynamics. "Hitters age, while pitchers break." |
| Baumann, November 2025 (A15) | [analysis] | "Not a single position player born before January 20, 1989 has a contract for next season" (that is, 36 and older entering 2026). Players born under Clinton (ages 24-32 in 2025) had 70.5% of 2025 plate appearances, 73.0% of innings and 72.6% of WAR. 11 position players aged 36-41 combined for 3,093 PA and 0.2 WAR; 30 pitchers aged 36-42 for 1,768.2 IP and 19.2 WAR. |
| Sawchik, MLB.com, June 2026 (P4) | [press, league-owned] | Pre-arbitration players produced **40.7% of fWAR (2019-25)**, accounted for 52% of 2025 service-time days, and are "about 60%" of players on the field. Free agents got 61.4% of salaries for 28.8% of fWAR. |
| Clemens, December 2025 (A13) | [analysis] | WAR share by service-time group, old vs new CBA: pre-arbitration 38.4% / 40.7%; arbitration 33.4% / 30.5%; free agents 28.2% / 28.8%. Salary shares in 5B. |
| ESPN 2022 (P5) and Lindbergh 2022 (A16) | [press] / [analysis] | Free agents are younger (581 under 30 in three offseasons vs 182 in 2012-14). Zimmerman: hitters age faster after 30. |
| AP, June 2026 (aging.md M8) | [press] | Hitters 35 and older produced little WAR in early 2026 vs their 2003 peak. Numbers are in aging.md and not repeated here. |
| FiveThirtyEight 2018, "Nobody wants baseball's 30-something free agents anymore" | [analysis] | NOT CONFIRMED. The page now redirects and the Wayback Machine is not reachable from here. Numbers were seen only in a search snippet and are not used. |

Aging-curve explanations (steeper post-30 hitter decline, survivor bias) are in aging.md, Section 1 (A6, A11, P1u) and are not repeated.

---

## 5. Payroll, revenue and salary shares

### 5A. Payrolls

| Item | Value | Source |
|---|---|---|
| Opening Day 2026 (40-man rosters plus injured list; 942 players; deferred money discounted) | Highest **Mets $352.2M** (record $355.4M in 2023); Dodgers $316.6M ($395.2M undiscounted); Yankees $297.2M; Phillies $282M; Blue Jays $269M; Tigers $206.7M; Cardinals $100.4M; Twins $96.5M; lowest **Guardians $62.3M**. Six clubs at $250M+, 10 at $200M+, 8 under $100M. | AP (P1) [press] |
| Opening Day 2026 salaries | Average **$5,335,966** (+3.4%; +28% over the five-year CBA, 5.6% a year). **Median $1.4M** (record $1.65M in 2015). 31 players at the **$780,000** minimum. 519 players (55%) at $1M+. 74 at $20M+, 19 at $30M+. Top 50 players earn 30% of salaries; top 100 earn 49%. | P1 [press] |
| Final 2025 payrolls | **Total $5.32B** (+3.1% from $5.16B in 2024). Dodgers $345.3M plus $169.4M tax = $514.6M (record). Mets $342.1M, Yankees $301.5M, Phillies $291.9M, Blue Jays $253.1M, Astros $236.4M, Rangers $229.9M, Padres $217.6M, Giants $182.9M, Orioles $165.6M, Cardinals $139.1M, White Sox $87.9M, lowest **Marlins $68.8M** (the text also says $68.7M). Average salary $4,611,595 (MLB) or $4,721,393 (players' union). | AP (P2) [press] |
| Early-season 2025, all 30 clubs (real-dollar payroll; RosterResource) | Minimum **$69.9M** (MIA), median **$158.75M**, mean $175.8M, maximum **$390.1M** (LAD), total $5.275B. CBT payroll: minimum $86.8M, median $182.15M, maximum $391.3M. Highest-to-lowest ratio 5.6. Median and total are derived from Becker's 30-team table. | Becker (A12) [analysis] |
| End-of-season 2025 and 2026, all 30 clubs (RosterResource estimated payroll) | 2025: min $69.5M (MIA), p25 $118.7M, **median $163.3M**, p75 $215.3M, max $404.4M (LAD, deferrals not discounted), sum $5.36B. 2026: min $75.9M (MIA), p25 $109.1M, **median $175.1M**, p75 $239.9M, max $412.0M (LAD), sum $5.53B. CBT median: $188.7M (2025), $206.7M (2026). | [data] D2. Embedded JSON on team payroll pages, field "estPayroll". Within about ±$5.5M of the AP 2025 finals for 11 of the 13 clubs AP lists; not LAD (deferrals) or HOU (off by $12M). The other total field ("TotalPayroll") is inconsistent (for example STL 2026 $32.8M) and is not used. |

Pre-arbitration bonus pool: $50M a year, $1.67M from each club. 2025 payouts by club ranged from $209,217 (MIN) to $4,742,392 (MIL) (A13).

### 5B. Share of salary by service class

| Source | Definition | Pre-arbitration | Arbitration | Free agent / 6+ years |
|---|---|---|---|---|
| Clemens, December 2025 (A13) [analysis] | Salary plus bonus by **service-time group only** (extended players stay in their group); 2019-2025 excluding 2020; Cot's plus RosterResource | Old CBA 14.20%; new CBA 15.36% | 25.16% / 23.93% | 60.64% / 60.71% |
| Sawchik, June 2026 (P4) [press] | "of all player salaries" | **9.4% (2025)** | NOT CONFIRMED | 61.4% (2019-25) |
| Becker, April 2025 (A12) [analysis], payroll-weighted (derived) | Contract status at the start of 2025 (guaranteed includes pre-free-agency extensions) | 7.3% (club range 1.2-39.1%) | 14.2% (club range 3.3-28.9%) | "Guaranteed" 78.6% (club range 39.5-93.0%) |
| Edwards via Baer 2016 (P6) [press, citing FanGraphs] | Service time, mid-2010s (season not stated) | Under 3 years of service: **38% of players, 7.2% of salaries**; average $885,000 | Average $3.9M | 7+ years: 183 players (under 25% of the sample) took "nearly 60 percent". 6+ years average $11.3M. |

Conflict: the pre-arbitration share ranges from 7% to 15% depending on definition.
- Clemens groups strictly by service time. Posted NPB/KBO free agents with zero MLB service, and pre-free-agency extensions, then count as pre-arbitration or arbitration.
- Becker and Sawchik use contract status.
- My own RosterResource cross-check (D2) undercounts pre-arbitration call-ups and gives an implausibly low pre-arbitration share (about 5%), so it is not used.

### 5C. Revenue

| Item | Value | Source |
|---|---|---|
| League revenue 2024 | **$12.1B**, a record (2023: $11.6B; 2014: about $9B) | Forbes, via CBS Sports (P3) [press] |
| League revenue 2025 | NOT CONFIRMED (no Forbes or Sportico figure found). Team sponsorship revenue alone was reported at $2.05B (+9%), but I did not read that report and do not use it. | - |
| Payroll as a share of revenue | 2024: $5.16B / $12.1B = about 43% (derived). Payroll excludes benefits, bonuses and minor leagues. P4 describes MLB's view that the player share "typically hovers" around 50-50. | P2, P3, P4 |
| Revenue growth since 2015 | MLB 2.7% a year vs NBA 10.7%, NFL 7.5%, NHL 6.8% | P4 [press, league-owned, during CBA talks] |

---

## 6. Exits and career length

### 6.1 Average career length

| Source | Population | Mean | Other |
|---|---|---|---|
| Witnauer et al. 2007 (J19) | Position players, debuts 1902-1993 | **5.6 years** | 20% single-year careers; exit risk at least 11% at every career point |
| [data] D3: distinct seasons with at least 1 MLB game, by debut decade | Hitters debuting 1990-99 (n=884) | **7.01 seasons** (median 6) | 14% one season; 51% 6+ seasons; 31% 10+ |
| same | Pitchers debuting 1990-99 (n=995) | **5.69** (median 4) | 21% one season; 41% 6+; 23% 10+ |
| same | Hitters debuting 2000-09 (n=913) | 6.24 (median 5) | 16% one season; 47% 6+; 26% 10+; 2% still active after 2021 |
| same | Pitchers debuting 2000-09 (n=1,163) | 5.30 (median 4) | 18% one season; 38% 6+; 17% 10+ |

Role is assigned by career games pitched (over 50% = pitcher). The 2010-16 debut cohort is still censored (26-29% active after 2021), so it is not shown.

### 6.2 Age at final MLB season [data] D3

Careers whose last MLB season fell in 2000-2021, so at least 4 later seasons (through 2025) confirm the end.

| Group | n | Mean (SD) | p10 | p25 | Median | p75 | p90 | 35+ | 38+ |
|---|---|---|---|---|---|---|---|---|---|
| All hitters | 2,153 | 30.5 (4.4) | 25 | 27 | 30 | 34 | 36 | 20% | 6% |
| All pitchers | 2,636 | 29.5 (4.4) | 25 | 26 | 29 | 32 | 36 | 14% | 5% |
| Hitters with 6+ seasons | 1,052 | 33.7 (3.3) | 29 | 31 | 34 | 36 | 38 | 40% | 13% |
| Pitchers with 6+ seasons | 1,003 | 33.4 (3.6) | 29 | 31 | 33 | 36 | 38 | 34% | 14% |
| All players with 10+ seasons | 1,107 | 35.6 (2.8) | 32 | 34 | 35 | 37 | 39 | 63% | 24% |

Era shift, careers of 6+ seasons: those ending 2000-09 (n=890) had a mean final age of 34.0 (median 34; 42% aged 35+). Those ending 2012-21 (n=981) had a mean of 33.1 (median 33; 33% aged 35+).

### 6.3 Exit hazard by age [data] D3

Each cell is n player-seasons / share that were the player's final MLB season (through 2025).
- "Regular" = 300+ PA for hitters, 100+ IP for pitchers (mostly starters).
- The 60-game 2020 season contributes no regular seasons.

| Age | Hitters, all: 2000-09 | Hitters, all: 2012-21 | Hitters 300+ PA: 2000-09 | Hitters 300+ PA: 2012-21 | Pitchers, all: 2000-09 | Pitchers, all: 2012-21 | Pitchers 100+ IP: 2000-09 | Pitchers 100+ IP: 2012-21 |
|---|---|---|---|---|---|---|---|---|
| 25 | 584 / 11% | 660 / 11% | 218 / 1% | 215 / 1% | 647 / 15% | 838 / 14% | 141 / 1% | 116 / 1% |
| 27 | 551 / 13% | 663 / 16% | 237 / 2% | 237 / 2% | 571 / 17% | 814 / 18% | 132 / 2% | 129 / 1% |
| 29 | 484 / 13% | 514 / 17% | 237 / 1% | 216 / 2% | 443 / 19% | 605 / 19% | 119 / 4% | 112 / 3% |
| 30 | 433 / 14% | 433 / 16% | 227 / 1% | 199 / 1% | 399 / 17% | 500 / 19% | 106 / 3% | 95 / 1% |
| 31 | 394 / 14% | 374 / 19% | 217 / 2% | 199 / 4% | 335 / 20% | 411 / 21% | 75 / 3% | 80 / 2% |
| 32 | 359 / 18% | 319 / 18% | 177 / 1% | 147 / 4% | 288 / 23% | 320 / 24% | 62 / 3% | 62 / 6% |
| 33 | 314 / 21% | 256 / 27% | 164 / 5% | 121 / **11%** | 245 / 23% | 269 / 28% | 49 / 6% | 54 / 7% |
| 34 | 243 / 21% | 208 / 36% | 123 / 3% | 85 / **14%** | 196 / 23% | 191 / 27% | 43 / 9% | 48 / 10% |
| 35 | 218 / 22% | 140 / **41%** | 107 / 4% | 56 / **20%** | 161 / 25% | 147 / 34% | 28 / 7% | 32 / 9% |
| 36 | 165 / 35% | 99 / 42% | 75 / 9% | 44 / 18% | 131 / 25% | 104 / 43% | 30 / 0% | 20 / 25% |
| 37 | 119 / 30% | 62 / 44% | 58 / 14% | 25 / 20% | 104 / 30% | 68 / 34% | 26 / 8% | 14 / 21% |
| 38 | 79 / 38% | 36 / 44% | 40 / 22% | 16 / 6% | 79 / 28% | 44 / 50% | 19 / 11% | 8 / 50% |
| 39 | 43 / 40% | 20 / 60% | 19 / 0% | 9 / 44% | 57 / 37% | 23 / 48% | 17 / 24% | 6 / 33% |
| 40 | 28 / 54% | 12 / 58% | 16 / 50% | 7 / 57% | 37 / 41% | 15 / 40% | 13 / 15% | 2 / 0% |

Read: in 2012-21 a regular hitter aged 33-35 had an 11-20% chance that this was his last season, vs 3-5% in 2000-09. Among all hitters at 34-35 the share is 36-41% vs 21-22%. Cells with n under about 30 are noisy.

### 6.4 Free-agent market exits [data] D1

Share of free agents who signed nothing during the following season, pooled over five offseasons:
- 6-9% at ages 28-32; 11% at 33; 17% at 34; 22% at 35; 36% at 36; 31% at 37+.
- At 36+ with negative platform WAR: **57%**. At 34-35 with negative WAR: 37%.
- Players with positive WAR rarely go unsigned (0-19%; see the matrix in 3C).

"No contract" mixes voluntary retirement, failing to find a job, and signing abroad.

### 6.5 Hazard by tenure (Groothuis & Hill, J20)

Annual exit hazard for hitters, 1990-2004, by year of MLB tenure:

| Tenure year | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| White | .182 | .130 | .109 | .101 | .130 | .070 | .069 | .151 | .174 | .128 | .212 | .276 | .247 | .375 | .303 |
| Black | .115 | .129 | .099 | .070 | .101 | .093 | .102 | .145 | .138 | .125 | .118 | .119 | .231 | .054 | .194 |
| Foreign-born Latino | .201 | .142 | .129 | .064 | .071 | .069 | .106 | .040 | .129 | .080 | .087 | .146 | .194 | .045 | .111 |

The paper describes the shape as U-shaped: the hazard declines for the first seven years, then climbs. Covariate effects are in 1D.

### 6.6 Released vs voluntarily retired

- **NOT CONFIRMED.** No study, peer-reviewed or analytics, that splits MLB career endings into release or non-signing vs voluntary retirement was found.
- Partial evidence:
  - In-season releases: 9.7% of matched control pitchers vs 23.6% after elbow ligament reconstruction, among retired pitchers (J21).
  - Witnauer 2011 (P13, *USA Today Magazine* essay): exit "often is involuntary". It cites an unnamed study in which "45.2% indicated forced retirement" of retired players. The original study is NOT CONFIRMED.
  - O'Neill (A1): 23.1% of contract-year observations for free-agent hitters (2006-2011) were retirements.
  - KBO 2025: 3 of 30 free-agent-eligible players had already retired before the filing window (P11).
- Groothuis & Hill (J20) show that exits are strongly predicted by performance (games, slugging, strikeouts, ERA, K/BB) and by age at fixed performance. This is consistent with mostly market-driven exits, but it does not measure voluntariness.

---

## 7. KBO and NPB: how many eligible players use free agency (measured behaviour only)

| League / year | Eligible | Exercised | Source |
|---|---|---|---|
| NPB, 2025 offseason | **104** free-agent-eligible: 18 newly domestic-eligible and 12 newly overseas-eligible, plus 20 domestic and 54 overseas already eligible (announced 2025-10-22) | **8 declared**: 4 domestic, 4 overseas (announced 2025-11-12), about 8% of eligible (derived) | Baseball King (P9, P10) [press] |
| KBO, 2025 offseason (for 2026) | **30** eligible: 13 first-time, 11 re-qualified, 6 who held eligibility without filing; 3 already retired | **21 approved** (6 A, 8 B, 7 C grade), 70% (derived) | OSEN (P11), Wikitree (P12) [press] |

Contract sizes and lengths for KBO/NPB free agents: NOT CONFIRMED (no systematic tally found).

---

## 8. Conflicts between sources

1. **Is $/WAR linear in quality?**
   - Swartz 2017 (actual WAR, relievers included): "still linear" (A4).
   - Edwards 2020 (relievers excluded): 0-2 WAR players about 25% cheaper per win (A6).
   - Clemens 2026: 2+ WAR $11.8M vs 0-1 WAR $7.4M, and the gap widened in 2024-26 (A2).
   - Projected vs actual WAR and the treatment of relievers drive much of the difference.
2. **Arbitration share.** The 40/60/80 rule (A8) vs about 21/33/55% for batters in 2013-21 (A7, derived). Relievers fit the rule; position players and starters do not.
3. **Pre-arbitration salary share.** 7.2% (mid-2010s, P6), 7.3% (2025, contract status, A12), 9.4% (2025, P4), 14-15% (2019-25, service-time grouping, A13). These are definitional differences (5B).
4. **Median salary.** AP: median $1.4M on 2026 Opening Day rosters (P1). MLB.com: "the median salary is the league minimum" and 57% of players at or near the minimum (P4). The populations differ (Opening Day roster vs everyone paid in a season); P4 is league-owned media during CBA talks.
5. **Does age lower free-agent pay at fixed performance?**
   - Congdon-Hohman & Lanning: age insignificant for first-year salary in 1998-2010 (W2).
   - Hakes & Turner: salaries drift down near the end of careers, but not significantly (J8).
   - My 2022-2026 count: median AAV per platform WAR falls only about 25% from 30-31 to 36+, while median years fall from 4 to 1.5 and the median guarantee per WAR falls about 75% (2C).
   - Reconciliation: age is priced mostly through contract length, which first-year-salary regressions do not see.
6. **Contract-year effect.** OLS and simple comparisons show none or a negative effect. Fixed effects with retirement controls show +4 to +7% OPS (J17, A1). Krautmann & Solow show +8 OPS points in the walk year (J15). Maxcy et al. reject strategic performance (J14).
7. **Free-agent exploitation ratio.** 0.21 for free agents in 2001-11 (J5) vs a mean of -0.30 (median 0.12) in 2012-19 (W1, an undergraduate thesis using a different revenue model). Both find rookies at about 0.87-0.89.
8. **Career length.** 5.6 years (J19: position players, 1902-93 debuts, life table) vs a mean of 7.0 seasons for hitters debuting in 1990-99 (D3: seasons with at least 1 game, not years of service). The era and the counting unit differ.
9. **Size of each free-agent market.** 130 (2024-25) and 142-144 (2025-26) guaranteed MLB free-agent deals by March (A10, A11) vs 181 and 193 first MLB contracts by season's end (D1). The difference is scope and timing: in-season signings, non-tenders and posted players.
10. **Pitchers vs hitters.** Hitters got younger (27.9 PA-weighted in 2025, the lowest of the period), while pitchers are older than at any time since 1995 (29.0). Clemens attributes this to workload protection of young pitchers (A14). A model that ages everyone alike will not reproduce both.

## 9. Gaps (NOT CONFIRMED)

- Scully (1974) and Hakes & Sauer (2006) own numbers: texts not readable here (JSTOR; AEA PDF returned 403).
- Lehn (1982), Krautmann, von Allmen & Berri (2009) MLB figures, Martin et al. (2011), White & Sheldon (2014): abstract only or not accessed.
- A published $/WAR breakdown by player age or contract length (FanGraphs, Baseball Prospectus, academic). Only my descriptive [data] table (2C) is available.
- A systematic split of career endings into release vs voluntary retirement.
- MLB league revenue for 2025.
- Cot's Contracts (blocked by bot protection) and Spotrac (paywalled, HTTP 402) payroll tables. RosterResource and AP were used instead.
- FiveThirtyEight 2018 on 30-something free agents (page redirects; archive not reachable).
- The MLB Trade Rumors contract tracker is paywalled. The free-agent counts here come from the FanGraphs tracker.
- KBO and NPB free-agent contract sizes and lengths (only eligibility and exercise counts).

---

## 10. References

**Peer-reviewed**
- J1 Scully, G.W. (1974). Pay and Performance in Major League Baseball. *American Economic Review* 64(5):915-930. (Not read; described via W1.)
- J2 Krautmann, A.C. (1999). What's Wrong with Scully-Estimates of a Player's Marginal Revenue Product. *Economic Inquiry* 37(2):369-381. https://ideas.repec.org/a/oup/ecinqu/v37y1999i2p369-81.html
- J3 Bradbury, J.C. (2013). What Is Right With Scully Estimates of a Player's Marginal Revenue Product. *J. Sports Economics* 14(1):87-96. DOI 10.1177/1527002511418981
- J4 Krautmann, A.C. (2013). Reply to Bradbury. *J. Sports Economics* 14(1):97-105. DOI 10.1177/1527002511419299
- J5 Blair, R.D., Humphreys, B.R. & Pyun, H. (2017). Monopsony Exploitation in Professional Sport: Evidence from Major League Baseball Position Players, 2000-2011. *Managerial and Decision Economics* 38(5):676-688. Working paper: https://ideas.repec.org/p/wvu/wpaper/15-48.html
- J6 Krautmann, A.C., von Allmen, P. & Berri, D. (2009). The Underpayment of Restricted Players in North American Sports Leagues. *Int. J. Sport Finance* 4(3):161-175. https://ideas.repec.org/a/jsf/intjsf/v4y2009i3p161-175.html
- J7 Hakes, J.K. & Sauer, R.D. (2006). An Economic Evaluation of the Moneyball Hypothesis. *J. Economic Perspectives* 20(3):173-186. DOI 10.1257/jep.20.3.173 (abstract via https://ideas.repec.org/a/aea/jecper/v20y2006i3p173-186.html)
- J8 Hakes, J.K. & Turner, C. (2011). Pay, productivity and aging in Major League Baseball. *J. Productivity Analysis* 35(1):61-74. Working paper 2007: https://mpra.ub.uni-muenchen.de/4326/1/MPRA_paper_4326.pdf
- J9 Kahn, L.M. (1993). Free Agency, Long-Term Contracts and Compensation in Major League Baseball: Estimates from Panel Data. *Review of Economics and Statistics* 75(1):157-164. https://ideas.repec.org/a/tpr/restat/v75y1993i1p157-64.html
- J10 Krautmann, A.C. & Oppenheimer, M. (2002). Contract Length and the Return to Performance in Major League Baseball. *J. Sports Economics* 3(1):6-17. DOI 10.1177/1527002502003001002
- J11 Link, C. & Yosifov, M. (2012). Contract Length and Salaries: Compensating Wage Differentials in Major League Baseball. *J. Sports Economics* 13(1):3-19. DOI 10.1177/1527002510396984
- J12 Solow, J.L. & Krautmann, A.C. (2020). Do You Get What You Pay for? Salary and Ex Ante Player Value in Major League Baseball. *J. Sports Economics* 21(7):705-722. DOI 10.1177/1527002520930259
- J13 Terry, R.P., McGee, J.E. & Kass, M.J. (2018). The not-so-free agent: Non-performance factors that contribute to free agent compensation premiums. *Sport Management Review* 21(2):189-201. DOI 10.1016/j.smr.2017.06.006
- J14 Maxcy, J.G., Fort, R.D. & Krautmann, A.C. (2002). The Effectiveness of Incentive Mechanisms in Major League Baseball. *J. Sports Economics* 3(3):246-255. DOI 10.1177/1527002502003003002
- J15 Krautmann, A.C. & Solow, J.L. (2009). The Dynamics of Performance Over the Duration of Major League Baseball Long-Term Contracts. *J. Sports Economics* 10(1):6-22. DOI 10.1177/1527002508327382
- J16 Ahlstrom, D., Si, S. & Kennelly, J. (1999). Free-agent performance in Major League Baseball: Do teams get what they expect? *J. Sport Management* 13(3):181-196. DOI 10.1123/jsm.13.3.181
- J17 O'Neill, H.M. (2013). Do Major League Baseball Hitters Engage in Opportunistic Behavior? *Int. Advances in Economic Research* 19(3):215-232. DOI 10.1007/s11294-013-9419-y
- J18 Lehn, K. (1982). Property Rights, Risk Sharing, and Player Disability in Major League Baseball. *J. Law & Economics* 25(2):343-366. DOI 10.1086/467020 (not read)
- J19 Witnauer, W.D., Rogers, R.G. & Saint Onge, J.M. (2007). Major League Baseball Career Length in the 20th Century. *Population Research and Policy Review* 26(4):371-386. DOI 10.1007/s11113-007-9038-5
- J20 Groothuis, P.A. & Hill, J.R. (2008). Exit Discrimination in Major League Baseball: 1990-2004. *Southern Economic Journal* 75(2):574-590. Working paper (2007): https://econ.appstate.edu/RePEc/pdf/wp0702.pdf
- J21 Erickson, B.J. et al. (2017). Reasons for Retirement Following Ulnar Collateral Ligament Reconstruction Among Major League Baseball Pitchers. *Orthopaedic J. Sports Medicine* 5(12). DOI 10.1177/2325967117745021 (abstract via Europe PMC)

**Working papers and theses**
- W1 Hyman, J.C. (2020). Monopsony Exploitation in Major League Baseball: Using Wins Above Replacement to Estimate Marginal Revenue Product. UC Berkeley Senior Honors Thesis. https://econ.berkeley.edu/sites/default/files/Economics%20Senior%20Honors%20Thesis%20-%20Jake%20Hyman.pdf
- W2 Congdon-Hohman, J. & Lanning, J.A. (2017). Beyond Moneyball: Changing Compensation in MLB. College of the Holy Cross WP 17-02. https://hcapps.holycross.edu/hcs/RePEc/hcx/HC1702-Congdon-Lanning_FreeAgencyRewards.pdf
- W3 Cuccolo, B. (2025). Decade-Long Deals: The Forces Driving MLB's Lengthening Contracts. Ursinus College honors paper. https://digitalcommons.ursinus.edu/bus_econ_hon/63
- W4 Humphreys, B.R. & Pyun, H. (2015). WVU Economics WP 15-48 (working paper of J5). https://ideas.repec.org/p/wvu/wpaper/15-48.html

**Analysis**
- A1 O'Neill, H.M. (2014-11-12). Do Hitters Boost Their Performance During Their Contract Years? SABR *Baseball Research Journal*, Fall 2014. https://sabr.org/journal/article/do-hitters-boost-their-performance-during-their-contract-years/
- A2 Clemens, B. (2026-02-24). What Are Teams Paying For A Win In Free Agency? 2026 Edition. FanGraphs. https://blogs.fangraphs.com/what-are-teams-paying-for-a-win-in-free-agency-2026-edition
- A3 Clemens, B. (2026-02-26). More Musings on What Teams Are Paying for a Win in Free Agency. FanGraphs. https://blogs.fangraphs.com/more-musings-on-what-teams-are-paying-for-a-win-in-free-agency
- A4 Swartz, M. (2017-07-11). The Recent History of Free-Agent Pricing. FanGraphs. https://blogs.fangraphs.com/the-recent-history-of-free-agent-pricing
- A5 Swartz, M. (2017-07-18). How Do Baseball Teams Discount the Future? FanGraphs. https://blogs.fangraphs.com/how-do-baseball-teams-discount-the-future
- A6 Edwards, C. (2020-01-09). Is the Cost of a Win in Free Agency Still Linear? FanGraphs. https://blogs.fangraphs.com/is-the-cost-of-a-win-in-free-agency-still-linear
- A7 Clemens, B. (2022-02-04). An Arbitration Compensation Update. FanGraphs. https://blogs.fangraphs.com/an-arbitration-compensation-update
- A8 Nicholson-Smith, B. (2011-08-02). When The 40/60/80 Arbitration Rule Doesn't Work. MLB Trade Rumors. https://www.mlbtraderumors.com/2011/08/when-the-406080-arbitration-rule-doesnt-work.html
- A9 Chamberlain, A. (with S. Dolinar) (2015-04-24). Modeling Salary Arbitration: Introduction. FanGraphs. https://blogs.fangraphs.com/modeling-salary-arbitration-introduction
- A10 Becker, J. (2025-03-07). Matrix Reloaded: Offseason Breakdown Tables Extravaganza. FanGraphs. https://blogs.fangraphs.com/matrix-reloaded-offseason-breakdown-tables-extravaganza
- A11 Becker, J. (2026-03-13). Matrix Reloaded: 2025-26 Offseason Summary. FanGraphs. https://blogs.fangraphs.com/matrix-reloaded-2025-26-offseason-summary
- A12 Becker, J. (2025-04-08). The 2025 Payrolls... And Beyond! FanGraphs. https://blogs.fangraphs.com/the-2025-payrolls-and-beyond/
- A13 Clemens, B. (2025-12-04). The Pre-Arb Bonus Pool Is A Success. FanGraphs. https://blogs.fangraphs.com/the-pre-arb-bonus-pool-is-a-success
- A14 Clemens, B. (2026-01-07). Hitters Keep Getting Younger. Pitchers Stay The Same Age. FanGraphs. https://blogs.fangraphs.com/hitters-keep-getting-younger-pitchers-stay-the-same-age
- A15 Baumann, M. (2025-11-11). They Haven't Killed off All the Old Guys Yet. FanGraphs. https://blogs.fangraphs.com/they-havent-killed-off-all-the-old-guys-yet
- A16 Lindbergh, B. (2022-12-22). Five Reasons MLB Free Agents Are Signing Longer Deals Than Ever. The Ringer. https://www.theringer.com/mlb/2022/12/22/23522591/mlb-free-agents-long-term-contracts
- A17 Polishuk, M. (2025-11-08). Players Entering Minor League Free Agency. MLB Trade Rumors (credits Baseball America's M. Eddy). https://www.mlbtraderumors.com/2025/11/players-electing-minor-league-free-agency.html

**Press**
- P1 Blum, R. (AP) (2026-04-09). AP study: MLB average salary hits a record $5.34M as the Mets lead spending again. https://www.clickondetroit.com/sports/2026/04/09/ap-study-mlb-average-salary-hits-a-record-534m-as-the-mets-lead-spending-again/
- P2 Blum, R. (AP) (2026-04-16). The Dodgers shattered MLB's spending record at $515 million in 2025, 7 times the lowest payroll. https://www.news4jax.com/sports/2026/04/16/the-dodgers-shattered-mlbs-spending-record-at-515-million-in-2025-7-times-the-lowest-payroll/
- P3 Snyder, M. (2025-01-27). MLB reports record $12.1 billion in revenues for 2024 season (citing Forbes). CBS Sports. https://www.cbssports.com/mlb/news/mlb-reports-record-12-1-billion-in-revenues-for-2024-season
- P4 Sawchik, T. (2026-06-30). Analysis: MLB's salary cap and floor proposals. MLB.com (league-owned). https://www.mlb.com/news/mlb-cba-proposal-salary-cap-floor-system-analysis
- P5 Gonzalez, A. & Rogers, J. (2022-12-23). How decade-plus MLB megadeals suddenly became a thing. ESPN. https://global.espn.com/mlb/story/_/id/35292038/how-decade-plus-mlb-megadeals-suddenly-became-thing
- P6 Baer, B. (2016-03-14). Pre-arb players make up 38% of MLB player pool but take only 7.2% of total salaries (citing C. Edwards, FanGraphs). NBC Sports. https://www.nbcsports.com/mlb/news/pre-arb-players-make-up-38-of-mlb-player-pool-but-take-only-7-2-of-total-salaries
- P7 University of Colorado (2007-07-08). Average Major League Baseball Career 5.6 Years, According To New Study. https://www.colorado.edu/today/2007/07/08/average-major-league-baseball-career-56-years-according-new-study
- P8 University of Iowa release via Newswise (date not recorded). Baseball free agents shirk, but not as much as fans think. https://www.newswise.com/articles/baseball-free-agents-shirk-but-not-as-much-as-fans-think
- P9 Baseball King (2025-10-23). 2025年度のフリーエージェント有資格選手名簿を公示. https://baseballking.jp/ns/678934/
- P10 Baseball King (2025-11-12). 東浜、則本、松葉 "90年世代"の投手がFA市場を盛り上げる！？ https://baseballking.jp/ns/680725/
- P11 이상학 (OSEN) (2025-11-05). 이미 3명이나 은퇴했는데…FA 자격 선수 30명 공시. https://osen.co.kr/article/G1112691701
- P12 Wikitree (2025-11-08). 프로 야구 눈치 싸움 시작…21명 FA 승인. https://www.wikitree.co.kr/articles/1093478
- P13 Witnauer, W.D. (2011). Making the show — and staying there. *USA Today Magazine* 140(2794) (via vLex). https://law-journals-books.vlex.com/vid/making-show-compressed-skewed-substantially-311260050

**Data (my own counts)**
- D1 FanGraphs RosterResource Free Agent Tracker, seasons 2021-2026, downloaded 2026-11-17. https://www.fangraphs.com/roster-resource/free-agent-tracker?season=2026 (change `season`).
- D2 FanGraphs RosterResource team payroll pages, 2025 and 2026, all 30 clubs, downloaded 2026-11-17. For example https://www.fangraphs.com/roster-resource/payroll/yankees?season=2026.
- D3 Lahman Baseball Database, 2026 edition, via the CRAN R package `Lahman` 14.0-0 (published 2026-02-12; Batting, Pitching and Appearances through 2025). https://cran.r-project.org/web/packages/Lahman/index.html.
- D4 Baseball-Reference, MLB Batting and Pitching Year-by-Year Averages (BatAge, PAge), accessed 2026-11-17. https://www.baseball-reference.com/leagues/majors/bat.shtml and https://www.baseball-reference.com/leagues/majors/pitch.shtml
