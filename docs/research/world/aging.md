# Aging, development and prospect outcomes: published evidence

Compiled 2026-11-16 for the physics-based player model (fictional players, ages 16 to about 40).
Every number below is tied to a source ID; the full citations are in Section 7.

**Source-type tags**
- **[PR]** peer-reviewed journal article
- **[WP]** academic working paper, preprint or manuscript
- **[BRJ]** SABR *Baseball Research Journal* (an edited research journal, not an academic peer-reviewed one)
- **[AN]** analytics article (FanGraphs, The Hardball Times (THT), Baseball Prospectus (BP), blogs, Substack)
- **[PRESS]** league media or news outlet
- **[PRAC]** practitioner or industry write-up (training or coaching data)

**Method terms used in the tables**
- **Delta method.** Take every player who appears in two consecutive seasons, compute his year-2 minus year-1 change, average the changes within each age pair (usually weighted by the harmonic mean or the minimum of the two playing-time totals), then chain the pairs into a curve.
- **Survivor bias.** Players who had an unlucky or poor year often do not return. The observed pairs therefore over-represent lucky year-1 seasons, which makes decline look larger and improvement look smaller. Lichtman (A2) adds "phantom" seasons for players who did not return. Judge (A6, A7) argues the bias may be small, or may even run the other way.
- **Fixed-effects or quadratic regression.** Each player gets his own intercept, and age enters as a curve. Fair (P1) uses two different quadratics joined at the peak. Bradbury (P2) uses a single symmetric parabola.
- **GAM.** Generalized additive model: a smooth, non-symmetric age curve fitted across players.
- **Cross-sectional.** The average of everyone observed at each age. It mixes aging with selection (who is still playing), so it is not a within-player curve.
- **Derived.** My own arithmetic from the published numbers.

**Headline numbers.** Each is restated from the tables below; see those rows for method and caveats.
- **Hitting peak.**
  - 26-28 by delta, imputation and fixed-effects methods (A1, A2, A6, P1, P4).
  - About 29 in long-career quadratic models (P2).
  - Better players peak about 2 years later than marginal ones (P3).
- **Hitting decline (OPS, Fair 2025).** About -0.5%/yr at 31, -1.0%/yr at 34, -1.4%/yr at 37 and -1.9%/yr at 40. About 12% total by 40.
  - Survivor-corrected delta method: -1.3 to -4.7 runs per 500 PA per year from 29 to 37 (A2).
- **Pitching (ERA).** Peak 26.5-26.7 (P1). Decline about +0.7%/yr at 31 to +2.0-2.2%/yr at 40.
- **Fastball velocity.** MLB curves peak about 21. About -4 mph from 21 to 38, or about -0.2 mph/yr on average (A19, A21). Slow before 28-30, faster after.
  - Amateur gains: about 5-6 mph/yr from 13 to 16, then 2-3 mph/yr from 16 to 18 (A57).
- **Sprint speed.** -0.12 to -0.17 ft/s per year at nearly every age from 22 to 36, mean -0.14 (A27).
- **Bat speed.** Flat until about 28-31, then about -0.3 mph/yr (A28, A29).
- **Exit velocity.** Peak about 27-29 (A31, A33). Informally about -0.2 mph/yr to 30 and -0.3 after (A32).
- **Plate discipline.** Walk rate holds or improves into the 30s. O-Contact% -3.5 points over 28-32 and -4.4 points over 33-37 (A15).
- **Defense.** Shortstop peak 24-28, losing 5-9 plays a year after (A37). DRS peak 26 (A38).
- **Draft.** Share of signed picks who reach MLB: round 1 about 67%, round 2 49%, round 3 40%, rounds 4-5 33-35%, rounds 6-10 20%, rounds 11-15 13%, rounds 16-20 10% (R1).
- **Prospect grades.** 50 FV hitters: 23% wash out, 53% project as regulars or better three years later (A42).
- **MLB debut age.** Average about 25 in 2015-2019 (M4).
- **Dominican signees.** About 5% reach MLB (M5).

---

## 1. Whole-player aging curves

### 1A. Academic studies

| Study | Type | Sample and metric | Method / bias handling | Peak age | Rates of change as published |
|---|---|---|---|---|---|
| Fair 2008 (P1) | PR | 1921-2004. Players with at least 10 "full-time" seasons: batters with 100+ games (441 players, 5,596 obs), pitchers with 450+ outs (144 players, 1,809 obs). Unadjusted OPS, OBP, ERA. | Nonlinear fixed effects with a player intercept. Separate quadratics before and after an estimated peak. Part-time (possible injury) seasons dropped, so this is "injury-free career player" aging. | OPS 27.59 (SE 0.23). OBP 28.30. ERA 26.54 (SE 1.40). | OPS %/yr at age 22, 25, 28, 31, 34, 37, 40: +2.28, +1.06, -0.05, -0.44, -0.82, -1.21, -1.59. ERA %/yr at the same ages (positive = worse): -1.69, -0.57, +0.24, +0.73, +1.22, +1.72, +2.21. Loss by age 40: OPS 9.8%, OBP 5.6%, ERA 14.9%. Players who debuted in 1966 or later: OPS 6.5%, OBP 4.0%, ERA 12.9% (the stability test was mixed). |
| Fair, updated April 2025 (P1u) | WP (same model) | 1921-2023. 582 batters (7,361 obs) and 178 pitchers (2,245 obs). | Same model. Runs the delta method on the same sample for comparison. | OPS 27.49 (SE 0.18). OBP 27.66 (SE 0.17). ERA 26.66 (SE 1.02). | OPS %/yr at age 22, 25, 28, 31, 34, 37, 40: +2.14, +0.97, -0.08, -0.53, -0.98, -1.43, -1.88. OPS points below peak at age 22, 25, 31, 34, 37, 40: -.047, -.010, -.007, -.025, -.054, -.094. ERA %/yr at age 22, 25, 28, 31, 34, 37, 40: -2.59, -0.92, +0.20, +0.66, +1.11, +1.57, +2.03. ERA runs above peak at age 22, 31, 34, 37, 40: +0.215, +0.051, +0.146, +0.290, +0.483. Loss by age 40: OPS 11.8%, OBP 7.6%, ERA 13.6%. Improvement before the peak is steeper than the decline after it. Debut cohorts up to 1965 vs 1966 on: no significant change for OBP or OPS; ERA rejected at 95% but not 99%. |
| Fair 2025, delta comparison (P1u, Table 2) | WP | Same 582 batters, OBP | Delta method vs fixed effects | - | OBP decline at 34, 37, 40: delta method 0.88%, 1.25%, 3.24% vs fixed effects 0.63%, 0.93%, 1.23%. Fair concludes the delta method overstates old-age decline. |
| Bradbury 2009 (P2), with his BP summary of 2010 (A3) | PR (+AN) | 1921-2006. Hitters with 10+ seasons and 5,000+ PA; pitchers with 4,000+ batters faced. Seasons with 300+ PA or 200+ BF. Ages 24-35. | Panel regression, symmetric quadratic in age. Controls for career-average ability, park and run environment (z-scores). A sample with only 1,000+ PA gave "virtually identical" peaks. | About 29 for both hitters and pitchers; "possibly 30 for hitters in modern times". | Peak age by skill, hitters: linear weights 29.4, OBP 30.0, SLG 28.6, AVG 28.4, walk rate 32.3, 2B+3B rate 28.3, HR rate 29.9. Pitchers: ERA 29.2, strikeout rate 23.6, walk rate 32.5, HR rate 27.4. The 23.6 lies below the 24-35 age window used, so it is extrapolated (a reader pointed this out in the comments). Hall of Famers peaked "a little earlier", if anything. Per-year rates: not found in the accessible text. |
| Hakes & Turner 2011 (P3) | PR | 1985-2005 position players. OPS indexed to league and position. Players split into quintiles by peak ability (third-best season). | Quadratic age paths estimated within each quintile. Shows that pooling all players biases the peak upward. | Quintiles 1-3: 25.6-26.8. Quintile 4: 27.5. Quintile 5 (best): 28.2. A pooled regression puts the peak 1.5 years above the median quintile peak. | Development and decline are more pronounced for the highest-peak players. Within-career variation is smaller than between-player variation. |
| Nguyen & Matthews 2024 (P4) | PR | Lahman data, debuts in 1985 or later, 2,323 players, seasons with 100+ PA. OPS (transformed), ages 21-39. | Treats dropout as missing data. Multiple imputation (MICE "2l.norm", 5 imputations). | 26 with imputation vs 30 when dropout is ignored. | Decline is steeper once missing seasons are imputed. No numeric rate in the text. |
| Schulz, Musa, Staszewski & Siegler 1994 (P6), as summarized by Fair | PR | 235 batters and 153 pitchers active in 1965 | Averages by age group | About 27 for batters, about 29 for pitchers | The authors note that these averages cannot give decline rates because of selection bias. |
| Schall & Smith 2000 (P5) | ASA magazine (*CHANCE*) | Citation verified; text not accessed | Secondary description only (P4): a logit model of career survival, and first-year performance vs career length | not found | not found |
| Albert 2002 (W1) | WP | Hitters with 5,000+ PA born 1910-1969 (473). Linear weights per PA. | Bayesian exchangeable model: a quadratic for each player | Median individual peak by birth decade 27.1-29.8. Lower-upper quartile: 1930s 25.6-28.5; 1960s 27.9-32.0. | Curvature only (not a yearly rate) |

