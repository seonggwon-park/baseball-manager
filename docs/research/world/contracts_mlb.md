# MLB contracts, free agency and payroll rules: research notes (state on 2026-11-17)

Research date: 2026-11-17. Purpose: sourced facts for designing and calibrating the free-agent market and contract
system. Every number or rule carries a tag and a source. Anything I could not confirm is marked **NOT CONFIRMED**.
Nothing here is estimated or filled in from memory. Values marked *(computed)* are plain arithmetic on sourced figures.

Not repeated here (see the companion files): `docs/research/world/mlb_notes.md` (roster sizes, IL rules, the 10/15-day
minimum stay after an option, the 2026-27 MLB calendar, CBA talks up to about Nov 15) and `docs/research/world/levels.md`
§4.2 (minor-league free agency). Where an item below touches them it says *confirms* or *corrects*.

**Tags.** [CBA] = agreement text (article/section given). [official] = MLB.com (including its news pages), MLBPA, MiLB
and the MLB glossary, plus the Major League Rules book. [analysis] = FanGraphs, Baseball Prospectus, MLB Trade Rumors
analysis, The Hardball Times. [press] = news reports. [wiki] = Wikipedia (not used).

**Documents cited repeatedly**

- **BA** = 2022-2026 Basic Agreement between the 30 clubs and the MLBPA. Effective 2022-03-10; ends 2026-12-01, 11:59 p.m. ET (Art. XXVI).
  https://agent.mlbplayers.com/_files/ugd/4d23dc_fa8575fb2a2c4f0ca117bc181aeb6206.pdf (440 pp.).
  A second MLBPA copy, https://mlbplayers.com/_files/ugd/4d23dc_d6dfc2344d2042de973e37de62484da5.pdf (442 pp.), has the same text
  except two extra uniform-logo pages. I compared the two line by line.
- **UPC** = Major League Uniform Player's Contract, BA Appendix A.
- **MiLB BA** = 2023-2027 Minor League Basic Agreement. Effective 2023-03-30; ends 2027-12-01, 11:59 p.m. ET (Art. XXV).
  https://agent.mlbplayers.com/_files/ugd/4d23dc_ebe9aae5da0b452b9f6dbfe5316df884.pdf
- **MLR 2026** = Major League Rules, 2026 edition (Office of the Commissioner), from the third-party copy used in mlb_notes:
  https://0359b39c-af98-4f91-8753-051c201c8092.filesusr.com/ugd/638123_e8a18872c61d4c0093ae389e26bc62f7.pdf .
  I checked every MLR passage used below against the 2024 edition hosted by the MLBPA,
  https://agent.mlbplayers.com/_files/ugd/4d23dc_168483d9e6a7481ab3da1f5f0ea90f41.pdf . That covers Rule 5 prices and eligibility,
  option limits, waiver claim tables, claim prices and priority, the 7-day DFA limit, release and re-signing rules, the Labor Day
  option-service rule, the minor-league free-agency date, the reserve-list date, Rule 11(e), PPI, the Voluntarily Retired List,
  the bonus ban and the Foreign Professional definition. All of them read the same in both editions. Tagged [official].
- **MLBTR offseason** = MLB Trade Rumors, "The 2026-27 Offseason Begins" (Franco and McDonald, 2026-10-29), https://www.mlbtraderumors.com/2026/10/the-2026-27-offseason-begins.html
- **NBC tracker** = NBC Sports Philadelphia, MLB lockout/CBA tracker (last updated 2026-11-04), https://www.nbcsportsphiladelphia.com/mlb/philadelphia-phillies/mlb-lockout-tracker-cba-talks-phillies/764481/

---

## 0. Key numbers at a glance

| Item | Value | Source |
|---|---|---|
| One year of MLB service | 172 days; at most one year per season | [CBA] BA Art. XXI(A)(1) |
| Salary arbitration | 3 to under 6 years of service. Super Two: 2 to 3 years, at least 86 days of service in the preceding season, and in the top 22 % of that class | [CBA] BA Art. VI(E)(1) |
| Free agency | 6 years of service and contract expired | [CBA] BA Art. XX(B)(1) |
| MLB minimum salary | 2022 $700,000 · 2023 $720,000 · 2024 $740,000 · 2025 $760,000 · 2026 $780,000 · 2027 **not set** | [CBA] BA Art. VI(A)(1) |
| Qualifying offer | $18.4M (2022 season) · $19.65M · $20.325M · $21.05M · $22.025M (2026) · $23.15M (2027) | [analysis] MLBTR; [official] MLB glossary (see 4.2) |
| CBT base threshold | $230M · $233M · $237M · $241M · $244M (2022-2026); no CBT after the 2026 season unless a new agreement creates one | [CBA] BA Art. XXIII(B)(2), XXIII(I) |
| Pre-arbitration bonus pool | $50M a year | [CBA] BA Art. XV(D)(2) |
| Revenue sharing | Transfers equal to a 48 % straight pool of Net Local Revenue | [CBA] BA Art. XXIV(A)(9), (B)(1) |

---

## 1. Reserve system and service time

### 1.1 Service time

| Rule | Value | Source |
|---|---|---|
| Accrual | 1 day of service for each day of the championship season on an MLB Active List. **172 days = 1 year**, at most 172 days per season. Counted from the first to the last regularly scheduled game, the same for every club | [CBA] BA Art. XXI(A)(1) |
| Counts as active | Major League Injured List, disciplinary suspension, active military duty up to 2 years, National Guard emergency duty up to 30 days | [CBA] BA Art. XXI(A)(2) |
| Short options | If optioned for a total of **less than 20 days in a season**, the player keeps MLB service for those days and the option is not charged. Exception: a player with zero MLB days that season before the option who is then released or outrighted and not added back to the Active List | [CBA] BA Art. XXI(B), XIX(E); [official] MLR 2026 Rule 7(c) |
| Counting option days | The option date counts and the recall date does not. Exceptions: an option after the start of an MLB game the player was eligible for does not count; a recall after the start of a minor-league game he was eligible for does count. For a player optioned and then DFA'd or released while on option, the DFA or release date is the last option day | [CBA] BA Art. XXI(B), Attachment 32 |
| Rehab assignment | Counts as MLB service, at MLB salary | [CBA] BA Art. XIX(C)(3)(d) |
| Designated for assignment | MLB salary and MLB service during the DFA period | [CBA] BA Art. XIX(G) |
| Optioned or outrighted before Opening Day rosters are due | No service for Opening Day or for a DFA period that starts from option | [CBA] BA Attachment 2 |
| September options | A player optioned after the third day after Labor Day receives MLB service (not salary) for the option period. A player optioned on or after Sept 1 also receives service if he was on the MLB Active Roster or IL continuously from June 1 **and** the option would keep him from arbitration or free agency that winter | [official] MLR 2026 Rule 7(f)(1)-(2) |
| Rookie of the Year credit | An "Eligible Prospect" (as defined in MLR 4(l)) who finishes **1st or 2nd in his league's ROY voting** is credited with a full year (172 days) for that season. At most 2 per league. The MLBPA may opt out for the next season by notice on or before Dec 15 | [CBA] BA Attachment 53 |
| Prospect Promotion Incentive (draft pick to the club) | Eligible Prospect = on Opening Day has 60 days or less of MLB service, keeps rookie status (no more than 130 AB / 50 IP / 45 days active), and is on at least 2 of the 3 preseason Top 100 lists (Baseball America, MLB.com, ESPN). The club gets a pick after the 1st round if he accrues 172 days with that club in such a season **and**, before he is arbitration-eligible, finishes 1st in ROY or top 3 in MVP or Cy Young voting. One PPI pick per player per career. Foreign Professionals and players who signed multi-year MLB deals before debut are excluded | [official] MLR 2026 Rule 4(l) |

Notation: service is written years.days (e.g. 2.133 = 2 years, 133 days). Source for the notation: https://www.mlbtraderumors.com/2022/11/super-two-status-set-at-2-128-years-of-service.html [analysis].

### 1.2 Arbitration eligibility and Super Two

| Rule | Value | Source |
|---|---|---|
| General rule | At least 3 but under 6 years of service: the player may take his salary to final, binding arbitration **without the club's consent**. With the club's consent, any player may | [CBA] BA Art. VI(E)(1)(a) |
| Super Two | At least 2 but under 3 years of service, **at least 86 days of service in the immediately preceding season**, and in the **top 22 %** (rounded to the nearest whole number) by total service of the class with 2 to 3 years and at least 86 days. All players tied at the 22 % line are eligible | [CBA] BA Art. VI(E)(1)(b) |
| History | Super Two began in 1991 at 17 % and rose to 22 % in 2013. Arbitration needed 2 years in 1974-85 and 3 years from 1986 | [press] ESPN via ABC7, 2026-11-05, https://abc7.com/post/louis-varland-andy-pages-among-super-2-arbitration-eligibles/20018398/ |
| Number of arbitration years | Super Twos get four arbitration years, other players three *(follows from VI(E)(1): 2 to 6 years versus 3 to 6)* | [CBA] BA Art. VI(E)(1) |

Super Two cutoff by offseason (the cutoff is whatever service the 22 % line falls on each year):

