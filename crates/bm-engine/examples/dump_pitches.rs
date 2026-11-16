//! Dump the pitches of simulated games as CSV, for comparing location
//! distributions with pitch-tracking data.
//!
//! cargo run --release -p bm-engine --example dump_pitches -- [games] [seed] > pitches.csv

use bm_engine::{play_game, Club, Hand, Level, MatchEnv, Observer, PitchEvent, Rng};
use std::io::{BufWriter, Write};

struct Dump<W: Write> {
    out: W,
}

impl<W: Write> Observer for Dump<W> {
    fn pitch(&mut self, e: &PitchEvent) {
        let f = &e.thrown.flight;
        let hand = |h: Hand| if h == Hand::Right { 'R' } else { 'L' };
        let _ = writeln!(
            self.out,
            "{},{},{},{},{},{:.4},{:.4},{:.4},{:.4},{}",
            e.balls,
            e.strikes,
            e.thrown.kind.code(),
            hand(e.pitcher_hand),
            hand(e.batter_hand),
            f.plate_pos.x,
            f.plate_pos.z,
            e.zone.bottom,
            e.zone.top,
            e.pitcher
        );
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let games: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(300);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(7);
    let env = MatchEnv::neutral_mlb();
    let level = Level::mlb();
    let mut rng = Rng::new(seed);
    let clubs: Vec<Club> = (0..30).map(|i| Club::random(&mut rng, 1000 * (i + 1), &level)).collect();
    let stdout = std::io::stdout();
    let mut dump = Dump { out: BufWriter::new(stdout.lock()) };
    let _ = writeln!(dump.out, "balls,strikes,pitch_type,p_throws,stand,plate_x,plate_z,sz_bot,sz_top,pitcher");
    for _ in 0..games {
        let a = rng.below(clubs.len());
        let h = (a + 1 + rng.below(clubs.len() - 1)) % clubs.len();
        let away = clubs[a].team_for_game(rng.below(5), env.rules.dh);
        let home = clubs[h].team_for_game(rng.below(5), env.rules.dh);
        play_game(&env, &away, &home, &mut rng, &mut dump);
    }
}