### 1B. Analytics studies (delta method and its critics)

| Study | Type | Sample | Method / survivor handling | Peak age | Yearly change as published |
|---|---|---|---|---|---|
| Lichtman 2009, Parts 1 and 2 (A1, A2) | AN | 1950-2008, all hitters. Linear-weights runs per 500 PA. | Delta method, pairs weighted by average PA. Part 2 corrects for survivors: each player who did not return gets a phantom next season: his last three seasons weighted 3/4/5 (oldest to newest), regressed with 500 PA of league average for his age, minus 5 runs, with the same PA as his year 1. | 27-28 after correction (plateau 26-28 before correction) | Corrected runs/500 PA by age pair: 21-22 +14.2, 22-23 +5.3, 23-24 +6.1, 24-25 +2.8, 25-26 +2.1, 26-27 +1.1, 27-28 0.0, 28-29 -1.9, 29-30 -1.3, 30-31 -1.8, 31-32 -2.3, 32-33 -2.3, 33-34 -3.8, 34-35 -4.1, 35-36 -4.4, 36-37 -4.7, 37-38 -3.9, 38-39 -6.8, 39-40 -4.9. Cumulative vs peak: -32.3 at 21 and -42.2 at 40 (uncorrected: -48.2 at 40). By era: at age 38, post-1980 players were 23 runs off peak vs 46 runs before 1980. Long careers (10+ years, 5,000+ PA): peak 28, plateau 27-30. Players with 1,000+ PA: nearly the same as all players. |
| Tango, "Aging Patterns", 2002 (A18) | AN | 1919-1999 hitters | Peak estimated with each player's final season removed (the final season is biased low) | About 27 (26 if final seasons are kept) | Level relative to peak: 1.000 at 27, about 0.939 at 33, 0.891 at 36, 0.843 at 40. Walk rate improves almost every year. Strikeouts are lowest around 29. Best HR ratio at 27. Success on balls in play drops almost immediately. SB success peaks at 24. The 1919-1979 and 1979-1999 curves are essentially the same. |
| Zimmerman, December 2013 (A8, A9) | AN | MLB; Triple-A 2006-2013 | Delta method, harmonic-mean PA weights; pre- vs post-2006 | MLB 2006-2013 wRC+: flat until about 26, then decline ("no peak"). Triple-A wRC+: peak at 25. | Recent decline rate is "almost the same as the pre-PED" rate (no number given). The share of matched seasons aged 25 or younger fell from 23.2% (1963-72) to 13.1% (1993-2002). |
| Zimmerman 2020 (A11) | AN | 2005-12 vs 2012-19 | Delta method, harmonic-mean PA weights, league-wide trends removed | wRC+ about 26 | Hitters age faster after 30, driven mainly by K% rising faster after 30. ISO flat, then declining from the late 20s. BB% roughly flat. BABIP unchanged. Values appear only in charts. |
| Judge 2020 (A6) | AN | 1977-2016 for fitting, 2017-2019 held out; OPS relative to league; ages 21-41 | Compares delta, Lichtman-corrected delta, and a GAM across players, on out-of-sample error | Delta 26. Corrected delta 26. GAM about 27. Bradbury-style parabola 29, which Judge attributes to its forced symmetry. | Plain delta: "almost 300 points" of OPS lost from peak to 41 (implausible). Corrected delta: total range 99 OPS points. GAM: range 95 points. Out-of-sample mean absolute error: delta 0.089, corrected 0.085, GAM 0.084. The delta method is worst at ages 36+. Lichtman's correction is roughly +20 OPS points per year of rise up to 26/27 and -8 points per year from 28. Judge also notes that early arrivals are better players, so the delta peak (26) marks when league-wide performance peaks rather than the average player. |
| Judge 2020b (A7) | AN | Simulation of ages 31-35 | Monte Carlo | - | Regressions on survivors only understate aging by about 40% when dropouts are a truncated tail; the bias is negligible when dropouts are a random mixture. |
| Carleton 2010 (A5) | AN | 997 hitters whose careers ran after 1980 and ended before 2009; seasons with 200+ PA | Mixed model, grouped by debut age and by the last "hurdle" cleared (1st year, age 27, age 31) | 24-31, depending on debut age and career length. Debut at 22 or younger and lasted past 31: peak 31. Debut at 25 and lasted past 31: 28. Lasted only past 27: 26 (debut 22 or younger), 27 (debut 24), 29 (debut 26). | Survival tables are in Section 4D. |
| Gutwein 2021 (A16) | AN | Ten seasons before 2021; 150+ PA in both seasons | Delta on wRC+ | About 26 | For hitters 35+, the curve projected roughly -8 to -9 wRC+ per year. Average change at age 37: BB% +0.5, K% +0.75 (points), ISO -0.015, BABIP -0.010. |
| Bonney 2015 (A17) | AN | Retrosheet 1961-2013 | Delta method; each hitter's second season measured against the same pitchers; final seasons dropped | - | The 1991-2005 and 2006-13 curves are statistically indistinguishable. 1961-90 vs 1991-2013 diverge from about age 30-31, with the older era declining faster. |
| Wyers 2009 (A4) | AN critique of P2 | 27 players who were 27 in 2006 and 29 in 2008 | Paired comparison | - | -.014 OPS from 27 to 29. About 48% of players were out of the league before 29. |
| FanGraphs Library, Slowinski 2010 (A59) | AN reference | - | Rule of thumb | - | About -0.5 WAR per year after 30 |
| nana_metrics 2026 (A35, A36) | AN (blog) | 3,204 hitters and 1,636 starting pitchers, 2000-2025 | GAM | Hitters: wRC+ 27 (26-28), OBP and SLG about 27, BB% 29, K% 27, ISO 28. Starters overall 25-27 (ERA 25, FIP 26, xFIP 27). | No yearly rates |

### 1C. Pitchers as a whole (in addition to Fair and Bradbury above)

| Finding | Value | Method / sample | Source |
|---|---|---|---|
| FIP change, ages 21 to 38 (all pitchers) | +1.7 | Delta method, harmonic-mean IP, 2002-2011 | Zimmerman 2012 (A21) [AN] |
| FIP change, ages 23 to 29 | Starters +0.54; relievers +1.20 | Same | Petti 2012 (A19) [AN] |
| Pitchers who kept their velocity within ±0.5 mph | FIP falls at first, then returns to its age-21 value; roughly flat from 21 to 38, best years 25-30 | Same | A21 [AN] |
| Curve shape after 2006 | "Constant and then down" | Delta method | Zimmerman 2013 (A8) [AN] |
| Curve shape, 2012-19 | ERA- no longer shows an immediate steady drop; more curved | Delta method | A11 [AN] |
| Starter peak | 25-27 | GAM | A36 [AN, blog] |

---

## 2. Component aging (mapped to the model's physical abilities)

### 2A. Fastball velocity