| Offseason | Cutoff | Super Two players | Source |
|---|---|---|---|
| 2021-22 | 2.116 | not given | [analysis] https://www.mlbtraderumors.com/2022/11/super-two-status-set-at-2-128-years-of-service.html |
| 2022-23 | 2.128 | not given | [analysis] same (2022-11-01) |
| 2023-24 | 2.118 | 24 | [analysis] https://www.mlbtraderumors.com/2023/11/super-two-status-set-at-2-118-years-of-service.html |
| 2024-25 | 2.132 | 29 | [press] AP via Fox Sports, 2024-11-08, https://www.foxsports.com/articles/mlb/duran-crawford-among-29-super-2s-eligible-for-salary-arbitration-perdomo-just-misses-cutoff |
| 2025-26 | 2.140 | 34 | [press] ESPN via ABC7, 2025-11-07, https://abc7news.com/post/mason-miller-zach-neto-among-34-super-2s-eligible-arbitration/18126531/ |
| 2026-27 | 2.133 | 28 | [press] ESPN via ABC7, 2026-11-05 (URL above) |
| Range 2009-2021 | 2.115 (2019) to 2.146 (2011) | - | [analysis] MLBTR 2022 list |

### 1.3 Free-agency eligibility

| Rule | Value | Source |
|---|---|---|
| Article XX(B) free agent | A player with **6 or more years of service** whose contract has run out and who has not signed for the next season | [CBA] BA Art. XX(B)(1) |
| Other free agents | Players who become free agents any other way (released, non-tendered, elected after an outright, etc.) may sign with any club "without any restrictions or qualifications". The qualifying offer and compensation apply only to Article XX(B) free agents | [CBA] BA Art. XX(B)(2) |
| Club reserve rights end when | The player becomes a free agent; the club terminates the contract (UPC ¶7(b)); the player terminates it (¶7(a)); the club misses the tender deadline (¶10(a)); the club fails to renew (¶10(a)); or the contract is assigned outright | [CBA] BA Art. XX(A) |

### 1.4 Tender, non-tender and renewal

| Rule | Value | Source |
|---|---|---|
| Tender Deadline | **The last Friday before Thanksgiving** (e.g. Nov 18, 2022). The Commissioner's Office sends the MLBPA a Central Tender Letter listing, by club, every player tendered (with salary terms) and those not tendered. The Office "will attempt to confirm" contract terms submitted before 8:01 p.m. ET that day | [CBA] BA Art. XX(A), Attachment 9; UPC ¶10(a) |
| 2026 | Friday **2026-11-20**. MLBTR gives 4:00 p.m. Central for the deadline | [analysis] MLBTR offseason |
| Non-tendered player | Becomes a free agent (failure to tender, Art. XX(A)(2)(c)) with no restrictions (Art. XX(B)(2)). MLBTR: "Non-tendered players become free agents immediately, without waivers" | [CBA] BA Art. XX(A), XX(B)(2); [analysis] MLBTR offseason |
| Tender errors | An inadvertent error leads to free agency only if the player gives written notice and the club does not fix it within 7 days | [CBA] BA Art. XX(A) |
| Renewal | If no agreement by **March 1**, the club may renew for one year **within 10 days after March 1**, at a salary it fixes, subject to the maximum-cut rule (2.5) | [CBA] UPC ¶10(a) |
| Minor leaguers moved to the 40-man | Putting them on the 40-man and tendering an MLB contract gives the club reserve rights; no renewal needed | [CBA] BA Attachment 12 |
| Rule 5 picks | Tender amounts go to the MLBPA in an addendum by 8 p.m. on the day of the Rule 5 draft | [CBA] BA Attachment 44 |

### 1.5 Pre-arbitration bonus pool

| Rule | Value | Source |
|---|---|---|
| Who is eligible | Players with under 3 years of service at the start of the season. Not Super Twos, and not players signed as Foreign Professionals or Extended Free Agents | [CBA] BA Art. XV(D)(1) |
| Size and funding | **$50 million a year**, funded centrally by the Commissioner's Office. In the CBT payroll each club carries 1/30th, i.e. $1,666,666.67 | [CBA] BA Art. XV(D)(2), XXIII(C)(1)(b) |
| Awards phase | MVP or Cy Young: 1st $2.5M, 2nd $1.75M, 3rd $1.5M, 4th/5th $1.0M. All-MLB First Team $1.0M, Second Team $500K. ROY 1st $750K, 2nd $500K. **Only the single highest award counts.** All-MLB bonuses require attending the awards ceremony | [CBA] BA Art. XV(D)(3)(a). I read the chart's column positions from the PDF layout. MLBTR lists the same amounts: https://www.mlbtraderumors.com/2025/11/paul-skenes-tops-2025-pre-arbitration-bonus-pool.html [analysis] |
| WAR phase | The rest of the pool goes to the **top 100 eligible players by "Joint WAR"**, in proportion to each player's share of their combined Joint WAR. The formula is set by a joint MLB-MLBPA committee | [CBA] BA Art. XV(D)(3)(b), (4) |
| Timing | Interim reports before the All-Star Game and Sept 1-10. Final report within 5 business days of the last award announcement (no later than Dec 15). Paid by Dec 31 | [CBA] BA Art. XV(D)(3)-(4) |
| Excluded | Pool money is not salary for the maximum-cut rule and is inadmissible in arbitration | [CBA] BA Art. VI(B)(2)(f), VI(E)(10)(b)(vi) |
| 2025 outcome (calibration) | 101 players paid; every club had at least one. Largest bonus: Paul Skenes **$3,436,343** (a record; $2.5M Cy Young award plus about $0.94M from the WAR phase *(computed)*) | [analysis] FanGraphs, Clemens, 2025-12-04, https://blogs.fangraphs.com/the-pre-arb-bonus-pool-is-a-success/ ; MLBTR 2025-11-25 (above) |

---

## 2. Salaries

### 2.1 MLB minimum salary (rate per season for each day of MLB service)

| Season | Minimum | Source |
|---|---|---|
| 2022 | $700,000 | [CBA] BA Art. VI(A)(1) |
| 2023 | $720,000 | same |
| 2024 | $740,000 | same |
| 2025 | $760,000 | same |
| 2026 | $780,000 | same |
| 2027 | **Not set.** The BA lists rates only through 2026 and expires Dec 1, 2026. Proposals only: MLB (Nov 3) $1,000,000 for players with at least 2 years of service and $900,000 below that; MLBPA (May 27) $1.5 million | [CBA] BA Art. VI(A)(1), XXVI; [press] NBC tracker; [official] https://www.mlb.com/news/mlb-increases-cap-and-floor-in-latest-proposal-sets-2028-target |

Part-season pay: a player in service for part of the season receives the share of his salary that his days of employment bear to the days in the season. [CBA] UPC ¶2; [official] MLR 2026 Rule 11(b).

### 2.2 Split contracts and 40-man players in the minors

A "split contract" is a Major League UPC with separate pay rates for minor-league and major-league service. [CBA] BA Art. XXIII(A)(3)

Minimum rate for **minor-league service** on a Major League contract, per season:

| Season | First MLB contract, no MLB service | Second or later MLB contract (not covering the same season as the first), or at least 1 day of MLB service | Source |
|---|---|---|---|
| 2022 | $57,200 | $114,100 | [CBA] BA Art. VI(A)(2)-(3) |
| 2023 | $58,800 | $117,400 | same |
| 2024 | $60,300 | $120,600 | same |
| 2025 | $62,000 | $123,900 | same |
| 2026 | $63,600 | $127,100 | same |

| Related rule | Value | Source |
|---|---|---|
| Pay on option | The minor-league rate in his MLB contract, or the BA minimum if that applies | [official] MLR 2026 Rule 7(h) |
| Optioned below his level | A player who spent at least 3 months active at a minor-league level before his option and is then optioned to a lower level must be paid at the higher level's rate | [official] MLR 2026 Rule 7(c) |
| Outrighted after Labor Day | A player with at least 1 day of MLB service who is outrighted after Labor Day and tendered a salary addendum may not get less than 80 % of his last monthly minor-league rate (Addendum C) or the BA minor-league minimum for players with MLB service, whichever is greater. A player who had a full year of service that season keeps at least 80 % of his MLB contract's minor-league rate | [CBA] BA Attachment 43 |
| Minor-league deals with MLB terms | MLB terms written into a minor-league UPC are a floor: the club may not sign the player to an MLB contract on worse terms (absent a prior release) | [CBA] BA Attachment 24 |

### 2.3 Minor-league pay scale (2023-2027 Minor League BA)

The rates are **weekly**. They apply to players on their first minor-league UPC (Foreign Professionals excepted).

| Level / period | 2023 | 2024 | 2025 | 2026 | 2027 |
|---|---|---|---|---|---|
| Triple-A (championship season) | $1,200 | $1,200 | $1,225 | $1,250 | $1,275 |
| Double-A | $1,000 | $1,000 | $1,020 | $1,040 | $1,060 |
| High-A | $900 | $900 | $920 | $935 | $955 |
| Single-A | $850 | $850 | $870 | $885 | $905 |
| Rookie | $675 | $675 | $700 | $710 | $720 |
| Spring training | not listed | $625 | $650 | $660 | $670 |
| Off-season, on-site (club invites the player to a facility) | $625 | $625 | $650 | $660 | $670 |
| Off-season, off-site | $250 | $250 | $255 | $260 | $265 |

