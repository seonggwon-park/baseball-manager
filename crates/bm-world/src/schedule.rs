//! Regular-season schedules, built to the formats of the real leagues
//! (`docs/research/world/*_notes.md`).
//!
//! The Korean league: ten clubs, 144 games each, 16 against each opponent. Three-game
//! series from Tuesday to Thursday and from Friday to Sunday, Mondays off. 135 games per
//! club are scheduled before the season; the last game of every pairing is left out and
//! placed in the closing weeks with the rained-out games. Five clubs play 73 games at
//! home and five 71, swapping every year.

use crate::calendar::{Date, Weekday};
use bm_engine::Rng;

/// One game: indices into the league's club list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Game {
    pub date: Date,
    pub home: usize,
    pub away: usize,
}

/// Rounds of a single round robin for `n` clubs (circle method): every round pairs
/// every club with another, every pair meets once.
fn round_robin(n: usize) -> Vec<Vec<(usize, usize)>> {
    assert!(n >= 2 && n % 2 == 0);
    let mut ring: Vec<usize> = (1..n).collect();
    let mut rounds = Vec::with_capacity(n - 1);
    for _ in 0..n - 1 {
        let mut round = vec![(0, ring[0])];
        for i in 1..n / 2 {
            round.push((ring[i], ring[n - 1 - i]));
        }
        rounds.push(round);
        ring.rotate_right(1);
    }
    rounds
}

/// Orientation of every pairing: `host[a][b]` is true when `a` holds the larger share of
/// the home games against `b`. Clubs in `more_home` hold it against five opponents, the
/// others against four (73 and 71 home games out of 144 with 16 games per pairing).
fn host_shares(n: usize, more_home: &[bool], rng: &mut Rng) -> Vec<Vec<bool>> {
    host_shares_k(n, more_home, n / 2, rng)
}

/// As [`host_shares`], with clubs in `more_home` holding the larger share against `k`
/// opponents and the others against `k - 1`.
fn host_shares_k(n: usize, more_home: &[bool], k: usize, rng: &mut Rng) -> Vec<Vec<bool>> {
    let want = |i: usize| if more_home[i] { k } else { k - 1 };
    for _ in 0..10_000 {
        let mut host = vec![vec![false; n]; n];
        let mut count = vec![0; n];
        let mut pairs: Vec<(usize, usize)> = (0..n).flat_map(|a| (a + 1..n).map(move |b| (a, b))).collect();
        rng.shuffle(&mut pairs);
        let mut ok = true;
        for (a, b) in pairs {
            // Give the larger share to whichever club is further from its quota.
            let (need_a, need_b) = (want(a) as i32 - count[a], want(b) as i32 - count[b]);
            let a_hosts = if need_a == need_b { rng.chance(0.5) } else { need_a > need_b };
            let (h, g) = if a_hosts { (a, b) } else { (b, a) };
            host[h][g] = true;
            count[h] += 1;
            if count[h] > want(h) as i32 {
                ok = false;
                break;
            }
        }
        if ok && (0..n).all(|i| count[i] == want(i) as i32) {
            return host;
        }
    }
    panic!("no orientation found for the home-game quotas");
}

