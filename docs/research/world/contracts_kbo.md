# KBO contracts, free agency and recent FA markets

Research date: 2026-11-17. This is for the free-agent market and contract system of the simulation. It complements `docs/research/world/kbo_notes.md` and `docs/research/world/nationality.md`, which it does not repeat except to confirm or correct them (section 7).

**Tags.** [official] = KBO bylaws, KBO regulations, KBO press releases, KBO's own data pages. [press] = Korean sports press. [wiki] = namu.wiki / Wikipedia, used only as a pointer (none was needed in the end). [data] = a table or figure I put together myself; the method is given each time.

**Rule followed.** Every number comes from a source I read and is cited next to it. "NOT CONFIRMED" means I found no source. Where sources disagree, both are given (also collected in section 11).

**Money.** Amounts are Korean won as published: 1억원 = 100,000,000 won; 1만원 = 10,000 won. "4년 80억" = 4 years, 8.0 billion won in total. In the tables, "억" means 100 million won.

## 0. Sources

| Short name | What it is | URL |
|---|---|---|
| Bylaws | 2026 KBO 야구규약 (bylaws), including the uniform player contract, the 외국인선수 고용규정 (foreign-player employment rules) and the 한·미 / 한·일 player-contract agreements. 268-page PDF, server last-modified 22 Apr 2026, sha256 127a572c…6a9cff. Article numbers refer to the KBO 규약 part unless I write "Foreign-player rules" or "Contract form". | https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf |
| League regulations | 2026 KBO 리그 규정. Checked: it holds almost nothing on contracts (only the Futures-club rules in Art. 78); all contract and FA rules are in the bylaws. | https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_리그규정.pdf |
| KBO release number N | KBO press release no. N, with its attachments (docx/xlsx/pdf) | https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=N |
| Movement data | KBO "선수 이동 현황" (player movement) page. I pulled every record for 2020-2026 in the categories FA contract, FA compensation player, release (자유계약선수), waiver, 임의해지, trade and secondary draft. The page loads its data from `POST https://www.koreabaseball.com/ws/Player.asmx/GetTradeList`. The dates are KBO publication/approval dates, which can be a few days after a club's announcement. | https://www.koreabaseball.com/Player/Trade.aspx |
| KBO player pages | Official birth dates of retired players, e.g. https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=74339 | (individual links in 9.8) |

---

## 1. Quick reference for the game