Source: [CBA] MiLB BA Art. VII(A)-(C) and Addendum C to the Minor League UPC (Appendix A).

| Related rule | Value | Source |
|---|---|---|
| Spring training | At least 4 weeks of spring-training pay (exceptions listed). No minor-league salary for players on MLB contracts or in MLB camp drawing MLB allowances | [CBA] MiLB BA Art. VII(B) |
| Dead Period | Saturday before Thanksgiving to Jan 1: no activities, no salary | [CBA] MiLB BA Art. VI(C)(3) |
| Arizona Fall League | Paid at the Double-A rate | [CBA] MiLB BA Art. VII(C)(4) |
| Minimum season length (shorter seasons are paid up to the minimum) | AAA 24 weeks, AA 23, A/A+ 22, Rookie 16 | [CBA] MiLB BA Art. VI(A)(1) |
| Players on later contracts (non-first-year) may negotiate | A championship-season rate above the scale, **up to $8,400/week**; guaranteed payments (signing bonus, termination pay) **up to $100,000 in total**; at most **3 clauses** obliging the club to release him if not added to the MLB roster by a date, and at most 3 obliging assignment to another club's MLB roster; a term of **at most 2 seasons**; MLB terms if he is added to the roster, **at most $4 million** in total compensation and bonuses; housing, his own hotel room, an interpreter, the right to negotiate with a foreign team, an MLB camp invitation | [CBA] MiLB BA Art. III(B)(2) |
| No extras | Clubs may not give minor leaguers any benefits beyond the agreement | [CBA] MiLB BA Art. VIII |

### 2.4 Release of minor leaguers (for completeness)

| Rule | Value | Source |
|---|---|---|
| Termination | The club may end a Minor League UPC "in its sole discretion and for any reason, at any time". Pay stops on termination | [CBA] MiLB BA Appendix A, UPC ¶17(b), ¶6(a) |
| Injured on duty and released | 2 weeks' salary from the date of injury | [CBA] MiLB BA UPC ¶17(c) |

### 2.5 Maximum salary reduction (renewals and re-signing of reserved players)

| Rule | Value | Source |
|---|---|---|
| MLB salary | A club may not tender, sign or renew a reserved player at a cut of **more than 20 %** from his MLB salary of the previous season, or **more than 30 %** from his MLB salary two seasons earlier | [CBA] BA Art. VI(B)(1)(a) |
| Minor-league salary (split contracts) | No cut of **more than 50 %** from the previous season's minor-league salary | [CBA] BA Art. VI(B)(1)(b) |
| What counts as previous salary | Base salary + signing bonus (prorated over a multi-year deal) + performance bonuses (the club either counts the full bonus or repeats it on the same terms) + deferred money at present value. If a multi-year deal's yearly rates would break the rule, its average annual value is used instead. Amounts lost to fines or suspensions count. Exercised option years count. Pre-arb pool money does not | [CBA] BA Art. VI(B)(2) |
| Larger cut with the Commissioner's consent | A player may accept a renewal below 80 % of the preceding rate with the Commissioner's consent, if the salary is agreed and the request reaches the Commissioner before Jan 10 | [official] MLR 2026 Rule 11(e) |

---

## 3. Salary arbitration

### 3.1 Calendar

| Step | Rule | Source |
|---|---|---|
| Eligible list | MLBPA and the league's Labor Relations Department confirm the list "promptly after the tender date". The 2022-26 text has **no separate filing date** | [CBA] BA Art. VI(E)(2) |
| Exchange of figures | 2023: Fri Jan 13 · 2024: Fri Jan 12 · 2025: Fri Jan 10 · 2026: Fri Jan 9. A settlement not reported by 1 p.m. ET on that day is not confirmed until after the exchange | [CBA] BA Art. VI(E)(2)-(3) |
| Hearings | Scheduled within 5 days after the exchange, for dates **Jan 26 to Feb 20** unless the parties agree otherwise. One site, preferably Tampa/St. Petersburg or Phoenix | [CBA] BA Art. VI(E)(6), (13) |
| Decision | The panel may rule the same day and must try to rule within 24 hours. It must pick one of the two figures, with no written opinion. Individual votes are disclosed on March 15 | [CBA] BA Art. VI(E)(13) |
| 2027 | MLBTR lists **Jan 8, 2027** for the exchange and early February for hearings, both flagged as unlikely to happen as planned under a lockout. There is no 2027 date in the BA | [analysis] MLBTR offseason |
| Lockout precedent | In 2022 the hearings ran in May-June after the lockout (first decisions May 11, Swanson/Duvall June 8, Fried June 22) | [press] https://www.tsn.ca/braves-beat-riley-cards-defeat-o-neill-in-arbitration-1.1798264 ; https://www.mlbtraderumors.com/2022/06/dansby-swanson-wins-arbitration-case-braves-win-case-over-adam-duvall.html ; https://www.espn.com/mlb/story/_/id/34132444/atlanta-braves-max-fried-wins-arbitration-case-gets-685-million |

### 3.2 Hearing rules that matter for a model

| Rule | Value | Source |
|---|---|---|
| Format | Final offer: each side submits a single figure. Three-arbitrator panels. Each side gets 1 hour plus 30 minutes for rebuttal. Neither side carries the burden of proof. Costs are split | [CBA] BA Art. VI(E)(4)-(9) |
| Criteria | Last season's contribution (performance, leadership, public appeal), length and consistency of career, past pay, comparable salaries, physical or mental defects, the club's recent record and attendance. Except for players with 5+ years of service, the panel weighs players **no more than one service class above** most heavily | [CBA] BA Art. VI(E)(10)(a) |
| Not admissible | Club or player finances; press comments (recognised awards allowed); offers made before arbitration; costs of representation; salaries in other sports; the pre-arb pool; Statcast and wearable data. Only public statistics are allowed (subscription sites count). Nobody may mention the CBT | [CBA] BA Art. VI(E)(10)(b)-(c), (12) |
| Salary data | The panel receives a confidential list of every rostered player's salary as of Aug 31, by service class | [CBA] BA Art. VI(E)(11) |
| Guarantee | A tendered arbitration-eligible player who **settles before his hearing** gets full in-season termination pay (the whole agreed salary) if released for lack of skill before Opening Day. The clause does not mention salaries awarded at a hearing | [CBA] BA Art. VI(E)(3) |

### 3.3 Hearings held and who won

| Year | Hearings | Players won | Teams won | Source |
|---|---|---|---|---|
| 2022 (May-June) | 13 | 4 | 9 | [press] ESPN, 2022-06-22 ("Teams have a 9-4 advantage", with only Judge still scheduled), URL in 3.1; AP 2023 ("19 hearings this year were up from 13 last year"). 9 + 4 = 13, so Judge's case did not reach a decision *(computed)* |
| 2023 | 19 | 6 | 13 | [press] AP via Fox Sports, 2023-02-18, https://www.foxsports.com/articles/mlb/ms-hernandez-loses-in-arbitration-teams-go-136-vs-players |
| 2024 | 15 | 9 | 6 | [press] AP via Fox Sports, 2025-02-15 ("players went 9-6" in 2024), https://www.foxsports.com/articles/mlb/nathaniel-lowe-loses-to-washington-nationals-in-final-salary-arbitration-case-of-the-year |
| 2025 | 9 | 4 | 5 | same |
| 2026 | 12 | 8 | 4 | [press] AP via Fox Sports, 2026-02-14, https://www.foxsports.com/articles/mlb/marlins-win-arbitration-hearing-vs-pitcher-calvin-faucher-players-finish-84-against-teams |
| 2022-2026 total | 68 | 31 | 37 | *(computed)* |
| All-time, 1974-2026 | 640 | 278 | 362 | [press] AP 2026 (same URL). Consistent with AP's 347-257 after 2023 and 358-270 after 2025 *(computed check)* |

2026 detail (AP): Tarik Skubal won a record **$32M** (the club filed $19M). The other player wins ranged from $6.8M down to $1.525M; the team wins were at $4.4M to $1.8M.

### 3.4 Arbitration salary as a share of free-agent value

The studies define the share differently, so their figures are not directly comparable.

| Source | Definition | Arb 1 | Arb 2 | Arb 3 | Arb 4 |
|---|---|---|---|---|---|
| FanGraphs, Ben Clemens, "The Details of Our New Prospect Valuation Methodology", 2026-07-02, https://blogs.fangraphs.com/the-details-of-our-new-prospect-valuation-methodology/ [analysis] | Share of the **previous season's** WAR valued at free-agent $/WAR. Chosen because it "produced the lowest mean squared error when compared to actual arbitration payouts under the new CBA" | 15 % | 35 % | 50 % | 75 % |
| Rule of thumb quoted by FanGraphs (Sheryl Ring, 2019-02-11, quoting Travis Sawchik at FiveThirtyEight), https://blogs.fangraphs.com/lets-fix-mlbs-salary-arbitration-system-introducing-restricted-free-agency/ [analysis] | Share of open-market value ("though that estimate does not always apply") | ~40 % | 60 % | ~80 % | - |
| The Hardball Times, Matthew Murphy, 2014-05-22, own study (3 seasons, 200+ players), https://tht.fangraphs.com/the-net-value-of-draft-picks/ [analysis] | Share of market value, all players. Hitters 22/32/40; starters 26/35/70; relievers 52/80/143 | 28 % | 37 % | 62 % | - |
| Nate Silver (Baseball Prospectus, 2005), **as cited** by the THT article above; original not read [analysis] | Share of market value | 44 % | 61 % | 64 % | - |