/// The Korean league's regular season for `n` clubs (10), opening on `opening` (a
/// Tuesday or Friday) with an All-Star break of the given dates in which no game is
/// played. `more_home[i]` marks the clubs with 73 home games this year.
pub fn kbo(n: usize, opening: Date, all_star_break: (Date, Date), more_home: &[bool], rng: &mut Rng) -> Vec<Game> {
    assert_eq!(more_home.len(), n);
    let host = host_shares(n, more_home, rng);
    // Five three-game series per pairing; the club with the larger share hosts three of them.
    let mut series_left: Vec<Vec<u8>> = vec![vec![0; n]; n];
    for a in 0..n {
        for b in 0..n {
            if a != b && host[a][b] {
                series_left[a][b] = 3;
                series_left[b][a] = 2;
            }
        }
    }
    // 45 slots: five cycles of a round robin, each cycle in a shuffled order.
    let base = round_robin(n);
    let mut slots: Vec<Vec<(usize, usize)>> = Vec::new();
    for _ in 0..5 {
        let mut cycle = base.clone();
        rng.shuffle(&mut cycle);
        slots.extend(cycle);
    }
    let mut games = Vec::new();
    let mut day = opening;
    let in_break = |d: Date| d >= all_star_break.0 && d <= all_star_break.1;
    for slot in slots {
        // The next Tuesday or Friday that is not in the break.
        while !matches!(day.weekday(), Weekday::Tuesday | Weekday::Friday) || in_break(day) || in_break(day.add_days(2)) {
            day = day.add_days(1);
        }
        for (a, b) in slot {
            // Alternate the venue so a pairing does not stay in one park all season.
            let a_home = match (series_left[a][b], series_left[b][a]) {
                (0, _) => false,
                (_, 0) => true,
                (x, y) => x > y || (x == y && rng.chance(0.5)),
            };
            let (h, g) = if a_home { (a, b) } else { (b, a) };
            series_left[h][g] -= 1;
            for k in 0..3 {
                games.push(Game { date: day.add_days(k), home: h, away: g });
            }
        }
        day = day.add_days(3);
    }
    // The 16th game of every pairing, at the park of the club with the smaller share, in
    // the "remaining games" schedule after the series are over.
    let leftover: Vec<(usize, usize)> = (0..n).flat_map(|a| (0..n).map(move |b| (a, b))).filter(|&(a, b)| a != b && host[b][a]).collect();
    place_after_series(&mut games, leftover, rng);
    games
}

/// Put single games into the days after the last series, each on the first day on which
/// neither club plays; then sort the schedule by date.
fn place_after_series(games: &mut Vec<Game>, mut leftover: Vec<(usize, usize)>, rng: &mut Rng) {
    rng.shuffle(&mut leftover);
    let mut busy = std::collections::HashSet::new();
    for g in games.iter() {
        busy.insert((g.date, g.home));
        busy.insert((g.date, g.away));
    }
    let mut d = games.iter().map(|g| g.date).max().expect("a schedule").add_days(1);
    while !leftover.is_empty() {
        let mut i = 0;
        while i < leftover.len() {
            let (h, g) = leftover[i];
            if !busy.contains(&(d, h)) && !busy.contains(&(d, g)) {
                busy.insert((d, h));
                busy.insert((d, g));
                games.push(Game { date: d, home: h, away: g });
                leftover.swap_remove(i);
            } else {
                i += 1;
            }
        }
        d = d.add_days(1);
    }
    games.sort_by_key(|g| (g.date, g.home));
}

