# NPB contracts, free agency and player movement (research notes)

Research date: 2026-11-17 (the 2026-27 offseason is in progress). For the free-agent market and contract system of the game.

Companion files, not repeated here except to confirm or correct them: `docs/research/world/npb_notes.md` (roster limits, 育成 basics, calendar) and `docs/research/world/nationality.md` (foreign-player rules).

Source tags:

- **[official]**: agreement text (野球協約, FA規約, 統一契約書, 育成規約), npb.jp or jpbpa.net pages. MLB.com league releases on the posting agreement are also tagged [official] and marked "MLB".
- **[press]**: Japanese sports and news press (Daily Sports, Hochi, Sponichi, Kyodo, Nikkan Sports, Nikkan Gendai, J-CAST). Web baseball outlets (hb-nippon.com 高校野球ドットコム, baseballchannel.jp, SPAIA, Sportiva) are also tagged [press], with the outlet named. English press is named too.
- **[wiki]**: ja.wikipedia, used only as a pointer. In the end I used none: the Wikipedia API rate-limited me.
- **[data]**: my own count or computation from the listed official pages. The method is given each time.
- **NOT CONFIRMED**: no source found that I could read. Do not use as fact.

Main texts used:

| Short name | What it is | URL |
|---|---|---|
| 協約2025 | 日本プロフェッショナル野球協約 2025 edition (newest posted; no 2026 edition exists on jpbpa.net) | https://jpbpa.net/wp-content/uploads/jpbpa-pdf/ag2025.pdf |
| FA規約2009 | フリーエージェント規約, 2009 edition. The only edition published anywhere I could find. 協約2025 Art. 196 says the FA system is defined in this separate rule book. | https://jpbpa.net/wp-content/uploads/2021/12/fa2009.pdf |
| 統一契約書2018 | uniform player contract, 2018 edition (newest posted) | https://jpbpa.net/wp-content/uploads/2021/12/uc2018.pdf |
| 育成規約2014 | 日本プロ野球育成選手に関する規約, 2014 edition (newest posted) | https://jpbpa.net/wp-content/uploads/2021/12/tp2014.pdf |
| 日米協定2014 | 日米間選手契約に関する協定 (pre-2017 posting agreement; the 2017 text is not public) | https://jpbpa.net/wp-content/uploads/2021/12/ju14.pdf |
| NPB FA page 2019 | NPB's own "フリーエージェントについて" (last published edition; later years' links are commented out) | https://npb.jp/announcement/2019/fa_about.html |
| NPB FA page 2018 | same, 2018 edition, with the special day-count measures | https://npb.jp/announcement/2018/fa_about.html |
| Listing of all the above | JPBPA 野球協約・統一契約書 page | https://jpbpa.net/contract/ |