---

## 4. Free agency

### 4.1 Calendar

| Event | Rule | 2026-27 dates | Source |
|---|---|---|---|
| Free agency begins | 9 a.m. ET on the day after the day the last World Series game started | Oct 29 | [CBA] BA Art. XX(B)(2)(a); [analysis] MLBTR offseason |
| Quiet Period | From that moment until **5 p.m. ET on the 5th day** after the day the last WS game started. Any club may talk with a free agent (interest, role, length, guarantees, no-trade terms) but **may not negotiate terms or sign**. The former club **may negotiate and sign** | Ends Nov 2, 5 p.m. ET (4 p.m. CT) | [CBA] BA Art. XX(B)(2)(b); [analysis] MLBTR offseason |
| Signing with any club | From the end of the Quiet Period | From Nov 2 | [CBA] BA Art. XX(B)(2)(c) |
| Contract options and opt-outs | All exercise dates must fall within the Quiet Period | By Nov 2 | [CBA] BA Art. XX(B)(6)(d) |
| Qualifying offers made | During the Quiet Period. The league tells the MLBPA by 5 p.m. ET on its last day | Nov 2: 8 offers | [CBA] BA Art. XX(B)(3); [press] CBS 2026-11-02 (4.3) |
| QO acceptance period | Until the date clubs file reserve lists (MLR 1(a)). The MLBPA sends the list of acceptances by 4 p.m. ET that day; anyone not on it has rejected | **Nov 17, 4 p.m. ET** | [CBA] BA Art. XX(B)(3); [analysis] https://www.mlbtraderumors.com/2026/11/the-opener-skubal-meetings-qo-decisions-rule-5-deadline-bauers-pagan.html |
| Reserve lists filed (= Rule 5 protection) | On a date the Commissioner sets, between the Monday before the Tender Deadline and the day before it | Nov 17, 6 p.m. ET | [official] MLR 2026 Rule 1(a); [analysis] MLBTR Opener (above) |
| Tender Deadline | Last Friday before Thanksgiving | Nov 20 | see 1.4 |
| Minor-league free agents | 5 p.m. ET on Oct 15 or on the 5th day after the last day of the WS, whichever is later. They may sign from the first day MLB free agents may sign elsewhere | Nov 2 *(computed: WS ended Oct 28 per mlb_notes)* | [official] MLR 2026 Rule 9(c)(1) |
| Trades of newly signed free agents | An Article XX(B) free agent signed after the Quiet Period cannot be traded until after June 15. The only exception before June 16 needs his written consent and is limited to trades for other players' contracts and/or cash of $50,000 or less. MLBTR applies this to players who accept the QO | - | [CBA] BA Art. XX(B)(6)(a); [analysis] MLBTR offseason |
| CBA expiry | 11:59 p.m. ET, Dec 1, 2026 (lockout expected Dec 2; see 9) | - | [CBA] BA Art. XXVI |

### 4.2 Qualifying offer: amount and how it is computed

| Offseason | Season covered | QO amount | Source |
|---|---|---|---|
| 2021-22 | 2022 | $18.4M | [analysis] MLBTR, 2025-10-14, https://www.mlbtraderumors.com/2025/10/qualifying-offer-price-set-at-22-025mm.html |
| 2022-23 | 2023 | $19.65M | same |
| 2023-24 | 2024 | $20.325M | same |
| 2024-25 | 2025 | $21.05M | same |
| 2025-26 | 2026 | $22.025M | same (via AP) |
| 2026-27 | 2027 | **$23.15M**: up 5.1 % on the year and 74 % since the first QO ($13.3M in 2012-13) | [official] MLB glossary, https://www.mlb.com/glossary/transactions/qualifying-offer ; [press] FOS, 2026-10-21, https://frontofficesports.com/article/mlb-qualifying-offer-figure-rises-amid-labor-strife/ |

How it is computed ([CBA] BA Art. XX(B)(3) and Attachment 45 with its Addendum A):