/// Japan's two first-team leagues (`docs/research/world/npb_notes.md`): six clubs each,
/// 143 games per club. 25 against each club of the own league, split 13 / 12 at home;
/// three clubs of each league have 63 league home games and three 62, by turns. 18
/// interleague games, three against each club of the other league in one park, every
/// club hosting three of the six; `swap` reverses the hosts, as every other year. Clubs
/// `0..6` form one league and `6..12` the other. Three-game series, Mondays off; the
/// interleague period begins on `interleague` (a Tuesday) and lasts three weeks.
pub fn npb(opening: Date, interleague: Date, all_star_break: (Date, Date), more_home: &[bool; 12], swap: bool, rng: &mut Rng) -> Vec<Game> {
    let mut league_host = vec![vec![false; 12]; 12];
    for lg in 0..2 {
        let more: Vec<bool> = more_home[6 * lg..6 * lg + 6].to_vec();
        // 63 or 62 home games: the larger share (13) against three or two opponents.
        let shares = host_shares_k(6, &more, 3, rng);
        for a in 0..6 {
            for b in 0..6 {
                league_host[6 * lg + a][6 * lg + b] = shares[a][b];
            }
        }
    }
    let rounds = round_robin(6);
    // 40 in-league slots (eight cycles), and six interleague slots in a Latin square.
    let mut league_slots: Vec<Vec<(usize, usize)>> = Vec::new();
    for _ in 0..8 {
        let mut cycle: Vec<Vec<(usize, usize)>> = Vec::new();
        for r in &rounds {
            let mut both: Vec<(usize, usize)> = r.clone();
            both.extend(r.iter().map(|&(a, b)| (a + 6, b + 6)));
            cycle.push(both);
        }
        rng.shuffle(&mut cycle);
        league_slots.extend(cycle);
    }
    let mut inter_slots: Vec<Vec<(usize, usize)>> = (0..6).map(|k| (0..6).map(|i| (i, 6 + (i + k) % 6)).collect()).collect();
    rng.shuffle(&mut inter_slots);
    let mut series_left = vec![vec![4u8; 12]; 12];
    let in_break = |d: Date| d >= all_star_break.0 && d <= all_star_break.1;
    let inter_end = interleague.add_days(20);
    let mut games = Vec::new();
    let mut day = opening;
    let mut league_iter = league_slots.into_iter();
    let mut inter_iter = inter_slots.into_iter();
    loop {
        while !matches!(day.weekday(), Weekday::Tuesday | Weekday::Friday) || in_break(day) || in_break(day.add_days(2)) {
            day = day.add_days(1);
        }
        let in_inter = day >= interleague && day <= inter_end;
        let slot = if in_inter { inter_iter.next() } else { league_iter.next() };
        let Some(slot) = slot else {
            if in_inter {
                day = inter_end.add_days(1);
                continue;
            }
            break;
        };
        for (a, b) in slot {
            let (h, g) = if in_inter {
                // Club `a` of the first league hosts the three clubs of the other league that
                // follow it in a fixed rotation, and visits the other three.
                let first_hosts = ((b - 6 + 6 - a) % 6 < 3) != swap;
                if first_hosts {
                    (a, b)
                } else {
                    (b, a)
                }
            } else {
                let a_home = match (series_left[a][b], series_left[b][a]) {
                    (0, _) => false,
                    (_, 0) => true,
                    (x, y) => x > y || (x == y && rng.chance(0.5)),
                };
                let (h, g) = if a_home { (a, b) } else { (b, a) };
                series_left[h][g] -= 1;
                (h, g)
            };
            for k in 0..3 {
                games.push(Game { date: day.add_days(k), home: h, away: g });
            }
        }
        day = day.add_days(3);
    }
    assert!(inter_iter.next().is_none(), "the interleague period does not fit");
    // The 25th game of every in-league pairing, at the park of the club with the larger share.
    let leftover: Vec<(usize, usize)> = (0..12).flat_map(|a| (0..12).map(move |b| (a, b))).filter(|&(a, b)| a != b && league_host[a][b]).collect();
    place_after_series(&mut games, leftover, rng);
    games
}

