//! The clubs and ballparks the environment plays in.

use bm_engine::{Club, GameSim, MatchEnv, ModelParams, Rng, Rules, Team};
use bm_physics::BallSpec;
use bm_world::{Baseline, Squad, Talent, World};
use std::sync::Arc;

/// The 30 major-league ballparks of the world data under the 2025 MLB rules, and 30 clubs
/// of players generated from the attribute system at major-league talent: the games of
/// `bm validate --world` and `bm tactics --world`.
#[derive(Clone, Debug)]
pub struct League {
    pub envs: Vec<Arc<MatchEnv>>,
    pub clubs: Vec<Club>,
}

/// One game's line-up: the clubs, the park (the home club's) and the two teams.
#[derive(Clone, Debug)]
pub struct Fixture {
    pub away: usize,
    pub home: usize,
    pub env: usize,
    pub teams: [Arc<Team>; 2],
}

impl League {
    /// The league whose clubs are drawn from `seed` (the CLI's clubs for the same seed).
    pub fn mlb(seed: u64) -> Self {
        let rules = Rules::mlb_2025();
        let (envs, _) = World::bundled().park_envs(&["us-major"], &rules, &ModelParams::default(), BallSpec::mlb(), 1.0);
        let mut rng = Rng::new(seed ^ 0xC1B5);
        let base = Baseline::default();
        let clubs = (0..envs.len()).map(|i| Squad::random(&mut rng, 1000 * (i as u32 + 1), &Talent::MAJOR).to_engine_with(&base)).collect();
        League { envs: envs.into_iter().map(Arc::new).collect(), clubs }
    }

    /// Two different clubs at random, in the home club's park, each with a random
    /// starter out of its five-man rotation.
    pub fn fixture(&self, rng: &mut Rng) -> Fixture {
        let n = self.clubs.len();
        let away = rng.below(n);
        let mut home = rng.below(n - 1);
        if home >= away {
            home += 1;
        }
        let env = home % self.envs.len();
        let dh = self.envs[env].rules.dh;
        let teams = [Arc::new(self.clubs[away].team_for_game(rng.below(5), dh)), Arc::new(self.clubs[home].team_for_game(rng.below(5), dh))];
        Fixture { away, home, env, teams }
    }

    /// A new game of the fixture, with both teams run by the default AI managers.
    pub fn start(&self, f: &Fixture) -> GameSim {
        GameSim::new(self.envs[f.env].clone(), f.teams[0].clone(), f.teams[1].clone())
    }
}