| Age range | Finding | Method / sample | Source |
|---|---|---|---|
| 13-18 (amateur, longitudinal) | Mean of 25 future MLB pitchers (154 Perfect Game readings): 72 mph at 13, 78 at 14, 84 at 15, 89 at 16, 92 at 17, 94 at 18. About 90% of peak reached around 15.4. Peak "typically" at 18-20. | Individual tracking from public showcase data; small sample | Eisenmann 2026a (A56) [PRAC/WP, not peer-reviewed] |
| 13-18 (amateur, cross-sectional) | 50th percentile: 65 mph at 13 to 87 mph at 18. Gains of about 5-6 mph/yr from 13 to 16 and 2-3 mph/yr from 16 to 18. Percentile bands are 4-5 mph apart. | Scraped showcase data (service not named), cross-sectional; showcase attendees are an above-average group | Eisenmann 2026b (A57) [PRAC] |
| 10-22 (general youth, cross-sectional) | Males, mean ± SD: 18U (mean age 16.4, n=35) 73.36 ± 4.31 mph; 21U (18.5, n=14) 78.10 ± 3.69; 15U (14.3) 66.96 ± 4.45; 13U (12.5) 55.40 ± 6.79 | 150 male athletes; not an elite sample | Tremblay et al. 2022 (P7) [PR] |
| 21-38 (MLB, all pitchers) | Within 0.5 mph of peak through 28; about -1 mph by 29, with the loss accelerating each year after; total career loss 3.75 mph (cumulative curve) | Delta method; BIS fastball velocity (all fastball types), 2002-2011; harmonic-mean IP weights | Petti & Zimmerman 2012 (A19) [AN] |
| 21-38 | About -4 mph (3.7) from 21 to 38, about -0.22 mph/yr on average (derived). Pitchers who held velocity: -0.3 mph over the same span. Among year pairs within ±0.5 mph, 60% were declines and 40% flat or gains. | Same | A21 [AN] |
| Starters vs relievers | Average starter: about -2 mph from peak by 30. Relievers: do not lose 1 mph from peak until 32. | Same | Petti 2012 (A20) [AN] |
| Cross-sectional, survivors | Average fastball 91.1 mph at 24 vs 89.4 at 34 | Pitchers with 40+ IP in each year, 2002-2011 | Petti 2012 (A22) [AN] |
| 19-44 cross-sectional (PITCHf/x four-seam) | Peak at 21: 93.79 mph (n=57 seasons). 25: 93.02. 30: 92.30. 35: 91.23. 36: 90.90. 39: 90.86. 40: 89.77. 41: 89.08. Derived: -0.17 mph/yr from 21 to 30 and -0.23 mph/yr from 30 to 36. Four-seam movement roughly constant from 19 to 44. | Mean by age; pitchers with 300+ career IP 2007-2018 (n=537). Includes survivor selection and the league-wide rise in velocity. | Platt 2019 (A24) [AN, community] |
| 24-33 (conflicting early study) | About +1.5 mph until 28-29, then "just over four mph" lost over the next five years | PITCHf/x 2007 to early 2008; 143 pitchers with 100+ fastballs in both years; preliminary, survivor-biased | Kalk 2008 (A23) [AN] |
| Statcast era, 23-36 | Essentially no loss from 23 to 27; -0.21 mph/yr at 30-36 | Delta within pitch type, 2021-2023, 1,123 pitchers. Undated methodology page from a small analytics site; low confidence. | PitchIQ (A26) [AN] |
| Statcast era, 21-36 (four-seam) | Table values reach -1.8 mph by 36 relative to 21. The article calls them "year-to-year" changes, but readers argued they must be cumulative, so the presentation is ambiguous. | Delta (Zimmerman data), Statcast 2015-2017 | Sawchik 2018 (A25) [AN] |
| Effect on strikeouts | Starter K/9 falls about 0.55 per 1 mph lost (age-33 season) | A19 | [AN] |
| Context | 2026 MLB average fastball "north of 94 mph" (was "under 92" about 17 years earlier) | Press | AP 2026 (M8) [PRESS] |

A peer-reviewed or rigorous Statcast-era, per-age delta curve for velocity (mph/yr by age, split by starters and relievers): **not found**.

### 2B. Spin and movement

| Finding | Source |
|---|---|
| Four-seam spin falls by about 18.8 rpm and two-seam spin by about 105.9 rpm by ages 36 and 33 respectively (table presentation ambiguous; see 2A). The article's conclusion: "pitchers generally lose spin as they age," but the four-seamer loses relatively less spin than velocity (average about 93 mph and 2,200 rpm). | Sawchik 2018 (A25) [AN] |
| Spin "declines mildly" (no rate given). Release point, approach angle and horizontal break drift by less than about 0.6 units over a career. | PitchIQ (A26) [AN, low confidence] |
| Four-seam movement roughly constant from 19 to 44 (cross-sectional) | Platt 2019 (A24) [AN] |

### 2C. Command / location error (pitchers): no direct measure found; walk rate as a proxy

| Finding | Method / sample | Source |
|---|---|---|
| Pitcher walk-rate peak 32.5; strikeout-rate peak 23.6 (extrapolated); ERA peak 29.2; HR-rate peak 27.4 | Regression on long-career pitchers, 1921-2006 | Bradbury (P2, A3) [PR/AN] |
| Starters improve walk rate through 26, then decline subtly (stay within ±1 BB/9 of peak). Relievers' walks rise from the start, about +1 BB/9 above peak by 30. | Delta method, 2002-2011 | Petti 2012 (A20) [AN] |
| BB% bottoms out around 26, then rises | Delta method, 2005-2019 | A11 [AN] |
| BB% keeps improving until about 30 | GAM, starting pitchers 2000-2025 | A36 [AN, blog] |
| Location+ or Kirby Index aging curves | **not found** | - |

### 2D. Pitcher strikeout rate

| Finding | Source |
|---|---|
| The average pitcher's K% "pretty much peaks when he debuts, or shortly afterward"; starters may hold it until about 25 (delta method, 2002+). | Sarris/Zimmerman 2012 (A13) [AN] |
| Starters do not lose 1 K/9 until 32; relievers start declining at 31 and are more than 2 K/9 below peak by 34; starters reach that loss around 39 (survivor effects noted). | Petti 2012 (A20) [AN] |
| 2012-19: the K% curve became "curved" rather than a straight decline from the early 20s, attributed to pitchers keeping velocity and changing pitch mix. | A11 [AN] |

### 2E. Sprint speed (running)

| Finding | Method / sample | Source |
|---|---|---|
| Change per year, by age in year 1 (ft/s): 21: -0.05, 22: -0.15, 23: -0.13, 24: -0.17, 25: -0.12, 26: -0.13, 27: -0.16, 28: -0.15, 29: -0.13, 30: -0.16, 31: -0.07, 32: -0.14, 33: -0.21, 34: -0.09, 35: -0.12, 36: -0.13, 37: -0.26 (n=18). Derived: mean of the 22-36 rows is -0.137 ft/s/yr, and the chained total from 21 to 37 is about -2.1 ft/s. Cross-sectional year-1 speed: 28.2 at 21, 27.3 at 27, 26.7 at 30, 25.8 at 35, 24.0 at 40. Attrition (share not returning): 13-17% at ages 21-26, 17-26% at 27-32, 34-47% at 33-37. Catchers are slower at every age but decline at a similar rate. | Delta method, unweighted; player-seasons with 10+ competitive runs, 2015-2022; survivor bias deliberately ignored, on the argument that speed change is mostly unrelated to staying in MLB | Clemens 2022 (A27) [AN] |
| Over 650 consecutive qualified pairs (2015-2017): 23% lost at least 0.5 ft/s and 7% gained at least 0.5; two-thirds lost some speed; 2 players gained a full ft/s while 23 lost one. "Every single age pairing" declined on average. Cross-sectional: the group is above the 27 ft/s average only through 28, hovers just below it at 29-32, with clear decline from 33. Share above 27 ft/s: 78.5% (27 and under), 47.5% (28-32), 15.2% (33+). | Statcast 2015-2017 | Petriello 2017 (M1) [PRESS] |
| Qualified 20-year-olds averaged 28.5 ft/s; the curve declines steadily to about league average (27) near 28, slowly until about 37, then falls off a cliff. Fast non-catchers declined about 0.7% per season from 30 to 35 (derived: about 0.2 ft/s/yr at 29 ft/s). | Statcast 2015-2022, cross-sectional plus a cohort of fast players | Petriello 2023 (M2) [PRESS] |
| A 30+ year-old has reached the 30 ft/s "elite" level in a full season only three times in 11 years of tracking (2015-2025). | Statcast 2015-2025 | Petriello 2025 (M3) [PRESS] |
| Scale: MLB average 27 ft/s; range about 23 (slow) to 30 (elite); "Bolt" = a run of 30+ ft/s. Sprint speed = fastest one-second window, averaging the best two-thirds of qualified runs. | Definitions | M1, M7 [PRESS] |
| "A human's sprint speed begins a free fall with a peak around age 22" (citing an outside source) | - | Zimmerman 2018, "Rotisserie Category Aging Patterns" (A63) [AN] |
| Youth: the 30-yard sprint improves about 4% (0.17 s) from 13U/14U to 15U, then about 1% (0.04 s) per age group, and the two oldest groups show no difference. | USA Baseball development program, group means | Eisenmann 2026c (A58) [PRAC] |

### 2F. Bat speed / swing speed