/// A full-season minor league (`docs/research/world/mlb_notes.md`): an opening three-game
/// series, then six-game series from Tuesday to Sunday with every Monday off, and one
/// more three-game series (Friday to Sunday) straight after the All-Star break; 150
/// games at Triple-A, 138 at Double-A and 132 at the two A levels. Most games are played
/// inside the division (Double-A Biloxi, 2027: 84 of 138). `division[i]` is the division
/// of club `i` (all 0 in a league without divisions); `in_division` is the share of a
/// club's series against its own division.
pub fn milb(division: &[usize], opening: Date, games: u32, all_star_break: (Date, Date), in_division: f64, rng: &mut Rng) -> Vec<Game> {
    let n = division.len();
    assert!(n >= 4 && n % 2 == 0, "a minor league needs an even number of clubs");
    assert!(games >= 6 && (games - 6) % 6 == 0, "games = two three-game series plus six-game series");
    let sixes = ((games - 6) / 6) as usize;
    let slots = sixes + 2;
    // How many series each pairing should have: the in-division share spread evenly over
    // the division, the rest over the other clubs.
    let mut want = vec![vec![0.0f64; n]; n];
    for a in 0..n {
        let same: Vec<usize> = (0..n).filter(|&b| b != a && division[b] == division[a]).collect();
        let other: Vec<usize> = (0..n).filter(|&b| division[b] != division[a]).collect();
        let share = if other.is_empty() {
            1.0
        } else if same.is_empty() {
            0.0
        } else {
            in_division
        };
        for &b in &same {
            want[a][b] = share * slots as f64 / same.len() as f64;
        }
        for &b in &other {
            want[a][b] = (1.0 - share) * slots as f64 / other.len() as f64;
        }
    }
    let mut met = vec![vec![0u32; n]; n];
    let mut last_home = vec![vec![None::<bool>; n]; n];
    // One pairing of every club per slot: the most overdue pairs first, with a random
    // tie-break; retried until every club has an opponent.
    let pairing = |met: &Vec<Vec<u32>>, rng: &mut Rng| -> Vec<(usize, usize)> {
        for _ in 0..200 {
            let mut free: Vec<usize> = (0..n).collect();
            rng.shuffle(&mut free);
            let mut out = Vec::new();
            while let Some(a) = free.pop() {
                let best = free
                    .iter()
                    .enumerate()
                    .map(|(i, &b)| (i, (want[a][b] + want[b][a]) / 2.0 - met[a][b] as f64 + 0.01 * rng.f64()))
                    .max_by(|x, y| x.1.total_cmp(&y.1));
                match best {
                    Some((i, _)) => out.push((a, free.swap_remove(i))),
                    None => break,
                }
            }
            if out.len() == n / 2 {
                return out;
            }
        }
        unreachable!("an even number of clubs can always be paired");
    };
    let mut games_out = Vec::new();
    let mut home_games = vec![0i32; n];
    let mut play = |pairs: Vec<(usize, usize)>, start: Date, len: i32, met: &mut Vec<Vec<u32>>, rng: &mut Rng| {
        for (a, b) in pairs {
            // Keep every club near half its games at home; otherwise alternate the venue
            // of a pairing.
            let a_home = if home_games[a] + 3 < home_games[b] {
                true
            } else if home_games[b] + 3 < home_games[a] {
                false
            } else {
                match last_home[a][b] {
                    Some(h) => !h,
                    None => rng.chance(0.5),
                }
            };
            home_games[if a_home { a } else { b }] += len;
            last_home[a][b] = Some(a_home);
            last_home[b][a] = Some(!a_home);
            met[a][b] += 1;
            met[b][a] += 1;
            let (h, g) = if a_home { (a, b) } else { (b, a) };
            for k in 0..len {
                games_out.push(Game { date: start.add_days(k), home: h, away: g });
            }
        }
    };
    let opening_pairs = pairing(&met, rng);
    play(opening_pairs, opening, 3, &mut met, rng);
    let mut day = opening.add_days(3);
    let mut sixes_left = sixes;
    let mut after_break_done = false;
    while sixes_left > 0 || !after_break_done {
        if !after_break_done && day.add_days(6) > all_star_break.0 {
            // The All-Star week: the three-game series begins on the Friday after the break.
            let mut fri = all_star_break.1.add_days(1);
            while fri.weekday() != Weekday::Friday {
                fri = fri.add_days(1);
            }
            let pairs = pairing(&met, rng);
            play(pairs, fri, 3, &mut met, rng);
            after_break_done = true;
            day = fri.add_days(3);
            continue;
        }
        while day.weekday() != Weekday::Tuesday {
            day = day.add_days(1);
        }
        if sixes_left == 0 {
            // The season ends before the All-Star break (not the case for any real league).
            break;
        }
        let pairs = pairing(&met, rng);
        play(pairs, day, 6, &mut met, rng);
        sixes_left -= 1;
        day = day.add_days(6);
    }
    games_out.sort_by_key(|g| (g.date, g.home));
    games_out
}

