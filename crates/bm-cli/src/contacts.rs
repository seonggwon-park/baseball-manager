//! `bm contacts`: the balls the bat met, with the geometry of the swing that hit them, and
//! the collision model run backwards on them.
//!
//! `bm contacts --out FILE` plays games as `bm validate` does and writes one row per ball
//! the bat met, fair (`res` X) or foul (F): the pitch (type, location, zone, velocity and spin at the plate), the swing
//! as bat tracking measures it (bat speed, attack angle, horizontal direction, tilt of the
//! swing plane, depth of contact), the batted ball (exit speed, launch angle, spray) and the
//! impact parameter the engine drew.
//!
//! `bm contacts --invert FILE --out FILE` reads rows with the same columns, simulated or
//! from pitch tracking, and finds for each the impact parameter with which the collision
//! model sends the ball off at its measured launch angle. The bat is placed as the engine
//! places it: square to the sweet spot's travel in the swing plane, turned further round
//! by the engine's rule for the face of the bat (plus `--face-lead` degrees), and levelled
//! as the wrists roll over.

use bm_engine::{play_game_styled, BattedInfo, Club, ManagerStyle, MatchEnv, Miss, Observer, PitchEvent, PitchResult, PlayKind, Rng};
use bm_physics::{collide, launch_angle, spin_components, spray_angle, BallSpec, BatSpec, Contact, Vec3};
use std::io::{BufRead, BufWriter, Write};
use std::path::Path;

const HEADER: &str = "hand,kind,px,pz,zb,zt,vx,vy,vz,wx,wy,wz,bs,aa,dir,tilt,depth,la,ev,spray,offset,along,ax,ay,az,theta,move_z,move_x,time,res,hit,fbear";

#[derive(Default)]
struct Rows {
    lines: Vec<String>,
}

impl Observer for Rows {
    fn pitch(&mut self, e: &PitchEvent) {
        if e.approach.is_bunt() || !matches!(e.result, PitchResult::InPlay | PitchResult::Foul) {
            return;
        }
        let Some(shape) = e.swing else { return };
        if shape.exit_speed <= 0.0 {
            return;
        }
        // Fouls that never became batted balls (back into the screen, off the end of the
        // bat) carry only the speed and elevation of the ball off the bat.
        let info = e.batted.copied().unwrap_or(BattedInfo {
            exit_speed: shape.exit_speed,
            launch_angle: shape.elevation,
            spray: f64::NAN,
            backspin: f64::NAN,
            sidespin: f64::NAN,
            gyrospin: f64::NAN,
            bat_speed: shape.bat_speed,
            attack_angle: shape.attack,
            bunt: false,
            shape,
        });
        let f = &e.thrown.flight;
        let pull = e.batter_hand.pull_sign();
        self.lines.push(format!(
            "{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.2},{:.2},{:.2},{:.4},{:.5},{:.5},{:.5},{:.4},{:.5},{:.4},{:.5},{:.5},{:.4},{:.5},{:.5},{:.5},{:.5},{:.4},{:.4},{:.4},{},{},{:.4}",
            if e.batter_hand.outside_sign() > 0.0 { 'R' } else { 'L' },
            e.thrown.kind.code(),
            f.plate_pos.x,
            f.plate_pos.z,
            e.zone.bottom,
            e.zone.top,
            f.plate_vel.x,
            f.plate_vel.y,
            f.plate_vel.z,
            f.spin.x,
            f.spin.y,
            f.spin.z,
            info.bat_speed,
            info.attack_angle,
            shape.azimuth,
            shape.tilt,
            shape.depth,
            info.launch_angle,
            info.exit_speed,
            info.spray * pull,
            shape.offset,
            shape.along,
            shape.axis.x,
            shape.axis.y,
            shape.axis.z,
            shape.direction,
            f.move_z,
            f.move_x,
            f.time,
            if e.result == PitchResult::InPlay { 'X' } else { 'F' },
            e.play.map_or(-1, |p| i32::from(matches!(p.kind, PlayKind::Single | PlayKind::Double | PlayKind::Triple | PlayKind::HomeRun | PlayKind::GroundRuleDouble))),
            // Bearing of the spot where the ball was first touched (or left the field), + toward the pull side.
            e.play.map_or(f64::NAN, |p| p.field_pos.x.atan2(p.field_pos.y) * pull),
        ));
    }
}