| Item | Value | Source |
|---|---|---|
| Season of service toward FA | ≥145 days on the first-team active roster in a regular season | [Bylaws Art. 162(2)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Seasons needed for first FA | High-school entrants 8, four-year college graduates 7 (9 and 8 until the 2022 season) | [Bylaws Art. 162(1),(5)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Re-FA | 4 more seasons after exercising FA rights | [Bylaws Art. 164](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| FA calendar | List ≤5 days after the Korean Series → 2 days to apply → approved list the next day → open market for all clubs (incl. foreign) from the day after | [Bylaws Arts. 165-168](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Outside FAs a club may sign | 1 / 2 / 3 / 4 when 1-10 / 11-20 / 21-30 / 31+ players applied (2026-27: 22 applied, so 3) | [Bylaws Art. 173](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf), [N12243](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12243) [official] |
| Compensation, grade A | 200% of previous salary + 1 player outside 20 protected, or 300% | [Bylaws Art. 172](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Compensation, grade B | 100% + 1 player outside 25 protected, or 200% | same |
| Compensation, grade C (also new FAs aged 35+, 3rd+ FAs) | 150%, no player | same |
| Contract length / total limits | None in the bylaws | [Bylaws](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Minimum salary | 30,000,000 won in 2026; 33,000,000 won from 2027 | [Bylaws Art. 71 + change table](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf), [N11814](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11814) [official] |
| Limit on salary cuts | None (abolished in 2008) | [Kyunghyang 2008-02-21](https://www.khan.co.kr/article/200802211753415) [press]; none in the 2026 bylaws [official] |
| Competitive-balance tax cap (top-40 payroll) | 2023-24: 114억 2,638만; 2025: 137억 1,165만; 2026: 143억 9,723만; 2027: 151억 1,709만; 2028: 158억 7,294만 | [N8558](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8558), [N10160](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10160), [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687) [official] |
| Payroll floor | 60억 6,538만 from 2027, raised 5% a year | [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687) [official] |
| Reserve list | Max 63 (military and developmental players not counted); due 25 Nov (18 Nov from 2027); published 30 Nov | [Bylaws Arts. 56-57](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf), [N12184](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12184) [official] |
| Posting to MLB | After 7 KBO seasons; club decides; 1 player per club per year; window 1 Nov-5 Dec; fee 20% / 17.5% / 15% of guaranteed value | [Bylaws Art. 104 and KBO-MLB agreement](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Average salary (excl. rookies and foreigners) | 2025: 1억 6,071만 (519 players); 2026: 1억 7,536만 (529 players, Asia-quota players also excluded) | [N11394](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11394), [N11861](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11861) [official] |

---

## 2. FA eligibility (question 1)

### 2.1 How a service season is counted

All from [Bylaws Arts. 162-163](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] unless marked.
- A player earns one "regular season" toward FA in a season where he has **145 or more days on the first-team active roster (현역선수 등록일수)** (Art. 162(2)3). It was 150 days until 2005.
- Older criteria for players first registered before the 2006 season: batters who played 2/3 of the team's games, or pitchers who threw 2/3 of the qualifying innings (games × 1 inning). Players first registered in 2006 or later are judged on days only (Art. 162(2)).
- **Counting window**: from opening day to the player's club's last regular-season game. Spring training games and the postseason are excluded (Art. 162(2),(4)).
- **Short seasons add up.** The records of several seasons below the threshold are added together. Once the total meets a criterion, it counts as one season (Art. 163).
- If force majeure (e.g. COVID-19) shortens the season, the 145 days are scaled down in proportion (Art. 162(2)).
- **Two players on the roster at once** (batter and pitcher): only the day criterion is used (Art. 162(3)).
- **Days that count without being on the active roster**:
  - Injured-list days count as roster days (league regulations Art. 14.3; summarised in kbo_notes.md).
  - From 2026, if a club forgot to file an injured-list request for a player out 30+ days with an obvious injury, KBO may credit the days retroactively, at most 3 times per club per year ([N11879](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11879)) [official].
  - Bereavement or family leave, including reserve-forces training from 2026, counts up to 5 days ([N11878](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11878)) [official].
  - Players called up for the 2023 Hangzhou Asian Games got back the roster days they missed ([N8750](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8750)) [official]. NOT CONFIRMED whether the same was done for the 2026 Asian Games.

### 2.2 Seasons required

| Player | Until the 2022 season | From the end of the 2022 season | Source |
|---|---|---|---|
| High-school entrant (any player not covered below) | 9 seasons | **8 seasons** | [Bylaws Art. 162(1)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf); [N7651](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=7651) (decision of 21 Jan 2020) [official] |
| Four-year college graduate (registered 4 years as a college player with the KBSA) | 8 seasons (9 if going abroad) | **7 seasons**; the separate "9 seasons to go abroad" rule was deleted | [Bylaws Art. 162(5)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Re-FA after exercising FA rights, or after returning from a foreign club under Art. 104 | 4 seasons | 4 seasons | [Bylaws Art. 164](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |

- NOT CONFIRMED: whether "early draft" picks (2nd-year university players, allowed since 2022, [Bylaws Art. 110(7)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) count as college graduates for the 7-season rule. Art. 162(5) requires 4 registered college years.
- **The KBO lists confirm the counts.** Each FA list gives "인정 연수" (seasons credited): 8 for new FAs from high school, 7 for those marked "대졸" (college), 4 for re-FAs. Example: [N11753 attachment](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11753) [official].
- **Multi-year contracts defer FA status.** A player whose contract (a multi-year deal or an exercised extension option) does not end that year cannot use his FA rights. He is published as a "FA자격유보선수" (FA eligibility deferred), and his eligibility waits until the contract ends. Clubs must report their multi-year players before the Korean Series starts ([Bylaws Arts. 162(6), 164(3), 165(2)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf), amended 11 Jan 2024) [official]. Deferred players: 11 in the 2025 list, 14 in 2026, 9 in 2027 ([N10294](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10294), [N11753](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11753), [N12242](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242)) [official].
- **Eligibility is kept.** A player who qualifies but does not apply keeps his eligibility (listed as "자격유지") and may apply in a later year ([Bylaws Art. 165(1)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf); the lists in section 9) [official].

### 2.3 Calendar

**The rule** ([Bylaws Arts. 165-169](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]:
1. Within 5 days after the Korean Series ends, the commissioner publishes the eligible players and their grades.
2. A player applies within 2 days. He gives his FA application to his own club, which forwards it. A player who does not apply has given up his rights for that year only.
3. The approved FAs are published the day after the application deadline.
4. From the next day they may negotiate and sign with **all clubs, foreign clubs included**. Talks with foreign clubs must follow the posting/contract agreements (Art. 168(2)).
5. Approved FAs must still attend their old club's and KBO events until 30 November (Art. 167(2)).
6. A contract with a new club is sent to KBO the day after it is signed, and KBO publishes it the next day (Art. 169(2)).

There is no exclusive negotiating window with the old club. It was abolished in 2016 ([OSEN 2016-03-07](https://osen.co.kr/article/G1110368144)) [press].

**Actual dates** (all [official], from the KBO releases linked in the first column):

| Market (KBO name) | Eligible list | Apply by | Approved list | Open market from | Last FA contract published (movement data) |
|---|---|---|---|---|---|
| 2021-22 ("2022 FA", [N8215](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8215), [N8218](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8218)) | 22 Nov 2021 | 24 Nov | 25 Nov | 26 Nov | 6 Jan 2022 |
| 2022-23 ([N8556](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8556), [N8562](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8562)) | 13 Nov 2022 | 15 Nov | 16 Nov | 17 Nov | 30 Mar 2023 |
| 2023-24 ([N9883](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9883), [N9887](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9887)) | 15 Nov 2023 | 17 Nov | 18 Nov | 19 Nov | 31 Jan 2024 |
| 2024-25 ([N10294](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10294), [N10297](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10297)) | 2 Nov 2024 | 4 Nov | 5 Nov | 6 Nov | 5 Feb 2025 |
| 2025-26 ([N11753](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11753), [N11762](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11762)) | 5 Nov 2025 | 7 Nov | 8 Nov | 9 Nov | 9 Feb 2026 |
| 2026-27 ([N12242](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242), [N12243](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12243)) | 13 Nov 2026 | 15 Nov | 16 Nov | 17 Nov | (market open) |

The last column comes from the [movement data](https://www.koreabaseball.com/Player/Trade.aspx).

### 2.4 Players who go unsigned

- **No signing deadline** appears in the 2026 bylaws. KBO can approve contracts with "FA 미계약선수" (unsigned FAs) after 1 February ([Bylaws Art. 44(2)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].
  - Example: 이용찬 signed with NC on 22 May 2021, in the middle of the season, and the compensation player (박정수, NC→Doosan) followed on 28 May ([movement data](https://www.koreabaseball.com/Player/Trade.aspx)) [official].
- **Pay is pro-rated.** Under the uniform contract, a contract signed after 1 February loses 1/300 of the annual salary for each day from 1 February to the day before signing ([Contract form Art. 7(3)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]. From 2027 the participation period starts on 25 January (section 10).
- **Compensation stays attached for 3 years.** It is still owed if the FA joins another club within 3 years of his approval ([Bylaws Art. 172(3)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].
- **After 3 years unsigned** the player is published as a 자유계약선수 (free agent) and may sign anywhere ([Bylaws Art. 169(3)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].
- **Late signings change the compensation lists.** If an FA signs after 1 February (25 January in the 2027 schedule), that year's rookies are also exempt from the compensation lists ([Bylaws Art. 172(8)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf); [N12242 attachment 2](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242)) [official].
- **Sign-and-trade** is used: the old club re-signs the player and trades him on the same day ([movement data](https://www.koreabaseball.com/Player/Trade.aspx)) [official].
  - 이명기, NC→Hanwha, 14 Feb 2023
  - 이지영, Kiwoom→SSG, 12 Jan 2024
  - 김민성, LG→Lotte, 26 Jan 2024
- **Recent unsigned cases.**
  - 강윤구 (강리호), grade C, 2022-23: played in an independent league and retired on 8 Jul 2023 ([OSEN 2023-07-09](http://www.osen.co.kr/article/G1112140612)) [press].
  - 문성현, grade C, 2024-25: no offer, independent league, retired 5 May 2026 ([SPOTV 2026-05](https://www.spotvnews.co.kr/news/articleView.html?idxno=814811)) [press].
  - 손아섭, grade C, 2025-26: unsigned until 5 Feb 2026, then 1 year / 1억 with Hanwha ([Newsis 2026-02-05](https://www.newsis.com/view/NISX20260205_0003504439)) [press].

---

## 3. FA grades and compensation (question 2)

### 3.1 How grades are set

Sources: [Bylaws Art. 172(1)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official]; grade table in [N12242 attachment 2](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242) [official]; introduced after the 2020 season, [N7651](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=7651) [official].

- **Ranking measure**: average salary plus options received over the **last 3 years**.
- **Who is ranked**: everyone except players already on FA contracts and players on non-FA multi-year contracts.
  - A non-FA multi-year player's **final contract year** is included, at the contract's average salary (total value ÷ years).
  - The non-FA exclusion dates from 21 Jan 2025 ([N11353](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11353)) [official].

| Grade | Rank within the club | Rank in the league |
|---|---|---|
| A | 1-3 | 1-30 |
| B | 4-10 | 31-60 |
| C | 11 or lower | 61 or lower |

- **Both ranks must fit the grade**; otherwise the player drops one grade. A club rank in A with a league rank in C (or the reverse) gives B.
- **First year only (2020-21 market)**: any player in the top 30 of the league was A regardless of his club rank.
- **New FAs aged 35+** (counted by birth year: born in or before season year − 35, e.g. "만 35세 이상(1989년생)" in the 2025 list): compensation is 150% cash only, and KBO lists them as grade C ([N10294 attachment](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10294)) [official].
- **Re-FAs**:
  - A 2nd FA is compensated like grade B. If his first FA was grade C, he stays C.
  - A 3rd or later FA gets 150% cash only.
  - KBO lists show 2nd FAs as B or C and 3rd+ FAs as C.
- **FA returning from abroad**: compensated by the grade published before he left (Art. 172(1)4).

### 3.2 What the old club receives

Source: [Bylaws Art. 172](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official]. "Previous salary" is the FA's salary for the season just ended, since signing happens between the postseason and the next season.

| Grade | Option 1 | Option 2 | Who chooses |
|---|---|---|---|
| A | 200% of previous salary + 1 player not among the signing club's **20** protected players | 300% of previous salary | Old club, within 3 days of receiving the list |
| B (and most 2nd FAs) | 100% + 1 player not among **25** protected | 200% | Old club |
| C, new FA aged 35+, 3rd+ FA, re-FA who was C at first FA | 150%, no player | - | (no choice) |

**Procedure and exceptions** (same article):
- **Timing.** The signing club sends its list (everyone outside the protected players) within 3 days of KBO publishing the contract. The old club chooses within 3 days. Cash is due within 7 days of the tax invoice; if late, legal interest + 20% per day is added.
- **Who is never on the lists** (neither protected nor available): military-reserve players, this year's FAs, foreign players, players who moved as FA compensation this year, this year's secondary-draft picks, and developmental players (including those in military service). If the FA signs after 1 February, that year's rookies are also excluded.
- **A compensation player who refuses to move** becomes 임의해지 (voluntarily retired) for his old club and is banned for 3 seasons; the old club then takes the cash-only option (Art. 172(2)).
- **Order.** One club signing several FAs compensates in signing order; signings on the same day are compensated in reverse order of the previous season's standings. Until compensation is finished, the signing club may not release, voluntarily retire or trade players.
- **Worked examples** [press]:
  - 최원태, grade A, LG→Samsung: 8억 (200%) + player, or 12억 (300%), so his previous salary was 4억 ([Yonhap 2024-12-08](https://www.yna.co.kr/view/AKR20241208020100007)).
  - 박찬호, grade A, KIA→Doosan: 9억 + player, or 13억 5천만 ([Yonhap 2025-11-18](https://www.yna.co.kr/view/AKR20251118075851007)).

**Related limits** ([Bylaws](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]:
- A club may sign at most 1-4 FAs from other clubs, depending on how many applied (Art. 173; table in section 1).
- An FA signed from another club cannot be traded for 1 year (Art. 174).
- An FA and his club cannot use salary arbitration (Art. 171).
- Contracts that break the FA rules are void. Sanctions: the club loses its 1st-round pick for 3 years; the player loses that year's FA right and is 임의해지 for 1 year (Arts. 175-176).

---

## 4. Contracts (question 3)

### 4.1 Contract length and total value

- **No cap on length or value.** The 2026 bylaws have no limit on the length or the total value of domestic contracts. I searched the whole text for 상한, 총액 and 계약기간; the only caps are the competitive-balance tax and the foreign-player cost caps ([Bylaws](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].
- **Long deals seen recently**:
  - 오지환 (LG), FA, 6 years / 124억, Jan 2024 ([News1 via Naver 2024-01-26](https://m.sports.naver.com/kbaseball/article/421/0007316846)) [press]
  - 하영민 (Kiwoom), non-FA, 8 years / 80억, 13 Jul 2026 ([Sports Hankook 2026-07-13](https://sports.hankooki.com/news/articleView.html?idxno=6936358)) [press]
  - 양의지 (Doosan), FA, 4+2 years / 152억, Nov 2022 ([OSEN 2026-11-17](http://www.osen.co.kr/article/G1112911300)) [press]
- **Required form** ([Bylaws Arts. 37-39](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]:
  - Every contract uses the uniform contract form, and every side clause must be written into the filed contract.
  - Clubs may not pay a player's taxes.
  - An undisclosed deal costs the club its next 1st-round pick and a 10억 fine, and the player is suspended for 1 year.
- **Disclosure.** KBO's FA schedule requires that salary, signing bonus, options and length be announced when an FA contract is signed ([N12242 attachment 2](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242)) [official].

### 4.2 Pay structure

All from [Bylaws](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official].
- **Salary**: fixed in the contract, paid in 10 monthly instalments during the participation period, 1 Feb-30 Nov (Arts. 70, 72; Art. 69). Pay tied to gate receipts or game results is banned (Art. 70(2)).
- **Signing bonus (계약금)**:
  - Allowed only for rookies, free agents/FAs and players the bylaws specifically allow (Art. 81(1)); paid in 2 instalments (Art. 81(3); Contract form Art. 7(2)).
  - A released player who re-signs with the club that released him within 2 years cannot get one (Art. 81(2)).
  - Any other money or goods counts as a signing bonus (Art. 82). Breaking this costs the next 2nd-round pick and a 10억 fine (Art. 83).
- **Top-up for low earners**: a player earning under 6,500만 gets an extra (6,500만 − salary) ÷ 300 for each day on the first-team roster (Art. 73(1)). The threshold was 5,000만 until the board of 31 Jul 2024 ([N10160](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10160)) [official].
- **Cut for high earners**: a player earning 3억 or more who is off the first-team roster for poor form loses 50% of 1/300 of salary per day (batters from opening day; pitchers after the club's 7th game). Injury absences are not cut until rehab is over (Art. 73(2)).
- **Suspensions**: players suspended from games lose 50% of 1/300 per game day (Art. 32(2)). Restricted or suspended-status players lose 1/300 per day (Art. 36(2)).
- **Unsigned reserve players**: those not signed by 31 January get a reserve allowance of 25% of 1/300 of last salary per day. Military-reserve players get 25% of salary, capped at 1,200만 and paid over 10 months (Art. 62).
- **Trades**: salary is unchanged on a trade (Art. 74(1)). The transfer fee belongs to the club (Art. 74(2)).

### 4.3 Non-FA multi-year contracts

- **Since when**: KBO allowed multi-year contracts for all players, FAs included, in **July 2021** ([Nocut News 2021-12-14](https://nocutnews.co.kr/news/5673622)) [press]. I found no KBO press release on it (searched the KBO notice board for 다년, 다년 계약, 비FA).
- **First deals**: SSG on 14 Dec 2021 ([same article](https://nocutnews.co.kr/news/5673622)) [press]:
  - 박종훈: 5 years / 65억 (salary 56억 + options 9억)
  - 문승원: 5 years / 55억 (salary 47억 + options 8억)
- **Conditions in the 2026 bylaws** ([Bylaws](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]:
  1. The club files the contract the day after signing or announcing it, and KBO publishes it the next day; late filing can be sanctioned (Art. 44(3), 11 Jan 2024). Approval is possible after 1 February (Art. 44(2)).
  2. FA eligibility earned during the contract is deferred until it ends (Art. 162(6)).
  3. The player is left out of other players' FA-grade rankings, except in his final year (Art. 172; [N11353](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11353)).
  4. For contracts signed after 23 Sep 2025, the payroll-tax count uses (total salary + total bonus) ÷ years + options paid. This stops front- or back-loading from dodging the cap (Art. 184(1); [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687)).
- **Inference (not stated outright in any source)**: non-FA deals cannot include a signing bonus, because Art. 81 limits bonuses to rookies and free agents. Every non-FA deal I saw was salary + options: SSG 2021 above; 하영민, "연봉과 옵션 포함 총액 80억원" ([Sports Hankook](https://sports.hankooki.com/news/articleView.html?idxno=6936358)) [press].
- **Foreign players**: multi-year contracts only from the player's 2nd contract year ([Foreign-player rules Art. 8(5)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].

### 4.4 Minimum salary

| Period | Minimum | Source |
|---|---|---|
| 2005 | 2,000만 | [N11814](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11814) [official] |
| 2010 | 2,400만 | same |
| 2015 | 2,700만 | same |
| 2021-2026 | **3,000만** (the 2026 rookies' listed salaries are all 3,000만, [N11861 xlsx](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11861)) | [N11814](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11814); [N7651](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=7651); [2026 bylaws change table](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| From 2027 | **3,300만** (+10%) | [Bylaws Art. 71(1)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf), [N11814](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11814) [official] |

- Developmental players (육성선수) are exempt from the minimum ([Bylaws Art. 71(2)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].
- Players of Futures-only member clubs (Ulsan) are paid the bylaw minimum ([League regulations Art. 78](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_리그규정.pdf)) [official].

### 4.5 Limits on salary cuts

- **None.** The KBO board abolished the old cap on cuts (20-40% depending on salary) on 19 Feb 2008 ([Kyunghyang 2008-02-21](https://www.khan.co.kr/article/200802211753415)) [press].
- The 2026 bylaws have no limit on cutting the next year's salary. The only reductions are the in-season ones in 4.2 ([Bylaws Arts. 69-83](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].

### 4.6 Salary arbitration (연봉 중재)

The process was called 연봉조정 until 2025, when it was renamed 연봉 중재 ([StarNews 2026-01-19](https://www.starnewskorea.com/sports/2026/01/19/2026011909393212488)) [press]; the bylaws use 중재.

Rules ([Bylaws Arts. 75-80](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]:
- **Who**: a club or a reserved player may apply. Players with less than 3 full years since starting as KBO players cannot.
- **Timetable**: apply by 10 January, 18:00, stating the salary wanted; each side files its evidence within 5 days; the panel decides by 30 January (23 January from 2027, [N12184](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12184)).
- If only one side files evidence, its figure wins. If neither does, the case is withdrawn.
- **After the award** the contract must be signed within 10 days.
  - A player who refuses becomes 임의해지.
  - A club that refuses loses him to free agency, keeps paying 1/10 of the award per month (February-May only) until he signs elsewhere, and receives the award plus the amounts it paid as a transfer fee.
- **FA contracts** cannot go to arbitration (Art. 171).
- The panel is 5 people (lawyers or academics), one nominated by each side ([N7950](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=7950)) [official]. NOT CONFIRMED whether the bylaw forces a choice between the two figures (final-offer arbitration); the text does not say so.

**How often it is used**:
- 2021: one filing, 주권 (KT): club 2억 2천만 vs player 2억 5천만, salary in 2020 1억 5천만. The panel chose the **player's figure** on 25 Jan 2021 ([N7944](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=7944), [N7953](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=7953)) [official].
- History ([StarNews 2026-01-19](https://www.starnewskorea.com/sports/2026/01/19/2026011909393212488)) [press]:
  - 99 filings since 1982, 21 hearings, 2 player wins (류지현 2002, 주권 2021), 19 club wins.
  - **No filings at all in 2022-2026** (5 years running). In 2026 the deadline moved to Monday 12 January because 10 January was a Saturday, and no application came in.
  - Hearings by decade: 1980s 8, 1990s 5, 2000s 5, 2010s 2, 2020s 1.
  - The hearing before 주권's was 이대호's in 2011, 10 years earlier ([Sports Kyunghyang 2021-01-24](https://sports.khan.co.kr/article/202101241910003)) [press].

### 4.7 Opt-outs and "self-release" clauses

- **The 김재환 case** ([Sports Kyunghyang 2025-12-17](https://sports.khan.co.kr/article/202512170001003); [SPOTV 2025-12](https://www.spotvnews.co.kr/news/articleView.html?idxno=789038)) [press]:
  - His 2021 FA deal with Doosan said that if he did not file for FA after 4 years and the two sides could not agree, Doosan would leave him off the reserve list.
  - He did not file, Doosan released him (movement data: 자유계약선수, 30 Nov 2025, [official]), and SSG signed him for 2 years / 22억 (16 guaranteed + 6 incentives) **without paying grade-B compensation**.
- **Opt-outs**: 홍건희 (Doosan) also used an opt-out to leave the remaining 2 years / 15억 of his deal (same SPOTV article) [press].
- **Proposed fix**: KBO and the clubs agreed to ban contracts that defeat FA compensation (the "김재환 룰") ([Sports Kyunghyang 2025-12-17](https://sports.khan.co.kr/article/202512170001003)) [press].
  - It was **not adopted** at the January 2026 board; KBO expected it in the first half of 2026, applying after the 2026 season ([Sports Kyunghyang 2026-01-29](https://sports.khan.co.kr/article/202601291033003)) [press].
  - It does not appear in the April 2026 bylaws or in any KBO release up to N12243 (16 Nov 2026). Adoption is **NOT CONFIRMED**.

---

## 5. Payroll limits: 경쟁균형세 (competitive-balance tax), formerly 샐러리캡 (question 4)

### 5.1 Cap by year

| Year | Cap (top-40 payroll per club) | Basis | Source |
|---|---|---|---|
| 2023 | 114억 2,638만 | 120% of the 2021-22 average club top-40 total (95억 2,199만) | [N8558](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8558), [Bylaws Art. 184](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| 2024 | 114억 2,638만 | same | [N10338](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10338) [official] |
| 2025 | 137억 1,165만 | +20%, board of 31 Jul 2024; renamed 경쟁균형세 (tax) and the "fine" renamed "baseball development fund" payment | [N10160](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10160) [official] |
| 2026 | 143억 9,723만 | +5% | [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687), [Bylaws Art. 184(3)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| 2027 | **151억 1,709만** | +5% | same |
| 2028 | 158억 7,294만 | +5% | same |

- The 2021-22 base figures ([N8558](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8558)) [official]:
  - Club average 81억 7,616만 (2021) and 108억 6,782만 (2022).
  - Highest: SSG 2022, 248억 7,512만. Lowest: Hanwha 2021, 41억 5,241만.
- Check [data]: (81.7616 + 108.6782) ÷ 2 × 1.2 = 114.264억; 137.1165 × 1.05^n gives the 2026-2028 figures.

### 5.2 How payroll is counted

Sources: [Bylaws Arts. 184-186](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official]; [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687) [official].
- **Who counts**: the club's **top 40** players by pay, leaving out foreign players and rookies.
- **What counts**: salary + options actually paid + FA signing bonus spread evenly over the contract years.
- **Which figures**: contracts filed by 31 January. Players traded or added in season count at the amount actually paid.
- **Multi-year deals signed after 23 Sep 2025**: (total salary + total bonus) ÷ years + options.
- **Exception player (예외 선수)**, introduced 23 Sep 2025:
  - Each club names one player with **7+ seasons registered with that club** (military-reserve time counts; developmental time and time at other clubs or abroad do not).
  - **50%** of his pay is left out.
  - He is named by the contract deadline and can be re-named within 3 business days if he moves.
- **Process**:
  - Clubs report bonuses, salaries and options by 1 December.
  - KBO notifies any overage by 15 December.
  - Payment is due by 31 January.
  - Withholding receipts are filed by early March.
  - The money goes to youth and amateur baseball.

### 5.3 Penalties for exceeding the cap

| Overage | 2023 to Sep 2025 ([N8558](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8558)) | From 23 Sep 2025 ([N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687); [Bylaws Art. 186(2)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) |
|---|---|---|
| 1st time | 50% of the excess | **30%** |
| 2 years in a row | 100% + next 1st-round pick moved down 9 places | **50%**, no pick penalty |
| 3+ years in a row | 150% + 1st-round pick down 9 places | **100% + 1st-round pick down 9 places** |
| Payments under-reported | - | 100% of the hidden amount; side deals punished under Art. 39 |

All [official].

### 5.4 Club top-40 totals, 2023-2025

Sources: [N9920](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9920) (2023), [N10338](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10338) (2024), [N11791](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11791) (2025) [official]. Values in 억원.

| Club | 2023 | 2024 | 2025 |
|---|---|---|---|
| Samsung | 104.4073 | 111.8100 | 132.0700 |
| LG | 107.9750 | **138.5616 (over by 24.2978; paid 12.1489 = 50%)** | 131.5486 |
| SSG | 108.4647 | 104.5700 | 131.1300 |
| Hanwha | 85.3100 | 107.1046 | 126.5346 |
| KIA | 98.7771 | 112.4900 | 123.0265 |
| Lotte | 106.4667 | 111.5018 | 122.1100 |
| Doosan | 111.8175 | 111.9436 | 105.5154 |
| KT | 94.8300 | 105.1641 | 105.1093 |
| NC | 100.8812 | 94.7275 | 89.4777 |
| Kiwoom | 64.5200 | 56.7876 | 43.9756 |
| Cap | 114.2638 | 114.2638 | 137.1165 |

- Only LG in 2024 exceeded the cap.
- KBO states the 2025 average per player as 2억 7,762만 [official]. My check: sum 1,110.4977억 ÷ 400 = 2.7762억 [data].

### 5.5 Payroll floor (하한액)

Sources: [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687) [official]; [Bylaws Arts. 187-188](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official].
- **Amount: 60억 6,538만**, the average of the lowest club's top-40 total in 2023 and 2024. My check: Kiwoom (64.5200 + 56.7876) ÷ 2 = 60.6538억 [data].
- **Starts in 2027** and rises **5% a year**.
- **Penalties**: 30% of the shortfall the first time, 50% two years running, 100% three or more years running, paid to the youth development fund.
- **2028 and 2029 values**: my arithmetic gives 63억 6,865만 (2028) and 66억 8,708만 (2029) [data]. Sports Seoul gives the same 2029 figure, reading 60억 6,538만 as the 2027 value ([2025-09-26](https://www.sportsseoul.com/news/read/1549182)) [press]. The bylaw itself prints only the formula, no figure.
- **Kiwoom's position**: its 2025 total was 16억 6,782만 below the 2027 floor ([OSEN 2025-12-19](http://www.osen.co.kr/article/G1112715334)) [press]; my arithmetic agrees. In July 2026 it signed 하영민 to a non-FA deal of 8 years / 80억, which the press linked to the floor ([Sports Hankook](https://sports.hankooki.com/news/articleView.html?idxno=6936358)) [press].

---

## 6. Player movement (question 5)

### 6.1 Rosters and the reserve list

Sources: [Bylaws Arts. 24, 56-63](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official]; 2027 dates from [N12184](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12184) [official].
- **Size limits**: a club may hold at most **68** players, reserved players included (65 until Jan 2026). The **reserve list** (보류선수) holds at most **63**; military-reserve and developmental-reserve players are not counted.
- **Dates**: the list is due **25 November** (18 November from 2027) and published **30 November**.
- **Reserve period**: runs from publication to 31 January of the year after next. A player still unsigned then becomes 임의해지.
- **Players left off the list**:
  - Their contracts end on 30 November and they become free agents.
  - The old club may not register them again for 1 year.
  - Another club that signs them must register them by 31 January.
- **Reserve allowance**: a club that fails to pay it within 14 days loses its reserve rights, and the player can ask to be declared a free agent (Art. 63).
- **Contracts for reserved players**: filed for approval 1 December-31 January (from 2027, by 24 January) (Art. 44(1)).

### 6.2 Releases and waivers

Sources: [Bylaws Arts. 30, 93-103](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official].
- **Release (자유계약선수 공시)**: the commissioner declares a player a free agent at the club's or player's request when the contract has been ended. A free agent may sign with any club (Art. 30).
- **During the participation period** a club wanting to drop a player (for club reasons or injury) **must put him on waivers**. Waiver requests are accepted **until the last day of the regular season**. A player claimed off waivers after 1 August cannot play in the postseason (Arts. 93-94).
- **Claims** are made within 7 days of the waiver notice. Priority goes in reverse order of the current standings; outside the season, reverse order of the previous final standings (Art. 95).
- **Claim fee** 300만, paid at once (Art. 100). The claiming club cannot waive the player again for 60 days (Art. 102).
- **Unclaimed** players become free agents on the 8th day but **cannot sign with any club that year** (Art. 99).
- A player may **refuse waivers** within 7 days and becomes 임의해지 (Art. 98).
- **After the season** releases happen by leaving players off the reserve list (6.1).

### 6.3 Voluntary retirement (임의해지)

Sources: [Bylaws Arts. 31, 66, 68, 79, 98, 172(2)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official]; called 임의탈퇴 until the October 2021 revision.
- **How a player becomes 임의해지**:
  - he asks to end the contract and the club agrees;
  - he is judged not to want a new contract;
  - the reserve period runs out;
  - he refuses waivers, an arbitration award or a move as FA compensation.
- He cannot train with the team; a breach costs the club 1,000만 and bars the player for 2 years.
- He may apply to return **1 year** after the notice and must return to the same club. The status can be cancelled (making him a free agent), but he still cannot rejoin the old club before the year is up.
- A compensation player who refuses to move is banned for 3 seasons.
- Foreign players can be made 임의해지 during the season without being paid the rest of their salary ([Foreign-player rules Art. 8(7)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].

### 6.4 Developmental players (육성선수)

Sources: [Bylaws Arts. 71(2), 84(1), 116-117](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official].
- Signed outside the draft and **not counted in the 68**.
- **High-school limit**: from each year's high-school graduates (drafted or not), a club may take at most **5**, or **3** if it drafted 7 or fewer players the year before.
- They can be added to the registered roster **from 1 May**. They are exempt from the minimum salary and can be traded.
- **Cannot become developmental players** that year:
  - unclaimed in-season waiver players;
  - draftees who refused to sign;
  - eligible players who did not enter the draft;
  - draft entrants who failed in talks with a foreign club;
  - 2nd-year university early-draft entrants who were not picked.
- Game rules (Futures and exhibition games only) are in kbo_notes.md.

### 6.5 Trades

Sources: [Bylaws Arts. 84-92, 115, 174, 181(3)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official].
- **When**: from the day after the postseason ends to **31 July**, with publication by 1 August.
- **Who**: registered and developmental players, including those in military service, after consulting the player.
- **What can be included**:
  - Up to **2 picks** in the next draft can be exchanged for players; the club receiving a pick cannot trade the resulting player for 1 year.
  - Cash is allowed but every payment must appear in the filed transfer agreement. A hidden payment voids the deal and costs both clubs their 1st-round pick and a 10억 fine each (Art. 88-2).
  - Loans and conditional (recall) deals are banned.
- **Moving allowance**: each club pays the player **100만**, 200만 in total; raised in July 2024 ([N10160](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10160)) [official].
- **Players who cannot be traded for a while**:
  - new draftees, for 1 year (player-for-player swaps excepted);
  - FAs signed from other clubs, for 1 year;
  - secondary-draft picks, for 1 year, unless the roster obligation is met or they were taken in round 3 or later.
- **Example of pick + cash**: Hanwha gave NC a 2026 3rd-round pick and 3억 for 손아섭 in July 2025 ([OSEN 2026-01-31](http://www.osen.co.kr/article/G1112737337)) [press].

### 6.6 Secondary draft (2차 드래프트)

Sources: [Bylaws Arts. 177-182](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf); [N8750](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8750); [N11438](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11438); [N11771](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11771) [official].

| Item | Rule |
|---|---|
| History | Held every other year 2011-2019 (5 times); abolished in 2021 and replaced by the "Futures FA" in the 2021-22 and 2022-23 offseasons; revived in July 2023 because the Futures FA moved few players |
| When | Every other year from 2023, after the regular season. Held 22 Nov 2023 ([N9894](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9894)) and 19 Nov 2025 ([N11771](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11771)). By that pattern the next is 2027 (not announced). |
| Timeline | Clubs file all players 5 days after the season ends; protected lists are due 10 days before the draft; status changes are frozen from the day after the season until the draft ends |
| Protection | **35 players** per club (40 before the revival) |
| Automatically exempt | Players in their 1st-3rd year (registered, developmental or in military service); 4th-year players with military history (from 2025); this year's FAs, including returnees from abroad; foreign players; protected players who left as FA compensation before the draft |
| Picks | Reverse order of standings; 3 rounds; the bottom 3 clubs get up to 2 extra picks; at most 4 picks from any one club; a club may pass and still pick in later rounds |
| Payment to the old club | 4억 (round 1), 3억 (round 2), 2억 (round 3), 1억 (round 4 and later), within 30 days of the invoice. In 2019 it was 3억 / 2억 / 1억. |
| Roster obligation | Next or following season: round 1 at least 50 days on the first-team roster (injured-list days count), round 2 at least 30 days, round 3+ none. A season with 30+ days on the injured or rehab list is excused. If the obligation is missed, the player returns to the old club after the 2nd season with a 50% refund, or becomes a free agent if the old club declines. |
| Picked player refuses | Banned from all clubs for 2 years; no fee is paid |
| Results | 2023: 22 players ([N9894](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9894)); 2025: 17 players ([N11771](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11771)). Movement data agrees: 22 and 17 records. |

**Futures FA (2021-22 and 2022-23 offseasons only)** ([N8179](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8179)) [official]:
- **Eligible**: players with 7+ seasons of 60 or fewer first-team days.
- **Signing club**: may sign up to 3, pays 100% of the player's last salary as compensation, must register him, may pay no signing bonus and no more than his last salary.
- **Unsigned** until the next Korean Series ends: he becomes a free agent.

### 6.7 Movement counts by calendar year

Tally of the [movement data](https://www.koreabaseball.com/Player/Trade.aspx) [data]. One row per player record (a two-player trade counts 2). 2026 is up to 17 Nov 2026.

| Category | 2021 | 2022 | 2023 | 2024 | 2025 | 2026 |
|---|---|---|---|---|---|---|
| 자유계약선수 (released) | 120 | 110 | 116 | 106 | 113 | 92 |
| of which on the reserve-list date (30 Nov-2 Dec) | 44 | 57 | 58 | 57 | 53 | - |
| of which on 31 January | 18 | 23 | 32 | 31 | 41 | 51 |
| 웨이버 (waivers) | 56 | 30 | 25 | 22 | 12 | 33 |
| 임의해지 | 3 | 10 | 9 | 7 | 12 | 8 |
| 트레이드 (traded players) | 23 | 23 | 21 | 19 | 17 | 6 |
| 군보류 자유계약 (released while in military service) | 14 | 22 | 20 | 33 | 42 | 17 |

- The bulk releases each **31 January** are probably players moved from registered to developmental contracts; NOT CONFIRMED, the data gives no reason.

---

## 7. Foreign players (question 6): confirmations and corrections to our notes

All from the [Foreign-player rules in the 2026 bylaws](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) and [Bylaws Art. 29](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official].

| Rule | Text in force | Our notes |
|---|---|---|
| Players under contract | At most **3**, not all registered at the same position. Injury replacements may push it above 3 for a while. Plus **1 Asia-quota player** (Art. 3; Bylaws Art. 29(2)-(3)). | Confirms both files. The position rule that nationality.md could not find is Art. 3 ("3명 전원이 동일 포지션으로 등록할 수 없다"). |
| In one game | 3, or 4 with the Asia-quota player (Art. 4) | Confirms |
| Cost cap, new signing | **US$1,000,000**: salary + signing bonus + options actually paid + transfer fee, tax excluded. Applies to returnees whose reserve rights had lapsed (Art. 8(1)). | Confirms nationality.md (which had press only) |
| Team cap | Since 2023 up to **US$4,000,000** for up to 3 players, **+US$100,000 per re-signing year** of re-signed players (Art. 8(1)) | Confirms nationality.md |
| Asia quota cost | Up to **US$200,000** (same items), **+US$100,000 a year** on re-signing; separate from the foreign cap (Art. 8(1)) | Confirms |
| Signings after 1 Feb, or new players after opening day | Up to **US$100,000 per remaining month** (Asia quota **US$20,000** per remaining month) (Art. 8(2)) | Clarifies nationality.md: the "$20,000 per month" applies to in-season / post-1-February signings, as kbo_notes.md says |
| Re-signing a player the club holds reserve rights to | Art. 8(3): paragraphs (1) and (2) "do not apply". Read literally this also lifts the team cap, although (1) builds re-signings into that cap; I report the wording as printed. | New |
| Penalty for exceeding the cost cap | 50% of the excess the first time; 100% + 2nd-round pick down 9 places twice in a row; 150% + 2nd-round pick down 9 places three or more times in a row (Art. 8(1)) | New |
| Multi-year deals | From the 2nd contract year only (Art. 8(5)) | New |
| Release / 임의해지 in season | Allowed; no further salary on 임의해지 (Art. 8(7)) | New |
| Replacements | **2** extra registrations a season for regular foreign players, **1** for the Asia-quota player; signing one through a trade uses a registration (unless both are unused, foreign-for-foreign trades until 30 June are free); players registered from **16 August** cannot play in the postseason (Art. 9) | Confirms kbo_notes.md |
| Injury replacement | Original player out **6+ weeks**; once per player; cost as in Art. 8(2); the club must then swap back (using a registration) or release (Art. 10) | Confirms |
| Intent to re-sign | Written notice by **25 November** (or the day after the Korean Series if later), reported to KBO within 24 hours (Art. 11) | New |
| Breaches | Contract void, player's registration barred 5 years, club barred from adding foreign players that year (Art. 12) | New |
| Who counts as foreign | No Korean nationality when the contract is signed, except foreign nationals who attended a Korean middle school or higher and were KBSA-registered; they go through the draft (Bylaws Art. 29(1); Foreign-player rules Art. 2) | New |

- **2027 proposals**: the holding limit is "to be eased in stages" ([N12184](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12184)) [official]; no decision. This matches the two notes files.

---

## 8. Moving abroad (question 7)

### 8.1 Posting to MLB

**KBO side** ([Bylaws Art. 104](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]:
- A club may transfer a player who has been active for **7 or more KBO regular seasons** since his first registration to a foreign professional club, with the commissioner's prior approval.
- The **7 seasons** are counted like FA seasons (Art. 162(2)).
- **At most 1 player per club per year.**

**KBO-MLB agreement** (text in the [2026 bylaws PDF](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf), Korean and English, signed 14 Jun 2018; it ran to 31 Oct 2021 and renews yearly unless either side gives notice 180 days ahead) [official]:
- **Club consent**:
  - A player on a KBO club's reserve, voluntarily-retired, restricted, disqualified, military, suspended or ineligible list can be approached only with his club's approval, given through KBO, and only by posting.
  - The club may not release him early so that he can sign in MLB (paragraphs 5-7).
  - Players on none of those lists may sign with MLB clubs freely, with no fee.
- **Window**: posting requests 1 November-5 December, with the full medical file. MLB clubs then have **30 days**, from 08:00 ET the day after the notice to 17:00 ET on day 30 (paragraphs 7-8).

**Release fee for a major-league contract** (paragraph 9), on the total guaranteed value (TGV: guaranteed years incl. player options, signing bonus, buyout of the first club option):

| Part of the guaranteed value | Fee rate |
|---|---|
| Up to $25,000,000 | 20% |
| $25,000,001-$50,000,000 | 17.5% |
| Above $50,000,000 | 15% |

- Example in the text: TGV $100,000,000 gives a fee of $16,875,000.
- **Supplemental fee**: 15% of bonuses and escalators actually earned and of salary in exercised club, vesting or mutual option years.
- **Payment schedule**: 50% within 14 days of the contract being confirmed; 25% within 12 months; 25% within 18 months. The KBO club releases the player within 5 business days of the first payment. No negotiation of the fee is allowed.
- **Minor-league contract**: fee of 25% of the signing bonus, plus 15% of major-league earnings if the contract has major-league terms (paragraph 10).
- **No deal by the deadline**: the player cannot be posted again until 1 November of the following year (paragraph 12; repeated in KBO's posting releases, e.g. [N12223](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12223)).
- **Recent posting requests** [official]: 이정후 and 고우석 (Nov 2023, [N9898](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9898), [N9901](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9901)); 김혜성 (Dec 2024, [N10320](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10320)); 송성문 (Nov 2025, [N11773](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11773)); 노시환 (31 Oct 2026, [N12223](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12223)).

### 8.2 Overseas FA and other rules

- **Overseas FA**: the same seasons as domestic FA, **8 for high-school entrants and 7 for four-year college graduates since the end of 2022**. The old rule making college graduates wait 9 seasons to go abroad was deleted then ([Bylaws Art. 162(1),(5)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]. An approved FA may negotiate with foreign clubs from the day after approval, following the agreements (Art. 168).
- **Returning FAs** ([Bylaws Arts. 164, 170, 172(1)4](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official]:
  - Compensation follows the grade he had when he left.
  - Re-FA comes 4 seasons after returning.
  - Clubs may pay him a signing bonus and offer multi-year deals.
- **Japan and Taiwan**: these agreements only require the club's approval through the commissioners for reserved players. They set no posting fee or window ([한·일 and 한·대만 agreements in the bylaws PDF](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf)) [official].
- **Amateurs who sign abroad directly** cannot sign with a KBO club for 2 years after that contract ends (7 years for coaching jobs) and must enter the draft. From January 2026 this applies from middle school, not high school ([Bylaws Art. 107](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf); [N11814](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11814)) [official].

---

## 9. Recent markets (question 8)

### 9.1 FA counts by offseason

Sources: eligible, applied and grades from the KBO releases in the first column [official]; outcomes from the [movement data](https://www.koreabaseball.com/Player/Trade.aspx) checked name by name against the approved lists [data].

| Offseason (KBO name) | Eligible (new / re-FA / kept eligibility) | Grades of eligible A/B/C | Applied (approved) | Grades of applied A/B/C | Did not apply | Deferred (multi-year) | Outside FAs allowed per club | Re-signed | Moved | Unsigned / retired |
|---|---|---|---|---|---|---|---|---|---|---|
| 2021-22 ("2022 FA", [N8215](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8215), [N8218](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8218)) | 19 (12/5/2) | 5/9/5 | 14 | 4/6/4 [data, from the list] | 5 | - | 2 | 8 | 6 | 0 |
| 2022-23 ([N8556](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8556), [N8562](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8562)) | **40** (29/7/4) | 11/14/15 | 21 | NOT CONFIRMED (attachment is .doc) | 19 | - | 3 | 9 | 11 | 1 unsigned (강윤구) |
| 2023-24 ([N9883](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9883), [N9887](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9887)) | 34 (12/8/14) | 8/14/12 | 19 | 3/9/7 | 15 | not published | 2 | 16 | 3 | 0 |
| 2024-25 ([N10294](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10294), [N10297](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10297)) | 30 (13/9/8) | 3/15/12 | 20 | 3/9/8 | 10 | 11 | 2 | 13 | 6 | 1 unsigned (문성현) |
| 2025-26 ([N11753](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11753), [N11762](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11762)) | 30 (13/11/6) | 7/13/10 | 21 | 6/8/7 | 9 | 14 | 3 | 13 | 7 | 1 retired (황재균) |
| 2026-27 (in progress, [N12242](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242), [N12243](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12243)) | 28 (15/6/7) | 6/14/8 | 22 | 6/10/6 | 6 | 9 | 3 | (2 so far) | - | - |

- **The 2022-23 spike** (40 eligible, 29 first-time) is the year the shorter 8/7-season rule took effect (section 2.2).
- **Outcomes by player** [data]:
  - **2021-22**:
    - Re-signed: 장성우, 황재균 (KT); 김재환 (Doosan); 백정현, 강민호 (Samsung); 김현수 (LG); 정훈 (Lotte); 최재훈 (Hanwha).
    - Moved: 허도환 KT→LG; 박건우 Doosan→NC; 박해민 Samsung→LG; 박병호 Kiwoom→KT; 나성범 NC→KIA; 손아섭 Lotte→NC.
    - Also 양현종 returned from MLB as an FA and signed with KIA (26 Dec 2021); he is not counted above.
  - **2022-23**:
    - Moved: 원종현 NC→Kiwoom; 박동원 KIA→LG; 유강남 LG→Lotte; 양의지 NC→Doosan; 채은성 LG→Hanwha; 노진혁 NC→Lotte; 이태양 SSG→Hanwha; 김상수 Samsung→KT; 박세혁 Doosan→NC; 오선진 Samsung→Hanwha; 한현희 Kiwoom→Lotte.
    - Re-signed: 장시환, 박민우, 김진성, 이재학, 오태곤, 신본기, 이명기 (sign-and-trade), 권희동, 정찬헌 (on 30 Mar 2023).
  - **2023-24**:
    - Moved: 김재윤 KT→Samsung; 안치홍 Lotte→Hanwha; 임창민 Kiwoom→Samsung.
    - The other 16 re-signed, 김민성 and 이지영 by sign-and-trade.
  - 2024-25 and 2025-26: see 9.2.
- **Compensation players** per offseason ([movement data](https://www.koreabaseball.com/Player/Trade.aspx), category "FA 보상선수") [data]: 2021-22: 4; 2022-23: 8; 2023-24: 1; 2024-25: 5; 2025-26: 4. All other moves were settled in cash: grade C, 35+, 3rd FA, or the old club took the cash-only option.

### 9.2 FA signings, 2024-25 and 2025-26

**How these tables were built** [data]:
- **Player, grade, type, birth date, clubs, date**:
  - Name, grade, type (new / re-FA / kept eligibility) and birth date come from KBO's approved lists ([N10297](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10297), [N11762](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11762)) [official].
  - Old club, new club and the KBO publication date come from the [movement data](https://www.koreabaseball.com/Player/Trade.aspx) [official].
  - **Age** is age in completed years on the KBO publication date [data].
- **Contract terms**: years ("2+1" = 2 years + 1 option year) and **maximum total** including options/incentives come from the press source in the last column [press]. "Bonus" = signing bonus (계약금); salary is the total over the contract.
- **Completeness**: every approved FA is listed, including the unsigned and retired ones. Every total is filled in; the breakdown is missing for some.

**2024-25 (2025 FA): 20 approved, 19 signed, 6 moved**

| Player | Pos | Age | Grade | Type | From → To | KBO date | Years | Max total (억) | Breakdown (억) | Source |
|---|---|---|---|---|---|---|---|---|---|---|
| 최정 | IF | 37 | C | 3rd FA | SSG (stayed) | 2024-11-09 | 4 | 110 | bonus 30 + salary 80, all guaranteed | [Yonhap 2024-11-06](https://www.yna.co.kr/view/AKR20241104080551007) |
| 심우준 | IF | 29 | B | new (kept eligibility) | KT → Hanwha | 2024-11-09 | 4 | 50 | guaranteed 42 + options 8 | [Xportsnews 2024-11-10](https://www.xportsnews.com/article/1927522) |
| 우규민 | P | 39 | C | 3rd FA | KT (stayed) | 2024-11-10 | 2 | 7 | bonus 2 + salary 4 + options 1 | [Xportsnews](https://www.xportsnews.com/article/1927522), [Yonhap 2024-11-06](https://www.yna.co.kr/view/AKR20241106112700007) |
| 허경민 | IF | 34 | B | 2nd FA | Doosan → KT | 2024-11-10 | 4 | 40 | bonus 16 + salary 18 + options 6 | [Xportsnews](https://www.xportsnews.com/article/1927522) |
| 엄상백 | P | 28 | B | new | KT → Hanwha | 2024-11-10 | 4 | 78 | bonus 34 + salary 32.5 + options 11.5 | [Xportsnews](https://www.xportsnews.com/article/1927522) |
| 장현식 | P | 29 | B | new | KIA → LG | 2024-11-13 | 4 | 52 | bonus 16 + salary 36, all guaranteed | [News1 via Naver 2024-11-11](https://m.sports.naver.com/kbaseball/article/421/0007900012) |
| 김원중 | P | 31 | A | new | Lotte (stayed) | 2024-11-18 | 4 | 54 | guaranteed 44 + incentives 10 | [Xportsnews](https://www.xportsnews.com/article/1927522) |
| 구승민 | P | 34 | A | new (college) | Lotte (stayed) | 2024-11-18 | 2+2 | 21 | - | [Sports Chosun 2024-11-18](https://www.sportschosun.com/baseball/2024-11-18/202411180100118070018448) |
| 노경은 | P | 40 | B | 2nd FA | SSG (stayed) | 2024-11-22 | 2+1 | 25 | bonus 3 + salary 13 + options 9 | [SBS 2024-11-22](https://news.sbs.co.kr/news/endPage.do?news_id=N1007883489) |
| 김헌곤 | OF | 36 | C | new, 35+ (kept eligibility, college) | Samsung (stayed) | 2024-11-28 | 2 | 6 | - | [Hankyoreh 2024-11-25](https://www.hani.co.kr/arti/sports/baseball/1169106.html) |
| 임정호 | P | 34 | C | new (college) | NC (stayed) | 2024-12-05 | 3 | 12 | - | [Xportsnews via Naver 2024-12-06](https://m.sports.naver.com/kbaseball/article/311/0001804057) |
| 최원태 | P | 27 | A | new | LG → Samsung | 2024-12-08 | 4 | 70 | - | [Yonhap 2024-12-08](https://www.yna.co.kr/view/AKR20241208020100007) |
| 김강률 | P | 36 | C | new, 35+ | Doosan → LG | 2024-12-15 | 3+1 | 14 | bonus 5 + salary up to 9 | [Yonhap 2024-12-13](https://www.yna.co.kr/view/AKR20241213052900007) |
| 류지혁 | IF | 30 | B | new | Samsung (stayed) | 2024-12-20 | 4 | 26 | incl. incentives 6 | [YTN 2024-12-16](https://www.ytn.co.kr/_ln/0107_202412161044230158) |
| 임기영 | P | 31 | B | new | KIA (stayed) | 2024-12-30 | 3 | 15 | - | [Kukmin Ilbo 2024-12-31](https://www.kmib.co.kr/article/view.asp?arcid=0024310931&code=61161211&cp=nv) |
| 서건창 | IF | 35 | C | new, 35+ (kept eligibility) | KIA (stayed) | 2025-01-13 | 1+1 | 5 | incl. options | [SBS 2025-01-09](https://news.sbs.co.kr/news/endPage.do?news_id=N1007941774) |
| 하주석 | IF | 30 | B | new | Hanwha (stayed) | 2025-01-13 | 1 | 1.1 | guaranteed 0.9 + options 0.2 | [Yonhap 2025-01-08](https://www.yna.co.kr/view/AKR20250108133100007) |
| 김성욱 | OF | 31 | C | new | NC (stayed) | 2025-01-17 | 2 | 3 | - | [Newsis 2025-01-16](https://www.newsis.com/view/NISX20250116_0003034603) |
| 이용찬 | P | 36 | B | 2nd FA | NC (stayed) | 2025-02-05 | 2+1 | 10 | - | [OSEN 2025-02-04](http://www.osen.co.kr/article/G1112498036) |
| 문성현 | P | 33 at end of season (b. 1991-11-09) | C | new | Kiwoom → unsigned | - | - | - | retired 5 May 2026 | [SPOTV 2026-05](https://www.spotvnews.co.kr/news/articleView.html?idxno=814811) |

- Total of the announced maximums: **599.1억** for 19 contracts [data].

**2025-26 (2026 FA): 21 approved, 20 signed, 7 moved**

| Player | Pos | Age | Grade | Type | From → To | KBO date | Years | Max total (억) | Breakdown (억) | Source |
|---|---|---|---|---|---|---|---|---|---|---|
| 박찬호 | IF | 30 | A | new | KIA → Doosan | 2025-11-20 | 4 | 80 | bonus 50 + salary 28 + incentives 2 | [Yonhap 2025-11-18](https://www.yna.co.kr/view/AKR20251118075851007) |
| 조수행 | OF | 32 | B | new (college) | Doosan (stayed) | 2025-11-20 | 4 | 16 | bonus 6 + salary 8 + options 2 | [Xportsnews 2025-11-28](https://www.xportsnews.com/article/2083595) |
| 강백호 | C | 26 | A | new | KT → Hanwha | 2025-11-22 | 4 | 100 | bonus 50 + salary 30 + options 20 | [Yonhap 2025-11-21](https://www.yna.co.kr/view/PYH20251120147300007) |
| 한승택 | C | 31 | C | new | KIA → KT | 2025-11-22 | 4 | 10 | bonus 2 + salary 6 + incentives 2 | [Newsis 2025-11-20](https://www.newsis.com/view/NISX20251120_0003411025) |
| 박해민 | OF | 35 | B | 2nd FA | LG (stayed) | 2025-11-26 | 4 | 65 | bonus 35 + salary 25 + incentives 5 | [Yonhap via Naver 2025-11-21](https://m.sports.naver.com/kbaseball/article/001/0015756480) |
| 이준영 | P | 33 | B | new (college) | KIA (stayed) | 2025-11-26 | 3 | 12 | bonus 3 + salary 6 + options 3 | [Xportsnews](https://www.xportsnews.com/article/2083595) |
| 김현수 | OF | 37 | C | 3rd FA | LG → KT | 2025-11-27 | 3 | 50 | bonus 30 + salary 20, all guaranteed | [Xportsnews](https://www.xportsnews.com/article/2083595); [Yonhap 2025-11-28](https://www.yna.co.kr/view/AKR20251128081900007) |
| 최원준 | OF | 28 | A | new | NC → KT | 2025-11-27 | 4 | 48 | bonus 22 + salary 20 + options 6 | [Xportsnews](https://www.xportsnews.com/article/2083595) |
| 이영하 | P | 28 | B | new | Doosan (stayed) | 2025-11-30 | 4 | 52 | bonus 23 + salary 23 + incentives 6 | [Newsis 2025-11-27](https://www.newsis.com/view/NISX20251127_0003419245) |
| 최원준 | P | 30 | A | new (college) | Doosan (stayed) | 2025-11-30 | 4 | 38 | bonus 18 + salary 16 + incentives 4 | [Yonhap 2025-11-28](https://www.yna.co.kr/view/AKR20251128078300007) |
| 양현종 | P | 37 | C | 3rd FA | KIA (stayed) | 2025-12-05 | 2+1 | 45 | bonus 10 + salary and incentives | [Yonhap 2025-12-04](https://www.yna.co.kr/view/AKR20251204078600007) |
| 최형우 | OF | 41 | C | 3rd FA | KIA → Samsung | 2025-12-05 | 2 | 26 | - | [Chosun via Naver 2025-12-04](https://m.sports.naver.com/kbaseball/article/023/0003944812) |
| 강민호 | C | 40 | C | 4th FA | Samsung (stayed) | 2026-01-08 | 2 | 20 | - | [Xportsnews 2026-01-08](https://www.xportsnews.com/article/2097890); [Maeil 2026-01-06](https://www.imaeil.com/page/view/2026010613592937250) |
| 김태훈 | P | 33 | A | new | Samsung (stayed) | 2026-01-08 | 3+1 | 20 | - | [Xportsnews](https://www.xportsnews.com/article/2097890) |
| 이승현 | P | 34 | B | new | Samsung (stayed) | 2026-01-08 | 2 | 6 | - | [Xportsnews](https://www.xportsnews.com/article/2097890) |
| 김상수 | P | 38 | B | 2nd FA | Lotte (stayed) | 2026-01-14 | 1 | 3 | - | [Yonhap 2026-01-08](https://www.yna.co.kr/view/AKR20260108113600007) |
| 김범수 | P | 30 | B | new | Hanwha → KIA | 2026-01-23 | 3 | 20 | - | [Newsis 2026-01-23](https://www.newsis.com/view/NISX20260123_0003488539) |
| 조상우 | P | 31 | A | new | KIA (stayed) | 2026-01-23 | 2 | 15 | - | [Kukmin Ilbo 2026-01-21](https://www.kmib.co.kr/article/view.asp?arcid=0029281305&code=61161211&cp=nv) |
| 장성우 | C | 36 | B | 2nd FA | KT (stayed) | 2026-01-23 | 2 | 16 | bonus 8 + salary 6 + incentives 2 | [Yonhap 2026-01-20](https://www.yna.co.kr/view/AKR20260120121600007) |
| 손아섭 | OF | 37 | C | 3rd FA | Hanwha (stayed) | 2026-02-09 | 1 | 1 | salary 1 | [Newsis 2026-02-05](https://www.newsis.com/view/NISX20260205_0003504439) |
| 황재균 | IF | 38 | C | 3rd FA | KT → retired | - | - | - | retired 19 Dec 2025 after a KT offer | [Sports Kyunghyang 2025-12-19](https://sports.khan.co.kr/article/202512191408003) |

- Total of the announced maximums: **643.0억** for 20 contracts [data].
- **Outside the FA process**: 김재환 was released by Doosan (30 Nov 2025) and signed with SSG for 2 years / 22억 (section 4.7) [official movement data / press].

**Summary by grade and age** [data, from the two tables; "years" counts option years]

| Market | Grade | Signed | Moved | Mean max total (억) | Max years |
|---|---|---|---|---|---|
| 2024-25 | A | 3 | 1 | 48.3 | 4, 4, 4 |
| 2024-25 | B | 9 | 4 | 33.0 | 1-4 (five 4-year deals) |
| 2024-25 | C | 7 (+1 unsigned) | 1 | 22.4 (최정's 110 included) | 2-4 |
| 2025-26 | A | 6 | 3 | 50.2 | 2-4 (five 4-year deals) |
| 2025-26 | B | 8 | 1 | 23.8 | 1-4 |
| 2025-26 | C | 6 (+1 retired) | 3 | 25.3 | 1-4 |

- **By age at signing**, two markets combined [data]:
  - 30 or younger: mean 3.5-3.8 years and 46-56억.
  - 31-34: about 3.2-3.3 years and 13-24억.
  - 35+: 2.3-2.9 years and 25-28억 (a few stars).

**2026-27, first deals (17 Nov 2026)** ([OSEN 2026-11-17](http://www.osen.co.kr/article/G1112911300)) [press]:
- 양의지 (Doosan, 39, 3rd FA, C): 2 years / up to 55억 (bonus 40 + salary 10 + incentives 5). He had an opt-out from his 4+2 year / 152억 deal, with 2 years / 42억 left.
- 오태곤 (SSG, 35, 2nd FA, C): 3 years / up to 11억 (bonus 2 + salary 7.4 + options 1.6).

### 9.3 Salaries 2025 and 2026

**League averages** ([N11394](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11394), [N11861](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11861) and their xlsx attachments) [official]:
- **2025**: 519 players, excluding 48 rookies and 30 foreigners. Average **1억 6,071만** (+3.7%); total 834억 660만.
- **2026**: 529 players, excluding 52 rookies, 30 foreigners and 10 Asia-quota players. Average **1억 7,536만** (+9.1%); total 927억 6,650만.
- **Earlier years** (same releases) [official]: 2022 1억 5,259만 ([N8273](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8273)); 2023 1억 4,648만 ([N8644](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8644)); 2024 1억 5,495만 ([N9985](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9985)).
- **Top salaries**:
  - 2026: 양의지 42억 (record raise of 26억); pitchers 고영표 26억, 박세웅 and 류현진 21억.
  - 2025: 김광현 30억.
  - Highest-paid foreign player: 180만 USD (네일 in 2026, 로하스 in 2025).
  - Top Asia-quota pay in 2026: 200,000 USD.

**Club payrolls, registered players excluding rookies and foreigners (and Asia-quota players in 2026)**, and **the top 28 (2025) / top 29 (2026) per club**, matching the first-team roster size and excluding foreigners and Asia-quota players. Units: 억원; averages in 만원. Sources: [N11394 xlsx sheets 07-08](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11394), [N11861 xlsx sheets 06-07](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11861) [official].

| Club | 2025 players | 2025 total | 2025 average | 2026 players | 2026 total | 2026 average | 2025 top-28 total / average | 2026 top-29 total / average |
|---|---|---|---|---|---|---|---|---|
| LG | 52 | 75.22 | 14,465 | 54 | 108.51 | 20,094 | 66.45 / 23,732 | 97.91 / 33,762 |
| Hanwha | 50 | 87.24 | 17,448 | 55 | 96.87 | 17,613 | 78.64 / 28,086 | 88.34 / 30,462 |
| SSG | 53 | 117.26 | 22,125 | 60 | 124.70 | 20,783 | 109.51 / 39,111 | 113.45 / 39,121 |
| Samsung | 54 | 99.42 | 18,411 | 52 | 98.085 | 18,863 | 87.42 / 31,221 | 88.19 / 30,410 |
| NC | 52 | 64.67 | 12,437 | 50 | 65.84 | 13,168 | 56.88 / 20,314 | 58.45 / 20,155 |
| KT | 53 | 83.72 | 15,796 | 51 | 101.38 | 19,878 | 74.56 / 26,629 | 92.59 / 31,928 |
| Lotte | 54 | 94.66 | 17,530 | 52 | 91.80 | 17,654 | 85.20 / 30,429 | 82.37 / 28,403 |
| KIA | 53 | 94.23 | 17,779 | 56 | 87.49 | 15,623 | 82.80 / 29,571 | 74.05 / 25,534 |
| Doosan | 49 | 73.886 | 15,079 | 50 | 103.88 | 20,776 | 66.99 / 23,925 | 96.47 / 33,266 |
| Kiwoom | 49 | 43.76 | 8,931 | 49 | 49.11 | 10,022 | 36.52 / 13,043 | 41.92 / 14,455 |
| League | 519 | 834.066 | **16,071** | 529 | 927.665 | **17,536** | 744.97 / **26,606** | 833.74 / see note |

- **Note on the 2026 top-29 average.** KBO's sheet gives a league average of 29,776만 (2억 9,776만), which is the 2026 total divided by **280**. Over 290 players (29 × 10) it is **28,750만** [data]. The club averages in the sheet do use 29.
- The 2024 top-28 figures were 710억 6,900만 total, 25,382만 average ([N11394 xlsx](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11394)) [official].
- These registered-player totals differ from the payroll-tax totals in 5.4, which take the top 40 and spread FA bonuses over the contract.

### 9.4 Players released after recent seasons

**Reserve-list exclusions** (players left off the reserve list, i.e. released or retired at season's end, foreign players included). Counts are from KBO's attached exclusion lists [official].

| After season | Left off the list | Same release: players registered that year → reserve list | Source |
|---|---|---|---|
| 2021 | 44 | 610 registered; in season 58 released or voluntarily retired, 43 military, 16 FA/Futures FA; 532 reserved | [N8222](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8222) |
| 2022 | 57 | 684 registered; 151 removed in all; 533 reserved | [N8576](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8576) |
| 2023 | 59 | 681 registered; 139 removed; 542 reserved | [N9907](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=9907) |
| 2024 | 57 | 668 registered; 114 removed; 554 reserved | [N10316](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10316) |
| 2025 | 54 (31 pitchers, 2 catchers, 10 infielders, 11 outfielders) | 597 registered; "29" removed; 568 reserved (see note) | [N11778](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11778) |
| 2026 | not yet published (due 25 Nov, published 30 Nov 2026) | - | - |

- **Note on 2025.** KBO's text says 597 − 29 = 568, yet the attached list names 54 players. The 597 seems to be the January registration count. I use the attachment's 54.
- **In-season releases and waivers** by year are in section 6.7.

### 9.5 Notable veteran retirements and their ages

Age is completed years at the end of the player's last season (mid-October), computed from KBO's official birth date [data]. The retirement itself is confirmed by the press source in the last column [press].

| Offseason | Player (last club) | Born (KBO page) | Age | Retirement source |
|---|---|---|---|---|
| 2021-22 | 송승준 (Lotte) | [1980-06-29](https://www.koreabaseball.com/Record/Retire/Pitcher.aspx?playerId=77591) | 41 | [Newsis 2021-11-19](http://www.newsis.com/view/?id=NISX20211119_0001657232) (retired while disciplined for a banned-drug case) |
| 2021-22 | 유한준 (KT) | [1981-07-01](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=74339) | 40 | [OhmyNews](http://www.ohmynews.com/NWS_Web/View/at_pg.aspx?CNTN_CD=A0002804899) (retired a week after the 2021 title) |
| 2022-23 | 이대호 (Lotte) | [1982-06-21](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=71564) | 40 | [Ilgan Sports 2022-10-13](https://isplus.com/2022/10/13/sports/baseball/20221013103710776.html) |
| 2022-23 | 오재원 (Doosan) | [1985-02-09](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=77248) | 37 | same |
| 2022-23 | 나지완 (KIA) | [1985-05-19](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=78629) | 37 | [SPOTV](https://www.spotvnews.co.kr/news/articleView.html?idxno=558148) |
| 2023-24 | 박석민 (NC) | [1985-06-22](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=74465) | 38 | [MyDaily 2023-10-31](https://mydaily.co.kr/page/view/2023103102224202885) |
| 2023-24 | 장원준 (Doosan) | [1985-07-31](https://www.koreabaseball.com/Record/Retire/Pitcher.aspx?playerId=74513) | 38 | [StarNews 2023-11-07](https://www.starnewskorea.com/stview.php?no=2023110710012781033) |
| 2024-25 | 추신수 (SSG) | [1982-07-13](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=51817) | 42 | [Newsis 2024-12-30](https://www.newsis.com/view/NISX20241229_0003013890) |
| 2024-25 | 김강민 (Hanwha) | [1982-09-13](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=71837) | 42 | same |
| 2024-25 | 박경수 (KT) | [1984-03-31](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=73113) | 40 | same |
| 2024-25 | 김재호 (Doosan) | [1985-03-21](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=74206) | 39 | same |
| 2024-25 | 정우람 (Hanwha) | [1985-06-01](https://www.koreabaseball.com/Record/Retire/Pitcher.aspx?playerId=74857) | 39 | [Kukmin Ilbo 2024-10-28](https://www.kmib.co.kr/article/view.asp?arcid=1729760012) |
| 2025-26 | 오승환 (Samsung) | [1982-07-15](https://www.koreabaseball.com/Record/Retire/Pitcher.aspx?playerId=75421) | 43 | [Xportsnews 2025-08-14](https://www.xportsnews.com/article/2040058) (announced 6 Aug 2025) |
| 2025-26 | 박병호 (Samsung) | [1986-07-10](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=75125) | 39 | [StarNews 2025-12-21](https://www.starnewskorea.com/sports/2025/12/21/2025122115002968543) (gives 39) |
| 2025-26 | 오재일 (KT) | [1986-10-29](https://www.koreabaseball.com/Record/Retire/Hitter.aspx?playerId=75334) | 38 (39 by the end of October) | [OSEN 2025-12-15](http://www.osen.co.kr/article/G1112713329) |
| 2025-26 | 황재균 (KT, FA) | [1987-07-28](https://www.koreabaseball.com/Record/Player/HitterDetail/Basic.aspx?playerId=76313) | 38 | [Sports Kyunghyang 2025-12-19](https://sports.khan.co.kr/article/202512191408003) |
| (FA, unsigned) | 문성현 (Kiwoom) | [1991-11-09](https://www.koreabaseball.com/Record/Retire/Pitcher.aspx?playerId=60336) | 34 at retirement (5 May 2026) | [SPOTV](https://www.spotvnews.co.kr/news/articleView.html?idxno=814811) |

- **Pattern** [data]: the notable veterans who retired after 2021-2025 were mostly **37-43**, with the stars at 40-43.
- **Out of range but useful**: 장시환 (LG, 38) retired in Oct 2026, when LG declined to re-sign 11 players ([Yonhap 2026-10-22](https://www.yna.co.kr/view/AKR20261022100400007)) [press]. He had been dropped by Hanwha in the 2025 exclusion list ([N11778](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11778)) [official].

---

## 10. Announced for 2027 (question 9)

| Item | 2027 | Source |
|---|---|---|
| Payroll cap | 151억 1,709만 (+5%) | [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687), [Bylaws Art. 184(3)](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Payroll floor | First year: 60억 6,538만 (+5% a year after; penalties 30/50/100%) | [N11687](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11687), [Bylaws Arts. 187-188](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| Minimum salary | 3,300만 | [N11814](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=11814) [official] |
| Contract calendar (from 1 Jan 2027, for contracts covering 2027) | Pay period 25 Jan-23 Nov; off-season 24 Nov-24 Jan; contract approval by 24 Jan; reserve allowance from 25 Jan for players unsigned by 24 Jan; arbitration decided by 23 Jan; reserve list due 18 Nov | [N12184](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12184) [official] |
| FA dates | The 2027 FA schedule already uses "after 25 January" for the late-signing compensation rule (bylaw text: 1 February) | [N12242 attachment 2](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242) [official] |
| Outside games and commercial work | Written club approval 7 days ahead | [N12184](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12184) [official] |
| FA rule against contracts that defeat compensation ("김재환 룰") | Planned to apply after the 2026 season; **adoption NOT CONFIRMED** | [Sports Kyunghyang 2026-01-29](https://sports.khan.co.kr/article/202601291033003) [press] |
| Foreign-player holding limit | "To be eased in stages" (no number or date) | [N12184](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12184) [official] |
| Secondary draft | Every other year from 2023, so due after the 2027 season; date not announced | [Bylaws Art. 177](https://6ptotvmi5753.edge.naverncp.com/KBO_FILE/ebook/pdf/2026_야구규약.pdf) [official] |
| 2027 FA market | 28 eligible, 22 approved, 9 deferred, up to 3 outside signings per club; market open since 17 Nov 2026 | [N12242](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242), [N12243](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12243) [official] |

NOT CONFIRMED: whether the 10 monthly salary instalments (Bylaws Art. 72) move with the new pay period, and whether the 10 January arbitration filing date moves.

---

## 11. Conflicts and gaps

**Conflicts**
1. **Late-signing compensation date**: the printed 2026 bylaws (Arts. 44, 172(8)) still say 1 February, while the 2027 FA schedule ([N12242](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=12242)) says 25 January, in line with the October 2026 calendar change.
2. **2026 top-29 league average**: KBO's sheet gives 2억 9,776만 (total ÷ 280); total ÷ 290 is 2억 8,750만 (9.3).
3. **2026 reserve list**: KBO's text says 29 players removed; its attachment names 54 (9.4).
4. **김현수's contract**: Newsis says "계약 기간 4년" in the body ([NISX20251125_0003416573](https://www.newsis.com/view/NISX20251125_0003416573)), but its headline, Yonhap and Xportsnews say 3 years / 50억. I use 3.
5. **우규민's breakdown**: Yonhap gives "salary 2억" (pieces add to 5억); Xportsnews gives salary 4억 (pieces add to the 7억 total). I use Xportsnews.
6. **Kiwoom's 2025 top-40 total**: Sports Hankook prints 43억9765만; KBO says 43억 9,756만. I use KBO's.
7. **Foreign-player Art. 8(3) vs 8(1)**: 8(3) says the cost caps do not apply when re-signing a reserved player, while 8(1) builds re-signing years into the team cap. Taken literally the text conflicts with itself (section 7).
8. **Cap period**: the 2022 release called the 114억 2,638만 cap a "2023-2025" cap ([N8558](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=8558)); the 2024 board raised the 2025 figure by 20% ([N10160](https://www.koreabaseball.com/MediaNews/Notice/View.aspx?bdSe=10160)). This is a change, not an error.
9. **Arbitration "zero" years**: the summary box on the StarNews page dates the run of zero filings to 2024, while the article body says 2022-2026. I use the body. The historical totals (99 / 21 / 2) come from that one article and were not checked elsewhere.

**Gaps (NOT CONFIRMED)**
- No KBO press release on the July 2021 permission for non-FA multi-year deals (press only).
- Whether non-FA deals may legally include a signing bonus (inferred "no" from Art. 81).
- Grade split of the 2022-23 applicants (the KBO attachment is an old .doc file).
- Whether 2026 Asian Games players got roster-day compensation.
- Whether early-draft (2nd-year university) players count as college graduates for the 7-season rule.
- Whether the payroll-floor penalties count from 2027 alone or use earlier years (no club has been tested).
- The breakdown of several contracts (dashes in 9.2).
- Foreign vs domestic split of the reserve-list exclusions. KBO marks foreigners with a symbol that was lost when converting the files; I did not separate them.
- The reason for the 31 January bulk releases.
- Adoption of the "김재환 룰".

---

## 12. Working files

Downloaded copies of the sources and the counting scripts were kept outside the repository; every figure above links to its source.