1. The **average salary of the 125 highest-paid players**, but never below the QO for the 2017 season ($17.2M per MLBTR's 2016-17 entry *(cross-reference)*).
2. The player pool is everyone on a 40-man roster (or outrighted) or on the 60-day IL on **Aug 31**. It also includes, for one year only, players who retired or went on the Restricted List since the prior season ended, and players released but still owed that season's salary as termination pay.
3. A player's salary = base salary (after escalators) + prorated signing bonus + prorated buyout of the first club or mutual option + bonuses earned by the end of the season. Deferred amounts are discounted to present value.
4. The average is raised **0.08 %** for award bonuses earned after the season, then **rounded to the nearest $50,000**.
5. The parties confer by the 7th day after the last regular-season game and confirm the figure by the 10th.

### 4.3 Qualifying offers made and accepted

| Offseason | Offers | Accepted | Who accepted | Source |
|---|---|---|---|---|
| 2012-2020 (9 offseasons) | 96 *(computed: sum of CBS's yearly rows)* | 10 *(computed)* | - | [press] CBS, 2026-11-02 (table), https://www.cbssports.com/mlb/news/mlb-qualifying-offer-2027-contract/ |
| 2021 | 14 | 1 | Brandon Belt (SF) | CBS table; [official] MLB.com, Simon, 2026-11-02, https://www.mlb.com/news/players-who-have-accepted-qualifying-offers |
| 2022 | 14 | 2 | Joc Pederson (SF), Martín Pérez (TEX) | same |
| 2023 | 7 | 0 | - | same |
| 2024 | 13 | 1 | Nick Martinez (CIN) | same |
| 2025 | 13 | 4 (record) | Brandon Woodruff (MIL), Shota Imanaga (CHC), Trent Grisham (NYY), Gleyber Torres (DET) | same |
| 2026 | 8 | pending at research time | see below | same |
| Total 2012-2025 | 157 | 18 (11.5 % *computed*) | - | CBS: "157 qualifying offers were made and only 18 accepted"; MLB.com: 165 offers including the 8 in 2026 |

The 2026 offers went to Adrián Morejón (SD), Trevor Rogers (BAL), Randy Arozarena (SEA), Seiya Suzuki (CHC), Ryan Jeffers (MIN),
Jake Bauers (MIL), Brandon Lowe (PIT) and David Bednar (NYY). [press] CBS (above)

Decisions reported by about midday on Nov 17:
- Bauers declined and agreed to 4 years, $58M with Boston. Milwaukee receives "a pick between the end of the first round and Competitive Balance Round A" in the 2027 draft. [official] https://www.mlb.com/news/mlb-rumors-and-2026-27-free-agent-news-november-17
- Morejón, Lowe and Suzuki are declining. [official] same page
- Jeffers will decline, per the Minnesota Star Tribune as relayed by MLBTR. [press] https://www.mlbtraderumors.com/2026/11/ryan-jeffers-to-decline-qualifying-offer.html
- Arozarena, Bednar and Rogers: **NOT CONFIRMED** (no decision found before the 4 p.m. ET deadline).

### 4.4 Draft-pick compensation and penalties

A Qualified Free Agent brings compensation **only if** three things happen: his former club made the QO; he declined it (or signed elsewhere during the acceptance period); and he signs a **Major League** contract with another club, confirmed **on or before the day before the next Rule 4 draft**. A bona fide minor-league contract never brings compensation. [CBA] BA Art. XX(B)(4)(a)

Former club (status in the season just completed):

| Former club | Compensation pick | Source |
|---|---|---|
| CBT payor (whatever its revenue-sharing status) | After the **4th round** | [CBA] BA Art. XX(B)(4)(b)(ii) |
| Revenue-sharing payee, not market-disqualified, and the player signs for a **total guarantee of $50M or more** | After the **1st round** (after the PPI picks) | [CBA] BA Art. XX(B)(4)(b)(i) |
| Everyone else, including payees whose player signs for less than $50M | After **Competitive Balance Round B** | [CBA] BA Art. XX(B)(4)(b); [official] MLB glossary (QO page above), which agrees |

- "Total guarantee" = guaranteed salary including player-option years, plus any signing bonus, plus the buyout of the first club or mutual option, plus bonuses, escalators and vesting options only to the extent his past performance would already have earned them. Deferrals are at present value. [CBA] BA Art. XX(B)(4)(b)
- Several picks after the same round go in reverse order of winning percentage; a club with more than one picks them in succession. [CBA] same

Signing club:

| Signing club | Forfeits | International bonus pool | Source |
|---|---|---|---|
| Revenue-sharing payee, not market-disqualified | Its **3rd-highest** available pick | no change | [CBA] BA Art. XX(B)(4)(c)(i)(A) |
| CBT payor | Its **2nd- and 5th-highest** picks | **minus $1,000,000** | [CBA] BA Art. XX(B)(4)(c)(i)(B) |
| All others | Its **2nd-highest** pick | **minus $500,000** | [CBA] BA Art. XX(B)(4)(c)(i)(C) |
| Each further QFA signed | The next-highest available pick(s). The BA's example: a CBT payor loses its 3rd- and 6th-round picks for its second QFA | same | [CBA] BA Art. XX(B)(4)(c)(ii)(C); [official] MLB glossary |

- Exempt from forfeiture: picks awarded under MLR 4(c)(2) and 4(k)(2). Compensation picks and Competitive Balance picks obtained in a trade **can** be forfeited. [CBA] BA Art. XX(B)(4)(c)(ii)(A)-(B)
- Forfeited pool money is shared equally among clubs that forfeited none. If a club's pool cannot cover the penalty, the rest comes out of later signing periods. [CBA] BA Art. XX(B)(4)(c)(iii)
- The Athletics count as a payee unless fully market-disqualified. [CBA] BA Art. XX(B)(4)(c)(iv)
- Picks are awarded and forfeited when the last QFA signs or on the day before the draft, whichever comes first. [CBA] BA Art. XX(B)(4)(c)(ii)

### 4.5 Who can and cannot receive a QO; QO side rules

| Rule | Value | Source |
|---|---|---|
| Eligible ("Qualified Free Agent") | Only an Article XX(B) free agent who has been **continuously under reserve to the same club** (majors or minors) **since Opening Day** of the season just ended, and who has **never before received a QO** from any club | [CBA] BA Art. XX(B)(3) |
| So these cannot get one | Players traded or claimed during the season ("In-season acquisitions are ineligible"); anyone who has had a QO before; players with under 6 years of service or who became free agents another way (release, non-tender, outright election) | [CBA] BA Art. XX(B)(1)-(3); [official] MLB glossary |
| Accepting | A signed one-year contract at the QO amount, with full in-season termination pay from the day of acceptance | [CBA] BA Art. XX(B)(3) |
| No side deals | A club and player may not agree that the club won't make a QO or that the player won't accept one. Penalty: discipline, possibly loss of draft picks | [CBA] BA Art. XX(B)(6)(c) |
| Outside organised baseball | No restriction on a free agent signing with a club outside organised baseball, and no compensation except under XX(B) | [CBA] BA Art. XX(B)(6)(b) |

### 4.6 Veterans (Article XX(B) free agents) on minor-league deals

| Rule | Value | Source |
|---|---|---|
| Uniform opt-outs | If he signs a minor-league UPC at least 10 days before the season, he may demand his release on three dates. **(i)** Effective the 4th day before Opening Day, requested by 2 p.m. ET on the 6th day before. **(ii)** Effective May 1, requested by 2 p.m. ET on April 27. **(iii)** Effective June 1, requested by 2 p.m. ET on May 28. Each time the club must put him on the 26-man roster or MLB IL (for (i), commit to that by noon ET on the 4th day before), or release him | [CBA] BA Art. XX(B)(5)(a) |
| No extra opt-out dates | No special covenant may give these players additional release dates | [CBA] BA Art. XX(B)(5)(b) |

---

## 5. Roster mechanics that keep, move or release players

### 5.1 40-man roster

| Rule | Value | Source |
|---|---|---|
| Limit | A club may hold up to **40** Major League contracts (the Major League Reserve List) | [CBA] BA Art. XX(A); *confirms* mlb_notes c (MLR 2(a)(2)(A)) |
| Restricted, Voluntarily Retired, Military and other lists | Kept under separate headings on the reserve list | [CBA] BA Art. XX(A) |

### 5.2 Options

| Rule | Value | Source |
|---|---|---|
| Option seasons | **3**. A season in which the player is optioned for less than 20 days in total is not charged | [official] MLR 2026 Rule 7(c); [CBA] BA Art. XIX(E) |
| Fourth option | Allowed for a player with fewer than **5 "seasons of service"** before the current season. A season of service = **90+ days** on an Active List or Development List in a championship season | [official] MLR 2026 Rule 7(c) Exception |
| Per-season limit | After **5** optional assignments in one season (spring training not counted), the next one needs outright waivers. A new club may option the player one more time. *Confirms* mlb_notes ("at most 5 times per season") | [official] MLR 2026 Rule 7(d) |
| How options are charged | One option year per season at most, charged only when that year's closed period begins. A player recalled after 20 or more days of the season is charged an option | [official] MLR 2026 Rule 7(k)(3)-(6) |
| Club cap | At most **15** players on option at once before Sept 1, **12** on or after | [official] MLR 2026 Rule 7(e) |
| Injured players | Cannot be assigned to the minors while on the MLB IL; must be healthy before an option | [CBA] BA Art. XIX(C)(1) |
| Recall wait (10/15 days) | Already in mlb_notes c; *confirmed* | [official] MLR 2026 Rule 7(b)(1) |

### 5.3 Designated for assignment, waivers, outright assignment

| Rule | Value | Source |
|---|---|---|
| DFA | Notice to the Commissioner that the club will release or assign a player when the roster is full. Within **7 days** the player must be released or his contract assigned. Christmas Day to New Year's Day does not count. During the DFA he is paid at his MLB rate, credited with service and cannot play | [CBA] BA Art. XIX(G); [official] MLR 2026 Rule 2(c)(5) |
| Two kinds of waivers | Outright assignment waivers (needed to send a player to the minors without right of recall) and unconditional release waivers. Requests are **irrevocable** | [official] MLR 2026 Rule 8(a), 8(d)(2) |
| Claim periods | Request by 2 p.m. ET. Release-waiver claims are due by **1 p.m. ET the next day**; outright-waiver claims by **1 p.m. ET two days later**. Any day counts from spring training through the season. In the off-season only Mon-Fri count (a Thursday outright request closes Monday) | [official] MLR 2026 Rule 8(b)(1), 8(b)(3) |
| Outright waivers last | Until 1 p.m. ET on the **7th day** after they are obtained | [official] MLR 2026 Rule 8(a)(2), 8(b)(4) |
| Claim priority | Lowest winning percentage, regardless of league. The first 30 days of the season and the off-season use the previous season's final standings. A club that has already claimed the same player on outright waivers since the last voluntary spring-training report date goes to the back of the queue | [official] MLR 2026 Rule 8(c)(3) |
| Claim price | **$50,000** on outright waivers. 50 % of the selection price for a Rule 5 pick or a draft-excluded player. **$1** on release waivers. The claiming club takes on the contract | [official] MLR 2026 Rule 8(c)(4), 9(b)(5); [CBA] UPC ¶7(d)(2) |
| Limits | At most 7 waiver requests per club per day. In season, a claimed player cannot be put on outright waivers or DFA'd until 48 hours after the claim is awarded or until he has played one game on the Active List, whichever comes first. In the off-season the wait is 7 days | [official] MLR 2026 Rule 8(d)(1), 8(d)(9), 8(g) |
| Injured players | Outright waivers on an IL player in season only after the minimum IL stay and with a guarantee that he is fit. Once waivers clear, outright or reinstate within 72 hours | [official] MLR 2026 Rule 8(d)(6) |
| Reacquiring an outrighted player | The same club may bring him back any time, except during the closed period or within 10 days (15 for pitchers) of an in-season outright (exceptions listed) | [official] MLR 2026 Rule 7(k)(1)-(2) |

### 5.4 Players who can refuse an outright or a minor-league assignment

| Player | Right | Source |
|---|---|---|
| **At least 3 years of MLB service**, or a Super Two as of the end of the prior season | May refuse an outright and become a free agent (without termination pay). If he accepts, he may still elect free agency in the **Election Period**: from the end of the MLB season to Oct 15, or to 8 p.m. ET on the 2nd day after his club's last postseason game if that is later, unless he is back on an MLB roster first. A Super Two who accepts his first outright loses this after-season right | [CBA] BA Art. XX(D)(1), (3) |
| **Outrighted for the 2nd time or later** in his career | Same right, regardless of service | [CBA] BA Art. XX(D)(2) |
| Procedure | Written notice no earlier than 4 days before the outright (8 days if he has no options left, or in the off-season). The player answers within **2 days** (3 in the off-season); no decision is final for 24 hours. An election that gives up termination pay takes effect only when the MLBPA confirms it in writing | [CBA] BA Art. XX(D)(3)-(4) |
| **At least 5 years of MLB service** (excluding Military List time; or 7 years including it) | Cannot be assigned to any club other than an MLB club without his written consent. He may consent, refuse, or elect free agency. A player who consents may still elect free agency after the season unless recalled. Same notice periods | [CBA] BA Art. XIX(A)(2) |
| Advance consent | A 5-year player may consent in advance to one named assignee club, given no more than 10 days before the season, for an assignment within 45 days. Players with at least 4.127 may give it effective once they reach 5 years | [CBA] BA Art. XIX(A)(3) |

### 5.5 Release and termination pay (MLB contracts)

| Moment of release for lack of skill (UPC ¶7(b)(2)) | Termination pay | Source |
|---|---|---|
| Off-season (end of season to start of spring training) | **30 days'** pay at his next-season rate (or the tendered rate) | [CBA] BA Art. IX(A) |
| Spring training, on or before the 16th day before Opening Day | **30 days'** pay | [CBA] BA Art. IX(B) |
| Spring training, after that | **45 days'** pay | [CBA] BA Art. IX(B) |
| In season | **The full unpaid balance of the season's salary** | [CBA] BA Art. IX(C) |
| Injured on duty | Unpaid balance of that year's salary, less workers' compensation | [CBA] BA Art. IX(E) |
| Split contracts | Minor-league rate in the off-season and early spring training. MLB rate after the 16th day before Opening Day. In season, the rate of wherever he is. A player who cannot be optioned without consent, or a Rule 5 pick, gets the MLB rate unless released in season while assigned to the minors | [CBA] BA Art. IX(D) |
| Offset | Total termination pay is reduced by whatever he earns from any club that year (the releasing club pays only the difference). A player who refuses a reasonable MLB offer from another club loses the part he would not have been owed | [CBA] BA Art. IX(F) |
| Procedure | Release waivers first ($1 claim). If claimed, the player may within **5 days** terminate the contract instead of reporting. If unclaimed, he is a free agent | [official] MLR 2026 Rule 9(b)(1)-(7); [CBA] UPC ¶7(d) |
| Re-signing by the releasing club | A player released in season cannot return to that club's Active List within **30 days** of the waiver request. A player released between Aug 31 and the next Opening Day cannot re-sign a **Major League** contract with that club until **May 15** (unless he signed an MLB deal elsewhere in between) | [official] MLR 2026 Rule 9(b)(9) |
| Guaranteed contracts | The BA does not use "guaranteed contract"; for the CBT a "Guaranteed Year" is one in which more than 50 % of base salary is guaranteed if he is released under UPC ¶7(b)(2) | [CBA] BA Art. XXIII(A)(8) |

### 5.6 Trade protection: 10-and-5 rights, no-trade clauses

| Rule | Value | Source |
|---|---|---|
| 10-and-5 | A player with **10+ years of MLB service, the last 5 with one club**, cannot be traded to another MLB club without his written consent | [CBA] BA Art. XIX(A)(1); [official] https://www.mlb.com/glossary/transactions/10-and-5-rights |
| Waiving 10-and-5 in advance | Possible only in a multi-year contract signed **before** he reaches 10-and-5, and only if it has a no-trade clause that limits trades in every year to no more than **16 clubs** he names | [CBA] BA Art. XIX(A)(1) |
| No-trade clauses | A normal subject of negotiation (listed among the Quiet Period topics). The BA sets no other limit on them that I found | [CBA] BA Art. XX(B)(2)(b)(vi) |
| Foreign assignments | No assignment outside the US and Canada without his consent | [CBA] BA Art. XIX(D) |

### 5.7 Rule 5 draft

| Rule | Value | Source |
|---|---|---|
| Who is eligible | Players on minor-league reserve lists (not the 40-man). A player who was **18 or younger on the June 5 before he signed** becomes eligible at the **5th** Rule 5 meeting after he signs; **19 or older**, the **4th**. Signing between the end of the season and that winter's Rule 5 meeting counts as signing after it | [official] MLR 2026 Rule 5(c)(1); [official] https://www.mlb.com/glossary/transactions/rule-5-draft |
| Other eligible players | Once eligible, always eligible while on a minor-league reserve list. Also eligible: outrighted players, minor-league free agents signed for next year, a player re-signed within a year of his release. A club may make any player selectable | [official] MLR 2026 Rule 5(c)(2)-(5) |
| Phases and price | MLB phase **$100,000**; Triple-A phase **$24,000** | [official] MLR 2026 Rule 5(b), 5(d) |
| Roster rule | Until the end of the next season the pick cannot be released or sent to the minors until all three of these have happened: a 15-day trial, outright waivers cleared, and an offer back to his old club for **50 % of the price** ($50,000), which it rejected. If he spends fewer than **90 days** on the Active List that season, the rules carry on until he reaches 90 days (IL time does not count). MLB glossary: picks "go directly to the selecting club's 26-man roster" | [official] MLR 2026 Rule 5(g)(1)-(3); MLB glossary (above) |
| Waiver timing | No outright waivers on a Rule 5 pick from his selection until 25 days before the next Opening Day; no assignment until 20 days before it | [official] MLR 2026 Rule 8(d)(3) |
| Order | Reverse order of the previous season's winning percentage, one pick per round, until clubs pass or their reserve lists are full | [official] MLR 2026 Rule 5(b) |
| 2026 schedule | MLB phase scheduled for Dec 9 but "unlikely" if there is a lockout; the MLBPA proposed moving it to November; the minor-league phase is to go ahead. The Rule 5 draft was cancelled during the 2021-22 lockout | [analysis] MLBTR offseason; MLBTR Opener (4.1) |

### 5.8 Minor-league free agency (*confirms and corrects* levels.md §4.2)

| Rule | Value | Source |
|---|---|---|
| Term of a first contract | **7 championship seasons**, or **6** if the player was **19 or older on the June 5 before he signed**. Foreign Professionals excluded | [CBA] MiLB BA Art. III(A)(1) |
| Effective date | Applies to all first-year contracts **entered into after March 30, 2023**. Contracts signed earlier keep their stated term. *Corrects* the fan-guide date (June 5, 2023) cited in levels.md §4.2 | [CBA] MiLB BA Art. III(A)(1), Art. III (existing contracts) |
| What counts as a season | **At least 1 day** in the season on a minor-league Active, Injured or Development List. Not counted: a season spent wholly on MLB lists, or one with 14+ days on the Restricted List for refusing to play. At most one season per calendar year. This is the "parts of" in "parts of six/seven seasons" | [CBA] MiLB BA Art. III(A)(3) |
| Later contracts | At most 2 seasons (see 2.3) | [CBA] MiLB BA Art. III(B)(2)(d) |
| When he becomes a free agent | 5 p.m. ET on Oct 15 or the 5th day after the WS, whichever is later, in the last year of his contract, unless the club has renewal options left or he is on the 40-man. Successor contracts may only be negotiated after his season and playoffs and before that time | [official] MLR 2026 Rule 9(c)(1)-(2) |
| Eligibility list | The Commissioner circulates a list of players with expiring contracts by Aug 1 | [official] MLR 2026 Rule 9(c)(3) |

### 5.9 Foreign professionals (relevant for KBO/NPB players)

A "Foreign Professional" is a player who, when he signs, is **at least 25** and has played **at least six seasons** in a foreign
professional league recognised by the Commissioner's Office. International amateurs may sign MLB contracts only if they qualify.
[official] MLR 2026 Rule 3(a)(1)(C). Foreign Professionals are excluded from the pre-arb pool ([CBA] BA Art. XV(D)(1)) and from PPI
([official] MLR 4(l)(1)). Posting agreements with KBO/NPB were **not researched** here.

---

## 6. Payroll limits

### 6.1 Competitive Balance Tax thresholds (Contract Year = Dec 2 to Dec 1)

| Contract Year | Base threshold | 1st surcharge | 2nd surcharge | 3rd surcharge |
|---|---|---|---|---|
| 2022 | $230M | $250M | $270M | $290M |
| 2023 | $233M | $253M | $273M | $293M |
| 2024 | $237M | $257M | $277M | $297M |
| 2025 | $241M | $261M | $281M | $301M |
| 2026 | $244M | $264M | $284M | $304M |
| 2027 | **None in the BA.** "There shall be no Competitive Balance Tax in place following the 2026 championship season" | | | |

Source: [CBA] BA Art. XXIII(A)(1), (B)(2), (B)(4)(a), (I).

### 6.2 Tax rates (marginal, by bracket)

| Payroll above the base threshold | First-time payor | Second-time | Third-time or more |
|---|---|---|---|
| Up to $20M (base rate) | 20 % | 30 % | 50 % |
| $20M-$40M (base + 12 %) | 32 % | 42 % | 62 % |
| $40M-$60M (base + 45 %; 42.5 % for first-timers) | 62.5 % | 75 % | 95 % |
| Over $60M (base + 60 %) | 80 % | 90 % | 110 % |

- First-time = was under the base threshold the previous year. Second-time = over it the previous year but not two years ago. Third-time = over it both previous years. Years under the 2017-21 agreement count. One year under the line therefore resets the club to the first-time rate *(follows from the definitions)*.
- Source: [CBA] BA Art. XXIII(B)(3)-(4).

| Mechanics | Value | Source |
|---|---|---|
| Measured | Final payroll calculated on the **Dec 2** after the Contract Year. Clubs are told on Dec 2 and pay by **Jan 21**. A late payer has up to 50 % of its Central Fund distributions withheld, plus interest | [CBA] BA Art. XXIII(A)(11), (B)(5) |
| Draft penalty | A club **at or above the 2nd surcharge threshold** has its **highest pick moved back 10 places**, every year it is there. If that pick is in the top 6 of the 1st round, its second-highest pick moves instead | [CBA] BA Art. XXIII(B)(4)(c) |
| Other penalties tied to CBT status | Signing a QFA costs the 2nd- and 5th-highest picks and $1M of international pool (4.4). Losing a QFA brings a pick only after the 4th round (4.4). A CBT payor that also pays into revenue sharing gives up part of its market-disqualification refund (6.5). There is no separate international-pool penalty for paying the tax in this agreement (not found in Art. XXIII) | [CBA] BA Art. XX(B)(4), XXIV(A)(12)(b) |
| Use of proceeds | First $3.5M to the player benefit plan; then 50 % to players' retirement accounts and 50 % to a fund for revenue-sharing payees that grow non-media local revenue | [CBA] BA Art. XXIII(H) |

### 6.3 What counts toward CBT payroll ("Actual Club Payroll")

| Item | Rule | Source |
|---|---|---|
| Components | 1/30 of player benefit costs + 1/30 of the pre-arb pool ($1,666,666.67) + the "Salary" of every player on an MLB contract, optioned players included | [CBA] BA Art. XXIII(C)(1) |
| Multi-year deals | **AAV** over the guaranteed years (base salaries + signing bonus + deferral costs, divided by guaranteed years). A deal signed after Opening Day counts its first year as a partial year | [CBA] BA Art. XXIII(E)(2) |
| Signing bonus | Spread evenly over the guaranteed years (all in year 1 if there are none) | [CBA] BA Art. XXIII(E)(3) |
| Bonuses | Counted when earned; potential bonuses are not in the AAV | [CBA] BA Art. XXIII(E)(4) |
| Options and buyouts | See section 7 | [CBA] BA Art. XXIII(E)(5) |
| Deferred money | At present value (federal mid-term rate) unless it carries interest within 1.5 points of that rate | [CBA] BA Art. XXIII(E)(6) |
| Trades | Salary is split at the trade date. The receiving club recalculates the AAV from the next day. Cash sent is charged to the payer over the remaining guaranteed years and credited to the receiver, which cannot go below zero ("can't go negative") | [CBA] BA Art. XXIII(C)(2)(b) |
| Benefit costs (calibration) | MLBTR's Ethan Hullihen projects **$18,561,327 per club** for 2027 | [analysis] https://www.mlbtraderumors.com/2026/11/mlb-proposal-salary-cap-2028-five-year-free-agency-million-league-minimum.html |

### 6.4 2025 tax bills (calibration)

| Club | 2025 CBT payroll | Tax |
|---|---|---|
| Dodgers | $417,341,608 | $169,375,768 |
| Mets | $346,670,456 | $91,637,501 |
| Yankees | $319,522,564 | $61,774,820 |
| Phillies | $314,329,912 | $56,062,903 |
| Blue Jays | $286,135,551 | $13,609,719 |
| Padres | $270,351,396 | $6,992,447 |
| Red Sox | $248,859,139 | $1,496,828 |
| Astros | $245,991,459 | $1,497,438 |
| Rangers | $241,380,966 | $190,483 |

- Source: [press] CBS Sports (Axisa, 2025-12-20, citing AP), https://www.cbssports.com/mlb/news/dodgers-record-169-million-luxury-tax-bill-blue-jays-yankees-cbt . Nine payors, which ties the record; "just under $403 million" in total.
- Check *(computed)*: a third-time payor at $417.34M pays 50 % × 20 + 62 % × 20 + 95 % × 20 + 110 % × 116.34 = $169.38M, which matches the Dodgers' bill. So the brackets in 6.2 are marginal.
- Conflict: other outlets give the league total as $402.6M or $401.3M (see 10).

### 6.5 Revenue sharing

| Rule | Value | Source |
|---|---|---|
| Size | Transfers equal to what a **48 % straight pool** of Net Local Revenue would move (Net Local Revenue = local revenue minus stadium operating expenses; local = total baseball revenue minus the club's share of central revenue; postseason revenue excluded) | [CBA] BA Art. XXIV(A)(3)-(9), (B)(1) |
| Formula | Each club puts in a blended Net Local Revenue (50 % of last year's, 25 % of two years ago, 25 % of three years ago). The pool is split equally; the net of what a club pays in and gets back sets its share of the transfer. Net payers are "Payor" clubs and net receivers "Payee" clubs | [CBA] BA Art. XXIV(A)(10) |
| Market disqualification | Clubs with a Market Score above 100 receive no revenue sharing. Their forfeited receipts are refunded to payor clubs | [CBA] BA Art. XXIV(A)(12) |
| Link to the CBT | A payor club that is also over the CBT gives up 0 % (1st straight year), 25 %, 50 %, 75 % or 100 % (5th straight year or more) of its refund | [CBA] BA Art. XXIV(A)(12)(b) |
| Discretionary fund | Commissioner's Discretionary Fund of at most $15M a year, at most $4M per club | [CBA] BA Art. XXIV(A)(11) |
| Central revenue | National media and similar revenue is not part of the plan; existing central-revenue arrangements are unchanged | [CBA] BA Art. XXIV(B)(2) |
| Press confirmation | "Teams currently share 48% of all local revenue" (ESPN, June 2026) | [press] https://www.espn.com/mlb/story/_/id/48959766/manfred-admits-concern-mlb-labor-talks-end-lockout |

### 6.6 International amateur bonus pools (context for the QO penalties)

- Hard pools for the 2023 signing period: **$5,685,500** for Competitive Balance Round A clubs, **$6,226,900** for Round B clubs and **$5,144,000** for all others.
- From 2024 the pools grow with industry revenue once it passes the 2019 level ($9.741B). A club may not exceed its pool.
- Signing periods run from Jan 15 to Dec 15.
- Source: [CBA] BA Attachment 46 §B.
- The actual 2024-2027 pool amounts are **NOT CONFIRMED**.

---

## 7. Contract forms: only the rules that constrain them

| Feature | Rule | Source |
|---|---|---|
| Guaranteed multi-year deals | Allowed. For the CBT a year is "guaranteed" if more than 50 % of its base salary survives a release for lack of skill | [CBA] BA Art. XXIII(A)(8) |
| Voidable years | **Banned**: no clause may let the club void a guaranteed year on an event. Options that extend a deal, including vesting options, are allowed | [CBA] BA Attachment 42 |
| Club option | Not a guaranteed year for the CBT. Any buyout is treated as a signing bonus, spread over the guaranteed years (only the earliest option's buyout); if the buyout is never paid, the charge is reversed | [CBA] BA Art. XXIII(E)(5)(a)(i), (b) |
| Player option | A guaranteed year for the CBT, unless the payment for declining it is more than 50 % of that year's base salary | [CBA] BA Art. XXIII(E)(5)(a)(ii) |
| Opt-out (player can void several years at once) | The years after the opt-out count as guaranteed, unless the opt-out payment is more than 50 % of their combined base salary | [CBA] BA Art. XXIII(E)(5)(a)(ii) |
| Low player-option years ("Valley Charge") | Compare the player-option year's value with 80 % of the smallest earlier guaranteed year. If that 80 % figure is below 75 % of the AAV, use 75 % of the AAV instead. Any shortfall is charged pro rata to the earlier guaranteed years | [CBA] BA Art. XXIII(E)(5)(d)(ii) |
| Mutual option | Treated as a club option | [CBA] BA Art. XXIII(E)(5)(a)(iv) |
| Dual option (either side may exercise) | Treated as a player option | [CBA] BA Art. XXIII(E)(5)(a)(iii) |
| Option and opt-out dates | Must all fall inside the 5-day Quiet Period after the World Series | [CBA] BA Art. XX(B)(6)(d) |
| Deferrals | **No limit** on the amount or share. Clubs must fully fund the present value (discounted at 5 % a year) by the **second July 1** after the season it is earned. For the CBT a retirement-triggered deferral is assumed to start at age 40 or at the end of the contract, whichever is later | [CBA] BA Art. XVI; XXIII(E)(6) |
| Signing bonus | Spread over the guaranteed years for the CBT; counts in the maximum-cut calculation | [CBA] BA Art. XXIII(E)(3); VI(B)(2) |
| Performance bonuses | **No bonus for playing, pitching or batting skill, and none for team standing.** Performance bonuses must be consistent with MLR 3(b); award bonuses (for a recognised award or honour) are a separate allowed category | [official] MLR 2026 Rule 3(b)(5); [CBA] BA Art. XXIII(A)(5)-(6) |
| "Most favoured nation" clauses | Banned | [CBA] BA Attachment 38 |
| No-trade clauses | See 5.6 | [CBA] BA Art. XIX(A)(1) |
| Loans to players | A loan below market interest adds imputed income to CBT salary; forgiven loans count as salary | [CBA] BA Art. XXIII(E)(7) |
| Contract approval | The Commissioner approves or rejects within 20 days (30 days between Feb 15 and Apr 15) | [CBA] BA Art. IV |

---

## 8. Exits

| Status | Rules | Source |
|---|---|---|
| **Voluntarily Retired List** | The player applies in writing through his club; the Commissioner may grant it. No reinstatement within 60 calendar days of the season counted from the application; no reinstatement application within 60 playing days of retiring; no MLB reinstatement from **Aug 1 to Oct 31**; if he applies after Feb 1, his club gets 30 days' notice. Does not count against roster limits. After **2 straight years** he drops off reserve lists but stays ineligible until reinstated, which restores his old status. The club need not tender him a contract while he is on the list. He may be released only with the Commissioner's approval and waivers. If he qualifies as a free agent he may become one, but he cannot sign or play for a new club until reinstated | [official] MLR 2026 Rule 2(c)(9), 9(b)(3)(B); [CBA] BA Attachments 10-11 |
| **Restricted List** | For a player who does not report or sign within 10 days of the season opening, who says he won't report until 30+ days after it, who breaks his contract, or in other unusual cases. **No salary**; cannot be released; does not count against roster limits. After 2 years he moves to a General Restricted List. A player restricted for not playing who becomes a free agent comes off the list. One restricted under the drug or domestic-violence policy can become a free agent but stays on the list until his suspension ends | [official] MLR 2026 Rule 2(c)(6); [CBA] BA Attachments 10-11 |
| Released players | A player released and unclaimed is a free agent | [official] MLR 2026 Rule 9(b)(7) |
| Retiring after release | **NOT CONFIRMED**: no rule found covering a released free agent who retires | - |
| Salary while on the Voluntarily Retired List | **NOT CONFIRMED** in the texts read (the Restricted List rule says no salary; the VRL rule is silent) | - |

---

## 9. 2026-27 collective bargaining: status on 2026-11-17

**Facts at research time (news checked through Nov 17)**

- **No agreement, and no item reported as agreed.**
  - NBC tracker (Nov 4): none agreed. On June 25 MLB accepted the union's proposals for earlier free agency for some players and for ending the QO, but only inside its salary-cap package. [press]
  - Front Office Sports (Nov 16): no agreed items reported. [press] https://frontofficesports.com/article/mlb-ramps-up-salary-cap-push-days-before-expected-lockout/
- **Latest proposal reported:** MLB's revised proposal of Nov 3 (MLBTR and ESPN date it Nov 3; the MLB.com article is dated Nov 4). I found no report of a bargaining session after that.
  - [official] https://www.mlb.com/news/mlb-increases-cap-and-floor-in-latest-proposal-sets-2028-target
  - [analysis] https://www.mlbtraderumors.com/2026/11/mlb-proposal-salary-cap-2028-five-year-free-agency-million-league-minimum.html
  - MLBTR's bargaining tag page shows no post after Nov 12 (https://www.mlbtraderumors.com/2026-27-collective-bargaining).
- **Owners' meeting:** quarterly meeting in New York, **Nov 17-19** (MLBTR offseason). FOS calls it "likely their last in-person gathering before a lockout". The outcome is **NOT CONFIRMED**; the meeting had just begun.
- **Lockout:**
  - Expected right after the deal expires (11:59 p.m. ET Dec 1) or on Dec 2. [analysis] MLBTR offseason; [press] FOS 2026-11-16
  - Neither MLB nor the clubs had **formally announced** one in anything I found.
  - MLB is preparing for the possibility that the union files a "disclaimer of interest" during a lockout (Wall Street Journal via MLBTR, 2026-11-12). [analysis] https://www.mlbtraderumors.com/2026/11/mlb-preparing-for-possibility-that-mlbpa-opts-for-disclaimer-of-interest-during-lockout.html
- **Public campaign:** FOS reported on Nov 16 that MLB had released a five-minute video for the cap and floor, presented by EVP Morgan Sword. The MLBPA's **interim executive director Bruce Meyer** said the cap "artificially suppresses player salaries". [press] FOS 2026-11-16
- **What a lockout freezes (MLBTR offseason):**
  - All MLB transactions stop, and teams may have no work-related contact with 40-man players. [analysis]
  - 40-man players could not play in affiliated games during a stoppage. [analysis]
- **Minor leaguers are not affected:** the clubs may not lock out minor-league players during the MiLB BA, which runs to Dec 1, 2027. [CBA] MiLB BA Art. XXII, XXV
- **Precedent:** the 2021-22 lockout lasted 99 days and ended with the 2022 agreement of March 10, 2022. [press] ESPN 2026-06-03 (URL in 6.5); [CBA] BA preamble

Proposals on the table that touch this file (none agreed; NBC tracker unless noted):

| Topic | MLB (latest: Nov 3) | MLBPA (latest known) |
|---|---|---|
| Cap / floor | **$258.2M cap, $180.2M floor from 2028**, using CBT-style payroll including benefits. Clubs may start at 90 % of the floor (about $162.2M) and reach it over 3 years. 50/50 split of "baseball revenue" plus central media. Earlier version (May 28): $245.3M / $171.2M from 2027 | No cap. Base CBT threshold **$300M** (May 27). Minimum revenue guarantee of $240M a year for each small-market club |
| 2027 CBT and revenue sharing | Current systems to continue in 2027 (MLB.com Nov 4). *Note:* the BA's CBT ends after 2026 (6.1), so 2027 needs a new agreement | Keep the CBT system |
| Minimum salary 2027 | $1,000,000 (2+ years), $900,000 (under 2 years) | $1.5M |
| Pre-arb pool | $65M in 2027, rising to $75M | $180M |
| Free agency | After **5 seasons at age 30**, with the current club holding a one-year option at the top-125 average | Free agency after 5 seasons for players 30 and older |
| Qualifying offer | Abolished, with its draft compensation | Abolished for traditional free agents |
| Arbitration | No change described | Wider eligibility (no figure given) |
| Contract limits (June 25) | New club: at most 5 years, first-year salary at most 15 % of the cap. Own "cornerstone" player: at most 6 years and 16 %. Raises at most 5 % a year. **New deferrals banned** | - |
| Options and roster | Options per season 5 → **4**; no fourth option year; weekly rosters; limits on shuttling players; Rule 5 eligibility one year earlier; a Rule 5 pick could be optioned | Options per season 5 → **3**; 28-man rosters for the first 15 days; Rule 5 eligibility earlier |
| Free-agent calendar (Aug 27) | Signing window to **Dec 21**, then a freeze until the first Monday in February | - |
| Sources | [press] NBC tracker; [official] MLB.com 2026-11-04 (URL above) | [press] NBC tracker |

*Confirms* mlb_notes e (Nov 3 cap and floor; options 5 → 4; earlier Rule 5; MLBPA options 5 → 3). One wording conflict is noted in section 10.

---

## 10. Conflicts between sources, and corrections

1. **Effective date of the 6/7-season minor-league rule.** The MiLB BA says "First-Year Contracts entered into after March 30, 2023". The fan guide cited in levels.md §4.2 says June 5, 2023. Use the BA date. *Correction to levels.md.*
2. **Outright refusal threshold.** The BA says "at least 3 years" (or Super Two status). The MLB glossary (https://www.mlb.com/glossary/transactions/outright-waivers) says "more than three years". The BA governs.
3. **How the QO amount is labelled.** One Dodger Blue item calls $21.05M "the qualifying offer for the 2024 season". MLBTR's list gives $21.05M for the 2024-25 offseason, i.e. the 2025 season. MLBTR's offseason labelling is used here.
4. **Tender deadline time.** The BA refers to terms submitted before 8:01 p.m. ET on the Tender Deadline. MLBTR gives 4:00 p.m. CT for 2026. Not resolved; the date (Nov 20) is not in doubt.
5. **MLB's option proposal.** MLBTR (Nov 3) says MLB proposed cutting "option years from five to four". MLB.com (Nov 4) and the NBC tracker say optional assignments **per season** 5 → 4, plus ending the fourth option year. The latter matches the current rules (3 option years, 5 options per season).
6. **2025 CBT total.** "Just under $403 million" (CBS, from AP figures), "$402.6 million" and "$401.3 million" in other reports (as summarised by search results; those pages were not opened).
7. **Pre-arb award phase.** The FanGraphs summary mentions a "$150,000" award for a 5th-place ROY finish. The BA chart has no ROY tier below 2nd, so that money presumably came from the WAR phase. **NOT CONFIRMED**.
8. **Lockout clock.** Some press says the lockout would start "at midnight ET on Dec. 2". The BA term ends at 11:59 p.m. ET on Dec 1. These describe the same moment.
9. **Arbitration share of free-agent value.** The four analyses in 3.4 differ a lot (15-75 %, 40-80 %, 28-62 %, 44-64 %). They measure different things (previous-season WAR value versus market value) over different eras.

---

## 11. Gaps (NOT CONFIRMED)

- 2026-27 QO decisions of Arozarena, Bednar and Rogers (deadline 4 p.m. ET Nov 17).
- Outcome of the Nov 17-19 owners' meeting; any formal lockout announcement.
- The 2027 minimum salary, CBT, pre-arb pool and arbitration rules (none set; depends on a new agreement).
- International bonus pool amounts for 2024-2027 (only the 2023 base amounts are in the BA).
- Salary rules for players on the Voluntarily Retired List; any rule on players who retire after release.
- Super Two count for 2022-23 (cutoff known, count not found).
- How the BA's ban on extra opt-out dates for Article XX(B) veterans (XX(B)(5)(b)) interacts with the MiLB BA's allowance of up to 3 release clauses (MiLB BA III(B)(2)(c)).
- KBO and NPB posting systems were not researched (out of scope).
- Several MLB glossary pages (service time, Super Two, DFA, options, non-tender) returned HTTP 406 to my fetch tool, so those items rest on the BA and MLR texts instead.