/// Every full swing (bunts excluded): outcome (W swinging strike, T foul tip, F foul,
/// X in play), bat speed, depth, attack angle, the pitch's descent at the plate, its type,
/// location and zone, and the impact parameter.
#[derive(Default)]
struct Swings {
    lines: Vec<String>,
}

const SWING_HEADER: &str = "result,hand,kind,px,pz,zb,zt,bs,depth,aa,descent,geo,offset,miss,along";

impl Observer for Swings {
    fn pitch(&mut self, e: &PitchEvent) {
        if e.approach.is_bunt() {
            return;
        }
        let Some(shape) = e.swing else { return };
        let code = match e.result {
            PitchResult::SwingingStrike => 'W',
            PitchResult::FoulTip => 'T',
            PitchResult::Foul => 'F',
            PitchResult::InPlay => 'X',
            _ => return,
        };
        let f = &e.thrown.flight;
        let descent = (-f.plate_vel.z / f.plate_vel.y.abs()).atan();
        self.lines.push(format!(
            "{code},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.5},{:.5},{},{:.5},{},{:.4}",
            if e.batter_hand.outside_sign() > 0.0 { 'R' } else { 'L' },
            e.thrown.kind.code(),
            f.plate_pos.x,
            f.plate_pos.z,
            e.zone.bottom,
            e.zone.top,
            shape.bat_speed,
            shape.depth,
            shape.attack,
            descent,
            u8::from(shape.tilt > 0.0),
            shape.offset,
            match e.miss {
                Some(Miss::Early) => 'E',
                Some(Miss::Late) => 'L',
                Some(Miss::OffTheEnd) => 'O',
                Some(Miss::Jammed) => 'J',
                Some(Miss::Over) => 'V',
                Some(Miss::Under) => 'U',
                None => '-',
            },
            shape.along,
        ));
    }
}

trait Lines: Observer + Default + Send {
    fn take(self) -> Vec<String>;
}
impl Lines for Rows {
    fn take(self) -> Vec<String> {
        self.lines
    }
}
impl Lines for Swings {
    fn take(self) -> Vec<String> {
        self.lines
    }
}

/// Play `games` games (as `bm validate` does) and write every ball the bat met to `out`,
/// or every full swing when `swings` is set.
#[allow(clippy::too_many_arguments)]
pub fn dump(envs: &[MatchEnv], clubs: &[Club], style: ManagerStyle, games: usize, seed: u64, threads: usize, out: &Path, swings: bool) {
    let (header, all) = if swings {
        (SWING_HEADER, collect::<Swings>(envs, clubs, style, games, seed, threads))
    } else {
        (HEADER, collect::<Rows>(envs, clubs, style, games, seed, threads))
    };
    let mut w = BufWriter::new(std::fs::File::create(out).expect("cannot create the output file"));
    writeln!(w, "{header}").unwrap();
    for l in &all {
        writeln!(w, "{l}").unwrap();
    }
    eprintln!("{} rows written to {}", all.len(), out.display());
}

fn collect<O: Lines>(envs: &[MatchEnv], clubs: &[Club], style: ManagerStyle, games: usize, seed: u64, threads: usize) -> Vec<String> {
    let per = games.div_ceil(threads);
    let mut master = Rng::new(seed);
    let seeds: Vec<Rng> = (0..threads).map(|_| master.fork()).collect();
    let mut all = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .into_iter()
            .enumerate()
            .map(|(i, mut rng)| {
                let n = per.min(games.saturating_sub(i * per));
                scope.spawn(move || {
                    let mut rows = O::default();
                    for _ in 0..n {
                        let (a, h) = crate::pick_pair(&mut rng, clubs.len());
                        let env = &envs[h % envs.len()];
                        let dh = env.rules.dh;
                        let away = clubs[a].team_for_game(rng.below(5), dh);
                        let home = clubs[h].team_for_game(rng.below(5), dh);
                        play_game_styled(env, &away, &home, [style; 2], &mut rng, &mut rows);
                    }
                    rows.take()
                })
            })
            .collect();
        for h in handles {
            all.extend(h.join().expect("simulation thread panicked"));
        }
    });
    all
}