| Finding | Method / sample | Source |
|---|---|---|
| Ages 22-31: about -0.02 mph/yr (flat). From 31 on: -0.31 mph/yr. The age 21-22 bucket (n=6) is distorted by one player who gained 2.4 mph. Example: 67 batters at 26 (21,141 swings) averaged 71.1 mph, then 70.8 at 27. | Delta method, chained, weighted by swings; 2023 (post-All-Star) to 2024 | Tango, January 2025 (A28) [AN] |
| Four-year groupings: hitters who were 33 in 2023 averaged 71.6, 71.5, 71.0 and 70.8 mph at 33-36. The plateau is at 24-28; swing speed "declines basically from 29 onwards, if not earlier" (selection bias acknowledged). | Four-season cohorts, 2023-2026 | Tango, June 2026 (A29) [AN] |
| Secondary paraphrase of Tango: "roughly consistent through age 31," then -0.25 to -0.5 mph/yr | - | Clemens 2025 (A30) [AN] |
| Bat speed peak 28; fast-swing% peak 27 (called provisional). Swing length increases slightly until about 26. Squared-up rate keeps improving into the 30s. | GAM, bat tracking 2023-2025 | nana_metrics 2026 (A35) [AN, blog] |
| League average bat speed about 72 mph in 2024; "fast swing" = 75+ mph | Fan site, citing Baseball Savant | Cameron 2025 (M9) [PRESS, fan site] |
| Driveline or other amateur bat-speed-by-age curves | **not found** (only vendor marketing ranges, not usable) | - |

### 2G. Exit velocity and power (ISO, HR)

| Finding | Method / sample | Source |
|---|---|---|
| Exit velocity peaks at about 27.5 (unweighted) or 31 (weighted by playing time, because of an age-30 anomaly); 29 after regressing that anomaly. Launch angle rises steadily with age. | Delta method; Statcast 2015 to mid-2019; at least 30 cases per age from 23 to 36; weights = the smaller balls-in-play total; no survivor correction | L'Oiseau 2019 (A31) [AN] |
| "Expect to see guys lose about .2 mph per season until around age 30, then .3 mph per season after that" (author's comment). Roughly constant at 26-30; sharp declines after 33-34. Hitters averaging 88+ mph lose exit velocity every season; those below 88 gain early. About 2 wOBA points per 0.2 mph. College-age average EV about 3 mph lower (author's limited data). | Unweighted delta method, 200+ PA, Statcast 2015-2017 | Perpetua 2018 (A32) [AN] |
| Average and max EV peak at 27, then steady decline. Hard-hit% rises from 20 to 25, holds 26-35, falls sharply after 35. Barrel% rises from 20 to 25 (noisy). | Delta method, unweighted, 50+ PA, 2015-2022 | Salorio 2023 (A33) [AN] |
| Savant power metrics peak at 27-30; "tough to show a significant power improvement after a batter's age-24 season" | Delta method, harmonic-mean PA | Zimmerman 2023 (A34) [AN] |
| Max EV peak 28; 90th-percentile EV and adjusted EV peak later and decline more slowly | GAM | A35 [AN, blog] |
| HR-rate peak 29.9 (hitters) | Regression, long careers | Bradbury (A3) [AN/PR] |
| ISO "a pretty definitive peak," earlier than expected; ground-ball rate bottoms out at 24; "don't leave your slugger in the fridge past 26" | Delta method, 2002-2011 | Sarris/Zimmerman 2012 (A12) [AN] |
| 2006-13: ISO declines "almost immediately" | Delta method | Zimmerman 2014 (A10) [AN] |

### 2H. Contact, strikeouts, walks and plate discipline (hitters)

| Finding | Method / sample | Source |
|---|---|---|
| Five-year age bands (percentage points, read as change across each band; the source does not state units). K%-BB%: 23-27 -0.5, 28-32 +3.3, 33-37 +4.0. Contact%: -0.8, -2.0, -3.2. O-Contact%: -0.5, -3.5, -4.4. Z-Contact%: -0.6, -0.8, -1.7. O-Swing%: 0.0, +1.2, +1.6. Z-Swing%: +1.8, +0.8, +1.6. Contact% drives K%; O-Swing% drives BB%. | Delta method, harmonic-mean PA (about 2016) | Zimmerman 2016 (A15) [AN] |
| O-Contact% falls almost 5 points by age 40 vs 21; no other discipline metric varies by more than 3 points. Contact% rises to about 29, then declines. O-Swing% falls early (hitters get more selective), rises after about 33. SwStr% rises from about 29. Pitchers' Zone% vs a hitter rises about 1 point from 28 to 34 and almost 3 by 39. | Delta-style, PITCHf/x 2007-2012 | Petti 2012 (A14) [AN] |
| K% rises 2 points from 24 to 30 (2006-2013). BB% peak 26-28 (earlier curves 29-30). | Delta method | Zimmerman 2014 (A10) [AN] |
| BB% peak 26-29; hitters have not lost 1 point of BB% from their peak until 35. K% improves until 24-25, then a long peak. Age 31 = the first significant decline in both. | Delta method, 2002-2011 | Sarris/Zimmerman 2012 (A12) [AN] |
| Walk-rate peak 32.3; AVG 28.4 | Regression, long careers | Bradbury (A3) [AN/PR] |
| Contact ability peak 26-27; O-Contact% declines noticeably after 30 | GAM | A35 [AN, blog] |
| Plate discipline (K%-BB%) improves until about 25, declines slowly to 28, then drops significantly | Delta method | Zimmerman 2016 (RotoWire column of 2016-07-01, A64) [AN] |

### 2I. Defense: range, hands and arm