/// The major leagues (`docs/research/world/mlb_notes.md`, format of 2025-2027): two
/// leagues of fifteen clubs in three divisions of five; 162 games per club, 81 at home.
/// 52 against the division (13 against each rival: three three-game series and one of
/// four), 62 against the rest of the league (two three-game series against eight clubs,
/// a three- and a four-game series against two) and 48 interleague games (two series
/// against the "prime" rival, one three-game series against each other club).
///
/// Clubs `0..15` are one league and `15..30` the other, five per division in order;
/// `rival[i]` is the index in `0..15` of club `i`'s interleague rival in the other
/// league. The season is 52 series: an opening series from Thursday to Sunday, two a
/// week (Monday to Thursday and Friday to Sunday) and one from Thursday to Sunday after
/// the All-Star break in the week of `all_star_monday`. Four-game series fit only the
/// Monday-to-Thursday and Thursday-to-Sunday windows; a three-game series there leaves a
/// day off. With 15 clubs per league every window needs an odd number of interleague
/// series. `swap` reverses the hosts of the pairings, as every other year.
pub fn mlb(opening: Date, all_star_monday: Date, rival: &[usize; 15], swap: bool, rng: &mut Rng) -> Vec<Game> {
    assert_eq!(opening.weekday(), Weekday::Thursday, "the opening series runs Thursday to Sunday");
    assert_eq!(all_star_monday.weekday(), Weekday::Monday);
    for attempt in 0..1000 {
        if let Some(games) = try_mlb(opening, all_star_monday, rival, swap, rng) {
            return games;
        }
        assert!(attempt < 999, "no major-league schedule found");
    }
    unreachable!()
}

/// One series still to be played: its length and whether the lower-numbered club hosts it.
type Series = (i32, bool);