/// One ball the bat met: the inputs of the inversion.
struct Row {
    /// +1 for a right-handed batter (outside = +x), -1 for a left-hander.
    out: f64,
    vel: Vec3,
    spin: Vec3,
    bat_speed: f64,
    attack: f64,
    /// Horizontal direction of the sweet spot's travel, rad, + = toward the pull side.
    dir: f64,
    tilt: f64,
    la: f64,
}

/// The bat at impact for a swing of the given measured geometry, placed as the engine
/// places it. Returns the sweet spot's direction of travel and the bat's axis.
fn bat_geometry(r: &Row, env: &MatchEnv, face_lead: f64) -> (Vec3, Vec3) {
    let p = &env.params;
    let (sa, ca) = r.attack.sin_cos();
    let (sd, cd) = r.dir.sin_cos();
    let forward = Vec3::new(-r.out * sd * ca, cd * ca, sa);
    let up = (Vec3::Z - forward * forward.z).normalized();
    let side = forward.cross(up) * r.out;
    // A plane holding `forward` is tilted at least as much as `forward` climbs.
    let droop = (r.tilt.cos() / ca).clamp(0.0, 1.0).acos();
    let square = side * droop.cos() - up * droop.sin();
    // The same turn measured within the swing plane, from the direction in that plane that
    // points straight at the pitcher (+ = toward the pull side, as the engine measures it).
    let normal = forward.cross(square).normalized();
    let mut straight = normal.cross(Vec3::X).normalized();
    if straight.y < 0.0 {
        straight = -straight;
    }
    let mut outward = normal.cross(straight);
    if outward.x * r.out < 0.0 {
        outward = -outward;
    }
    let theta = (-forward.dot(outward)).atan2(forward.dot(straight));
    let face = p.face_max * (p.face_gain * theta / p.face_max).tanh();
    let rolled = ((theta - p.roll_from) / p.roll_span).clamp(0.0, 1.0);
    let droop = droop * (1.0 - p.roll_flatten * rolled);
    let square = side * droop.cos() - up * droop.sin();
    let lead = face - theta + face_lead;
    (forward, square * lead.cos() + forward * lead.sin())
}

struct Launch {
    la: f64,
    ev: f64,
    spray: f64,
    backspin: f64,
    sidespin: f64,
}

fn launch(r: &Row, env: &MatchEnv, bat: &BatSpec, forward: Vec3, axis: Vec3, offset: f64) -> Option<Launch> {
    let along = bat.sweet_spot_from_tip;
    let c = Contact { ball_vel: r.vel, ball_spin: r.spin, bat_axis: axis, bat_vel: forward * r.bat_speed, offset, dist_from_tip: along };
    let res = collide(&env.ball, bat, &c).ok()?;
    let (wb, ws, _) = spin_components(res.vel, res.spin);
    let pull = -r.out;
    Some(Launch { la: launch_angle(res.vel), ev: res.vel.norm(), spray: spray_angle(res.vel) * pull, backspin: wb, sidespin: ws * pull })
}