| Finding | Method / sample | Source |
|---|---|---|
| Shortstops, outs per 4,000 balls in play. Unregressed: peak 22-24, average decline about 9 plays/yr (about 7 runs/yr). After regressing year 1 (for selection): peak 24-28 ("peaks no later than age 28"), decline about 5 plays/yr, gain about 8 plays/yr in the 3 years before 24. Regressed level vs peak (from the chain column, read as the level at the first age of each pair): -14 at 29, -19 at 30, -35 at 32, -36 at 35. Unregressed: -40 at 29, -80 at 32, -99 at 35. | Matched pairs ("with or without you"), Retrosheet 1956-2006 | Tango 2008 (A37) [AN] |
| Defensive peak at 26, steady decline, "more substantial" after 30. Example: a player at +5 runs saved at 25 is at +3 by 30 and 0 by 34. | Defensive Runs Saved (DRS), two-year rolling windows per 1,000 innings (1,800+ innings), delta, 2003+ | Simon 2020, Sports Info Solutions (A38) [AN, industry] |
| 1B range highest in the early 20s and declines until about 30. 2B: range peaks at 26-27, efficiency at 28-29. 3B efficiency peaks at 30-31. SS: range declines faster than sure-handedness, -18 plays from peak to 32 (vs Tango's -35). | Revised zone rating (efficiency) and out-of-zone plays (range); THT data 2004-2007; full-time players | Camden Depot 2010 (A39) [AN, blog] |
| By position, combined value: center fielders peak slightly earlier; second basemen are at about 95% of peak by 23 (league 84%) and decline faster in their 30s; shortstops age well to about 32; third basemen peak late (more valuable at 29 than 25) and collapse around 35 | Delta-style on EqR (offense plus position), post-WWII | Silver 2005 (A40) [AN] |
| Catchers: framing improves at 25, flat 26-30, declines from 31. Arm strength gains at 22-23, flat to about 28, gains at 29-32, clear decline after 32. Exchange time improves every year. Blocking peaks at 29. | Delta method, Baseball Savant 2021-2024; no numbers in text | Booth 2025 (A41) [AN, blog] |
| Outs Above Average (OAA) or UZR aging curve with per-year numbers | **not found** | - |
| Arm strength (throw velocity) aging for infielders and outfielders | **not found** (only single-player anecdotes) | - |

---

## 3. How much players differ (spread of development and aging)

| Finding | Source |
|---|---|
| Peak age rises with talent: 25.6-26.8 (bottom three quintiles) vs 28.2 (top). Between-player variation exceeds within-career variation. | Hakes & Turner 2011 (P3) [PR] |
| Spread of individual peak ages (players with 5,000+ PA): quartiles 25.6-28.5 (born 1930s) to 27.9-32.0 (born 1960s) | Albert 2002 (W1) [WP] |
| Peak depends on debut age and career length (24 to 31) | Carleton 2010 (A5) [AN] |
| Hitters with above-average K% peaked at 23 (1950-2014) or 21 (1990-2014); below-average K% hitters peaked at 26. Delta method, 600+ PA seasons, wOBA. | Jordan 2015 (A61) [AN, community] |
| Chance of a hitter improving by at least a threshold in one year (Statcast, delta). Age 21-22 (n=64): ISO +.020 45%, AvgEV +2 mph 20%, MaxEV +2 mph 44%, Barrel% +2 15%. 24-25 (n=389): 37%, 21%, 25%, 32%. 27-28 (n=438): 32%, 20%, 19%, 29%. 30-31 (n=273): 26%, 13%, 18%, 24%. 33-34 (n=140): 22%, 12%, 8%, 25%. 35-36 (n=60): 33%, 10%, 10%, 24%. | Zimmerman 2023 (A34) [AN] |
| Year-to-year exit-velocity change: the 5th-95th percentile spread is widest at 24-30 and narrows in the early 30s (probably survivor bias) | Perpetua 2018 (A32) [AN] |
| Sprint-speed pairs: 23% lost at least 0.5 ft/s and 7% gained at least 0.5 in one year | Petriello 2017 (M1) [PRESS] |
| Swing-speed change beyond ±1 to 1.5 mph in a year is a tail event (injury or mechanics) | Tango 2025 (A28) [AN] |
| Fastball velocity: among year pairs within ±0.5 mph, 60% were declines | Zimmerman 2012 (A21) [AN] |
| Pitch quality (Filth+, SD 12.8): residual SD of yearly change 2.04 points; projection cone ±2.0 at 1 year, ±3.5 at 3, ±4.6 at 5 | PitchIQ (A26) [AN, low confidence] |
| Players lose "around 25 percent of their value from one year to the next" on average (percent change in WAR, which includes regression to the mean); the spread depends on skill level (no value given). Average delta-method WAR changes are under 0.5 WAR per age step. | Frontin 2019 (A60) [AN] |
| Youth pitching velocity SD within an age group: 3.7-6.8 mph, largest at 13U (maturation) | Tremblay 2022 (P7) [PR] |
| Teen showcase velocity percentiles are 4-5 mph apart, and a pitcher who stays in his percentile band gains about 22 mph from 13 to 18 | Eisenmann 2026b (A57) [PRAC] |

---

## 4. Prospects: development and reaching MLB

### 4A. Minor-league development rates

| Finding | Source |
|---|---|
| Triple-A hitters (2006-2013): wRC+ curve "up and down with a peak at age 25" (no numbers) | Zimmerman 2013 (A9) [AN] |
| FanGraphs-style prospect projection: "a 19-year-old with a 100 wRC+ will be a 135 wRC+ player at peak" (peak "28-ish"); a 170 wRC+ in Class A is about 100 in MLB (2021-2023 level transitions) | Rosenblum 2024 (A46) [AN] |
| Players at least 4 years younger than their league (300+ PA, 1997-2011, 280 players): 65.7% reached MLB, and those who did were worth "roughly five and one-half" WAR (table: 5.6 WARP). Average MLB WARP by the level of the young season: Triple-A 7.5, Double-A 3.7, High-A 4.1, Class A 3.5. "Good" players who were 1-2 years young (23 or under): 52.3% reached, 2.4 WARP. | Ankrom 2012 (A45) [AN] |
| Per-year improvement by age within the minors (wRC+, K%, BB%, velocity) and its spread | **not found** |

### 4B. Reaching MLB by draft round

**Karcher 2017 (R1) [BRJ].** Signed players from the 1996-2011 drafts, rounds 1-20, outcomes as of 2016-08-01. The 1965-95 comparison is Allan Simpson's Baseball America study as quoted by Karcher.

| Round | Played in MLB | 3+ MLB years | 1965-95 played in MLB |
|---|---|---|---|
| 1st + supplemental | 66.7% | 46.8% | 67.0% |
| 2nd + supplemental | 49.4% | 31.5% | 46.5% |
| 3rd + supplemental | 39.7% | 21.6% | 36.2% |
| 4th | 35.0% | 18.6% | 28.3% |
| 5th | 33.3% | 18.6% | 28.5% |
| 6-10 combined | 20% | 9.0% | 20.4% |
| 11-15 combined | 12.7% | 5.2% | 12.1% |
| 16-20 combined | 9.9% | 4.4% | 9.8% |

The source's round-6 and round-8 tables are identical, which looks like a transcription error.

Reached MLB by player type, derived from Karcher's tables. High-school players drafted in 2010-11 had less time to reach MLB.

| Round | HS pitchers | HS position players | College pitchers | College position players |
|---|---|---|---|---|
| 1 | 59.9% (n=162) | 56.7% (180) | 71.7% (226) | 78.2% (156) |
| 2 | 46.0% | 34.4% | 52.3% | 67.3% |
| 3 | 41.2% | 31.5% | 41.9% | 43.3% |
| 4 | 31.8% | 30.0% | 34.8% | 41.6% |
| 5 | 38.7% | 26.7% | 30.3% | 37.9% |

Rounds 1-5: college position players 54.1% vs college pitchers 48.1%.

| Other draft findings | Source |
|---|---|
| June regular-phase first-rounders 1965-2009: 69.0% (827/1,198) reached MLB by 2014. By era: 62.9% (1965-74), 75.2% (1985-94), 67.7% (1995-2004), 76.3% (2005-09). Supplemental 1st 54.7%, 2nd 46.5%, 3rd 37.8%, 4th 31.3%, 5th 29.5%. All drafted players about 15%. First-round catchers 63.9%. | Simpson 2014 (A47) [AN] |
| Picks 1-100, 1984-99: 41% of high-school vs 59% of college picks reached MLB (college about 50% more likely). 15-year WARP: 1st round HS 14.31 vs college 20.32; 2nd round 4.37 vs 6.59; 3rd round 1.88 vs 5.34. | Jazayerli 2005 (A48) [AN] |
| All signed picks, 1987-2008: 17.2% ("roughly one in six") played in MLB. Top 20 rounds only: 25.4% (1987-96 26.3%; 1997-2008 24.6%, still rising in 2013). Top 20 rounds, 1965-95: 22.8%. | Eddy 2013, Baseball America via SABR (A49) [AN] |
| First-rounders 2000-2010, by pick range, reached MLB: picks 1-5 87%, 6-10 75%, 11-15 82%, 16-20 78%, 21-25 65%, 26-30 58% (some still in the minors at the time). | OTBB_Sean 2016 (A54) [AN, community; low weight] |

### 4C. Reaching MLB (and value) by prospect grade or rank

**FanGraphs Future Value (FV) outcomes** (Clemens 2025, A42 [AN]). FanGraphs lists 2019-2022, prospects graded 45+ FV. The outcome is the Steamer projection three years later, per 600 PA (hitters) or 200 IP (pitchers). Tiers in WAR per season: washout <0.5 or no MLB projection, backup 0.5-1.5, regular 1.5-2.5, above average 2.5-4, star 4+.

| FV | Hitters: washout / backup / regular / above avg / star (n) | Pitchers: washout / backup / regular / above avg / star (n) |
|---|---|---|
| 45 | 51 / 25 / 17 / 6 / 1% (295) | 53 / 26 / 16 / 5 / 0% (230) |
| 45+ | 52 / 18 / 19 / 11 / 1% (91) | 38 / 24 / 25 / 13 / 0% (68) |
| 50 | 23 / 24 / 30 / 21 / 2% (197) | 27 / 27 / 24 / 20 / 2% (96) |
| 55 | 17 / 17 / 30 / 31 / 6% (54) | 17 / 20 / 37 / 27 / 0% (30) |
| 60 | 14 / 12 / 19 / 38 / 17% (42) | 17 / 33 / 25 / 25 / 0% (12) |

Samples above FV 60 are only 1-3 players.

**FV scale definitions** (Longenhagen 2022, A43 [AN]), average WAR per season over the six pre-free-agency years:
- 80: more than 7.0 (top 5 overall)
- 70: 5.0-7.0
- 60: 3.4-4.9 (All-Star)
- 55: 2.5-3.3 (above-average regular)
- 50: 1.6-2.4 (average everyday player)
- 45: 0.8-1.5 (low-end regular or platoon)
- 40: 0.0-0.7 (bench player)

**Baseball America Top 100, 1990-2006** (Perez 2013 update of McKinney 2011, A44 [AN]). Every ranking counts separately. Outcome is average WAR per cost-controlled season. "Didn't play" = never had a season with 100+ PA or 25+ IP. Bust = under 1.5 average WAR. Success = 1.5+. Superior = 2.5+.
- Didn't play: pitchers 17.0%, position players 12.8%, all 14.65%.
- Bust: pitchers 77.5%, position players 66.0%, all 71.1%.
- Success: all 28.9%; superior 16.35%.
- Success by rank: 1-10 51.8% (superior 32.4%), 11-20 44.7%, 21-30 28.8%, 31-40 28.2%, 41-50 25.3%, 51-60 21.8%, 61-70 20.6%, 71-80 24.7%, 81-90 18.8%.

### 4D. Age at MLB debut, time to MLB, survival

| Finding | Source |
|---|---|
| Average debut age: 25.0 in 2019 (262 debuts); 25.0-25.3 for 2015-2019; 23.3 in 1976; 23.0 in 1971 | Justice 2020, MLB.com research (M4) [PRESS] |
| Median debut age stayed between 24 and 25 over 20 years; about 96% of debuts happen at ages 20-29; 24- and 25-year-olds were over 50% of debuts in some years | Ginsborg 2020 (A51) [AN, student org] |
| Share surviving past year 1, past 27, and past 31, by debut age (post-1980 hitters). Here "debut" = first season with 200+ PA, and "survived" = another 200+ PA season after that point. Debut 22 or younger: 93.8%, 72.5%, 47.8%. Debut 23: 88.1%, 64.9%, 39.3%. Debut 24: 81.7%, 64.0%, 35.5%. Debut 25: 82.9%, 61.7%, 33.6%. Debut 26: 69.1%, 55.5%, 29.2%. Attrition spikes in the first year, at 27-28 and at 31-32. | Carleton 2010 (A5) [AN] |
| Debuting at 24 instead of 23: hazard ratio 1.26 for career ending; each year of later debut is worth about -0.5 runs of seasonal VORP (1950-1990 position players) | Arthur 2014 (A50) [AN] |
| College draftees reach MLB faster, especially in the first 4 years after the draft | Jazayerli 2005 (A48) [AN] |
| First-rounders 2008-2011 (87 of 200 had debuted by early 2015): simple average 3.8 years (HS) vs 2.5 (4-year college); adjusted for those who had not yet debuted, 6.5 vs 4.5 | "someone" 2015 (A55) [AN, community; low weight] |
| Hitters 35+ produced 5.6 WAR in about the first third of 2026, vs 71.3 WAR in 2003, the peak year for older hitters (FanGraphs data, as reported). Context: few late-30s regulars today. | AP 2026 (M8) [PRESS] |

### 4E. International amateurs (signed at 16)

| Finding | Source |
|---|---|
| Eligibility: signing window January 15 to December 15; "He must turn 17 before Sept. 1 of the year in which he signs." | MLB.com glossary (M6) [PRESS] |
| Dominican signees 2004-2012: 5.4% (213 of 3,911) reached MLB; yearly range 3.1% (2012 class) to 7.6% (2011). The 1990 class: 9.2% (26 of 281). Also quotes Mother Jones 2013 (2.6% of 832 Dominicans in the 2006 Dominican Summer League) and Time 2010 (2%). | Pérez Neró 2019, Diario Libre (M5) [PRESS; own analysis using Baseball-Reference] |
| Baseball America's top-30 bonus lists, 2010-2012 classes (115 players): 33% reached MLB; about 10% became "players of significance" (5+ career fWAR or one 3.0 fWAR season); 0.83 fWAR per player on average | Anonymous 2023 (A53) [AN, blog; low weight] |
| International bonus spending ROI (classes 2004-2015): rolling ROI from 1279% (2004) to 307% (2015, still maturing). Reach rates by bonus tier not given. | McDaniel 2019 (A52) [AN] |

---

## 5. Conflicts between sources

1. **Overall peak age.** Different methods give different answers:
   - Delta method and its corrected form: 26-28 (A1, A2, A6, A8, A11).
   - Fair's fixed effects: 27.5-28.3 (P1, P1u).
   - Bradbury's symmetric quadratic on long careers: about 29 (P2).
   - Imputation: 26, vs 30 when dropout is ignored (P4).
   - The explanations offered are survivor and selection bias (A2, A7), the forced symmetry of a single parabola (A6), late-peaking long-career samples (A1, A5, P3), and early arrivals being better players (A6).
2. **Who peaks later.** Hakes & Turner find the best players peak about 2 years later (P3). Bradbury found Hall of Famers peak "a little earlier", if anything (A3).
3. **Walk-rate peak (hitters).** 26-29 by FanGraphs delta curves (A10, A12) vs 32.3 (Bradbury, P2) vs 29 (GAM blog, A35). Tango (A18) has walk rate improving almost every year.
4. **Power peak.** About 24-26, or immediate decline after 2006 (A10, A12), vs 27-30 (A33, A34, A35; HR rate 29.9 in P2; HR ratio 27 in A18).
5. **Fastball velocity.** Most curves peak at about 21 with slow decline (A19, A21, A24, A26). Kalk 2008 found gains to 28-29 (A23; small, partial-season sample). Petti's "within 0.5 mph of peak through 28" for all pitchers (A19) does not fit easily with "-2 mph for the average starter by 30" (A20). Amateur tracking puts peak velocity at 18-20 (A56), which MLB-only data cannot see.
6. **Bat speed.** Decline starts at 31 (A28, January 2025), from about 29 (A29, June 2026, using more seasons), or peaks at 28 (A35). The rate after onset is -0.31 mph/yr (A28) vs "0.25-0.5" (A30, secondary).
7. **Sprint speed.** Within-player changes are negative at every age from 21 (A27, M1). The cross-sectional "clear decline begins at 33" (M1) describes the league population, not individual aging.
8. **Era effects.**
   - Fair 2008 found smaller decline for later cohorts; the 2025 update found no significant change for OBP and OPS.
   - Lichtman found much slower decline after 1980.
   - Zimmerman found no early-career rise after 2006.
   - Bonney found 1991-2005 and 2006-13 indistinguishable, but different from 1961-90 after about 30.
9. **Survivor bias direction.** Lichtman: it exaggerates decline (A2). Judge: under truncated dropout, survivor-only regressions *understate* aging by about 40% (A7). Clemens chose to ignore it for sprint speed (A27).
10. **Draft reach rates depend on the denominator.** All signed picks reach MLB at 17.2%, top-20-round picks at 25.4% (both 1987-2008, A49), and all drafted players at about 15% (A47). First-round rates range from 66.7% (R1, signed, 1996-2011) to 69.0% (A47, 1965-2009) and 76.3% (A47, 2005-09) depending on years and phase.
11. **Pitcher strikeout peak.** 23.6 (P2, extrapolated outside the data window) vs "at debut", with starters holding to about 25 (A13), vs starters not losing 1 K/9 until 32 (A20).

## 6. Open gaps

**Not found:**
- Statcast-era, per-age delta curves for fastball velocity with starter/reliever splits from a rigorous source. Only an undated small-site page (A26) and pre-2019 FanGraphs work exist.
- Spin, extension and induced-break aging with clean per-year numbers. The only table (A25) has a disputed presentation.
- Pitcher command or location-error aging (Location+, Kirby Index or miss distance by age).
- Arm-strength (throw velocity) aging for infielders and outfielders. Catchers are covered qualitatively only (A41).
- OAA or UZR aging curves with per-year values. DRS (A38) gives only the shape and one example; the play-based shortstop curve (A37) is from 2008.
- Hitter "timing" or reaction aging measured directly. The FiveThirtyEight result that older hitters' gap vs fast and slow fastballs stays constant was seen only in a search excerpt (Lindbergh & Arthur 2016); the SABR repost contains no numbers.
- Minor-league per-year development rates (wRC+, K%, BB%, velocity, exit velocity by age within MiLB) and their spread.
- Amateur bat speed and exit velocity norms by age from a disclosed-method source. Driveline: no public age curve found.
- Full text of Bradbury 2009 (paywalled; component peaks taken from his BP summary, A3).
- The findings of Schall & Smith 2000 (citation only).
- Draft-reach numbers from the peer-reviewed draft papers: Spurr 2000, *J. Sports Economics* 1(1):66-85, DOI 10.1177/152700250000100106; Burger & Walters 2009, *J. Sports Economics* 10(5):485-501, DOI 10.1177/1527002509332350.
- International signee reach rates by bonus tier, signing age (16 vs 17-18) or country from a rigorous source.
- Post-2020 (20-round draft era) reach rates by round. Too recent to have matured.

**Unverified leads** (seen only in search-engine excerpts; the pages were blocked or paywalled; do not use without checking):
- Baseball America 2019, "How Many MLB Draftees Make It To The Majors": 2nd round about 51%, 3rd round about 40%, drafts since 1981.
- Baseball America, "Numbers Game: Age Isn't Everything On The International Market": 54.9% of MLB players who signed as Dominican or Venezuelan amateurs signed at 16; 93% of such position players signed at 16 or 17; 45% of such pitchers signed after 16.
- Baseball America: "a majority of college pitchers add velocity by their second season as a professional" (2019-20 college classes); 13 Top-100 pitchers added at least 1 mph in 2024.
- Medium (Matan K.): defensive value (FanGraphs Def per 600 PA) declines about 1-4 runs faster after 35 in recent seasons.

Injury-driven decline is out of scope. Note, though, that the delta-method speed and velocity curves (A27, M1, A19) include injury effects, while Fair (P1) explicitly removes likely injury seasons.

## 7. References

**Peer-reviewed and academic**
- P1: Fair, R.C. (2008). Estimated Age Effects in Baseball. *Journal of Quantitative Analysis in Sports* 4(1), Art. 1. DOI 10.2202/1559-0410.1074. Copy: https://fairmodel.econ.yale.edu/rayfair/pdf/2005dol5.PDF [PR]
- P1u: Fair, R.C. (updated April 2025). Estimated Age Effects in Baseball, revised data 1921-2023. https://fairmodel.econ.yale.edu/RAYFAIR/pdf/2005d.PDF [WP]
- P2: Bradbury, J.C. (2009). Peak athletic performance and ageing: Evidence from baseball. *Journal of Sports Sciences* 27(6):599-610. DOI 10.1080/02640410802691348 [PR]
- P3: Hakes, J.K. & Turner, C. (2011). Pay, productivity and aging in Major League Baseball. *Journal of Productivity Analysis* 35(1):61-74. DOI 10.1007/s11123-009-0152-8. Working paper: https://mpra.ub.uni-muenchen.de/4326/1/MPRA_paper_4326.pdf [PR]
- P4: Nguyen, Q. & Matthews, G.J. (2024). Filling the gaps: A multiple imputation approach to estimating aging curves in baseball. *Journal of Sports Analytics* 10(1):77-85. DOI 10.3233/JSA-240744. arXiv:2210.02383 [PR]
- P5: Schall, T. & Smith, G. (2000). Career Trajectories in Baseball. *CHANCE* 13(4):35-38. DOI 10.1080/09332480.2000.10542233 [ASA magazine; not accessed]
- P6: Schulz, R., Musa, D., Staszewski, J. & Siegler, R.S. (1994). The relationship between age and major league baseball performance: Implications for development. *Psychology and Aging* 9(2):274-286. DOI 10.1037/0882-7974.9.2.274 [PR; cited via P1]
- P7: Tremblay, M., Tétreau, C., Corbin-Berrigan, L.-A. & Descarreaux, M. (2022). Anthropometrics, athletic abilities and perceptual-cognitive skills associated with baseball pitching velocity in young athletes aged between 10 and 22 years old. *Frontiers in Sports and Active Living* 4. DOI 10.3389/fspor.2022.822454 [PR]
- W1: Albert, J. (2002). Smoothing Career Trajectories of Baseball Hitters. Manuscript, Bowling Green State University. https://bayesball.github.io/papers/career_trajectory.pdf [WP]
- R1: Karcher, R.T. (2017). The Chances of a Drafted Baseball Player Making the Major Leagues: A Quantitative Study. SABR *Baseball Research Journal*, Spring 2017. https://sabr.org/journal/article/the-chances-of-a-drafted-baseball-player-making-the-major-leagues-a-quantitative-study/ [BRJ]

**Analytics**
- A1: Lichtman, M. (2009-12-21). How do baseball players age? (Part 1). THT. https://tht.fangraphs.com/how-do-baseball-players-age-part-1/
- A2: Lichtman, M. (2009-12-29). How do baseball players age? (Part 2). THT. https://tht.fangraphs.com/how-do-baseball-players-age-part-2/
- A3: Bradbury, J.C. (2010-01-11). How Do Baseball Players Age?: Investigating the Age-27 Theory. BP. https://baseballprospectus.com/?p=9933
- A4: Wyers, C. (2009-11-17). Is peak at age 29? THT. https://tht.fangraphs.com/tht-live/is-peak-at-age-29/
- A5: Carleton, R.A. (2010-02-15). Baseball Therapy: That Peak Age Thing, Part 1. BP. https://www.baseballprospectus.com/news/article/10058/baseball-therapy-that-peak-age-thing-part-1/
- A6: Judge, J. (2020-07-07). The Delta Method, Revisited: Rethinking Aging Curves. BP. https://www.baseballprospectus.com/news/article/59972/the-delta-method-revisited/
- A7: Judge, J. (2020-06-15). An Approach to Survivor Bias in Baseball. BP. https://www.baseballprospectus.com/?p=59491
- A8: Zimmerman, J. (2013-12-13). Are Aging Curves Changing? (originally titled "Hitters No Longer Peak, Only Decline"). FanGraphs. https://blogs.fangraphs.com/hitters-no-longer-peak-only-decline
- A9: Zimmerman, J. (2013-12-18). More on Changing Hitter Aging Curves. FanGraphs. https://blogs.fangraphs.com/more-on-changing-hitter-aging-curves
- A10: Zimmerman, J. (2014-01-15). Component Changes in New Hitter Aging Curves. FanGraphs. https://blogs.fangraphs.com/component-changes-in-new-hitter-aging-curves
- A11: Zimmerman, J. (2020-02-12). Aging Curves Revisited: Damn Strikeouts. THT. https://tht.fangraphs.com/aging-curves-revisited-damn-strikeouts/
- A12: Sarris, E. (2012-01-12). Hitters Age Like Wine — Power Like Cheese? (curves by J. Zimmerman). FanGraphs. https://blogs.fangraphs.com/hitters-age-like-wine-power-like-cheese
- A13: Sarris, E. (2012-01-11). Aging Strikeouts: You'll Never Be This Good Again. FanGraphs. https://blogs.fangraphs.com/youll-never-be-as-good-again/
- A14: Petti, B. (2012-10-08). Hitter Aging Curves: Plate Discipline. FanGraphs. https://blogs.fangraphs.com/hitter-aging-curves-plate-discipline
- A15: Zimmerman, J. (2016-07-07, last updated). The Saber's Edge: The Intersection of Plate Discipline, Age and Contact. RotoWire. https://web9.rotowire.com/baseball/article/the-sabers-edge-the-intersection-of-plate-discipline-age-and-contact-28828
- A16: Gutwein, C. (2021-10-04). Checking In on the Aging Curve. FanGraphs. https://blogs.fangraphs.com/checking-in-on-the-aging-curve/
- A17: Bonney, P. (2015-08-20). Maybe Players Aren't Aging Differently After All. THT. https://tht.fangraphs.com/maybe-players-arent-aging-differently-after-all/
- A18: Tango, T. (c. 2002). Aging Patterns. Tango on Baseball archives. https://www.tangotiger.net/archives/artAging.shtml
- A19: Petti, B. (with J. Zimmerman) (2012-04-30). Pitcher Aging Curves: Introduction. FanGraphs. https://blogs.fangraphs.com/?p=84093
- A20: Petti, B. (2012-05-02). Pitcher Aging Curves: Starters and Relievers. FanGraphs. https://blogs.fangraphs.com/pitcher-aging-curves-starters-and-relievers/
- A21: Zimmerman, J. (2012-05-09). Pitcher Aging Curves: Maintaining Velocity. FanGraphs. https://blogs.fangraphs.com/pitcher-aging-curves-maintaining-velocity
- A22: Petti, B. (2012-06-06). Velocity Decline and Pitcher Attrition by Age. FanGraphs. https://blogs.fangraphs.com/pitcher-attrition-and-velocity-decline-by-age
- A23: Kalk, J. (2008-05-27). Preliminary aging curve for fastball speed. THT. https://tht.fangraphs.com/preliminary-aging-curve-for-fastball-speed/
- A24: Platt, B. (2019-06-06). "Stuff" and Father Time. FanGraphs Community Research. https://community.fangraphs.com/stuff-and-father-time/
- A25: Sawchik, T. (2018-03-27). Rich Hill Has a Theory About Spin and Aging (data: J. Zimmerman). FanGraphs. https://blogs.fangraphs.com/rich-hill-has-a-theory-about-spin-and-aging/
- A26: PitchIQ (undated, accessed 2026-11-16). Filth+ Aging Projection — Methodology & Accuracy. https://pitchiq.prospecttilt.com/aging-methodology
- A27: Clemens, B. (2022-09-29). Why Are Catchers So Dang Slow? FanGraphs. https://blogs.fangraphs.com/why-are-catchers-so-dang-slow
- A28: Tango, T. (2025-01-14). Aging Curve - Swing Speed. Tangotiger Blog. http://tangotiger.com/index.php/site/article/aging-curve-swing-speed
- A29: Tango, T. (2026-06-23). When does Swing Speed start to decline? Tangotiger Blog. https://tangotiger.com/index.php/site/article/when-does-swing-speed-start-to-decline
- A30: Clemens, B. (2025-02-13). Early Notes on the New Bat Speed Data Release. FanGraphs. https://blogs.fangraphs.com/early-notes-on-the-new-bat-speed-data-release
- A31: L'Oiseau, P. (2019-07-25). Creating Aging Curves for Statcast Metrics. THT. https://tht.fangraphs.com/creating-aging-curves-for-statcast-metrics/
- A32: Perpetua, A. (2018-02-15). Quick and Dirty Aging Curves With Exit Velocity. RotoGraphs. https://fantasy.fangraphs.com/quick-and-dirty-aging-curves-with-exit-velocity/
- A33: Salorio, A. (2023-01-27). Examining Aging Curves for Statcast Metrics. Substack. https://adamsalorio.substack.com/p/examining-aging-curves-for-statcast
- A34: Zimmerman, J. (2023-06-22). Savant Hitter Aging Curves & Improvement Chances. RotoGraphs. https://fantasy.fangraphs.com/savant-hitter-aging-curves-improvement-chances/
- A35: nana_metrics (2026-07-31). [SABR] A Study on the Peak of Hitting Metrics in the "Batter Aging Curve". note.com (Japanese blog). https://note.com/nana_metrics/n/nf8e2d361540e
- A36: nana_metrics (2026-08-03). [SABR] Analysis of Peaks by Pitching Metrics in the "Pitcher Age Curve". note.com. https://note.com/nana_metrics/n/n7254de2e1c8b
- A37: Tango, T.M. (2008-02-07). Fielding aging curves. THT. https://tht.fangraphs.com/fielding-aging-curves/
- A38: Simon, M. (2020-05-08). A Closer Look at Defensive Aging Curves. Sports Info Solutions (excerpt from *The Fielding Bible, Vol. V*). https://www.sportsinfosolutions.com/2020/05/08/a-closer-look-at-defensive-aging-curves/
- A39: Camden Depot (2010-06-26, a repost of 2008 work). Depot Retro: Infield Age Curves. https://camdendepot.blogspot.com/2010/06/depot-retro-infield-age-curves.html
- A40: Silver, N. (2005-09-22). Lies, Damned Lies: A New Look at Aging. BP. https://www.baseballprospectus.com/news/article/4464/lies-damned-lies-a-new-look-at-aging/
- A41: Booth, Q. (2025-03-12). How do Catchers Age? Substack. https://quinnbooth.substack.com/p/how-do-catchers-age
- A42: Clemens, B. (2025-02-10). How Do Prospect Grades Translate to Future Outcomes? FanGraphs. https://blogs.fangraphs.com/how-do-prospect-grades-translate-to-future-outcomes
- A43: Longenhagen, E. (2022-02-21). Prospect Week Primer. FanGraphs. https://blogs.fangraphs.com/prospect-week-primer
- A44: Perez, M. (2013-12-03). Death to TINSTAAPP: Updating McKinney's Work on Prospect Success. Camden Depot (updates S. McKinney 2011, Royals Review; original not accessed). https://camdendepot.blogspot.com/2013/12/death-to-tinstaapp-updating-mckinneys.html
- A45: Ankrom, B. (2012-06-07). The Process: How to Evaluate Precocious Prospects. BP. https://baseballprospectus.com/prospects/article/17276/the-process-how-to-evaluate-precocious-prospects
- A46: Rosenblum, J. (2024-04-08). The Top 30 Projected Hitting Prospects Entering 2024. RotoGraphs. https://fantasy.fangraphs.com/the-top-30-projected-hitting-prospects-entering-2024
- A47: Simpson, A. (2014-06-09). Perfect Game Presents: The Baseball Draft, A 50-Year Retrospective, Part Two. BP. https://www.baseballprospectus.com/prospects/article/23821/perfect-game-presents-the-baseball-draft-a-50-year-retrospective-part-two
- A48: Jazayerli, R. (2005-05-19). Doctoring The Numbers: The Draft, Part Two. BP. https://baseballprospectus.com/news/article/4042/doctoring-the-numbers-the-draft-part-two
- A49: Eddy, M. (2013-07-23). How and why more amateur draft picks are making the majors. Baseball America, reposted by SABR. https://sabr.org/latest/eddy-how-and-why-more-amateur-draft-picks-are-making-the-majors/
- A50: Arthur, R. (2014-07-08). Moonshot: Survival of the Fittest: Position Players. BP. https://www.baseballprospectus.com/?p=24096
- A51: Ginsborg, A. (2020-11-29). Let the Kids Play? Sports Analytics Group at Berkeley. https://sportsanalytics.studentorg.berkeley.edu/articles/let-the-kids-play.html
- A52: McDaniel, K. (2019-04-04). We Analyzed the Value of International Signing Bonus Money. FanGraphs. https://blogs.fangraphs.com/we-analyzed-the-value-of-international-signing-bonus-money
- A53: Anonymous (2023-01-27). Bonus Bust: Big bucks do little in international free agency. https://areyouwatchingthis.com/mlb/news/867039-Bonus-Bust-Big-bucks-do-little-in-international-free-agency
- A54: OTBB_Sean (2016-06-15). Success Rate of MLB First-Round Draft Picks by Slot. FanGraphs Community. https://community.fangraphs.com/success-rate-of-mlb-first-round-draft-picks-by-slot/
- A55: "someone" (2015-02-21). Time from Draft Until Majors. FanGraphs Community. https://community.fangraphs.com/time-from-draft-until-majors/
- A56: Eisenmann, J.C. (2026-06-18). Chasing Velo: A Preliminary Analysis. *Archives of IronMan Performance* (Substack). https://joeeisenmann.substack.com/p/chasing-velo-a-preliminary-analysis [PRAC]
- A57: Eisenmann, J.C. (2026-06-21). Chasing Velo: Benchmarks for 13-18 Year Old Competitive Pitchers. https://joeeisenmann.substack.com/p/chasing-velo-benchmarks-for-13-18 [PRAC]
- A58: Eisenmann, J.C. (2026-07-29). Performance Metrics for USA Baseball National Teams & Athlete Development Programs, 12U-Collegiate. https://joeeisenmann.substack.com/p/performance-metrics-for-usa-baseball [PRAC]
- A59: Slowinski, P. (2010-02-27). Aging Curve. FanGraphs Library. https://library.fangraphs.com/principles/aging-curve
- A60: Frontin, C. (2019-10-17). Baseball ProGUESTus: Casting Some Uncertainty on How Players Age. BP. https://baseballprospectus.com/news/article/54551/baseball-proguestus-casting-some-uncertainty-on-how-players-age
- A61: Jordan, D. (2015-04-22). Different Aging Curves For Different Strikeout Profiles. FanGraphs Community. https://community.fangraphs.com/different-aging-curves-for-different-strikeout-profiles/
- A63: Zimmerman, J. (2018-03-13). Rotisserie Category Aging Patterns. THT. https://tht.fangraphs.com/rotisserie-category-aging-patterns/
- A64: Zimmerman, J. (2016-07-01, last updated). The Saber's Edge: How Age Impacts Plate Discipline. RotoWire. https://www.rotowire.com/baseball/article/the-sabers-edge-how-age-impacts-plate-discipline-28776

**Press and league media**
- M1: Petriello, M. (2017-06-29). Sprint speed leaderboards dominated by youth. MLB.com. https://www.mlb.com/news/statcast-sprint-speed-shows-speed-peaks-young-c239376598
- M2: Petriello, M. (2023-03-21; first published December 2022). Can Trea's speed defy Father Time? MLB.com. https://www.mlb.com/news/how-trea-turner-s-speed-will-age
- M3: Petriello, M. (2025-07-23). Much has changed in 10 years. MLB's fastest players haven't. MLB.com. https://www.mlb.com/news/byron-buxton-trea-turner-maintaining-elite-speed
- M4: Justice, R. (2020-03-05). MLB execs explain trend of young superstars (research by J. Bernard & D. Adler). MLB.com. https://www.mlb.com/news/mlb-executives-youth-revolution
- M5: Pérez Neró, N. (2019-03-26). El porcentaje de criollos que llega a las Grandes Ligas sube hasta el 5.4%. Diario Libre. https://www.diariolibre.com/deportes/beisbol/el-porcentaje-de-criollos-que-llega-a-las-grandes-ligas-sube-hasta-el-5-4-AG12412543
- M6: MLB.com Glossary. International Amateur Free Agency & Bonus Pool Money (accessed 2026-11-16). https://www.mlb.com/glossary/transactions/international-amateur-free-agency-bonus-pool-money
- M7: MLB.com Glossary. Sprint Speed (accessed 2026-11-16). https://www.mlb.com/glossary/statcast/sprint-speed
- M8: Associated Press (2026-06-09). MLB's hitters are struggling to thrive after 35 in the age of analytics and increased velocity (via WTOP). https://wtop.com/sports/2026/06/mlbs-hitters-are-struggling-to-thrive-after-35-in-the-age-of-analytics-and-increased-velocity/
- M9: Cameron, J. (2025-01-24). Should Byron Buxton Fear the Swing Speed Aging Curve? Twins Daily (fan site). https://twinsdaily.com/news-rumors/minnesota-twins/should-byron-buxton-fear-the-swing-speed-aging-curve-r17674/