I diffed the 2024 and 2025 協約 texts sentence by sentence. The only changes are two ballpark names (SoftBank's dome) [data], so the 2025 text is effectively the 2024 text.

---

## 0. The numbers that matter most

| Item | Value | Source |
|---|---|---|
| One FA "season" | 145 days on the active roster (出場選手登録) in one season. Shorter seasons are added together until they reach 145 days. | [official] NPB FA page 2019; FA規約2009 Art. 2 |
| Domestic FA | 8 seasons. 7 for players drafted in 2007 or later who were in university or company (日本野球連盟) baseball when drafted. | same |
| International FA | 9 seasons | same |
| Re-acquisition | 4 more seasons after exercising FA (domestic or international); the re-acquired right is international FA | same; FA規約2009 Art. 5 |
| FA-qualified / declared, 2026 | 105 qualified (36 domestic-only, 69 international). 10 declared (8 domestic, 2 international). | [official] https://npb.jp/news/detail/20261019_01.html , https://npb.jp/news/detail/20261110_02.html |
| Declarations 2020-2025 | 42 in total. 18 stayed with their club, 20 moved within NPB, 4 went to MLB, 0 unsigned. | [data] from NPB `fa_filed` pages, section 6 |
| Compensation ranks until the 2026 offseason | A = top 3 Japanese salaries in the old club; B = 4th-10th; C = 11th and below. A/B: player (from outside a 28-man protected list) plus 50%/40% of salary, or 80%/60% in cash only. Second FA: 25%/20% or 40%/30%. | [official] FA規約2009 Art. 10 |
| New in 2026 offseason | No compensation at all for FA movers aged 35 or older on 1 April of the next year | [press] Nikkan Sports, Hochi, 2026-10-30 |
| From the 2027 offseason | Player compensation (人的補償) abolished. Old club chooses a draft pick (A: extra 2nd-round pick, B: extra 3rd-round pick) or cash (A: 80-130% of salary, B: 60-100%). B narrowed to ranks 4-8 (8th: cash 60% only); C from rank 9. Review in the 2029 offseason. | [press] Nikkan Sports, Sponichi, Daily, Hochi, 2026-10-30; [official] JPBPA statement https://jpbpa.net/2026/10/30/13504/ |
| Reduction limit (減額制限) | Salary over ¥100M: max 40% cut. ¥100M or less: max 25% cut. Larger cuts need the player's consent. | [official] 協約2025 Art. 92 |
| Minimum salary | 支配下 ¥4.2M (Art. 89). Top-up while on the active roster up to a ¥16M annual rate (Art. 89-2). 育成 ¥2.3M (2014 text). All exclude consumption tax. | [official] 協約2025, 育成規約2014 |
| Average salary (JPBPA survey of union members on 支配下 contracts; foreign players are not members) | 2025: ¥49.05M (725 players). 2026: ¥52.16M (713 players). Median ¥19M → ¥20M. | [official] JPBPA PDFs, section 6E |
| Released (not reserved on 2 Dec) | 126-151 支配下 players a year, 2020-2025. Plus 育成 releases rising from 56 (2020) to 139 (2026). | [data] NPB 自由契約 lists, section 4 |
| Release-notice deadline | 30 October since 2025, because of the Freelance Act; in 2024 it was the day after the Japan Series | [press] hb-nippon 2025-10-29; legal column, 新日本法規, 2025-12-26; Daily 2024-09-30 |
| 現役ドラフト (active-player draft) | Since 2022, held each December. 12 / 12 / 13 / 12 players moved, 2022-2025. 2026 edition: 9 December. | [official] https://c.npb.jp/geneki_draft/2025/ ; [press] Daily 2026-10-14 |
| Posting fee | 20% of the first $25M guaranteed + 17.5% of the next $25M + 15% above $50M. Minor-league deal: 25% of the signing bonus. Negotiating window 45 days. | [official, MLB] https://www.mlb.com/glossary/transactions/japanese-posting-system |

---

## 1. FA eligibility, calendar and negotiation

### 1A. How a season is counted

| Rule | Detail | Source |
|---|---|---|
| Unit | A season with 145 or more days on the active roster counts as 1 season. More than 145 days in a year still counts as only 1. | [official] NPB FA page 2019; JPBPA page (as of Sept 2010) https://jpbpa.net/transfer/ |
| Short seasons | All seasons under 145 days are summed; every 145 days of the total counts as 1 season | [official] FA規約2009 Art. 2(3); NPB FA page 2019 |
| Start date for counting | The scheduled opening day of the league season, not the date of the registration notice | [official] FA規約2009 Art. 2 note |
| Injured-player measure (故障者特例措置) | For a player who had 145 days the year before and is deregistered for an on-field injury suffered between 1 Feb and 30 Nov: the days from deregistration until his first farm official game are added, **up to 60 days**. The measure is not available in the season after one in which it lifted the player to 145 days. | [official] NPB FA page 2018 https://npb.jp/announcement/2018/fa_about.html ; JPBPA page (2010) says it began in 2007 |
| Pitcher measures (投手の登録日数加算) | A rotation starter (as designated by his club) registered within 7 days of opening day is treated as registered from opening day. A starter deregistered 7 or fewer days before the first All-Star Game and re-registered within 7 days after the last one gets the gap counted. | [official] NPB FA page 2018 |
| Starter relief proposed in 2023 | The players' association proposed adding 7 days when a starter is deregistered after a start and re-registered for another start within 2 weeks. Starters said they find it harder than position players to reach 145 days. **Whether this was adopted: NOT CONFIRMED.** | [press] Daily 2023-08-03 https://www.daily.co.jp/baseball/2023/08/03/0016657813.shtml , 2023-09-11 https://www.daily.co.jp/baseball/2023/09/11/0016798774.shtml |
| 2020 (120-game COVID season) | NPB and the players disputed how to count the shortened season (at most 141 registration days were possible). **The final rule is NOT CONFIRMED.** | [press] Nikkan Gendai https://www.nikkan-gendai.com/articles/view/sports/274043 |

### 1B. Seasons needed

| Right | Seasons | Conditions | Source |
|---|---|---|---|
| Domestic FA (国内FA): may sign with any NPB club, but no foreign club | 8 | All players | [official] NPB FA page 2019; FA規約2009 Art. 2(1) |
| Domestic FA, shorter rule | **7** | Drafted in **2007 or later**, and at the time of the draft a member of a university baseball federation (大学野球連盟) or of the Japan Amateur Baseball Association (日本野球連盟: company and club teams). High-school draftees, and by the wording also independent-league draftees, need 8. | [official] FA規約2009 Art. 2(1); NPB FA page 2019 ("2007年以降のドラフトで入団した大学生・社会人選手"); JPBPA table (2010) |
| International FA (海外FA): any club in Japan or abroad | 9 | Not if the player already exercised domestic FA before reaching 9 | [official] FA規約2009 Art. 2(2); NPB FA page 2019 |
| Re-acquisition | 4 more seasons after the FA declaration | Applies after any exercise of FA, including declaring and staying. What is re-acquired is international FA. | [official] FA規約2009 Art. 5; NPB FA page 2019 |

Still in force in 2026: Hochi (2026-10-30) describes current domestic FA as "大学生・社会人は入団から最短7年、高校生は同8年" [press] https://news.yahoo.co.jp/articles/3da05bf46d255ff7d6db79ded607369b9e082c83

The players' association has long asked for shorter FA periods, in parallel with the abolition of player compensation. [press] Hochi 2026-10-30 (above). **No shortening has been agreed as of 2026-11-17.** The union's exact demand (a uniform 6 years, per a search excerpt of a Tokyo Sports article that now opens to another story) is NOT CONFIRMED.

### 1C. Calendar

Rules:

| Step | Rule | Source |
|---|---|---|
| List of FA-qualified players published | FA規約2009 says 2 days after the later league's last regular-season game. In practice, publication now comes after the Climax Series (see dates below). **The current rule text is NOT CONFIRMED.** A player still short of days with games left is listed with his shortfall and added later. | [official] FA規約2009 Art. 3 |
| Declaration window | Within **7 days, excluding Saturdays, Sundays and holidays**, from the day after the Japan Series ends. The player tells his club. The declaration is filed jointly with the club representative to the commissioner. | [official] FA規約2009 Art. 6(1)-(2); NPB FA page 2019 |
| Not declaring | The right is kept (保留) for later years | [official] FA規約2009 Art. 4, Art. 6 note 1 |
| Declared players published | 15:00 on the day after the 7-day window | [official] FA規約2009 Art. 4 |
| Negotiation | From the day after publication, with **any club including the old club** | [official] FA規約2009 Art. 6(3); NPB FA page 2019 |
| Duty to the old club | A declared player stays bound by his contract and must attend club and NPB events **until 30 November** | [official] FA規約2009 Art. 12 |
| Signing published | Each signing is notified to the commissioner and published | [official] FA規約2009 Art. 6(4) |

Observed dates [official], pages `https://npb.jp/news/detail/<date>_0x.html`:

Japan Series end dates for 2020-2024 are from https://npb.jp/nippons/<year>/ [official].

| Offseason | FA-qualified list published | Japan Series ended | FA declarations published |
|---|---|---|---|
| 2020 | 17 Nov (n_20201117_03) | Wed 25 Nov | NOT FETCHED (first signings 8 Dec) |
| 2021 | 14 Nov (20211114_01) | Sat 27 Nov | 8 Dec (20211208_01) |
| 2022 | 17 Oct (20221017_02) | Sun 30 Oct | 10 Nov (20221110_01) |
| 2023 | 23 Oct (20231023_01) | Sun 5 Nov | 15 Nov (20231115_01) |
| 2024 | 23 Oct (20241023_03) | Sun 3 Nov | 14 Nov (20241114_01) |
| 2025 | 22 Oct (20251022_03) | Thu 30 Oct ([press] 新日本法規 column, below) | 12 Nov (20251112_01) |
| 2026 | 19 Oct (20261019_01) | 28 Oct (npb_notes A6) | 10 Nov (20261110_02) |

[data] Every observed publication date falls on the day after the 7th weekday after the Series ended, with holidays skipped: 3 Nov in 2022 and 2026; 4 Nov 2024, a substitute holiday; 3 Nov 2025. So the 2009 rule is still applied.

2026 calendar worked through the rule [data]:

| Step | Date |
|---|---|
| Declaration window | 29, 30 Oct, 2, 4, 5, 6, 9 Nov (3 Nov is a holiday). Closed **9 Nov at 17:00** ([press] hb-nippon 2026-11-09 https://www.hb-nippon.com/articles/19042) |
| Published | 10 Nov ([official]) |
| Negotiation opens | 11 Nov |
| Signings | As of 17 Nov none had been published: `fa_signed.html` for 2026 returns "page not found" [official] |

Signing dates 2020-2025 run from the publication day itself (宣言残留, e.g. 10 Nov 2022) to 26 Jan [official, section 6].

### 1D. Negotiation rules for declared players

| Rule | Detail | Source |
|---|---|---|
| Domestic-FA players and foreign clubs | May not negotiate or sign with any club outside NPB, independent leagues included. Until he signs with another NPB club, the player stays on his old club's reserve list. | [official] FA規約2009 Art. 6(5)-(6) |
| Reduction limit | Does not apply: an FA signing may cut salary by any amount | [official] FA規約2009 Art. 8 |
| Arbitration | Neither club nor player may ask for salary arbitration | [official] FA規約2009 Art. 9 |
| Salary cap on the new contract | **2009 text:** the signing club may not pay more than the player's previous NPB salary unless the commissioner approves a written request. **Whether this clause still exists is NOT CONFIRMED.** Recent deals such as Kai 2024 (5 years, about ¥3bn) look far above previous salaries, so it is either routinely approved or gone. | [official, 2009] FA規約2009 Art. 7 |
| Limit on signings | A club may sign at most 2 declared players from other NPB clubs. 3 if 21-30 players declared, 4 if 31-40, 5 if 41 or more. **C-rank players do not count.** Signing back your own player does not count. Last text: 2009. Already described as a "2人まで, Cランク除く" rule in a 2020 press article. | [official, 2009] FA規約2009 Art. 11; [press] Nikkan Gendai https://www.nikkan-gendai.com/articles/view/sports/281612/2 |
| Posting a domestic-FA signing | The signing club may not post the player until he would have reached international FA | [official, 2009] FA規約2009 Art. 13 |

### 1E. Staying after declaring (宣言残留)

- **Rule.** A declared player may negotiate with and re-sign with his old club. That still counts as exercising FA, and it restarts the 4-season clock in Art. 5. [official] FA規約2009 Art. 4 note, Art. 6(3)
- **Data.** In 2020-2025, 18 of 42 declared players (43%) re-signed with their own club. [data] from NPB `fa_filed` pages, section 6
- **Club policy is not a rule.** Example: Hiroshima, which "at times did not allow" it, said in the 2026 offseason it would accept 森下, 床田 and 坂倉 staying after declaring, and offered raises and multi-year deals. The three declared on 4, 5 and 6 Nov, the first time three Hiroshima players declared in one year. [press] Baseball Channel https://www.baseballchannel.jp/npb/301065/ , https://www.baseballchannel.jp/npb/300410/
- **No list of club-by-club policies found: NOT CONFIRMED.**

### 1F. Foreign players and FA (confirms `nationality.md`)

- A player who earns NPB FA rights stops counting as foreign for the roster limits from the next season. Only for that chapter of the agreement (Art. 82(4): "この章の規定の適用に関する場合に限り"). [official] 協約2025 Art. 82(4)
- For FA ranking, such players **still count as foreign** and are left out of the ranking. [official] FA規約2009 Art. 10(2)

---

## 2. FA rank and compensation

### 2A. Rules in force through the 2026 offseason

Rank is set by the player's salary in the year of declaration, ranked among the old club's players **excluding foreign players**. Ties: the player with more total registration days ranks lower, then the older player ranks lower. [official] FA規約2009 Art. 10(2)

| Rank | Salary rank in old club | First FA: cash plus player | First FA: cash only | Second or later FA: cash plus player | Second or later FA: cash only |
|---|---|---|---|---|---|
| A | 1-3 | 1 player + **50%** of previous salary | **80%** | 1 player + **25%** | **40%** |
| B | 4-10 | 1 player + **40%** | **60%** | 1 player + **20%** | **30%** |
| C | 11 and below | none | none | none | none |

Source: [official] FA規約2009 Art. 10(3)-(4). Same numbers on the JPBPA page (2010) https://jpbpa.net/transfer/ ; that table mislabels two of its rows "人的補償なし".

The B-rank figures were still the ones in use in 2025: "旧年俸の60%の金銭補償、または人的補償+旧年俸の40%" for 桑原. [press] Baseball Channel https://www.baseballchannel.jp/npb/242724/

| Detail | Rule | Source |
|---|---|---|
| Who chooses | The old club, between "player + cash" and "cash only" | [official] FA規約2009 Art. 10(3) |
| Protected list | The signing club protects **28** players. Its foreign players are also excluded. The old club picks **1 player per FA player signed** from the rest of the signing club's 支配下 list. | [official] FA規約2009 Art. 10(3)(1)ア |
| Deadline for the list | Within 2 weeks of the signing being published | same |
| Clashes | If two clubs want the same compensation player, the club in the same league as the signing club goes first, then reverse order of standings | same |
| Completion | All compensation within 40 days of the signing being published. Cash can be extended with the old club's consent. | [official] FA規約2009 Art. 10(5) |
| Player refuses to move | He becomes 資格停止 (suspended). The old club then gets the cash-only amount. | [official] FA規約2009 Art. 10(7) |
| Late signing | No compensation if the player signs with an NPB club only on or after 1 December of the second year after his declaration | [official] FA規約2009 Art. 10(6) |
| Moving abroad | No compensation and no limit on numbers for international-FA moves outside NPB | [official] JPBPA page (2010) https://jpbpa.net/transfer/ ; FA規約2009 Art. 10(1) covers only NPB-to-NPB moves |
| Relocation cost | A player moved as compensation gets the trade relocation allowance of ¥2M + tax | [official] 協約2025, memorandum after Art. 124, item 2 |
| Whether that year's new draftees are protected | NOT CONFIRMED | |

### 2B. New for the 2026 offseason only, before the 2027 reform

FA movers who are **35 or older on 1 April of the following year** need **no compensation**, cash or player, whatever their rank. NPB's legal office called it the first age-based cut-off. Nikkan Sports gives DeNA's 筒香 as an example of a player covered this year. [press] Nikkan Sports 2026-10-30 https://www.nikkansports.com/baseball/news/202610300001692.html ; Hochi via Yahoo https://news.yahoo.co.jp/articles/3da05bf46d255ff7d6db79ded607369b9e082c83

Everything else in 2A still applies in the 2026 offseason. The players' association had rejected NPB's draft-pick proposal on 12 Aug 2026 ("今オフは現行制度が維持される"). [press] Kyodo via Kumamoto Nichinichi https://kumanichi.com/articles/2024589

### 2C. From the 2027 offseason (agreed 30 Oct 2026, approved by the executive committee)

| Rank | Salary rank in old club (Japanese players) | Old club chooses |
|---|---|---|
| A | 1-3 (unchanged) | the next year's **2nd-round draft pick**, or cash of **80-130%** of salary |
| B | 4-7 | the next year's **3rd-round draft pick**, or cash of **60-100%** |
| B | 8 | cash of **60%** only, no pick |
| C | 9 and below (before: 11 and below) | nothing |

How the pick works:

- The old club makes **an extra pick** in that round, in the signing club's waiver-order slot. With an A-rank player, the old club picks twice in round 2: once in its own slot, once in the signing club's.
- The old club chooses pick or cash first, and may not switch afterwards.
- NPB checks that the cash stays in the 目安 (guideline) range.
- What happens when one club signs two players of the same rank is still under discussion.
- The pick system and the rank bands are to be reviewed around the **2029 offseason**, after two years of use.

Sources [press] (all 2026-10-30 or 31):

- Nikkan Sports https://www.nikkansports.com/baseball/news/202610300001692.html
- Sponichi https://www.sponichi.co.jp/baseball/news/2026/10/30/kiji/20261030s00001173204000c.html
- Daily https://www.daily.co.jp/baseball/2026/10/30/0020984189.shtml
- Hochi via Yahoo (above)
- Summary with the waiver-slot detail: https://www.draft-kaigi.jp/draft-blog/draft-system/90065/

Players' association statement [official]: https://jpbpa.net/2026/10/30/13504/ . It confirms that player compensation is abolished, that a draft-pick system is introduced, and that it will be reviewed in the 2029 offseason. It gives no figures.

**Gap:** NPB has published no page with the new rule text: NOT CONFIRMED in official form.

### 2D. Player compensation in practice

From NPB's trade lists, which mark each compensation transfer "※…選手のFA移籍に伴う契約譲渡". [official] `https://npb.jp/announcement/<year>/pn_traded.html` , 2020-2026.

| Offseason | FA mover (old club → new club) | Compensation player (signing club → old club) | Date published |
|---|---|---|---|
| 2020 | 梶谷隆幸 (DeNA → Giants) | 田中俊太 | 2020-12-22 |
| 2021 | 又吉克樹 (Chunichi → SoftBank) | 岩嵜翔 | 2022-01-06 |
| 2022 | 森友哉 (Seibu → Orix) | 張奕 | 2022-12-20 |
| 2022 | 近藤健介 (Nippon-Ham → SoftBank) | 田中正義 | 2023-01-19 |
| 2023 | 西川龍馬 (Hiroshima → Orix) | 日髙暖己 | 2024-01-09 |
| 2023 | 山川穂高 (Seibu → SoftBank) | 甲斐野央 | 2024-01-19 |
| 2024 | 茂木栄五郎 (Rakuten → Yakult) | 小森航大郎 | 2025-01-17 |
| 2024 | 甲斐拓也 (SoftBank → Giants) | 伊藤優輔 | 2025-01-22 |
| 2025 | 桑原将志 (DeNA → Seibu) | 古市尊 | 2026-01-19 |
| 2025 | 則本昂大 (Rakuten → Giants) | 田中千晴 | 2026-01-30 |

[data] **10 player-compensation cases among the 20 moves within NPB, 2020-2025.** The other 10 were C rank or cash-only.

Ranks reported in the press:

- 2024 [press] hb-nippon https://www.hb-nippon.com/articles/2390
  - A: 菅野 (to MLB), 大山 (stayed)
  - B: 九里 (Hiroshima took cash only: no transfer in NPB's list), 木下, 甲斐, 茂木
  - C: 石川柊太, 原口, 福谷
- 2025: 桑原 B, with a 2025 salary of ¥120M (estimate). [press] J-CAST https://www.j-cast.com/2025/11/28509700.html
- The rest: NOT CONFIRMED.

Long run: 40 players have moved as compensation since FA began in 1993; 27 of them were 26 or older at the next opening day. [press] THE DIGEST via Yahoo https://news.yahoo.co.jp/articles/66a1b0ffcaaea4b70f7fbfcb3327dcaaec78acdf

---

## 3. Salary rules

### 3A. Basics

| Rule | Detail | Source |
|---|---|---|
| Salary period | Salary (参稼報酬) covers **1 Feb - 30 Nov** (10 months). Payment terms are agreed and written in the contract. | [official] 協約2025 Art. 87(1)-(2); 統一契約書2018 Art. 3 |
| Tax | Amounts in contracts **exclude** consumption tax (since the 2014 edition) | [official] 協約2025 Art. 87(3) |
| Banned forms | No pay as a share of gate receipts and no piece-rate pay | [official] 協約2025 Art. 88 |
| Salary on transfer | Unchanged when a contract is transferred | [official] 協約2025 Art. 93 |
| Filing next year's contract | From **1 Dec** for current 支配下 players; from the day after the season for others | [official] 協約2025 Art. 52(2) |
| Bonus contracts (契約金) | Only for 自由契約 (released) players, players who never signed with an NPB club, and players named elsewhere in the rules. A released player rejoining his former club must wait 2 years for a bonus contract. Bonuses are lump sums. | [official] 協約2025 Art. 101-103 |

### 3B. Minimums

| Item | Amount | Source and note |
|---|---|---|
| 支配下 minimum | **¥4.2M a year** | [official] 協約2025 Art. 89. The same in the 2014-2025 editions. The **2013 edition said ¥4.4M** and had no tax clause, so the change in 2014 was a switch to tax-exclusive amounts, not a cut ([official] https://jpbpa.net/wp-content/uploads/2021/12/ag2013.pdf). Secondary sites that quote ¥4.4M are using the old figure. |
| Active-roster top-up (出場選手追加参稼報酬) | A player paid under **¥16M** gets, for each day on the active roster, 1/150 of the gap between ¥16M and his salary. Salary plus top-up is capped at ¥16M. | [official] 協約2025 Art. 89-2. The threshold was ¥14.3M in the 2014-2017 editions. |
| 育成 minimum | **¥2.3M a year**; standard signing payment (支度金) for a 育成 draftee ¥2.9M + tax; 育成 contract year 1 Jan - 31 Dec | [official] 育成規約2014 Art. 7(2). The 2011 and 2013 editions said **¥2.4M** (支度金 ¥3.0M), again the pre-2014 tax-inclusive basis. |

Conflicts:

- A secondary explainer (https://bb-news.jp/npb-salary-system-complete-guide/, not press) says the first-team minimum is ¥17M. On the same page it also gives "支配下 minimum ¥17M, 育成 ¥4.4M", which is plainly garbled. The 2025 agreement says ¥16M. **The agreement text wins until proven otherwise.**
- No newer 育成規約 than 2014 is published: a change since then is **NOT CONFIRMED either way**.
- No source showed a change for 2026.

### 3C. Reduction limit (減額制限) and refusal

| Item | Rule | Source |
|---|---|---|
| Limit | Next year's salary may not be cut by more than **40%** if this year's salary is **over ¥100M**, or by more than **25%** if it is **¥100M or less**. Unless the player consents. | [official] 協約2025 Art. 92; 統一契約書2018 Art. 31(1) |
| Base | The salary written in the contract. The active-roster top-up and game shares are excluded. | [official] 協約2025 Art. 92 |
| Effect at the boundary | [data] A ¥100M player can be cut to ¥75M at most; a ¥101M player to ¥60.6M | |
| Releasing instead | A club that wants a bigger cut and gets no consent can leave the player off the reserve list. He becomes 自由契約 on 2 Dec. | [official] 協約2025 Art. 66, 69 |
| Player still unsigned from 10 Jan | The club pays a **reserve allowance (保留手当)** of 25% of 1/365 of last year's salary per day, monthly, until signing or an arbitration application. It is deducted from the salary once he signs. If the club does not pay within 15 days of a request, it loses its rights. | [official] 協約2025 Art. 71-72; 統一契約書2018 Art. 33 |
| Player's lever from 10 Jan | If the player offers to renew at the maximum-cut salary and the club refuses, the club loses its reserve right and the player may ask to be made a free agent. Wording: "以上減額" in Art. 70, "超えて減額" in the uniform contract. | [official] 協約2025 Art. 70; 統一契約書2018 Art. 31(2) |
| Holding out a full year | If the player is still reserved and unsigned on 9 Jan two years later, he becomes **資格停止** (suspended list) | [official] 協約2025 Art. 74(1); 統一契約書2018 Art. 31(1) |
| Exceptions | FA-declared players (no limit, FA規約2009 Art. 8). Consent. | [official] |

### 3D. Salary arbitration (参稼報酬調停)

| Item | Rule or fact | Source |
|---|---|---|
| Who may apply | A reserved player or his club, when they disagree only on the salary amount | [official] 協約2025 Art. 94; 統一契約書2018 Art. 32 |
| Procedure | The commissioner forms a committee. It hears both sides' figures and reasons. Both sign a contract with the salary left blank; the committee fills it in within **30 days** of accepting the case. It is not final-offer arbitration: the committee sets its own number. | [official] 協約2025 Art. 95-96 |
| Excluded | FA-declared players | [official] FA規約2009 Art. 9 |
| How often | **Rare.** NPB's notices page lists one decision only: 西武 v 涌井秀章, 2011. Club ¥220M, player ¥250M plus incentives; decided **¥253M** ("前年度の概ね15％アップ"). The committee said salary should rest mainly on last season's results, plus earlier record, titles, drawing power, expectations, balance with other players and the club's finances. | [official] https://npb.jp/npb/20110128doc.html |
| Earlier cases | 下柳剛 (2000 offseason), Alfonso Soriano (1997 offseason) and 落合博満 (described as the first Japanese applicant; year not checked) | [press] Baseball Channel https://www.baseballchannel.jp/npb/144224/ , https://www.baseballchannel.jp/npb/144248 |
| Total number of cases, any case after 2011 | **NOT CONFIRMED** | |

### 3E. Multi-year contracts

- The uniform contract is written for one year (1 Feb - 30 Nov, Art. 3) and renewed under the reserve clause (Art. 31). [official] 統一契約書2018
- Neither the agreement nor the uniform contract has an article on multi-year deals. **How they are documented (special clauses, 特約, Art. 47) and whether the reduction limit applies inside them: NOT CONFIRMED.**
- They are common in practice. Examples:
  - SoftBank, 2022: 千賀 on a "変動制" (variable-salary) 5-year deal with an opt-out; 武田翔太 on a variable 4-year deal of over ¥600M. [press] Sportiva https://sportiva.shueisha.co.jp/clm/baseball/npb/2022/02/10/fagm/
  - Hanshin, Nov 2025: 近本光司 stayed without declaring FA on a 5-year, ¥2.5bn deal (estimate). [press] Daily https://www.daily.co.jp/tigers/2025/11/11/0019694767.shtml
- Players on multi-year deals are not eligible for the 現役ドラフト (section 4F).
- Clubs often offer multi-year deals to regulars close to FA; some players turn them down for one-year deals. [press] SPAIA https://spaia.jp/column/baseball/npb/30137

---

## 4. Player movement

### 4A. Reserve list (保留選手名簿) and the 70 limit

Confirms npb_notes D1; additions only:

- Clubs submit the full reserve list by **30 Nov**. It covers reserved contract players plus voluntarily retired, restricted, suspended and ineligible players. At most **70** reserved contract players (80 under the emergency relief clause). [official] 協約2025 Art. 66
- The commissioner publishes it on **2 Dec**. 支配下 players left off are free agents (自由契約) from that day. [official] 協約2025 Art. 67, 69
  - NPB's lists show the unreserved players dated 2 Dec in 2020, 2021, 2022, 2024 and 2025, and **1 Dec in 2023**. [official] `pn_released` pages
- Reserved players may not negotiate, train or play with any other club, Japanese or foreign. [official] 協約2025 Art. 68(2)
- The reserve right on a **voluntarily retired player (任意引退) lasts 3 years**; after that he counts as a free agent. [official] 協約2025 Art. 67(3)

### 4B. Release (自由契約) and release-notice periods (戦力外通告)

Mechanics:

- A free agent may sign with any club once the commissioner publishes him as 自由契約. [official] 協約2025 Art. 58
- Ways to get there:
  - not reserved on 2 Dec (Art. 69)
  - waiver unclaimed (Art. 120)
  - club termination (球団による契約解除)
  - reserve right given up after a posting (契約保留権放棄)
  - [official] NPB `pn_released` pages
- A 支配下 player can be re-signed as 育成 only after release, and not in the same season if released mid-season. [official] 育成規約2014 Art. 9(2) (as npb_notes D7)

Release-notice periods:

| Season | First period | Second period | Source |
|---|---|---|---|
| 2024 | 30 Sep - 11 Oct. The two Climax Series First Stage losers: until the day after their last game. | From the day after all CS games end to **the day after the Japan Series ends** | [press] Daily 2024-09-30 https://www.daily.co.jp/baseball/2024/09/30/0018178798.shtml |
| 2025 | | Ended **30 Oct**. "Before, the limit was 2 days after the Japan Series" (Daily 2024 says the day after). SoftBank and Hanshin released players **during** the 2025 Japan Series. | [press] hb-nippon 2025-11-01 https://www.hb-nippon.com/articles/9812 ; 2025-10-29 https://www.hb-nippon.com/articles/9782 |
| 2026 | Ended **9 Oct**. CS First Stage losers may notify on the day after and the second day after elimination. | To 30 Oct (same law) | [press] hb-nippon 2026-10-10 https://www.hb-nippon.com/articles/18184 |

- **Why 30 Oct.** The Freelance Act (特定受託事業者に係る取引の適正化等に関する法律, in force **1 Nov 2024**) requires 30 days' notice before ending a continuing contract. The 支配下 contract runs to 30 Nov, so notice is due by **30 Oct**. Two parts of this are the legal column's own interpretation: the contract term (taken from 統一契約書 Art. 3, since the form names no term) and the date. [press, legal column] 新日本法規, 2025-12-26 https://www.sn-hoki.co.jp/articles/article4500841/
- The start date of the 2026 first period: **NOT CONFIRMED**. Only non-press sites give dates, and they disagree.
- I found no NPB page announcing either period.

### 4C. How many are released (counted from NPB's official lists)

`https://npb.jp/announcement/<year>/pn_released.html` (支配下) and `.../pndev_released.html` (育成) [official]. Counts are mine [data].

| Season ("年度": runs to January) | Not reserved on 2 Dec | Of which foreign-style names (approx.) | Waived and unclaimed (season) | Reserve right given up (契約保留権放棄); of which postings | Club termination | 育成 released (date of the main list) |
|---|---|---|---|---|---|---|
| 2020 | 132 | 31 | 3 | 2; 1 posting (the other: チェン・グァンユウ) | 1 | 56 (30 Nov) |
| 2021 | 144 | 33 | 12 | 2; 1 posting (the other: 古谷優人) | 1 | 54 (1 Nov) |
| 2022 | 151 | 43 | 7 | 2; 2 postings | 0 (plus 1 voluntary-retirement listing cancelled, which makes the player a free agent: Art. 59(2)) | 71 (31 Oct) |
| 2023 | 148 | 33 | 14 | 3; 3 postings | 1 | 108 (31 Oct) |
| 2024 | 126 | 23 | 18 | 3; 3 postings | 0 | 115 (31 Oct) |
| 2025 | 150 | 28 | 13 | 3; 3 postings | 0 | 127 (31 Oct) |
| 2026 (to 17 Nov) | due 2 Dec | | 19 (18 with foreign-style names; the 19th is 宋家豪) | | 1 | **139 (30 Oct)** |

- "Foreign-style" = names written "X．Name" or all in katakana. Japanese players with katakana registered names would be miscounted, and Taiwanese players with kanji names are missed.
- Many of the 2 Dec players re-sign as 育成 with the same club; NPB's lists do not say which.
- Waived-and-unclaimed players are nearly all foreign.
- The rise in 育成 releases coincides with large 育成 squads at some clubs (npb_notes C5: 35-48 育成 players at the Giants, Seibu and SoftBank in 2026). The causal link is not sourced.

Press counts of 戦力外 (released or offered a 育成 deal, Japanese and foreign, retirements usually excluded):

- 2025: **157** by 30 Oct. [press] hb-nippon https://www.hb-nippon.com/articles/9812
- 2026: 84 notified and 77 戦力外 by 9 Oct; 92 notified and 85 戦力外 by 23 Oct. [press] hb-nippon https://www.hb-nippon.com/articles/18184 , https://www.hb-nippon.com/articles/18601
- **2026 final count: NOT FOUND.**

### 4D. Where released and retiring players go (NPB survey)

NPB, 戦力外／現役引退選手の進路調査結果 [official]. Pages:

- 2020 season: https://npb.jp/npb/careersupport2021_1.html
- 2021: https://npb.jp/npb/careersupport2022_1.html
- 2022: https://npb.jp/npb/careersupport2023_1.html
- 2023: https://npb.jp/npb/careersupport2024_1.html
- 2024: https://npb.jp/npb/careersupport2025_1.html
- 2025: https://npb.jp/npb/careersupport2026_1.html (published 2026-04-27)

Population: released and retired players, **excluding foreign players and players re-signed as 育成 by the same club**. Retirements are **included**, so this is not a pure "released players" table.

| Season | n | Mean age | Mean years in NPB | NPB 支配下 player | NPB 育成 player | NPB coach | NPB club staff | Farm-only clubs (Oisix, Hayate) | Independent leagues | Company (社会人) teams | Teams abroad | Commentator | Other baseball | Outside baseball | Undecided / unknown |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2020 | 133 | 28.1 | 7.7 | 8 | 23 | 9 | 28 | - | 16 | 10 | 1 | 4 | 2 (student-baseball coaches) | 18 | 14 |
| 2021 | 143 | 27.8 | 7.3 | 9 | 35 | 7 | 28 | - | 5 | 14 | 6 | 3 | 0 | 22 | 14 |
| 2022 | 145 | 27.8 | 7.7 | 4 | 41 | 8 | 30 | - | 9 | 12 | 0 | 2 | 6 | 22 | 11 |
| 2023 | 153 | 27.1 | 6.8 | 11 | 44 | 11 | 27 | 13 | 7 | 12 | 6 | 1 | 8 | 8 | 5 |
| 2024 | 157 | 26.3 | 6.3 | 5 | 48 | 9 | 31 | 6 | 3 | 16 | 3 | 1 | 1 | 13 | 21 |
| 2025 | 169 | 27.2 | 6.8 | 10 | 48 | 3 | 29 | 6 | 8 | 18 | **17** | 3 | 2 | 17 | 8 |

- Ages are as of 31 Oct from 2022 on.
- "Outside baseball" = company job, own business, school, other sport.
- 2025 had 17 moves to teams abroad. A legal column (Dec 2025) expects more moves abroad, citing the KBO's new Asia quota from 2026 (see `nationality.md`). [press] 新日本法規, above
- In the same column: of 7 支配下 players released during the 2025 Japan Series, by mid-November 2 had retired, 3 had re-signed 育成 with their club, and 2 had signed with other clubs (1 育成, 1 支配下).

**Ages at retirement:** the only statistic found is the mean age of this combined released-plus-retired group (26.3-28.1 years, 2020-2025). **A separate age distribution of voluntary retirements: NOT CONFIRMED.**

### 4E. Joint tryout (合同トライアウト)

| Year | Status | Source |
|---|---|---|
| 2001-2024 | NPB's 12-club joint tryout, from autumn 2001 | [press] Daily 2024-08-22 https://www.daily.co.jp/opinion-d/2024/08/22/0018033453.shtml |
| Aug 2024 | At NPB-union talks the clubs said they scout other clubs' players during the season and no longer sign from one tryout; holding it from 2025 was in doubt; they proposed the union host it | same |
| 2024 | 14 Nov, ZOZO Marine Stadium. 45 players, 3,420 spectators including 156 scouts. In the 2023 edition 2 participants got 育成 contracts. | [press] Daily 2024-11-14 https://www.daily.co.jp/baseball/2024/11/14/0018339590.shtml |
| 2025 | Hosted by the **players' association** at Mazda Stadium, Hiroshima. 39 entrants. | [press] まるスポ via Yahoo (below); hb-nippon 2026-11-09 https://www.hb-nippon.com/articles/19039 |
| 2026 | 「エイブル トライアウト2026」, **17 Nov**, Vantelin Dome Nagoya, union-hosted. 46 announced, 45 expected. Open also to players offered a 育成 deal after release, and to players trying to return to NPB. | [press] まるスポ via Yahoo https://news.yahoo.co.jp/articles/5cba8753897f0b3933bb60477fea043366d1e48d ; hb-nippon https://www.hb-nippon.com/articles/19039 |
| | Signings from the 2025 and 2026 tryouts | **NOT CONFIRMED** |

### 4F. Active-player draft (現役ドラフト)

Purpose: give players with too little playing time a move. Agreed by NPB and the players' association in Oct 2022 after 4 years of talks. Held privately; only the picks are announced. [press] Tokyo Shimbun 2022-11-20 https://www.tokyo-np.co.jp/article/214988 ; Baseball Channel https://www.baseballchannel.jp/npb/241961/

| Item | Rule | Source |
|---|---|---|
| Lists | Each club lists **2 or more** players | [press] Tokyo Shimbun, Baseball Channel |
| Order | Clubs tell the chair which listed player they want. The club drawing the most interest picks first. The next pick passes to the club that just lost a player, and so on. Every club takes at least 1, so **at least 12 move**. What happens when the turn passes to a club that has already picked: NOT CONFIRMED. | [press] Tokyo Shimbun 2022-11-20 |
| 2nd round | Optional (希望制). Since 2025, a club that only wants to give up a player may also take part. | [press] Baseball Channel 2025 https://www.baseballchannel.jp/npb/241961/ ; hb-nippon https://www.hb-nippon.com/articles/10422 |
| Not eligible | Foreign players; players on multi-year contracts; players with next-season salary **¥50M or more**; FA-qualified players and anyone who has used FA; 育成 players; players moved from 育成 to 支配下 after the season; players acquired by trade after October of the previous year | [press] Baseball Channel and hb-nippon (2025 rules) |
| Salary exception | Each club may list **one** player paid ¥50M to under ¥100M, but must still list at least 2 under ¥50M | [press] same |
| 2026 change | Players acquired by trade from **October of the previous year to 31 July** of the draft year are now eligible. NPB says the aim is a livelier 2nd round. | [press] Daily 2026-10-14 https://www.daily.co.jp/baseball/2026/10/14/0020926602.shtml |
| Dates | 9 Dec 2022 (Tokyo Shimbun); 8 Dec 2023, same rules as 2022 (Daily https://www.daily.co.jp/baseball/2023/10/02/0016875411.shtml ); 2024 NOT CONFIRMED; 9 Dec 2025 (Baseball Channel); **9 Dec 2026** (5th edition, Daily 2026-10-14) | [press] |
| Fee between clubs | **NOT CONFIRMED.** Not to be confused with the dormant 選抜会議 in 協約2025 Art. 125-132: one-fifth of each club's players listed, fee = salary + ¥2M / + ¥1M / salary, up to 3 losses a club. | [official] for the Art. 125-132 text |

Results [official] https://c.npb.jp/geneki_draft/2022/ (also /2023/, /2024/, /2025/):

| Year | Round 1 | Round 2 | Total moved |
|---|---|---|---|
| 2022 | 12 | none | 12 |
| 2023 | 12 | none | 12 |
| 2024 | 12 | 1 (Hiroshima took 鈴木健矢 from Nippon-Ham) | 13 |
| 2025 | 12 | none | 12 |

Transfers are dated about 11 days later in NPB's trade lists ("※現役ドラフトに伴う契約譲渡", e.g. 2022-12-20). [official] `pn_traded` pages

### 4G. 育成 contracts (adds to npb_notes D7)

Text: 育成規約2014 [official]. Only additions or confirmations here.

- **Pay and contract year:** contract year 1 Jan - 31 Dec; minimum ¥2.3M. (Art. 7(2), see 3B)
- **After 3 seasons without promotion:** automatic free agency, with notice by 31 Oct and release effective 30 Nov. Anyone, the old club included, may sign him from the next day, as 支配下 or 育成, one season at a time after that. A player who joins on or after 1 July does not have that year counted. (Art. 10(1), Art. 4(10))
- **育成 reserve list:** due by **31 Oct**. Players left off are free from **1 Nov**. (Art. 10(2))
  - NPB published it on 31 Oct in 2022-2025 and on **30 Oct in 2026**. [official] https://npb.jp/news/detail/20261030_01.html
  - So in practice 育成 releases are settled a month before 支配下 releases.
- **Trades:** 育成 contracts can be traded until 31 July. Relocation costs, paid by the receiving club: ¥300k + tax if he joins as 支配下, ¥100k + tax if he stays 育成. (Art. 11)
- **Foreign 育成 players aged 26 or over:** a newly signed one must be promoted by 31 March (Art. 9(3)). This confirms `nationality.md`, which cited Wikipedia.
- **Later amendments: NOT CONFIRMED.** I found none in the press either.

### 4H. Waivers (ウエイバー)

| Rule | Detail | Source |
|---|---|---|
| When | A club wanting to end a contract during the season must first ask for waivers | [official] 協約2025 Art. 115; 統一契約書2018 Art. 27 |
| Claim period | 7 days | [official] 協約2025 Art. 115 |
| Withdrawal | Within 3 days of applying | [official] 協約2025 Art. 117 |
| Priority | Same-league clubs first, in reverse order of winning percentage (current season in season, previous season otherwise) | [official] 協約2025 Art. 119 |
| Fee | ¥3.9M + tax | [official] 協約2025 Art. 121 |
| Player refuses | 資格停止 | [official] 協約2025 Art. 118 |
| Unclaimed | 自由契約, and **no club may sign him for that season** | [official] 協約2025 Art. 120 |
| Re-waiving a claimed player | Not within 60 days | [official] 協約2025 Art. 123 |
| Use | Unclaimed waivers: 3-19 a year, 2020-2026, almost all foreign players (4C). Number of successful claims: **NOT CONFIRMED**. | [data] |

### 4I. Trades

- Trade window: from the day after the season ends to **31 July**; waiver claims are exempt. [official] 協約2025 Art. 108 (as npb_notes D1)
- No loans, recall rights or conditional transfers. [official] Art. 107
- Players agree to future transfers in the uniform contract. [official] Art. 106
- Players get no share of transfer fees. [official] Art. 90
- Relocation allowance ¥2M + tax, split equally between the clubs, paid only if the player actually moves house (moving into a club dormitory does not count). Travel and moving costs are paid by the receiving club. [official] Art. 114 and the 2010 memorandum after Art. 124

---

## 5. Moving abroad

### 5A. Posting system: the 2017 agreement and its status now

| Item | Terms | Source |
|---|---|---|
| Agreement | New NPB-MLB protocol, consented to by the MLB players' union. Effective **1 Nov 2018**. Ratified by MLB clubs Dec 2017. 2017 postings (Ohtani, Makita) still ran under the 2013 protocol with a $20M fee cap. | [official, MLB] https://www.mlb.com/press-release/mlb-clubs-ratify-new-posting-system-agreement-with-npb-262712102 |
| Major-league contract fee | **20%** of guaranteed value up to $25M. Then **17.5%** of the part from $25M to $50M. Then **15%** of the part above $50M. Example: a $100M deal pays about $16.9M. | [official, MLB] glossary https://www.mlb.com/glossary/transactions/japanese-posting-system |
| Supplemental fee | 15% of bonuses and escalators actually earned, and of option-year pay when an option is exercised | same |
| Minor-league contract | 25% of the signing bonus. Supplemental fee if a deal with major-league terms leads to the 25-man roster. | same |
| Negotiating window | **45 days** after posting (the 2017 release said 30) | [official, MLB] glossary; NBC Sports / The Athletic 2026-11-02 says MLB changed it "before the 2021-22 lockout" ([press] https://www.nbcsports.com/fantasy/baseball/player-news/2026-11-02/rosenthal-npb-players-need-to-sign-before-lockout) |
| If unsigned | Returns to his club; cannot be posted again until the next offseason | [official, MLB] glossary |
| Posting period | 2017 release: **1 Nov - 5 Dec**. Current: **1 Nov - 15 Dec**, per the 2026 Hanshin announcement ("期限は12月15日"). | [official, MLB] 2017 release; [press] Daily 2026-10-19 https://www.daily.co.jp/tigers/2026/10/19/0020945504.shtml |
| Young players | Players under 25, or with under 6 professional seasons in a recognised foreign league, fall under MLB's international amateur bonus pools. That kept Sasaki's fee under the 20% band. | [official, MLB] glossary; https://www.mlb.com/news/roki-sasaki-faq |
| Text | The current agreement text is not public. The 2014 text (日米協定2014) is the last one posted. | [official] https://jpbpa.net/wp-content/uploads/2021/12/ju14.pdf |

[data] Worked example of the fee formula: for Yamamoto's $325M contract (12 years: MLB.com Sasaki FAQ) it gives $5M + $4.375M + $41.25M = **$50.625M** to Orix. I did not open a report of the actual payment.

### 5B. Club consent

- Posting starts only at the **club's request**. Under the 2014 agreement the NPB club asks NPB to notify MLB that it "wishes to make the player available". [official] 日米協定2014 §7-8
- The JPBPA explains the same; it adds that under the old system the club could refuse the winning bidder. [official] https://jpbpa.net/transfer/ (2010)
- In practice clubs announce "容認" (approval). Example: Hanshin approved 佐藤輝明 on 19 Oct 2026 and said it was not done on the player's wish alone. [press] Daily https://www.daily.co.jp/tigers/2026/10/19/0020945650.shtml
- A club that signed a player through domestic FA may not post him before he would have had international FA. [official] FA規約2009 Art. 13

### 5C. International FA

- 9 seasons (1B). No club consent, no compensation, no limit on numbers. [official] FA規約2009 Art. 2(2); JPBPA (2010)
- A declared international FA may also sign within NPB. 2020-2025 examples: 近藤 (to SoftBank), 九里 (to Orix), 則本 (to the Giants), 伊藤光 and 桑原 (within NPB, with NPB compensation rules applying). [official] `fa_filed` pages

### 5D. Departures to MLB, 2020-21 to 2025-26 offseasons

[official] NPB lists: postings https://npb.jp/announcement/pn_released_posting.html ; FA `fa_filed` pages. The dates are NPB's 自由契約 notice dates, not the MLB signing dates.

| Offseason | Posted and signed (MLB club, NPB notice date) | International FA to MLB |
|---|---|---|
| 2020-21 | 有原航平 (Rangers, 2021-01-12) | 澤村拓一 (Red Sox) |
| 2021-22 | 鈴木誠也 (Cubs, 2022-03-30) | |
| 2022-23 | 吉田正尚 (Red Sox, 2022-12-26), 藤浪晋太郎 (Athletics, 2023-01-27) | 千賀滉大 (Mets) |
| 2023-24 | 山本由伸 (Dodgers, 2024-01-10), 今永昇太 (Cubs, 2024-01-30), 上沢直之 (Rays, 2024-02-14) | 松井裕樹 (Padres) |
| 2024-25 | 青柳晃洋 (Phillies, 2025-01-28), 小笠原慎之介 (Nationals, 2025-02-12), 佐々木朗希 (Dodgers, 2025-02-21) | 菅野智之 (Orioles) |
| 2025-26 | 村上宗隆 (White Sox, 2026-01-09), 岡本和真 (Blue Jays, 2026-01-19), 今井達也 (Astros, 2026-01-20) | none |

Posted but unsigned:

- 2025-26: 髙橋光成 (Seibu) was posted in late November 2025. Negotiations failed (交渉不成立) with the deadline of 4 Jan US time (5 Jan Japan), and he re-signed with Seibu on 4 Jan 2026. The club said he had had major-league contract talks. [press] Daily https://www.daily.co.jp/baseball/2026/01/04/0019877790.shtml
- In 2026 he declared international FA instead (6A).
- **Earlier unsigned postings: NOT CHECKED.**

### 5E. The 2026-27 offseason so far

- 佐藤輝明 (Hanshin, 2026 Central League Triple Crown): club approved on 19 Oct; posting completed and accepted by MLB on **2 Nov**. [press] Daily (above); hb-nippon https://www.hb-nippon.com/articles/18898
- NBC Sports, citing The Athletic, also names 伊藤大海 (Nippon-Ham) and 平良海馬 (Seibu) as posted. **NOT CONFIRMED by a Japanese source.** [press] NBC Sports (5A)
- **A lockout may start on 1 Dec 2026.** NBC/The Athletic report that MLB will not change the posting window this time, so posted players probably have to sign before a lockout or go back to Japan. They also report that MLB amended the agreement before the 2021-22 lockout. [press] NBC Sports (above)

---

## 6. Recent markets

### 6A. FA market by offseason

From NPB `fa_qualified`, `fa_filed`, `fa_signed` pages and news posts [official]. Outcomes are counted by me [data].

| Offseason | FA-qualified (new domestic / new intl / held domestic / held intl) | Declared (domestic + intl) | Declared % of qualified | Stayed | Moved within NPB | To MLB | Unsigned |
|---|---|---|---|---|---|---|---|
| 2020 | 97 (21/16/17/43) | 7 (6+1) | 7.2% | 4 | 2 | 1 | 0 |
| 2021 | 97 (23/11/17/46) | 3 (2+1) | 3.1% | 2 | 1 | 0 | 0 |
| 2022 | 109 (24/19/20/46) | 8 (5+3) | 7.3% | 3 | 4 | 1 | 0 |
| 2023 | 106 (21/20/15/50) | 7 (5+2) | 6.6% | 3 | 3 | 1 | 0 |
| 2024 | 111 (24/7/19/61) | 9 (7+2) | 8.1% | 3 | 5 | 1 | 0 |
| 2025 | 104 (18/12/20/54) | 8 (4+4) | 7.7% | 3 | 5 | 0 | 0 |
| 2026 | 105 (22/17/14/52) | 10 (8+2) | 9.5% | open | open | open | open |
| **2020-25** | | **42** | | **18 (43%)** | **20 (48%)** | **4 (10%)** | **0** |

- 2015-2019 declarations, for the longer trend: 6, 6, 7, 5, 6. [official] `fa_filed` 2015-2019 pages; pages before 2015 return "page not found".
- 2026 declared players [official] https://npb.jp/announcement/2026/fa_filed.html :
  - Domestic: 太田光 (Rakuten C), 小島和哉 (Lotte P), 宗佑磨 and 中川圭太 (Orix IF), 上林誠知 (Chunichi OF), 森下暢仁 and 床田寛樹 (Hiroshima P), 坂倉将吾 (Hiroshima C)
  - International: 髙橋光成 (Seibu P), 戸柱恭孝 (DeNA C)
- Conflict: Baseball King (2026-10-19) gives 21 new domestic qualifiers and 104 in total; NPB says 22 and 105. **NPB is used.** [press] https://baseballking.jp/ns/710245/

### 6B. Every declared player, 2020-2025

Outcome and date from NPB [official]; compensation from NPB trade lists (2D); contract terms from press, all **estimates**.

| Offseason | Player (club, position, type) | Outcome (NPB date) | Comp. player | Reported contract [press] |
|---|---|---|---|---|
| 2020 | 熊代聖人 (Seibu OF, dom.) | stayed 2020-12-08 | | |
| 2020 | 増田達至 (Seibu P, dom.) | stayed 2020-12-08 | | |
| 2020 | 松永昂大 (Lotte P, dom.) | stayed 2021-01-28 | | |
| 2020 | 小川泰弘 (Yakult P, dom.) | stayed 2020-12-25 | | 4 years (SPAIA https://spaia.jp/column/baseball/npb/30137) |
| 2020 | 梶谷隆幸 (DeNA OF, dom.) | Giants 2020-12-14 | 田中俊太 | NOT FOUND |
| 2020 | 井納翔一 (DeNA P, dom.) | Giants 2020-12-14 | none | NOT FOUND |
| 2020 | 澤村拓一 (Lotte P, intl.) | MLB Red Sox | | |
| 2021 | 又吉克樹 (Chunichi P, dom.) | SoftBank 2021-12-20 | 岩嵜翔 | 4 years ¥650M; his salary was ¥42M (Sportiva, above) |
| 2021 | 岡田雅利 (Seibu C, dom.) | stayed 2021-12-08 | | |
| 2021 | 大和 (DeNA IF, intl.) | stayed 2021-12-08 | | |
| 2022 | 伏見寅威 (Orix C, dom.) | Nippon-Ham 2022-11-24 | none | NOT FOUND |
| 2022 | 森友哉 (Seibu C, dom.) | Orix 2022-11-28 (announced 16 Nov) | 張奕 | 4 years ¥1.8bn (Daily https://www.daily.co.jp/baseball/2022/11/17/0015814770.shtml); 4 years ¥1.6bn including incentives (Tokyo Sports https://www.tokyo-sports.co.jp/articles/-/245247) |
| 2022 | 外崎修汰 (Seibu IF, dom.) | stayed 2022-11-10 | | |
| 2022 | 嶺井博希 (DeNA C, dom.) | SoftBank 2022-11-24 | none | 4 years ¥300M (Nikkan Gendai https://www.nikkan-gendai.com/articles/view/sports/315837/2) |
| 2022 | 岩崎優 (Hanshin P, dom.) | stayed 2022-11-10 | | |
| 2022 | 千賀滉大 (SoftBank P, intl.) | MLB Mets | | |
| 2022 | 近藤健介 (Nippon-Ham OF, intl.) | SoftBank 2022-12-16 | 田中正義 | 7 years about ¥5bn (Gendai https://gendai.media/articles/-/103404 ; Nikkan Gendai, above) |
| 2022 | 西勇輝 (Hanshin P, intl.) | stayed 2022-11-28 | | |
| 2023 | 西川龍馬 (Hiroshima OF, dom.) | Orix 2023-12-04 | 日髙暖己 | 4 years ¥1.2bn or more (SPAIA https://spaia.jp/column/baseball/npb/30598) |
| 2023 | 石田健大 (DeNA P, dom.) | stayed 2023-12-21 | | |
| 2023 | 山﨑福也 (Orix P, dom.) | Nippon-Ham 2023-12-06 | none | 4 years ¥1.0bn (Daily https://www.daily.co.jp/baseball/2023/12/06/0017102922.shtml); 4 years ¥800M, against reported rival offers of 4y ¥1.2bn (SoftBank) and 4y ¥1.0bn (Giants) (Nikkan Gendai https://www.nikkan-gendai.com/articles/view/sports/332551) |
| 2023 | 平井克典 (Seibu P, dom.) | stayed 2023-12-13 | | |
| 2023 | 山川穂高 (Seibu IF, dom.) | SoftBank 2023-12-21 | 甲斐野央 | 4 years ¥1.2bn plus incentives (Kyodo via Ryukyu Shimpo https://ryukyushimpo.jp/news/entry-2603027.html); "over ¥1.2bn" (SPAIA) |
| 2023 | 田村龍弘 (Lotte C, intl.) | stayed 2023-12-13 | | |
| 2023 | 松井裕樹 (Rakuten P, intl.) | MLB Padres | | |
| 2024 | 大山悠輔 (Hanshin IF, dom., A) | stayed 2024-12-19 (announced 29 Nov) | | 5 years ¥1.7bn plus incentives (Daily https://www.daily.co.jp/tigers/2024/11/29/0018393798.shtml) |
| 2024 | 原口文仁 (Hanshin IF, dom., C) | stayed 2024-12-19 | | |
| 2024 | 福谷浩司 (Chunichi P, dom., C) | Nippon-Ham 2025-01-06 | none | NOT FOUND |
| 2024 | 木下拓哉 (Chunichi C, dom., B) | stayed 2024-12-20 | | |
| 2024 | 石川柊太 (SoftBank P, dom., C) | Lotte 2024-12-19 | none | NOT FOUND |
| 2024 | 甲斐拓也 (SoftBank C, dom., B) | Giants 2024-12-20 | 伊藤優輔 | 5 years about ¥1.5bn (Daily https://www.daily.co.jp/baseball/2024/12/27/0018487420.shtml) |
| 2024 | 茂木栄五郎 (Rakuten IF, dom., B) | Yakult 2024-12-11 | 小森航大郎 | NOT FOUND |
| 2024 | 菅野智之 (Giants P, intl., A) | MLB Orioles | | |
| 2024 | 九里亜蓮 (Hiroshima P, intl., B) | Orix 2024-12-26 | none (cash) | NOT FOUND |
| 2025 | 東浜巨 (SoftBank P, dom.) | stayed 2026-01-26 | | |
| 2025 | 石井一成 (Nippon-Ham IF, dom.) | Seibu 2026-01-06 | none | NOT FOUND |
| 2025 | 松本剛 (Nippon-Ham OF, dom.) | Giants 2025-11-28 | none | NOT FOUND |
| 2025 | 辰己涼介 (Rakuten OF, dom.) | stayed 2026-01-19 | | |
| 2025 | 則本昂大 (Rakuten P, intl.) | Giants 2026-01-20 (club announced 16 Jan) | 田中千晴 | NOT FOUND |
| 2025 | 伊藤光 (DeNA C, intl.) | Rakuten 2025-11-19 | none | NOT FOUND |
| 2025 | 桑原将志 (DeNA OF, intl., B, salary ¥120M) | Seibu 2025-12-22 (club announced 25 Nov) | 古市尊 | NOT FOUND |
| 2025 | 松葉貴大 (Chunichi P, intl.) | stayed 2025-12-04 | | |

Ranks shown are from press (2D); the rest are not known.

### 6C. Contract lengths and values: what the sample says

- **Completeness.** Terms were found for **9 of the 20 moves** and **2 of the 18 stays**.
- **Bias.** These are press estimates (推定), and the large deals are the ones that get reported.
- [data] **Lengths** in the sample: 4 years (又吉, 森, 嶺井, 西川, 山﨑, 山川, 小川), 5 years (甲斐, 大山), 7 years (近藤).
- [data] **Average annual values:** ¥75M (嶺井), about ¥160M (又吉), ¥250M (山﨑), ¥300M or more (西川, 山川, 甲斐), ¥340M (大山), ¥450M (森), about ¥710M (近藤).
- For comparison, a non-FA extension: 近本 5 years ¥2.5bn (¥500M a year).
- The value of the remaining moves, many of them C-rank veterans, is **NOT CONFIRMED**.

### 6D. Released players

See 4C (official counts) and 4D (official destinations).

Short version for the model, 2020-2025:

- 126-151 支配下 players are not reserved each 2 Dec. About a quarter have foreign-style names.
- Among Japanese players released or retired (n = 133-169 a year, same-club 育成 re-signs excluded):
  - only **4-11 (3-7%) sign another NPB 支配下 contract**
  - **23-48 (17-31%) sign an NPB 育成 contract**
  - **29-40 stay in NPB as coaches or staff**
  - the rest go to company or independent teams, abroad, or out of baseball

### 6E. Salaries: JPBPA survey (年俸調査)

Source [official]: JPBPA research pages https://jpbpa.net/2026rr/ and https://jpbpa.net/2025rr/ . PDFs `https://jpbpa.net/wp-content/uploads/jpbpa-pdf/26nenpou1.pdf` … `26nenpou7.pdf` (and `25nenpou*`).

Population: union members on 支配下 contracts at **opening day**. Foreign players and some others are not members. The 2026 results were released 27 Apr 2026. [press] Kyodo via Sakigake https://www.sakigake.jp/news/article/20260427CO0087/

Per-club averages and medians:

| Club | 2025 players | 2025 average (¥M) | 2025 median (¥M) | 2026 players | 2026 average (¥M) | 2026 median (¥M) |
|---|---|---|---|---|---|---|
| SoftBank | 59 | 69.56 | 20.5 | 57 | **87.06** | 22.0 |
| Hanshin | 62 | 57.99 | 25.0 | 61 | 77.89 | 22.0 |
| Giants | 56 | **78.00** | **28.0** | 56 | 68.80 | **27.0** |
| Orix | 60 | 48.53 | 23.0 | 58 | 54.06 | 20.0 |
| Nippon-Ham | 62 | 40.62 | 23.0 | 59 | 49.64 | 26.0 |
| Rakuten | 63 | 43.79 | 16.0 | 63 | 44.19 | 21.0 |
| Seibu | 61 | 37.44 | 14.0 | 60 | 43.33 | 15.0 |
| Hiroshima | 64 | 40.07 | 19.0 | 62 | 41.39 | 16.0 |
| Chunichi | 58 | 38.98 | 18.5 | 59 | 41.02 | 22.0 |
| DeNA | 59 | 44.66 | 22.0 | 56 | 40.84 | 21.0 |
| Lotte | 61 | 42.16 | 25.0 | 61 | 40.41 | 20.0 |
| Yakult | 60 | 49.77 | 17.0 | 61 | 39.91 | 18.0 |
| **Central League** | 359 | 51.28 | 20.0 | 355 | 51.58 | 20.0 |
| **Pacific League** | 366 | 46.85 | 19.0 | 358 | 52.72 | 20.0 |
| **All** | **725** | **49.05** | **19.0** | **713** | **52.16** | **20.0** |

- 2026 by position: pitchers 355 players, ¥49.73M; catchers 81, ¥42.87M; infielders 164, ¥55.38M; outfielders 113, ¥61.75M. [official] 26nenpou1
- History, all clubs [official] 26nenpou2, 26nenpou3, 26nenpou6:

| Year | Players | Average (¥M) | Median (¥M) |
|---|---|---|---|
| 2020 | 727 | 41.89 | 15.0 |
| 2021 | 730 | 41.74 | 15.0 |
| 2022 | 721 | 43.12 | 15.0 |
| 2023 | 714 | 44.68 | 16.0 |
| 2024 | 716 | 47.13 | 18.0 |
| 2025 | 725 | 49.05 | 19.0 |
| 2026 | 713 | 52.16 | 20.0 |

- Amounts exclude tax from 2016 (table note).
- **Distribution:** the JPBPA publishes only averages and medians, by club, league and position. **Percentiles or a full distribution: NOT CONFIRMED.**
  - [data] Mean / median = 2.6 in 2026: a very right-skewed distribution.
- **Satisfaction with the 2025 renewals:** the share of players satisfied ranged from 66.67% (Hanshin) to 28.85% (Orix). [official] 26nenpou7

### 6F. Ages of retirement

Only the NPB survey in 4D: mean age **26.3-28.1** and mean NPB tenure **6.3-7.7 years** for released-plus-retired Japanese players, 2020-2025. [official]

**A breakdown by age band, or for voluntary retirements alone: NOT CONFIRMED.**

---

## 7. Announced for 2027 and beyond

| Item | Status | Source |
|---|---|---|
| FA compensation: no player compensation from the **2027 offseason**; draft pick (A: extra 2nd round, B: extra 3rd round) or cash (A 80-130%, B 60-100%); B narrowed to ranks 4-8; C from rank 9 | Agreed and approved 30 Oct 2026; review around the 2029 offseason; treatment of multiple same-rank signings still open | [press] Nikkan Sports, Sponichi, Daily, Hochi (2C); [official] JPBPA statement |
| No compensation for movers aged 35+ (on 1 Apr of the next year) | From the **2026** offseason | same |
| 現役ドラフト | 5th edition **9 Dec 2026**. Players traded between last October and 31 July now eligible. | [press] Daily 2026-10-14 |
| FA years | Union asks for shorter periods; nothing agreed. The exact demand is NOT CONFIRMED. | [press] Hochi 2026-10-30 says the shortening was requested "in parallel" with abolishing player compensation |
| Posting | No published change to the 2017 terms. The MLB labour deal expires 1 Dec 2026, and a lockout may cut short this winter's 45-day windows. | [press] NBC Sports / The Athletic 2026-11-02 |
| Joint tryout | Run by the union since 2025; held 17 Nov 2026 | [press] (4E) |
| Release-notice deadline 30 Oct | Follows from the 2024 Freelance Act, so it will recur each year | [press, legal] (4B) |
| Minimum salary or reduction limit for 2027 | No announcement found | NOT CONFIRMED |
| NPB business plan to Sept 2027 | Lists only routine items: running the drafts and the 現役ドラフト, "talks with the union on various issues", editing the 協約 and contract form, publishing FA notices | [official] https://npb.jp/organization/docpb_09_2026.pdf |

---

## 8. Conflicts and corrections

1. **JPBPA 2026 salary table (6), "前年比" column.** It prints +10.7% (all clubs), +17.2% (PL) and +4.8% (CL) for 2026. Those are the changes from **2024**. The real changes from 2025 to 2026 are **+6.3%, +12.5% and +0.6%** [data, from the same table's averages]. Kyodo confirms +¥3.11M, i.e. 6.3%. The per-club tables (4) and (5) compute the change correctly. [official] 26nenpou6; [press] Sakigake/Kyodo
2. **Minimum salary ¥4.2M versus ¥4.4M, and 育成 ¥2.3M versus ¥2.4M.** The higher figures are the pre-2014, tax-inclusive basis (2013 editions). Some 2025-26 explainers still quote them. Use **¥4.2M / ¥2.3M plus tax** (3B).
3. **Active-roster top-up threshold.** ¥16M in the 2025 agreement versus ¥17M on a garbled secondary explainer (bb-news.jp). The agreement wins.
4. **Release-notice deadline before 2025.** "The day after the Japan Series" (Daily 2024) versus "2 days after" (hb-nippon 2025). Moot from 2025.
5. **2026 FA-qualified count.** NPB: 22 new domestic and 105 in total. Baseball King, same day: 21 and 104. NPB is used.
6. **When FA-qualified lists are published.** FA規約2009 says 2 days after the regular season. Since 2022 the lists came after the Climax Series (17-23 Oct). That part of the FA規約 has evidently been amended; the current text is not public. The declaration timetable, by contrast, still matches the 2009 text (1C).
7. **Contract estimates differ between papers.** 森: ¥1.8bn (Daily) v ¥1.6bn including incentives (Tokyo Sports). 山﨑: ¥1.0bn (Daily) v ¥800M (Nikkan Gendai). Both are given in 6B.
8. **Posting window end.** The 2017 MLB release says 5 Dec; Hanshin 2026 says 15 Dec. NBC also writes that posted players "have until December 15th to sign", which does not fit a 45-day window starting in November. Treat **15 Dec** as the last posting date and **45 days** as the window.
9. **Where the 45-day window came from.** NBC/Rosenthal says it was changed "before the 2021-22 lockout"; a search summary said December 2022. Unresolved.
10. **Confirms the companion notes.**
    - `npb_notes.md` D1 (70 limit, 31 July deadline, Art. 64/79/108) and D7 (育成 3-year rule, 31 July promotion deadline, 26+ foreign 育成 rule) match the texts.
    - Addition for D7: the 育成 reserve list and releases come on **31 Oct** (30 Oct in 2026), a month before 支配下 releases.
    - `nationality.md` FA exception (Art. 82(4)): confirmed. Note that it applies only to the roster chapter, and that FA ranking still treats these players as foreign (FA規約2009 Art. 10(2)).
11. **JPBPA's own 2010 FA page** labels two compensation rows wrongly ("人的補償なし" for the player-plus-cash rows). The FA規約 text is correct.

## 9. Not confirmed (gaps)

- **Current FA規約 text.** Only the 2009 edition is published.
  - Not confirmed for today: the salary-cap clause (Art. 7), the signing limits (Art. 11), the 40-day completion rule, the protected-list details (whether that year's draftees count).
  - Partly confirmed by 2025 press: the 28-man list and the 40% / 60% B-rank amounts.
- **Official NPB text of the 2027 compensation system.** Press only, plus a JPBPA statement without figures.
- Whether the 2023 starting-pitcher day-count proposal was adopted; how the 2020 short season was counted.
- **Club-by-club policies on staying after declaring.** Only Hiroshima's 2026 change was found.
- **Salary arbitration:** cases after 2011 and the all-time total.
- **Multi-year contracts:** how they are documented; whether the reduction limit applies when one ends.
- **育成規約** amendments after 2014; any 2026-27 change in minimum salaries.
- **現役ドラフト:** official rule text and any fee between clubs.
- **Waivers:** number of successful claims.
- **Contract terms** for 11 of the 20 FA moves and 16 of the 18 stays, 2020-2025.
- **Releases:** the final 2026 release count (the official list is due 2 Dec 2026); signings that came from the 2024-2026 tryouts.
- **Retirement ages:** a breakdown of voluntary retirements by age.
- **2026-27 postings:** 伊藤大海 and 平良海馬 are confirmed only by English press; outcomes are pending.

## 10. Local copies

Downloaded copies of the sources and the counting scripts were kept outside the repository; every figure above links to its source.