// Pairings are indexed by club numbers on both sides; index loops read more plainly here.
#[allow(clippy::needless_range_loop)]
fn try_mlb(opening: Date, all_star_monday: Date, rival: &[usize; 15], swap: bool, rng: &mut Rng) -> Option<Vec<Game>> {
    let n = 30;
    let league = |c: usize| c / 15;
    let division = |c: usize| c / 5;
    let is_rival = |a: usize, b: usize| a < 15 && b >= 15 && rival[a] == b - 15;
    // Series of every pairing (a < b), with the host decided up front so that every club
    // has 81 home games.
    let mut left: Vec<Vec<Vec<Series>>> = vec![vec![Vec::new(); n]; n];
    // Division: a regular tournament on each division (each club takes the larger share,
    // the four-game series plus one three-game series, against two of its four rivals).
    for a in 0..n {
        for b in a + 1..n {
            if league(a) == league(b) && division(a) == division(b) {
                let d = (b - a) % 5; // 1..4 inside a division of five
                let a_more = (d == 1 || d == 2) != swap;
                left[a][b] = vec![(4, a_more), (3, a_more), (3, !a_more), (3, !a_more)];
            }
        }
    }
    // Rest of the league: two clubs per club get a three- and a four-game series (a
    // 2-regular graph oriented around its cycles, so every club hosts one four-game
    // series), the others two three-game series, one at each park.
    for lg in 0..2 {
        let clubs: Vec<usize> = (15 * lg..15 * lg + 15).collect();
        let cycle_edges = loop {
            let mut order = clubs.clone();
            rng.shuffle(&mut order);
            // A Hamiltonian cycle through clubs of different divisions: consecutive clubs
            // differ in division, and so do the last and the first.
            let ok = (0..15).all(|i| division(order[i]) != division(order[(i + 1) % 15]));
            if ok {
                break (0..15).map(|i| (order[i], order[(i + 1) % 15])).collect::<Vec<_>>();
            }
        };
        for &(x, y) in &cycle_edges {
            // `x` hosts the four-game series against `y`.
            let (a, b) = (x.min(y), x.max(y));
            let a_hosts_four = (a == x) != swap;
            left[a][b] = vec![(4, a_hosts_four), (3, !a_hosts_four)];
        }
        for &a in &clubs {
            for &b in &clubs {
                if a < b && division(a) != division(b) && left[a][b].is_empty() {
                    left[a][b] = vec![(3, true), (3, false)];
                }
            }
        }
    }
    // Interleague: the rival home and away; every other club once, each club hosting
    // seven of the fourteen.
    for a in 0..15 {
        for b in 15..30 {
            if is_rival(a, b) {
                left[a][b] = vec![(3, true), (3, false)];
            } else {
                let d = (b - 15 + 15 - rival[a]) % 15; // 1..14 for the non-rivals
                left[a][b] = vec![(3, (d <= 7) != swap)];
            }
        }
    }
    // The 52 windows.
    let mut windows: Vec<(Date, i32)> = vec![(opening, 4)];
    let mut monday = opening.add_days(4);
    while windows.len() < 52 {
        if monday == all_star_monday {
            windows.push((monday.add_days(3), 4));
        } else {
            windows.push((monday, 4));
            windows.push((monday.add_days(4), 3));
        }
        monday = monday.add_days(7);
    }
    windows.truncate(52);
    let mut games = Vec::new();
    for &(start, days) in &windows {
        // A perfect matching of the 30 clubs among the series still to be played: the club
        // with the fewest possible opponents first, paired with the opponent it has the most
        // series left against.
        let fits = |l: &Vec<Series>| !l.is_empty() && (days == 4 || l.iter().any(|s| s.0 == 3));
        let mut pairs = Vec::new();
        let mut matched = false;
        for _ in 0..60 {
            let mut free: Vec<usize> = (0..n).collect();
            pairs.clear();
            let mut ok = true;
            while !free.is_empty() {
                let options =
                    |c: usize, free: &Vec<usize>| -> Vec<usize> { free.iter().copied().filter(|&d| d != c && fits(&left[c.min(d)][c.max(d)])).collect() };
                let (ci, _) =
                    free.iter().enumerate().map(|(i, &c)| (i, options(c, &free).len() as f64 + 0.5 * rng.f64())).min_by(|x, y| x.1.total_cmp(&y.1)).unwrap();
                let c = free.swap_remove(ci);
                let opts = options(c, &free);
                let Some(&d) = opts.iter().max_by(|&&x, &&y| {
                    let lx = left[c.min(x)][c.max(x)].len() as f64 + rng.f64();
                    let ly = left[c.min(y)][c.max(y)].len() as f64 + rng.f64();
                    lx.total_cmp(&ly)
                }) else {
                    ok = false;
                    break;
                };
                free.retain(|&x| x != d);
                pairs.push((c.min(d), c.max(d)));
            }
            if ok {
                matched = true;
                break;
            }
        }
        if !matched {
            return None;
        }
        for &(a, b) in &pairs {
            let l = &mut left[a][b];
            // A four-game series when the window allows one, else a three-game series.
            let i = l.iter().position(|s| days == 4 && s.0 == 4).or_else(|| l.iter().position(|s| s.0 == 3)).expect("a series that fits");
            let (len, a_hosts) = l.swap_remove(i);
            let (h, g) = if a_hosts { (a, b) } else { (b, a) };
            // A three-game series in a Monday-to-Thursday window starts on Tuesday.
            let first = if len == 3 && days == 4 { start.add_days(1) } else { start };
            for k in 0..len {
                games.push(Game { date: first.add_days(k), home: h, away: g });
            }
        }
    }
    if left.iter().flatten().any(|l| !l.is_empty()) {
        return None;
    }
    games.sort_by_key(|g| (g.date, g.home));
    Some(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_korean_season_has_the_real_shape() {
        for seed in 0..10 {
            korean_season(seed);
        }
    }

    fn korean_season(seed: u64) {
        let mut rng = Rng::new(seed);
        let more_home = [false, true, false, true, false, true, false, true, false, true];
        let opening = Date::new(2027, 3, 26);
        let brk = (Date::new(2027, 7, 9), Date::new(2027, 7, 14));
        let games = kbo(10, opening, brk, &more_home, &mut rng);
        assert_eq!(games.len(), 720);
        for (club, &more) in more_home.iter().enumerate() {
            let mine: Vec<&Game> = games.iter().filter(|g| g.home == club || g.away == club).collect();
            assert_eq!(mine.len(), 144, "club {club}");
            let home = mine.iter().filter(|g| g.home == club).count();
            assert_eq!(home, if more { 73 } else { 71 }, "club {club}");
            for other in (0..10).filter(|&o| o != club) {
                let n = mine.iter().filter(|g| g.home == other || g.away == other).count();
                assert_eq!(n, 16, "{club} v {other}");
            }
            // Never two games on one day.
            let mut days: Vec<Date> = mine.iter().map(|g| g.date).collect();
            days.sort();
            days.dedup();
            assert_eq!(days.len(), 144, "club {club} plays twice on a day");
        }
        // 135 games per club before the closing weeks, on Tuesday to Sunday, none in the break.
        let monday = games.iter().filter(|g| g.date.weekday() == Weekday::Monday).count();
        assert!(monday <= 45, "{monday} Monday games");
        assert!(games.iter().all(|g| g.date < brk.0 || g.date > brk.1));
        assert!(games.iter().all(|g| g.date >= opening));
        let last = games.iter().map(|g| g.date).max().unwrap();
        assert!(last < Date::new(2027, 10, 15), "season ends {last}");
    }

    #[test]
    fn the_japanese_season_has_the_real_shape() {
        for seed in 0..10 {
            japanese_season(seed);
        }
    }

    fn japanese_season(seed: u64) {
        let mut rng = Rng::new(seed);
        let mut more_home = [false; 12];
        for i in [1, 3, 5, 6, 8, 10] {
            more_home[i] = true;
        }
        let opening = Date::new(2027, 3, 26);
        let inter = Date::new(2027, 6, 1);
        let brk = (Date::new(2027, 7, 19), Date::new(2027, 7, 22));
        for swap in [false, true] {
            let games = npb(opening, inter, brk, &more_home, swap, &mut rng);
            assert_eq!(games.len(), 12 * 143 / 2);
            for (club, &more) in more_home.iter().enumerate() {
                let mine: Vec<&Game> = games.iter().filter(|g| g.home == club || g.away == club).collect();
                assert_eq!(mine.len(), 143, "club {club}");
                let same_league = |o: usize| (o < 6) == (club < 6);
                let league_home = mine.iter().filter(|g| g.home == club && same_league(g.away)).count();
                assert_eq!(league_home, if more { 63 } else { 62 }, "club {club}");
                let inter_home = mine.iter().filter(|g| g.home == club && !same_league(g.away)).count();
                assert_eq!(inter_home, 9, "club {club}");
                for other in (0..12).filter(|&o| o != club) {
                    let n = mine.iter().filter(|g| g.home == other || g.away == other).count();
                    assert_eq!(n, if same_league(other) { 25 } else { 3 }, "{club} v {other}");
                }
                let mut days: Vec<Date> = mine.iter().map(|g| g.date).collect();
                days.sort();
                days.dedup();
                assert_eq!(days.len(), 143, "club {club} plays twice on a day");
            }
            // Interleague games only in the three-week period, and only interleague games there.
            for g in &games {
                let inter_game = (g.home < 6) != (g.away < 6);
                let in_window = g.date >= inter && g.date <= inter.add_days(20);
                assert_eq!(inter_game, in_window, "{g:?}");
                assert!(g.date < brk.0 || g.date > brk.1);
            }
            // Swapping the year swaps the interleague hosts.
            let host_0_vs_6 = games.iter().any(|g| g.home == 0 && g.away == 6);
            assert_eq!(host_0_vs_6, !swap);
        }
    }

    #[test]
    fn minor_league_seasons_have_the_real_shape() {
        // Double-A South: eight clubs in two divisions of four, 138 games.
        let division = [0, 0, 0, 0, 1, 1, 1, 1];
        let opening = Date::new(2027, 4, 2);
        let brk = (Date::new(2027, 7, 12), Date::new(2027, 7, 15));
        for seed in 0..20 {
            let mut rng = Rng::new(seed);
            let games = milb(&division, opening, 138, brk, 0.6, &mut rng);
            for club in 0..8 {
                let mine: Vec<&Game> = games.iter().filter(|g| g.home == club || g.away == club).collect();
                assert_eq!(mine.len(), 138, "club {club}");
                let home = mine.iter().filter(|g| g.home == club).count();
                assert!((60..=78).contains(&home), "club {club}: {home} home games");
                let inside = mine.iter().filter(|g| division[g.home] == division[g.away]).count();
                assert!((70..=98).contains(&inside), "club {club}: {inside} games inside the division");
                let mut days: Vec<Date> = mine.iter().map(|g| g.date).collect();
                days.sort();
                days.dedup();
                assert_eq!(days.len(), 138, "club {club} plays twice on a day");
            }
            assert!(games.iter().all(|g| g.date.weekday() != Weekday::Monday), "Mondays are off");
            assert!(games.iter().all(|g| g.date < brk.0 || g.date > brk.1));
            let last = games.iter().map(|g| g.date).max().unwrap();
            assert!(last <= Date::new(2027, 9, 13), "season ends {last}");
        }
        let mut rng = Rng::new(9);
        // A league without divisions.
        let games = milb(&[0; 6], opening, 132, brk, 0.6, &mut rng);
        assert_eq!(games.len(), 6 * 132 / 2);
    }

    #[test]
    fn the_major_league_season_has_the_real_shape() {
        // Prime rivals pair clubs of the same position in the other league.
        let rival: [usize; 15] = std::array::from_fn(|i| i);
        let opening = Date::new(2027, 3, 25);
        let all_star = Date::new(2027, 7, 12);
        for seed in 0..3 {
            let mut rng = Rng::new(seed);
            let games = mlb(opening, all_star, &rival, seed == 1, &mut rng);
            assert_eq!(games.len(), 30 * 162 / 2);
            for club in 0..30 {
                let mine: Vec<&Game> = games.iter().filter(|g| g.home == club || g.away == club).collect();
                assert_eq!(mine.len(), 162, "club {club}");
                assert_eq!(mine.iter().filter(|g| g.home == club).count(), 81, "club {club} home games");
                let opp = |g: &Game| if g.home == club { g.away } else { g.home };
                let (mut division, mut league, mut inter) = (0, 0, 0);
                for g in &mine {
                    let o = opp(g);
                    if o / 15 != club / 15 {
                        inter += 1;
                    } else if o / 5 == club / 5 {
                        division += 1;
                    } else {
                        league += 1;
                    }
                }
                assert_eq!((division, league, inter), (52, 62, 48), "club {club}");
                let mut sevens = 0;
                for o in (0..30).filter(|&o| o != club) {
                    let k = mine.iter().filter(|g| opp(g) == o).count();
                    let expected = if o / 15 != club / 15 {
                        if rival[club % 15] == o % 15 {
                            6
                        } else {
                            3
                        }
                    } else if o / 5 == club / 5 {
                        13
                    } else {
                        if k == 7 {
                            sevens += 1;
                        }
                        if k == 7 {
                            7
                        } else {
                            6
                        }
                    };
                    assert_eq!(k, expected, "{club} v {o}");
                }
                assert_eq!(sevens, 2, "club {club}: seven-game pairings");
                let mut days: Vec<Date> = mine.iter().map(|g| g.date).collect();
                days.sort();
                days.dedup();
                assert_eq!(days.len(), 162, "club {club} plays twice on a day");
            }
            let last = games.iter().map(|g| g.date).max().unwrap();
            assert_eq!(last, Date::new(2027, 9, 26), "season ends {last}");
            assert!(games.iter().all(|g| g.date < all_star || g.date > all_star.add_days(2)), "nothing in the break");
        }
    }
}