/// Impact parameter that reproduces the row's launch angle; `None` when the angle is out
/// of the model's reach for this swing. Launch angle rises with the impact parameter except
/// at the very edge of the bat, where grazed balls fold back toward the pitch's own
/// direction, so the root is looked for between the lowest and the highest launch.
fn invert_one(r: &Row, env: &MatchEnv, bat: &BatSpec, ball: &BallSpec, face_lead: f64) -> Option<(f64, Launch)> {
    let (forward, axis) = bat_geometry(r, env, face_lead);
    let reach = ball.radius + bat.radius_at(bat.sweet_spot_from_tip) - 1e-4;
    let at = |o: f64| launch(r, env, bat, forward, axis, o).map(|l| l.la);
    const N: usize = 141;
    let grid: Vec<(f64, f64)> = (0..N)
        .filter_map(|i| {
            let o = reach * (2.0 * i as f64 / (N - 1) as f64 - 1.0);
            at(o).map(|la| (o, la))
        })
        .collect();
    let lowest = grid.iter().enumerate().min_by(|a, b| a.1 .1.total_cmp(&b.1 .1))?.0;
    let highest = grid.iter().enumerate().max_by(|a, b| a.1 .1.total_cmp(&b.1 .1))?.0;
    if highest <= lowest {
        return None;
    }
    let k = (lowest..highest).find(|&i| grid[i].1 <= r.la && grid[i + 1].1 >= r.la)?;
    let (mut lo, mut hi) = (grid[k].0, grid[k + 1].0);
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        if at(mid)? < r.la {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let o = 0.5 * (lo + hi);
    launch(r, env, bat, forward, axis, o).map(|l| (o, l))
}

/// Read rows with the columns of [`HEADER`] (extra columns are kept, missing simulated
/// ones may be absent) and write them back with the fitted impact parameter and what the
/// collision model then predicts. `face_lead_deg` turns the face of the bat further toward
/// the pull side than the engine's rule does, to test how much the fit depends on that rule;
/// `dir_shift_deg` turns the whole swing (travel and bat together) toward the pull side.
pub fn invert(env: &MatchEnv, input: &Path, out: &Path, face_lead_deg: f64, dir_shift_deg: f64) {
    let f = std::fs::File::open(input).expect("cannot open the input file");
    let mut lines = std::io::BufReader::new(f).lines();
    let header = lines.next().expect("empty input").expect("unreadable input");
    let cols: Vec<&str> = header.split(',').collect();
    let col = |name: &str| cols.iter().position(|c| *c == name).unwrap_or_else(|| panic!("input has no column {name}"));
    let idx = [col("hand"), col("vx"), col("vy"), col("vz"), col("wx"), col("wy"), col("wz"), col("bs"), col("aa"), col("dir"), col("tilt"), col("la")];
    let bat = BatSpec::mlb_wood();
    let mut w = BufWriter::new(std::fs::File::create(out).expect("cannot create the output file"));
    writeln!(w, "{header},fit_offset,fit_ev,fit_spray,fit_backspin,fit_sidespin").unwrap();
    let (mut n, mut missed) = (0usize, 0usize);
    for line in lines {
        let line = line.expect("unreadable input");
        let v: Vec<&str> = line.split(',').collect();
        let num = |i: usize| v.get(idx[i]).and_then(|s| s.parse::<f64>().ok()).unwrap_or(f64::NAN);
        let row = Row {
            out: if v.get(idx[0]) == Some(&"L") { -1.0 } else { 1.0 },
            vel: Vec3::new(num(1), num(2), num(3)),
            spin: Vec3::new(num(4), num(5), num(6)),
            bat_speed: num(7),
            attack: num(8),
            dir: num(9) + dir_shift_deg.to_radians(),
            tilt: num(10),
            la: num(11),
        };
        n += 1;
        let ok = [row.bat_speed, row.attack, row.dir, row.tilt, row.la, row.vel.y].iter().all(|x| x.is_finite());
        match ok.then(|| invert_one(&row, env, &bat, &env.ball, face_lead_deg.to_radians())).flatten() {
            Some((o, l)) => {
                writeln!(w, "{line},{:.5},{:.4},{:.5},{:.1},{:.1}", o, l.ev, l.spray, l.backspin, l.sidespin).unwrap();
            }
            None => {
                missed += 1;
                writeln!(w, "{line},,,,,").unwrap();
            }
        }
    }
    eprintln!("{n} rows, {missed} without a solution, written to {}", out.display());
}
