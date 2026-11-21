//! Side-by-side comparison of simulated aggregates with the measured 2025 MLB values.
//!
//! Targets come from two public sources:
//! * MLB Stats API, 2025 regular-season team totals (league-wide rates), and
//! * a 30-day, 126,266-pitch sample of 2025 Statcast pitch data from Baseball
//!   Savant (plate discipline, batted-ball and bat-tracking distributions).

// The tables below pair simulated arrays with constant target arrays by index.
#![allow(clippy::needless_range_loop)]

use crate::tally::{Tally, CALL_EDGES, DEPTH_EDGES, DIST_EDGES, EFF_BAT_EDGES, EFF_LA_EDGES, ELEV_EDGES, NB, NC, ND, NE, REGIONS, REGION_NAMES};
use bm_engine::PitchKind;

pub struct Row {
    pub label: String,
    pub sim: f64,
    pub target: f64,
}

fn pct(a: f64, b: f64) -> f64 {
    if b > 0.0 {
        100.0 * a / b
    } else {
        f64::NAN
    }
}

fn line(label: &str, sim: f64, target: f64, digits: usize) {
    let diff = sim - target;
    let rel = if target.abs() > 1e-9 { 100.0 * diff / target } else { f64::NAN };
    println!("  {:<34} {:>9.d$} {:>9.d$} {:>+9.d$} {:>+7.1}%", label, sim, target, diff, rel, d = digits);
}

fn header(title: &str) {
    println!("\n== {title}");
    println!("  {:<34} {:>9} {:>9} {:>9} {:>8}", "", "sim", "MLB 2025", "diff", "rel");
}

/// League-level rates (MLB Stats API, 2025 regular season).
pub fn league_rows(t: &Tally) -> Vec<Row> {
    let ab = t.pa - t.bb - t.hbp - t.sf;
    let h = t.singles + t.doubles + t.triples + t.hr;
    let tb = t.singles + 2.0 * t.doubles + 3.0 * t.triples + 4.0 * t.hr;
    let r = |label: &str, sim: f64, target: f64| Row { label: label.to_string(), sim, target };
    vec![
        r("R / team game", t.runs / t.team_games, 4.447),
        r("PA / team game", t.pa / t.team_games, 37.64),
        r("AVG", h / ab, 0.2452),
        r("OBP", (h + t.bb + t.hbp) / (ab + t.bb + t.hbp + t.sf), 0.3152),
        r("SLG", tb / ab, 0.4038),
        r("BABIP", (h - t.hr) / (ab - t.so - t.hr + t.sf), 0.2906),
        r("K %", pct(t.so, t.pa), 22.22),
        r("BB %", pct(t.bb, t.pa), 8.41),
        r("HBP %", pct(t.hbp, t.pa), 1.054),
        r("HR %", pct(t.hr, t.pa), 3.089),
        r("1B %", pct(t.singles, t.pa), 14.28),
        r("2B %", pct(t.doubles, t.pa), 4.234),
        r("3B %", pct(t.triples, t.pa), 0.343),
        r("SF %", pct(t.sf, t.pa), 0.714),
        r("ROE % (Statcast sample)", pct(t.roe, t.pa), 0.54),
        r("GIDP / team game", t.gidp / t.team_games, 0.642),
        r("pitches / PA", t.pa_pitches / t.pa, 3.882),
        r("SB attempts / team game", (t.sb + t.cs) / t.team_games, 0.911),
        r("SB success %", pct(t.sb, t.sb + t.cs), 77.67),
    ]
}

pub fn print(t: &Tally) {
    header("League rates (MLB Stats API, 2025 regular season)");
    for row in league_rows(t) {
        let digits = if row.target < 1.0 { 4 } else { 3 };
        line(&row.label, row.sim, row.target, digits);
    }
    println!(
        "  wild pitches / team game {:.3}; runners thrown out on the bases / team game {:.3}; pitchers used / team game {:.2}",
        t.wp / t.team_games,
        t.runner_outs / t.team_games,
        1.0 + t.pitchers_used / t.team_games
    );
    println!("  innings / game {:.3}; ties {:.2} %", t.innings / t.games, pct(t.ties, t.games));

    header("Tactics (MLB Stats API totals; bunt details from the Statcast sample)");
    let x = &t.tactics;
    line("sacrifice bunts / team game", x.sac_bunts / t.team_games, 0.1152, 4);
    line("intentional walks / team game", x.intentional_walks / t.team_games, 0.1144, 4);
    let offers: f64 = x.bunt_pitches[1..].iter().sum();
    line("bunt attempts: missed %", pct(x.bunt_pitches[1], offers), 8.35, 2);
    line("bunt attempts: fouled off %", pct(x.bunt_pitches[2], offers), 50.77, 2);
    line("bunt attempts: in play %", pct(x.bunt_pitches[3], offers), 40.88, 2);
    line("bunts in play: exit velocity, mph", x.bunt_ev.mean(), 33.8, 2);
    line("bunts in play: exit velocity SD", x.bunt_ev.sd(), 8.0, 2);
    line("bunts in play: launch angle, deg", x.bunt_la.mean(), -22.6, 2);
    line("bunts in play: popped up (> 20 deg) %", pct(x.bunt_popped, x.bunt_pitches[3]), 14.6, 2);
    let s = &x.sac_situation;
    line("runner on first, < 2 outs: sacrifice %", pct(s[1], s[0]), 71.6, 2);
    line("   ... bunt hit %", pct(s[2], s[0]), 15.8, 2);
    line("   ... batter out, nobody advances %", pct(s[3], s[0]), 8.4, 2);
    line("   ... lead runner out %", pct(s[4], s[0]), 4.2, 2);
    line("bases empty: bunt hit %", pct(x.empty_situation[1], x.empty_situation[0]), 51.0, 2);
    let total: f64 = x.bunt_fielder.iter().sum();
    let share = |i: usize| pct(x.bunt_fielder[i], total);
    println!(
        "  bunts fielded by, % (sim / MLB): P {:.0}/48  C {:.0}/9  1B {:.0}/11  3B {:.0}/32  others {:.0}/1",
        share(0),
        share(1),
        share(2),
        share(4),
        100.0 - share(0) - share(1) - share(2) - share(4)
    );
    println!(
        "  bunts in play / team game {:.3} (MLB about 0.24); pulled back {:.0} % of bunt pitches; reached on error {:.1} % of sacrifice situations",
        x.bunt_pitches[3] / t.team_games,
        pct(x.bunt_pitches[0], offers + x.bunt_pitches[0]),
        pct(s[5], s[0])
    );
    println!(
        "  hit-and-run pitches / team game {:.3}; pinch hitters / team game {:.3}; pinch runners / team game {:.3}; steals of third: {:.3} attempts / team game, {:.0} % safe",
        x.hit_and_run_pitches / t.team_games,
        x.pinch_hitters / t.team_games,
        x.pinch_runners / t.team_games,
        x.steals_of_third[0] / t.team_games,
        pct(x.steals_of_third[1], x.steals_of_third[0])
    );

    header("Pitch outcomes, % of pitches (Statcast sample)");
    line("ball", pct(t.res[0], t.pitches), 35.70, 2);
    line("called strike", pct(t.res[1], t.pitches), 16.22, 2);
    line("swinging strike (incl. tips)", pct(t.res[2] + t.res[4], t.pitches), 12.11, 2);
    line("foul", pct(t.res[3], t.pitches), 18.12, 2);
    line("in play", pct(t.res[5], t.pitches), 17.58, 2);
    line("hit by pitch", pct(t.res[6], t.pitches), 0.274, 3);

    let wh: f64 = t.miss.iter().sum();
    println!(
        "  swings and misses by cause: early {:.1} %, late {:.1} %, off the end {:.1} %, jammed {:.1} %, over the ball {:.1} %, under the ball {:.1} %",
        pct(t.miss[0], wh),
        pct(t.miss[1], wh),
        pct(t.miss[2], wh),
        pct(t.miss[3], wh),
        pct(t.miss[4], wh),
        pct(t.miss[5], wh)
    );

    header("Plate discipline (Statcast sample)");
    line("Zone %", pct(t.zone, t.pitches), 50.73, 2);
    line("Swing %", pct(t.swings, t.pitches), 47.97, 2);
    line("Z-Swing %", pct(t.z_swings, t.z_n), 67.08, 2);
    line("O-Swing %", pct(t.o_swings, t.o_n), 28.29, 2);
    line("Contact %", pct(t.contact, t.swings), 74.67, 2);
    line("Z-Contact %", pct(t.z_contact, t.z_swings), 82.83, 2);
    line("O-Contact %", pct(t.o_contact, t.o_swings), 54.73, 2);
    line("foul share of contact %", pct(t.fouls_of_contact, t.contact), 50.77, 2);
    line("called strike | take in zone %", pct(t.cs_z, t.takes_z), 87.71, 2);
    line("called strike | take out of zone %", pct(t.cs_o, t.takes_o), 4.54, 2);

    println!("\n== By count: sim / MLB (share of pitches, zone %, swing %, Z-swing %, O-swing %, fastball %)");
    #[rustfmt::skip]
    let target: [[[f64; 5]; 3]; 4] = [
        [[25.78, 54.9, 32.1, 45.1, 16.2], [12.99, 46.5, 49.8, 74.8, 28.2], [6.75, 32.7, 50.6, 87.2, 32.7]],
        [[9.73, 56.5, 43.1, 58.6, 22.9], [10.06, 50.9, 54.3, 77.0, 30.7], [9.82, 38.9, 57.3, 88.2, 37.6]],
        [[3.24, 59.6, 41.1, 55.9, 19.3], [5.19, 57.5, 58.4, 77.1, 33.2], [8.38, 48.1, 64.9, 89.6, 42.1]],
        [[0.99, 63.9, 7.9, 10.9, 2.4], [2.09, 64.0, 55.1, 71.8, 25.6], [4.98, 59.6, 70.2, 89.2, 42.1]],
    ];
    for b in 0..4 {
        for s in 0..3 {
            let n = t.count_n[b][s];
            let on = n - t.count_zn[b][s];
            let tg = target[b][s];
            println!(
                "  {b}-{s}  share {:5.2}/{:5.2}  zone {:5.1}/{:5.1}  swing {:5.1}/{:5.1}  zsw {:5.1}/{:5.1}  osw {:5.1}/{:5.1}  fastballs {:5.1}",
                pct(n, t.pitches),
                tg[0],
                pct(t.count_zone[b][s], n),
                tg[1],
                pct(t.count_swing[b][s], n),
                tg[2],
                pct(t.count_zswing[b][s], t.count_zn[b][s]),
                tg[3],
                pct(t.count_oswing[b][s], on),
                tg[4],
                pct(t.count_fast[b][s], n)
            );
        }
    }

    println!("\n== Swing % by distance outside the zone edge (in): sim / MLB for all counts, 0, 1, 2 strikes");
    #[rustfmt::skip]
    let sw_t: [[f64; 4]; NB] = [
        [76.4, 58.0, 87.8, 97.1], [74.6, 57.3, 84.5, 94.4], [70.3, 51.8, 79.8, 91.9], [64.4, 44.8, 72.7, 87.5],
        [60.2, 39.7, 66.8, 83.0], [54.6, 35.0, 59.8, 78.1], [48.2, 31.0, 53.1, 68.1], [43.7, 27.1, 46.3, 64.7],
        [39.9, 23.3, 41.5, 59.7], [35.1, 19.8, 35.4, 53.7], [29.7, 16.9, 29.0, 44.4], [21.8, 11.8, 20.1, 33.6],
        [15.7, 7.8, 14.5, 24.2], [10.0, 5.6, 9.5, 13.6], [5.3, 2.0, 5.5, 7.0], [2.5, 1.3, 2.2, 3.1],
    ];
    let con_t: [f64; NB] = [88.8, 86.8, 84.0, 81.3, 77.9, 74.5, 70.3, 65.3, 60.3, 56.8, 49.1, 37.5, 28.1, 13.9, 2.7, 2.1];
    for i in 0..NB {
        let mut s = format!("  [{:5.1},{:5.1})", DIST_EDGES[i], DIST_EDGES[i + 1]);
        for g in 0..4 {
            s += &format!("  {:5.1}/{:5.1}", pct(t.swing_dist_s[g][i], t.swing_dist_n[g][i]), sw_t[i][g]);
        }
        s += &format!(
            "   contact|swing {:5.1}/{:5.1}  (pitch share {:4.1} %)",
            pct(t.contact_dist_c[i], t.contact_dist_n[i]),
            con_t[i],
            pct(t.swing_dist_n[0][i], t.pitches)
        );
        println!("{s}");
    }

    println!("\n== Called-strike % on takes by distance outside the zone edge (in): sim / MLB");
    let call_t: [f64; NC] = [100.0, 99.0, 95.7, 90.7, 82.4, 75.1, 61.8, 51.1, 32.5, 25.4, 18.0, 11.6, 5.4, 1.7, 0.2];
    let mut s = String::new();
    for i in 0..NC {
        s += &format!("  [{:4.1},{:4.1}) {:5.1}/{:5.1}", CALL_EDGES[i], CALL_EDGES[i + 1], pct(t.call_s[i], t.call_n[i]), call_t[i]);
        if i % 4 == 3 {
            s += "\n";
        }
    }
    println!("{s}");

    println!("\n== Pitch types: usage % sim/MLB, velocity mph sim/MLB, whiff per swing % sim/MLB, zone % sim/MLB, O-swing % sim/MLB, EV, LA, GB %  sim/MLB");
    #[rustfmt::skip]
    let kt: [(PitchKind, [f64; 8]); 8] = [
        (PitchKind::FourSeam, [31.66, 94.5, 21.7, 56.7, 23.6, 91.2, 20.5, 31.0]),
        (PitchKind::Sinker, [15.55, 93.8, 13.9, 58.1, 25.1, 89.3, 5.0, 57.2]),
        (PitchKind::Cutter, [7.24, 89.9, 21.8, 53.5, 26.0, 87.8, 15.6, 38.6]),
        (PitchKind::Slider, [14.54, 86.3, 33.0, 47.3, 32.0, 87.9, 13.4, 41.4]),
        (PitchKind::Sweeper, [7.67, 82.5, 32.7, 45.3, 30.7, 85.7, 19.1, 35.2]),
        (PitchKind::Curve, [8.73, 79.9, 29.9, 44.2, 27.9, 88.6, 10.4, 46.8]),
        (PitchKind::Changeup, [10.45, 86.0, 30.9, 39.8, 34.0, 86.8, 8.9, 48.3]),
        (PitchKind::Splitter, [3.21, 86.4, 36.1, 36.1, 34.6, 87.1, 7.0, 51.8]),
    ];
    for (kind, tg) in kt {
        let k = &t.kinds[kind.index()];
        println!(
            "  {}  use {:5.1}/{:5.1}  v {:5.1}/{:5.1}  whiff {:5.1}/{:5.1}  zone {:5.1}/{:5.1}  osw {:5.1}/{:5.1}  EV {:5.1}/{:5.1}  LA {:5.1}/{:5.1}  GB {:5.1}/{:5.1}",
            kind.code(),
            pct(k.n, t.pitches),
            tg[0],
            k.speed.mean(),
            tg[1],
            pct(k.whiffs, k.swings),
            tg[2],
            pct(k.zone, k.n),
            tg[3],
            pct(k.out_swings, k.out_n),
            tg[4],
            k.ev.mean(),
            tg[5],
            k.la.mean(),
            tg[6],
            pct(k.gb, k.bip),
            tg[7]
        );
    }
    // MLB: swing % at pitches in the zone and called strikes, % of pitches (2025 Statcast sample:
    // the rulebook zone widened by the ball's radius).
    #[rustfmt::skip]
    let kc: [[f64; 2]; 8] = [[68.8, 17.0], [60.2, 22.4], [68.1, 16.2], [69.0, 14.3], [64.7, 16.1], [57.7, 18.7], [76.7, 9.1], [80.4, 7.0]];
    println!("  zone swing % sim/MLB, called strikes % of pitches sim/MLB:");
    for ((kind, _), tg) in kt.iter().zip(kc) {
        let k = &t.kinds[kind.index()];
        println!("    {}  zsw {:5.1}/{:5.1}  called {:5.1}/{:5.1}", kind.code(), pct(k.zone_swings, k.zone), tg[0], pct(k.called, k.n), tg[1]);
    }
    // MLB: whiff per swing at pitches in the zone, against same-side and opposite-side batters.
    #[rustfmt::skip]
    let kz: [[f64; 3]; 8] = [[18.2, 23.4, 20.6], [10.8, 13.5, 14.7], [16.1, 23.8, 20.6], [19.3, 34.7, 30.7], [19.5, 35.5, 27.3], [15.3, 30.2, 29.9], [20.5, 34.2, 30.1], [22.6, 38.1, 35.4]];
    println!("  whiff per swing, sim/MLB: in the zone, against same-side batters, against opposite-side batters; sim misses by cause, % of swings");
    for ((kind, _), tg) in kt.iter().zip(kz) {
        let k = &t.kinds[kind.index()];
        println!(
            "    {}  zone {:5.1}/{:5.1}  same {:5.1}/{:5.1}  opposite {:5.1}/{:5.1}   early {:4.1}  late {:4.1}  off the end {:4.1}  jammed {:4.1}  over {:4.1}  under {:4.1}",
            kind.code(),
            pct(k.zone_whiffs, k.zone_swings),
            tg[0],
            pct(k.side[0][1], k.side[0][0]),
            tg[1],
            pct(k.side[1][1], k.side[1][0]),
            tg[2],
            pct(k.miss[0], k.swings),
            pct(k.miss[1], k.swings),
            pct(k.miss[2], k.swings),
            pct(k.miss[3], k.swings),
            pct(k.miss[4], k.swings),
            pct(k.miss[5], k.swings)
        );
    }
    // MLB (Statcast sample, 9-parameter fit): spin-induced vertical and arm-side movement
    // over the whole flight from release to the plate, in; mean plate height, ft.
    const MLB_RISE: [f64; 8] = [16.6, 7.9, 8.6, 2.3, 1.1, -10.3, 5.0, 4.0];
    const MLB_ARM: [f64; 8] = [9.1, 16.8, -0.9, -2.9, -12.7, -8.4, 16.0, 12.0];
    #[rustfmt::skip]
    let km: [[f64; 3]; 8] = [[MLB_RISE[0], MLB_ARM[0], 2.87], [MLB_RISE[1], MLB_ARM[1], 2.38], [MLB_RISE[2], MLB_ARM[2], 2.53], [MLB_RISE[3], MLB_ARM[3], 1.93],
                             [MLB_RISE[4], MLB_ARM[4], 2.01], [MLB_RISE[5], MLB_ARM[5], 1.85], [MLB_RISE[6], MLB_ARM[6], 1.83], [MLB_RISE[7], MLB_ARM[7], 1.77]];
    println!("  movement over the whole flight (in) and plate height (ft), sim/MLB: induced rise, arm-side, height");
    for ((kind, _), tg) in kt.iter().zip(km) {
        let k = &t.kinds[kind.index()];
        println!(
            "    {}  rise {:5.1}/{:5.1}  arm side {:5.1}/{:5.1}  height {:4.2}/{:4.2}",
            kind.code(),
            k.rise.mean(),
            tg[0],
            k.arm_side.mean(),
            tg[1],
            k.height.mean(),
            tg[2]
        );
    }
    // MLB (Statcast sample): plate location toward the outside edge, ft, and zone %,
    // against same-side and opposite-side batters.
    #[rustfmt::skip]
    let kx: [[f64; 4]; 8] = [[0.161, 57.4, 0.123, 56.3], [-0.161, 57.8, 0.165, 58.6], [0.458, 52.7, -0.194, 54.0], [0.543, 45.9, -0.127, 49.3],
                             [0.642, 43.7, -0.008, 48.0], [0.338, 44.0, 0.040, 44.3], [-0.050, 41.8, 0.449, 39.4], [-0.059, 36.6, 0.357, 35.9]];
    println!("  location by the batter's side, sim/MLB: toward the outside edge (ft) and zone %, same-side batters, then opposite-side");
    for ((kind, _), tg) in kt.iter().zip(kx) {
        let k = &t.kinds[kind.index()];
        println!(
            "    {}  same  {:+5.2}/{:+5.2}  zone {:4.1}/{:4.1}   opposite  {:+5.2}/{:+5.2}  zone {:4.1}/{:4.1}",
            kind.code(),
            k.side_x[0].mean(),
            tg[0],
            pct(k.side_zone[0][1], k.side_zone[0][0]),
            tg[1],
            k.side_x[1].mean(),
            tg[2],
            pct(k.side_zone[1][1], k.side_zone[1][0]),
            tg[3]
        );
    }
    println!("  four-seam whiff per swing by velocity, sim/MLB:");
    let fb_t = [17.4, 19.2, 19.3, 21.5, 24.5, 25.4, 30.3, f64::NAN];
    let mut s = String::from("   ");
    for i in 0..7 {
        s += &format!(" [{}-{}) {:4.1}/{:4.1}", 88 + 2 * i, 90 + 2 * i, pct(t.whiff_fb_speed[i][1], t.whiff_fb_speed[i][0]), fb_t[i]);
    }
    println!("{s}");
    println!("  whiff per swing by pitch height (ft), sim/MLB for fastballs, breaking, offspeed:");
    #[rustfmt::skip]
    let wh_t: [[f64; 3]; 8] = [[74.3, 82.3, 76.4], [36.6, 57.4, 52.9], [16.4, 30.4, 31.1], [11.5, 18.6, 19.1], [14.5, 14.1, 15.6], [24.6, 23.9, 20.1], [42.6, 39.1, f64::NAN], [57.7, f64::NAN, f64::NAN]];
    for (i, row) in wh_t.iter().enumerate() {
        let mut s = format!("    z [{:.1},{:.1})", 0.5 + 0.5 * i as f64, 1.0 + 0.5 * i as f64);
        for (g, tg) in row.iter().enumerate() {
            s += &format!("  {:5.1}/{:5.1}", pct(t.whiff_height[g][i][1], t.whiff_height[g][i][0]), tg);
        }
        println!("{s}");
    }

    header("Batted balls in play (Statcast sample)");
    line("exit velocity mean, mph", t.ev.mean(), 88.91, 2);
    line("exit velocity SD", t.ev.sd(), 14.95, 2);
    line("launch angle mean, deg", t.la.mean(), 13.14, 2);
    line("launch angle SD", t.la.sd(), 28.67, 2);
    line("hard-hit % (>= 95 mph)", pct(t.hard, t.bip), 41.18, 2);
    line("barrel %", pct(t.barrel, t.bip), 8.55, 2);
    line("sweet-spot % (8-32 deg)", pct(t.sweet, t.bip), 35.87, 2);
    line("bat speed on balls in play, mph", t.bat_speed_bip.mean(), 71.19, 2);
    line("bat speed, all swings", t.bat_speed_all.mean(), 69.89, 2);
    line("bat speed SD, all swings", t.bat_speed_all.sd(), 8.37, 2);
    let q = [0.05, 0.25, 0.50, 0.75, 0.95];
    let evp: Vec<String> = q.iter().map(|&x| format!("{:.1}", Tally::percentile(&t.ev_hist, 0.0, 1.0, x))).collect();
    let lap: Vec<String> = q.iter().map(|&x| format!("{:.1}", Tally::percentile(&t.la_hist, -90.0, 1.0, x))).collect();
    println!("  EV percentiles 5/25/50/75/95: sim [{}]  MLB [61.2 80.6 92.0 100.1 107.3]", evp.join(" "));
    println!("  LA percentiles 5/25/50/75/95: sim [{}]  MLB [-36.0 -5.0 14.0 31.0 62.0]", lap.join(" "));
    if t.spin_fit[0] > 10.0 {
        let n = t.spin_fit[0];
        let slope = (n * t.spin_fit[4] - t.spin_fit[1] * t.spin_fit[2]) / (n * t.spin_fit[3] - t.spin_fit[1].powi(2));
        let icpt = (t.spin_fit[2] - slope * t.spin_fit[1]) / n;
        println!("  backspin vs launch angle (0-40 deg): sim {icpt:.0} + {slope:.0} * LA rpm   Statcast fit (Nathan) -763 + 120 * LA rpm");
        // Multiple regression on launch angle and pull-side direction (Nathan: +21 rpm per
        // degree toward the opposite field).
        let f = &t.spin_fit3;
        let (n, mx, mz, my) = (f[0], f[1] / f[0], f[2] / f[0], f[3] / f[0]);
        let (sxx, szz, sxz) = (f[4] / n - mx * mx, f[5] / n - mz * mz, f[6] / n - mx * mz);
        let (sxy, szy) = (f[7] / n - mx * my, f[8] / n - mz * my);
        let det = sxx * szz - sxz * sxz;
        let (b_la, b_pull) = ((sxy * szz - szy * sxz) / det, (szy * sxx - sxy * sxz) / det);
        let a = my - b_la * mx - b_pull * mz;
        println!("  backspin vs launch angle and pull-side direction (0-40 deg): sim {a:.0} + {b_la:.0} * LA {b_pull:+.0} * pull rpm   Statcast fit (Nathan) -763 + 120 * LA -21 * pull rpm");
    }

    if t.side_fit[0] > 10.0 {
        let n = t.side_fit[0];
        let slope = (n * t.side_fit[4] - t.side_fit[1] * t.side_fit[2]) / (n * t.side_fit[3] - t.side_fit[1].powi(2));
        let icpt = (t.side_fit[2] - slope * t.side_fit[1]) / n;
        println!("  hook spin vs pull-side direction (LA 10-40 deg): sim {icpt:.0} + {slope:.0} * angle rpm   Statcast fit (Nathan) -849 + 94 * angle rpm");
    }

    header("Collision efficiency: exit velocity / (1.23 * bat speed + 0.23 * plate speed), balls in play (Statcast sample)");
    line("mean", t.eff_all.ratio.mean(), 0.8350, 4);
    line("SD", t.eff_all.ratio.sd(), 0.1240, 4);
    line("share at or above 0.80 (squared up), %", 100.0 * t.eff_all.share_at_least(0.80), 66.29, 2);
    line("share at or above 0.90, %", 100.0 * t.eff_all.share_at_least(0.90), 36.59, 2);
    line("share at or above 0.95, %", 100.0 * t.eff_all.share_at_least(0.95), 17.80, 2);
    line("share at or above 1.00, %", 100.0 * t.eff_all.share_at_least(1.00), 2.97, 2);
    let q = [0.05, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99];
    let rp: Vec<String> = q.iter().map(|&x| format!("{:.3}", t.eff_all.percentile(x))).collect();
    println!("  percentiles 5/25/50/75/90/95/99: sim [{}]  MLB [0.603 0.759 0.860 0.932 0.972 0.991 1.017]", rp.join(" "));
    // MLB: EV, bat speed, mean ratio, median, 90th, 99th percentile.
    const EFF_LA_MLB: [[f64; 6]; 7] = [
        [75.6, 69.6, 0.724, 0.738, 0.922, 1.008],
        [89.8, 71.1, 0.845, 0.872, 0.976, 1.022],
        [93.4, 71.7, 0.873, 0.905, 0.989, 1.021],
        [93.9, 71.6, 0.878, 0.908, 0.986, 1.021],
        [91.6, 72.0, 0.853, 0.875, 0.962, 1.003],
        [86.6, 72.2, 0.804, 0.822, 0.911, 0.967],
        [81.0, 72.5, 0.751, 0.760, 0.859, 0.920],
    ];
    const EFF_BAT_MLB: [[f64; 6]; 6] = [
        [72.6, -0.4, 0.827, 0.862, 1.001, 1.093],
        [81.8, 8.7, 0.842, 0.871, 0.991, 1.031],
        [86.8, 11.7, 0.844, 0.870, 0.986, 1.021],
        [90.3, 14.5, 0.841, 0.866, 0.972, 1.011],
        [93.2, 16.3, 0.832, 0.855, 0.957, 0.997],
        [94.2, 17.5, 0.802, 0.823, 0.934, 0.977],
    ];
    println!("  by launch angle (sim / MLB): EV, bat speed, mean ratio, median, 90th, 99th percentile");
    for (i, (e, m)) in t.eff_la.iter().zip(&EFF_LA_MLB).enumerate() {
        println!(
            "    LA [{:4.0},{:4.0})  EV {:5.1}/{:5.1}  bat {:5.1}/{:5.1}  mean {:.3}/{:.3}  p50 {:.3}/{:.3}  p90 {:.3}/{:.3}  p99 {:.3}/{:.3}",
            EFF_LA_EDGES[i],
            EFF_LA_EDGES[i + 1],
            e.ev.mean(),
            m[0],
            e.bat.mean(),
            m[1],
            e.ratio.mean(),
            m[2],
            e.percentile(0.5),
            m[3],
            e.percentile(0.9),
            m[4],
            e.percentile(0.99),
            m[5]
        );
    }
    println!("  by bat speed (sim / MLB): share of balls in play, EV, mean launch angle, mean ratio, median, 90th, 99th percentile");
    const EFF_BAT_SHARE_MLB: [f64; 6] = [2.79, 10.22, 20.95, 32.45, 23.94, 9.48];
    for (i, (e, m)) in t.eff_bat.iter().zip(&EFF_BAT_MLB).enumerate() {
        println!(
            "    bat [{:2.0},{:2.0}) mph  share {:5.2}/{:5.2}  EV {:5.1}/{:5.1}  LA {:5.1}/{:5.1}  mean {:.3}/{:.3}  p50 {:.3}/{:.3}  p90 {:.3}/{:.3}  p99 {:.3}/{:.3}",
            EFF_BAT_EDGES[i],
            EFF_BAT_EDGES[i + 1],
            pct(e.ratio.n, t.eff_all.ratio.n),
            EFF_BAT_SHARE_MLB[i],
            e.ev.mean(),
            m[0],
            e.la.mean(),
            m[1],
            e.ratio.mean(),
            m[2],
            e.percentile(0.5),
            m[3],
            e.percentile(0.9),
            m[4],
            e.percentile(0.99),
            m[5]
        );
    }

    header("Swing geometry: full swings of 60 mph and up (Statcast bat tracking)");
    let g = &t.geo;
    let slow_all = pct(g.slow.iter().sum::<f64>(), g.all.iter().sum::<f64>());
    line("swings under 60 mph, % of all swings", slow_all, 6.9, 2);
    line("   ... of swings and misses", pct(g.slow[0], g.all[0]), 13.3, 2);
    line("   ... of fouls", pct(g.slow[1], g.all[1]), 6.1, 2);
    line("   ... of balls in play", pct(g.slow[2], g.all[2]), 3.5, 2);
    // MLB by strikes: miss %, caught tip %, foul %, in play %, bat speed, share under 60 mph.
    const STRIKES_MLB: [[f64; 6]; 3] = [[23.8, 2.2, 37.0, 37.0, 71.0, 5.3], [23.5, 2.1, 37.3, 37.1, 70.3, 6.0], [22.3, 2.4, 38.6, 36.7, 68.7, 8.8]];
    for s in 0..3 {
        let c = &g.by_strikes[s];
        let n: f64 = c[..4].iter().sum();
        let m = &STRIKES_MLB[s];
        println!(
            "  swings with {} strikes: miss {:5.1}/{:5.1}  caught tip {:4.1}/{:4.1}  foul {:5.1}/{:5.1}  in play {:5.1}/{:5.1}  bat speed {:5.1}/{:5.1}  under 60 mph {:4.1}/{:4.1} %",
            s,
            pct(c[0], n),
            m[0],
            pct(c[1], n),
            m[1],
            pct(c[2], n),
            m[2],
            pct(c[3], n),
            m[3],
            g.speed_by_strikes[s].mean(),
            m[4],
            pct(c[4], n),
            m[5]
        );
    }
    // MLB: bat speed mean and SD, depth SD (in), direction SD (deg) for whiffs, fouls, balls in play.
    const GEO_MLB: [[f64; 4]; 3] = [[71.54, 4.73, 15.49, 24.72], [71.11, 4.86, 10.38, 17.19], [72.01, 4.71, 7.69, 12.00]];
    for (i, name) in ["swing and miss", "foul", "in play"].iter().enumerate() {
        println!(
            "  {:<15} bat speed {:5.2}/{:5.2} (SD {:4.2}/{:4.2})  depth mean {:5.1} in, SD {:5.2}/{:5.2}  direction mean {:5.1}, SD {:5.2}/{:5.2} deg",
            name,
            g.speed[i].mean(),
            GEO_MLB[i][0],
            g.speed[i].sd(),
            GEO_MLB[i][1],
            g.depth[i].mean(),
            g.depth[i].sd(),
            GEO_MLB[i][2],
            g.direction[i].mean(),
            g.direction[i].sd(),
            GEO_MLB[i][3]
        );
    }
    line("attack angle on balls in play, deg", g.attack.mean(), 8.78, 2);
    // Statcast sample, full swings: swinging strikes 14.70, fouls 8.57, balls in play 8.77 deg.
    for (i, (name, mlb)) in
        [("attack angle, swings and misses", 14.70), ("attack angle, fouls", 8.57), ("attack angle, balls in play (all)", 8.77)].into_iter().enumerate()
    {
        line(name, g.attack_by[i].mean(), mlb, 2);
    }
    line("attack angle SD", g.attack.sd(), 8.36, 2);
    line("pull-side spray off the bat, mean deg", g.spray.mean(), 5.6, 2);
    line("pull-side spray SD", g.spray.sd(), 31.6, 2);
    // (label, MLB slope, MLB r)
    const FITS_MLB: [(&str, f64, f64); 7] = [
        ("bat speed (mph) per inch of depth", 0.168, 0.27),
        ("attack angle (deg) per inch of depth", 0.706, 0.65),
        ("direction toward pull (deg) per inch of depth", 1.264, 0.81),
        ("pull-side spray per degree of direction", 1.09, 0.41),
        ("depth (in) per inch of inside location", 0.000, 0.00),
        ("direction (deg) per inch of inside location", 0.218, 0.11),
        ("pull-side spray (deg) per inch of inside location", 0.790, 0.15),
    ];
    println!("  fits on balls in play: slope sim / MLB (correlation sim / MLB)");
    for (f, (label, slope, r)) in g.fits.iter().zip(&FITS_MLB) {
        println!("    {:<52} {:7.3} / {:6.3}   (r {:5.2} / {:4.2})", label, f.slope(), slope, f.r(), r);
    }
    // MLB by depth bin: whiff %, foul %, in play %; balls in play: bat speed, attack angle, direction to pull,
    // launch angle, exit velocity, efficiency, pull-side spray (NaN where the sample is too small).
    const NA: f64 = f64::NAN;
    const DEPTH_MLB: [[f64; 10]; ND] = [
        [60.1, 37.3, 2.6, NA, NA, NA, NA, NA, NA, NA],
        [35.7, 51.1, 13.2, 67.9, -5.9, -23.1, -13.9, 88.2, 0.855, -23.6],
        [20.4, 50.5, 29.1, 69.6, 0.1, -14.7, 0.8, 88.3, 0.841, -13.4],
        [12.6, 43.1, 44.3, 71.1, 5.2, -7.2, 11.3, 89.4, 0.838, -4.2],
        [9.4, 38.4, 52.2, 72.1, 8.5, -1.3, 15.5, 91.2, 0.846, 4.6],
        [9.9, 35.5, 54.6, 72.9, 11.4, 3.7, 19.3, 92.0, 0.849, 12.3],
        [15.9, 37.8, 46.2, 73.4, 13.8, 8.8, 20.5, 91.3, 0.840, 19.3],
        [31.2, 42.3, 26.5, 73.4, 16.6, 15.1, 18.4, 84.9, 0.785, 23.5],
        [60.2, 29.4, 10.3, 73.4, 20.1, 23.6, 1.8, 72.1, 0.670, 16.1],
        [91.6, 6.2, 2.2, NA, NA, NA, NA, NA, NA, NA],
    ];
    println!("  by contact depth (in, relative to the usual contact point; MLB bins relative to 30 in): sim / MLB");
    println!(
        "    depth          whiff %      foul %     in play %   | in play: bat speed    attack      direction   launch      EV          efficiency    spray"
    );
    for i in 0..ND {
        let n: f64 = g.by_depth[i].iter().sum();
        let b = &g.depth_bip[i];
        let m = &DEPTH_MLB[i];
        println!(
            "    [{:3.0},{:3.0})  {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:5.1}/{:5.1}   | {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:.3}/{:.3}  {:5.1}/{:5.1}   (share of swings {:4.1} %, {:4.1} in from the end of the bat)",
            DEPTH_EDGES[i],
            DEPTH_EDGES[i + 1],
            pct(g.by_depth[i][0], n),
            m[0],
            pct(g.by_depth[i][1], n),
            m[1],
            pct(g.by_depth[i][2], n),
            m[2],
            b[0].mean(),
            m[3],
            b[1].mean(),
            m[4],
            b[2].mean(),
            m[5],
            b[3].mean(),
            m[6],
            b[4].mean(),
            m[7],
            b[5].mean(),
            m[8],
            b[6].mean(),
            m[9],
            pct(n, g.all.iter().sum::<f64>() - g.slow.iter().sum::<f64>()),
            b[7].mean()
        );
    }
    // MLB by depth bin and location class: miss (with caught tips) %, foul %, in play %.
    const DEPTH_ZONE_MLB: [[[f64; 3]; 3]; ND] = [
        [[54.0, 42.9, 3.2], [67.4, 30.2, 2.3], [71.8, 24.7, 3.5]],
        [[36.1, 52.5, 11.5], [43.2, 48.7, 8.1], [50.6, 41.8, 7.6]],
        [[20.9, 50.6, 28.5], [25.7, 50.0, 24.3], [31.2, 44.4, 24.4]],
        [[10.2, 43.0, 46.8], [13.6, 40.9, 45.5], [19.5, 37.8, 42.7]],
        [[7.0, 32.8, 60.2], [7.4, 33.8, 58.8], [15.7, 36.0, 48.3]],
        [[5.1, 29.7, 65.2], [5.8, 33.5, 60.7], [18.0, 38.7, 43.3]],
        [[5.6, 35.1, 59.3], [9.8, 42.9, 47.3], [29.7, 39.7, 30.6]],
        [[14.9, 51.9, 33.2], [26.9, 46.5, 26.6], [52.7, 33.5, 13.8]],
        [[50.6, 37.1, 12.2], [55.9, 33.4, 10.7], [79.1, 16.3, 4.6]],
        [[92.6, 4.1, 3.4], [98.6, 1.4, 0.0], [96.6, 2.1, 1.3]],
    ];
    // MLB: share of full swings, miss %, foul %, in play %, share of the class's swings more than 10 in early.
    const ZONE_MLB: [[f64; 5]; 3] = [[46.6, 13.9, 40.5, 45.7, 13.2], [16.2, 19.4, 40.7, 39.9, 16.3], [37.2, 36.3, 35.4, 28.3, 26.3]];
    println!("  by location of the pitch (sim / MLB): miss % / foul % / in play % by depth of contact");
    let full: f64 = g.all.iter().sum::<f64>() - g.slow.iter().sum::<f64>();
    for z in 0..3 {
        let mut tot = [0.0; 3];
        let mut early = 0.0;
        for i in 0..ND {
            for k in 0..3 {
                tot[k] += g.depth_zone[i][z][k];
                if DEPTH_EDGES[i] >= 10.0 {
                    early += g.depth_zone[i][z][k];
                }
            }
        }
        let n: f64 = tot.iter().sum();
        let m = &ZONE_MLB[z];
        println!(
            "    pitch {:<21} share {:4.1}/{:4.1}   miss {:5.1}/{:5.1}   foul {:5.1}/{:5.1}   in play {:5.1}/{:5.1}   more than 10 in early {:4.1}/{:4.1} %",
            ZONE_NAMES[z],
            pct(n, full),
            m[0],
            pct(tot[0], n),
            m[1],
            pct(tot[1], n),
            m[2],
            pct(tot[2], n),
            m[3],
            pct(early, n),
            m[4]
        );
    }
    for i in 0..ND {
        let mut row = String::new();
        for z in 0..3 {
            let c = &g.depth_zone[i][z];
            let n: f64 = c.iter().sum();
            let m = &DEPTH_ZONE_MLB[i][z];
            row.push_str(&format!("   {:4.0}/{:4.0} {:4.0}/{:4.0} {:4.0}/{:4.0}", pct(c[0], n), m[0], pct(c[1], n), m[1], pct(c[2], n), m[2]));
        }
        println!("    [{:3.0},{:3.0}){}", DEPTH_EDGES[i], DEPTH_EDGES[i + 1], row);
    }
    // MLB, swings within 4 in of the batter's usual depth and 5 deg of his usual direction (7150 swings):
    // by elevation of the ball off the bat: fouls % of swings and EV, in play % of swings and EV.
    const CENTRED_MLB: [[f64; 4]; NE] = [
        [0.1, 56.7, 0.1, NA],
        [2.6, 67.0, 0.7, 70.0],
        [2.6, 75.0, 2.1, 81.7],
        [1.4, 77.4, 5.2, 89.7],
        [0.2, 71.6, 6.6, 94.2],
        [0.6, 75.5, 7.6, 96.7],
        [1.5, 75.2, 8.3, 96.3],
        [1.6, 71.6, 9.4, 95.3],
        [2.1, 69.6, 6.9, 91.9],
        [2.9, 71.6, 4.3, 88.6],
        [4.7, 73.9, 3.2, 85.9],
        [4.9, 76.4, 2.3, 82.5],
        [2.8, 76.1, 0.9, 76.9],
        [0.6, 76.3, 0.3, 77.3],
    ];
    let cn = g.centred[0];
    let fouls: f64 = g.centred_foul.iter().map(|m| m.n).sum();
    let bip: f64 = g.centred_bip.iter().map(|m| m.n).sum();
    println!(
        "  well-timed swings (within 4 in of the usual depth and 5 deg of the usual direction), {:.1} % of full swings: sim / MLB",
        pct(cn, g.all.iter().sum::<f64>() - g.slow.iter().sum::<f64>())
    );
    println!(
        "    swing and miss {:5.1}/{:5.1}   foul, with caught tips {:5.1}/{:5.1}   in play {:5.1}/{:5.1}   (caught tips {:4.1}/{:4.1}; MLB fouls without launch data 4.4)",
        pct(g.centred[1], cn),
        7.2,
        pct(fouls + g.centred[2], cn),
        34.6,
        pct(bip, cn),
        58.2,
        pct(g.centred[2], cn),
        1.6
    );
    // MLB: share of well-timed swings, miss (with caught tips) %, foul %, in play %.
    const ZONE_NAMES: [&str; 3] = ["well inside the zone", "on the edge", "outside the zone"];
    const CENTRED_ZONE_MLB: [[f64; 4]; 3] = [[53.7, 6.4, 32.1, 61.6], [16.4, 6.8, 34.0, 59.2], [29.9, 14.3, 34.3, 51.5]];
    for z in 0..3 {
        let c = &g.centred_zone[z];
        let n: f64 = c.iter().sum();
        let m = &CENTRED_ZONE_MLB[z];
        println!(
            "    pitch {:<21} share {:4.1}/{:4.1}   swing and miss {:5.1}/{:5.1}   foul {:5.1}/{:5.1}   in play {:5.1}/{:5.1}",
            ZONE_NAMES[z],
            pct(n, cn),
            m[0],
            pct(c[0], n),
            m[1],
            pct(c[1], n),
            m[2],
            pct(c[2], n),
            m[3]
        );
    }
    let cm = &g.centred_miss;
    let cmn: f64 = cm.iter().sum::<f64>().max(1.0);
    println!(
        "    well-timed misses by cause: early {:.1} %, late {:.1} %, off the end {:.1} %, jammed {:.1} %, over the ball {:.1} %, under the ball {:.1} %",
        pct(cm[0], cmn),
        pct(cm[1], cmn),
        pct(cm[2], cmn),
        pct(cm[3], cmn),
        pct(cm[4], cmn),
        pct(cm[5], cmn)
    );
    // MLB by pitch type (depth relative to the batter's own mean on balls in play): mean, SD,
    // share more than 10 in late, share more than 10 in early, whiff % when more than
    // 10 in late, 5-10 in late, within 5 in, 5-10 in early, more than 10 in early.
    #[rustfmt::skip]
    const KIND_DEPTH_MLB: [(PitchKind, [f64; 9]); 8] = [
        (PitchKind::FourSeam, [-6.0, 7.4, 28.1, 1.5, 38.9, 20.0, 9.2, 4.2, 9.6]),
        (PitchKind::Sinker, [-4.6, 7.3, 21.8, 2.1, 20.5, 11.2, 8.3, 12.5, 23.7]),
        (PitchKind::Cutter, [-0.4, 9.1, 14.0, 12.2, 41.3, 20.1, 12.0, 14.3, 35.9]),
        (PitchKind::Slider, [5.8, 10.1, 5.7, 33.0, 36.0, 17.8, 11.5, 18.7, 57.6]),
        (PitchKind::Sweeper, [7.5, 10.0, 4.0, 40.0, 42.5, 20.3, 11.3, 16.8, 47.9]),
        (PitchKind::Curve, [8.9, 10.2, 2.8, 44.9, 18.8, 7.9, 7.3, 14.0, 44.0]),
        (PitchKind::Changeup, [6.9, 9.4, 3.4, 36.7, 18.2, 12.8, 12.1, 18.8, 53.8]),
        (PitchKind::Splitter, [7.2, 9.5, 3.7, 37.9, 27.8, 7.8, 17.1, 23.9, 57.1]),
    ];
    let usual = g.depth[2].mean();
    // The histogram is in steps of 2.5 in from -40 in; classes are taken about the mean depth of balls in play.
    let shift = (usual / 2.5).round() as i64;
    println!("  timing by pitch type (sim / MLB), depth of contact relative to the mean of balls in play, + = out in front:");
    println!("        mean in       SD           >10 in late %  >10 in early % | whiff %: >10 late    5-10 late    within 5     5-10 early   >10 early");
    for (kind, m) in KIND_DEPTH_MLB {
        let kd = &g.kind_depth[kind.index()];
        let class = |lo: i64, hi: i64| {
            let (mut n, mut w) = (0.0, 0.0);
            for (i, h) in kd.hist.iter().enumerate() {
                let j = i as i64 - 16 - shift;
                if j >= lo && j < hi {
                    n += h[0];
                    w += h[1];
                }
            }
            (n, w)
        };
        let all = kd.depth.n.max(1.0);
        let c = [class(-99, -4), class(-4, -2), class(-2, 2), class(2, 4), class(4, 99)];
        println!(
            "    {}  {:+5.1}/{:+5.1}  {:5.1}/{:5.1}   {:5.1}/{:5.1}    {:5.1}/{:5.1}   |        {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:5.1}/{:5.1}  {:5.1}/{:5.1}",
            kind.code(),
            kd.depth.mean() - usual,
            m[0],
            kd.depth.sd(),
            m[1],
            pct(c[0].0, all),
            m[2],
            pct(c[4].0, all),
            m[3],
            pct(c[0].1, c[0].0),
            m[4],
            pct(c[1].1, c[1].0),
            m[5],
            pct(c[2].1, c[2].0),
            m[6],
            pct(c[3].1, c[3].0),
            m[7],
            pct(c[4].1, c[4].0),
            m[8]
        );
    }
    println!("    elevation off the bat      fouls, % of swings (EV, mph)        in play, % of swings (EV, mph)");
    for i in 0..NE {
        let m = &CENTRED_MLB[i];
        println!(
            "    [{:3.0},{:3.0})   {:5.1}/{:5.1}  ({:5.1}/{:5.1})   {:5.1}/{:5.1}  ({:5.1}/{:5.1})",
            ELEV_EDGES[i],
            ELEV_EDGES[i + 1],
            pct(g.centred_foul[i].n, cn),
            m[0],
            g.centred_foul[i].mean(),
            m[1],
            pct(g.centred_bip[i].n, cn),
            m[2],
            g.centred_bip[i].mean(),
            m[3]
        );
    }
    // MLB by launch-angle class: pull-side spray mean and SD, direction to pull, depth relative to 30 in, attack angle.
    const LA_MLB: [[f64; 5]; 4] =
        [[16.14, 27.35, 2.60, -1.1, 6.9], [5.82, 25.92, 1.05, -0.1, 8.5], [-2.30, 25.22, -1.81, 1.3, 10.5], [-19.24, 47.57, -6.32, 3.3, 13.0]];
    println!("  by launch-angle class (sim / MLB): pull-side spray mean and SD, direction, depth, attack angle");
    for (i, name) in ["GB", "LD", "FB", "PU"].iter().enumerate() {
        let c = &g.by_la[i];
        let m = &LA_MLB[i];
        println!(
            "    {}  spray {:6.1}/{:6.1} (SD {:4.1}/{:4.1})  direction {:5.1}/{:5.1}  depth {:5.1}/{:5.1} in  attack {:5.1}/{:5.1}",
            name,
            c[0].mean(),
            m[0],
            c[0].sd(),
            m[1],
            c[1].mean(),
            m[2],
            c[2].mean(),
            m[3],
            c[3].mean(),
            m[4]
        );
    }

    println!("\n== Batted-ball types (launch angle classes: GB < 10, LD 10-25, FB 25-50, PU > 50 deg)");
    println!("  note: Statcast's own bb_type labels are assigned by its classifier, so shares differ slightly from pure launch-angle classes");
    #[rustfmt::skip]
    let tt: [(&str, [f64; 7]); 4] = [
        ("GB", [42.24, 85.6, 0.249, 0.000, 0.224, 0.023, 0.002]),
        ("LD", [24.18, 94.2, 0.621, 0.014, 0.416, 0.178, 0.013]),
        ("FB", [26.61, 92.4, 0.252, 0.153, 0.047, 0.045, 0.007]),
        ("PU", [6.96, 77.6, 0.011, 0.000, 0.009, 0.002, 0.000]),
    ];
    for (i, (name, tg)) in tt.iter().enumerate() {
        let b = &t.types[i];
        println!(
            "  {name}  share {:5.1}/{:5.1}  EV {:5.1}/{:5.1}  AVG {:.3}/{:.3}  HR {:.3}/{:.3}  1B {:.3}/{:.3}  2B {:.3}/{:.3}  3B {:.3}/{:.3}  pull-side mean {:5.1} deg",
            pct(b.n, t.bip),
            tg[0],
            b.ev.mean(),
            tg[1],
            b.hits / b.n,
            tg[2],
            b.hr / b.n,
            tg[3],
            b.singles / b.n,
            tg[4],
            b.doubles / b.n,
            tg[5],
            b.triples / b.n,
            tg[6],
            b.pull.mean()
        );
    }
    println!("  MLB pull-side mean by Statcast type: GB +12.1 (RHB) / +18.5 (LHB), LD +4.3 / +6.4, FB -4.5 / -1.9 deg");

    println!("\n== By launch-angle bin: share %, mean EV, AVG, HR rate, landing distance (ft); sim / MLB");
    #[rustfmt::skip]
    let lt: [[f64; 5]; 11] = [
        [2.38, 75.6, 0.066, 0.000, 3.0], [4.43, 83.9, 0.068, 0.000, 5.0], [7.75, 87.5, 0.118, 0.000, 8.0], [11.39, 91.2, 0.240, 0.000, 22.0],
        [13.91, 93.4, 0.483, 0.000, 97.0], [14.98, 94.0, 0.711, 0.003, 236.0], [14.30, 93.2, 0.467, 0.178, 327.0], [10.57, 90.6, 0.253, 0.158, 325.0],
        [5.97, 87.5, 0.068, 0.023, 278.0], [4.31, 85.1, 0.030, 0.000, 220.0], [3.31, 82.2, 0.014, 0.000, 156.0],
    ];
    for (i, tg) in lt.iter().enumerate() {
        let b = &t.la_bins[i];
        println!(
            "  [{:4},{:4})  share {:5.2}/{:5.2}  EV {:5.1}/{:5.1}  AVG {:.3}/{:.3}  HR {:.3}/{:.3}  dist {:4.0}/{:4.0}",
            -40 + 10 * i as i32,
            -30 + 10 * i as i32,
            pct(b.n, t.bip),
            tg[0],
            b.ev.mean(),
            tg[1],
            b.hits / b.n,
            tg[2],
            b.hr / b.n,
            tg[3],
            b.dist.mean(),
            tg[4]
        );
    }
    println!("\n== Launch angle 20-35 deg by exit velocity: HR rate and projected distance (mean, SD); sim / MLB");
    // Statcast 2025: distances from the full season, SDs from the 22,095-ball sample.
    let ht: [[f64; 3]; 7] = [
        [0.000, 281.0, 21.7],
        [0.002, 303.0, 22.1],
        [0.021, 328.0, 23.0],
        [0.102, 354.0, 23.1],
        [0.382, 377.0, 25.3],
        [0.680, 398.0, 26.4],
        [0.859, 413.0, 30.6],
    ];
    for (i, tg) in ht.iter().enumerate() {
        println!(
            "  EV [{:3},{:3})  HR {:.3}/{:.3}  dist {:4.0}/{:4.0}  SD {:4.1}/{:4.1}   (n = {:.0})",
            80 + 5 * i,
            85 + 5 * i,
            t.hr_ev_hr[i] / t.hr_ev_n[i],
            tg[0],
            t.hr_ev_dist[i].mean(),
            tg[1],
            t.hr_ev_dist[i].sd(),
            tg[2],
            t.hr_ev_n[i]
        );
    }
    println!("\n== Home-run rate by launch angle and exit velocity; sim / MLB (Statcast 2025 sample, n in brackets)");
    let hc: [[(f64, u32); 5]; 7] = [
        [(0.000, 196), (0.000, 274), (0.000, 365), (0.000, 233), (0.000, 69)],
        [(0.000, 225), (0.000, 304), (0.000, 355), (0.026, 228), (0.068, 59)],
        [(0.000, 231), (0.014, 296), (0.132, 319), (0.425, 207), (0.755, 53)],
        [(0.018, 221), (0.137, 307), (0.536, 304), (0.916, 155), (1.000, 37)],
        [(0.045, 222), (0.163, 257), (0.524, 231), (0.835, 103), (0.923, 13)],
        [(0.011, 177), (0.063, 206), (0.314, 153), (0.690, 42), (0.857, 7)],
        [(0.004, 284), (0.014, 214), (0.103, 136), (0.435, 23), (0.667, 3)],
    ];
    println!("  LA \\ EV        [ 90, 95)          [ 95,100)          [100,105)          [105,110)          [110,120)");
    for (r, (row, name)) in t.hr_cell.iter().zip(["10-15", "15-20", "20-25", "25-30", "30-35", "35-40", "40-50"]).enumerate() {
        print!("  {name:<8}");
        for (c, (sim, tg)) in row.iter().zip(&hc[r]).enumerate() {
            let _ = c;
            print!("   {:.3}/{:.3} ({:3})", sim.1 / sim.0.max(1.0), tg.0, tg.1);
        }
        println!();
    }
    println!("\n== Home-run share by projected distance (ft), launch angle 15 deg and up, by the bearing of the landing point; sim / MLB");
    // Statcast 2025 sample (10,739 balls): the line sector is within 11.25 deg of a foul line,
    // centre within 11.25 deg of straightaway centre field.
    let hd: [[f64; 8]; 3] =
        [[0.004, 0.034, 0.349, 0.907, 1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.012, 0.157, 0.635, 0.957, 0.969, 1.0], [0.0, 0.0, 0.0, 0.0, 0.043, 0.531, 0.914, 1.0]];
    print!("  distance  ");
    for i in 0..8 {
        print!("   [{:3},{:3})", 300 + 20 * i, 320 + 20 * i);
    }
    println!();
    for (s, name) in ["line", "alley", "centre"].iter().enumerate() {
        print!("  {name:<8}  ");
        for (c, tg) in t.hr_dist[s].iter().zip(&hd[s]) {
            print!("  {:.2}/{:.2}", c.1 / c.0.max(1.0), tg);
        }
        println!();
    }
    println!("\n  inside-the-park home runs: {:.2} % of home runs", 100.0 * t.hr_inside.0 / t.hr_inside.1.max(1.0));
    println!("\n== Home-run share by projected distance beyond the park's fence at the landing bearing (ft), launch angle 15 deg and up; sim / MLB");
    // Statcast 2025 sample against the five published distances of each park, linear in bearing.
    let hm: [[f64; 9]; 3] = [
        [0.0, 0.0, 0.0, 0.02, 0.24, 0.67, 0.85, 0.95, 0.99],
        [0.0, 0.0, 0.0, 0.10, 0.42, 0.75, 0.97, 0.99, 0.99],
        [0.0, 0.0, 0.0, 0.02, 0.22, 0.68, 0.95, 1.0, 1.0],
    ];
    print!("  margin    ");
    for i in 0..9 {
        print!("  [{:3},{:3})", -40 + 10 * i, -30 + 10 * i);
    }
    println!();
    for (s, name) in ["line", "alley", "centre"].iter().enumerate() {
        print!("  {name:<8}  ");
        for (c, tg) in t.hr_margin[s].iter().zip(&hm[s]) {
            print!("  {:.2}/{:.2}", c.1 / c.0.max(1.0), tg);
        }
        println!();
    }
    println!("\n== Spray (pull-side angle, deg): sim / MLB");
    let st = [[3.2, 32.3, 42.1, 28.9, 29.0], [7.7, 31.5, 46.8, 27.8, 25.3]];
    for (h, name) in ["RHB", "LHB"].iter().enumerate() {
        let n: f64 = t.pull_n[h].iter().sum();
        println!(
            "  {name}  mean {:5.1}/{:5.1}  SD {:5.1}/{:5.1}  pull {:5.1}/{:5.1}  centre {:5.1}/{:5.1}  oppo {:5.1}/{:5.1}",
            t.spray_pull[h].mean(),
            st[h][0],
            t.spray_pull[h].sd(),
            st[h][1],
            pct(t.pull_n[h][0], n),
            st[h][2],
            pct(t.pull_n[h][1], n),
            st[h][3],
            pct(t.pull_n[h][2], n),
            st[h][4]
        );
    }
    println!("\n== Launch angle of balls in play by pitch height (ft): sim / MLB");
    let hz = [f64::NAN, 6.0, 11.6, 17.0, 21.9, 26.7];
    let mut s = String::new();
    for i in 0..6 {
        s += &format!("  [{:.1},{:.1}) {:5.1}/{:5.1}", 1.0 + 0.5 * i as f64, 1.5 + 0.5 * i as f64, t.la_height_la[i] / t.la_height_n[i], hz[i]);
    }
    println!("{s}");
    attack_regions(t);
    kind_zone(t);
    println!("\n== Base states at the start of a plate appearance, % (sim / MLB)");
    let bs_t = [56.35, 18.32, 8.78, 6.56, 2.77, 2.66, 2.36, 2.18];
    let names = ["empty", "1B", "2B", "1B+2B", "3B", "1B+3B", "2B+3B", "loaded"];
    let mut s = String::from(" ");
    for i in 0..8 {
        s += &format!(" {} {:.2}/{:.2} ", names[i], pct(t.base_state[i], t.pa), bs_t[i]);
    }
    println!("{s}");
    println!(
        "  runner on third with fewer than two outs: {:.2}/5.20 % of PA; sacrifice fly in that situation: {:.2}/15.25 %",
        pct(t.third_lt2, t.pa),
        pct(t.sf_opp, t.third_lt2)
    );
    println!(
        "  runner on first with fewer than two outs: {:.2}/18.15 % of PA; GIDP in that situation: {:.2}/10.39 %",
        pct(t.first_lt2, t.pa),
        pct(t.gidp, t.first_lt2)
    );
    println!("\n== Base running, % (sim / MLB)");
    let adv_t = [
        ("runner on first reaches third on a single", 33.7),
        ("runner on second scores on a single", 65.2),
        ("runner on first scores on a double", 40.7),
        ("runner on second takes third on a ground out", 69.7),
        ("runner on second tags to third on an air out", 35.1),
        ("ground-ball out with a runner on first, <2 outs: double play", 50.6),
        ("   ... force out at second only", 28.6),
        ("   ... out at first only", 20.8),
    ];
    for (i, (label, tg)) in adv_t.iter().enumerate() {
        println!("  {:<62} {:5.1} / {:5.1}", label, pct(t.adv[i][1], t.adv[i][0]), tg);
    }
    println!("\n== Outs on balls in play by fielder, % (sim / MLB): P C 1B 2B 3B SS LF CF RF");
    #[rustfmt::skip]
    let of_t: [[f64; 9]; 4] = [
        [8.4, 1.0, 15.1, 26.5, 22.4, 26.5, 0.0, 0.0, 0.0],
        [1.7, 0.0, 4.3, 7.7, 5.9, 7.8, 20.5, 28.9, 23.3],
        [0.0, 0.0, 0.2, 1.7, 0.5, 2.0, 27.7, 37.1, 30.9],
        [0.7, 9.3, 23.6, 22.0, 23.4, 20.8, 0.0, 0.1, 0.1],
    ];
    for (i, name) in ["GB", "LD", "FB", "PU"].iter().enumerate() {
        let tot: f64 = t.outs_by_fielder[i].iter().sum();
        let mut s = format!("  {name} ");
        for j in 0..9 {
            s += &format!(" {:4.1}/{:4.1}", pct(t.outs_by_fielder[i][j], tot), of_t[i][j]);
        }
        println!("{s}");
    }
    println!("\n== Double plays: GIDP per ground-ball out with a runner on first and fewer than two outs: {:.3}", t.gidp / t.dp_opp_gb.max(1.0));
}

/// Pitch types in and out of the rulebook zone (grown by the ball's radius): swing rate, whiffs
/// (and foul tips) per swing, and run value per 100 pitches valued with the count-by-result table
/// of the 2025 Statcast sample (the mean `delta_run_exp` of each result in each count), against
/// that sample (30 dates, 121,915 pitches of these types; bunts and automatic or intentional
/// pitches left out). The table makes the value of a result independent of how the engine scores
/// runs, so a difference is in the mix of results.
fn kind_zone(t: &Tally) {
    // [count = balls * 3 + strikes][class as `tally::pitch_class`]
    const RV: [[f64; 12]; 12] = [
        [0.0330, -0.0393, -0.0404, -0.0407, -0.0399, 0.3333, -0.2515, 0.4093, 0.7188, 1.0116, 1.5633, 0.4308],
        [0.0265, -0.0653, -0.0667, -0.0670, -0.0661, 0.3697, -0.2161, 0.4502, 0.7551, 1.0949, 1.6012, 0.4732],
        [0.0265, -0.1570, -0.1551, -0.1572, 0.0000, 0.4476, -0.1502, 0.5156, 0.8278, 1.1652, 1.5925, 0.5669],
        [0.0655, -0.0464, -0.0459, -0.0452, -0.0461, 0.2948, -0.2859, 0.3862, 0.6848, 0.9887, 1.5245, 0.3676],
        [0.0463, -0.0656, -0.0663, -0.0668, -0.0662, 0.3556, -0.2399, 0.4323, 0.7216, 1.0564, 1.5401, 0.4371],
        [0.0397, -0.1818, -0.1833, -0.1809, 0.0001, 0.4269, -0.1768, 0.5034, 0.7907, 1.1198, 1.6163, 0.5247],
        [0.0967, -0.0664, -0.0635, -0.0636, -0.0663, 0.1967, -0.3536, 0.3256, 0.6045, 0.9260, 1.2961, 0.3766],
        [0.0973, -0.0730, -0.0722, -0.0716, -0.0723, 0.3463, -0.2902, 0.3878, 0.6771, 0.9345, 1.5613, 0.3996],
        [0.0888, -0.2208, -0.2241, -0.2274, 0.0000, 0.3674, -0.2167, 0.4571, 0.7349, 1.0548, 1.4759, 0.4668],
        [0.1254, -0.0656, -0.0643, -0.0637, -0.0578, 0.0000, -0.4445, 0.2693, 0.4830, 0.9070, 1.0945, 0.3730],
        [0.1909, -0.0807, -0.0797, -0.0866, -0.0821, 0.2178, -0.3829, 0.2969, 0.5715, 0.7617, 1.3929, 0.2824],
        [0.2756, -0.3096, -0.3150, -0.3062, 0.0000, 0.2899, -0.3035, 0.3734, 0.6668, 0.9682, 1.5284, 0.3672],
    ];
    // [in, out]: swing %, whiff % of swings, run value per 100 pitches, its standard error.
    const MLB: [[[f64; 4]; 2]; 8] = [
        [[68.6, 18.2, -2.33, 0.17], [23.3, 35.6, 2.94, 0.10]],
        [[60.0, 10.9, -2.58, 0.24], [24.9, 24.4, 3.05, 0.15]],
        [[67.9, 16.2, -1.79, 0.40], [25.7, 39.4, 2.66, 0.19]],
        [[68.9, 19.4, -1.48, 0.31], [31.9, 59.7, 0.96, 0.13]],
        [[64.4, 19.7, -2.19, 0.41], [30.5, 55.8, 1.34, 0.17]],
        [[57.5, 15.4, -1.30, 0.43], [27.9, 54.0, 1.77, 0.19]],
        [[76.6, 20.6, -1.81, 0.40], [33.9, 46.5, 1.26, 0.15]],
        [[80.4, 22.7, -1.30, 0.76], [34.6, 53.9, 0.63, 0.26]],
    ];
    println!("\n== Pitch types in and out of the zone: sim / MLB (run value of the results with the MLB count-by-result table)");
    println!("  type zone   swing %       whiff %       runs/100 pitches (MLB s.e.)");
    let mut chi2 = 0.0;
    for (k, m) in MLB.iter().enumerate() {
        for (z, name) in ["in ", "out"].iter().enumerate() {
            let (mut n, mut sw, mut wh, mut rv) = (0.0, 0.0, 0.0, 0.0);
            for c in 0..12 {
                for j in 0..12 {
                    let all = t.pitch_classes[k][c][j];
                    let inz = t.pitch_classes_zone[k][c][j];
                    let v = if z == 0 { inz } else { all - inz };
                    n += v;
                    rv += v * RV[c][j];
                    if (2..=4).contains(&j) || j >= 6 {
                        sw += v;
                    }
                    if matches!(j, 2 | 3) {
                        wh += v;
                    }
                }
            }
            let r = 100.0 * rv / n.max(1.0);
            chi2 += ((r - m[z][2]) / m[z][3]).powi(2);
            println!(
                "  {:<4} {name}   {:5.1}/{:5.1}   {:5.1}/{:5.1}   {:+5.2}/{:+5.2} ({:.2})",
                PitchKind::ALL[k].code(),
                pct(sw, n),
                m[z][0],
                pct(wh, sw),
                m[z][1],
                r,
                m[z][2],
                m[z][3]
            );
        }
    }
    println!("  chi-square of the run values over the 16 cells (MLB s.e.): {chi2:.0}");
}

/// Swing outcomes, bat speed and contact quality by attack region of the plate
/// (`tally::REGION_NAMES`) and the swing's geometry by pitch height, against the 2025 Statcast
/// sample (30 dates, 125,396 pitches; bunts left out; NA where the sample has fewer than 150
/// swings or balls in play).
fn attack_regions(t: &Tally) {
    const NA: f64 = f64::NAN;
    // swing %, whiff % of swings, foul % of swings, bat speed (mph, all swings), exit velocity
    // (mph), collision efficiency (`Efficiency`), launch angle (deg); [0-1 strikes, two strikes].
    const REGION_MLB: [[[f64; 7]; 2]; REGIONS] = [
        [[68.6, 13.0, 40.3, 72.7, 94.8, 0.868, 16.4], [95.4, 11.5, 40.1, 71.4, 92.7, 0.862, 14.8]],
        [[60.8, 17.9, 39.9, 71.9, 91.7, 0.847, 14.7], [90.8, 14.1, 42.1, 70.6, 90.9, 0.852, 14.2]],
        [[57.6, 12.4, 45.2, 71.3, 84.3, 0.784, 16.8], [81.9, 12.0, 47.2, 70.1, 82.5, 0.778, 18.2]],
        [[45.5, 24.6, 35.1, 70.7, 87.4, 0.818, 8.5], [81.1, 19.6, 37.6, 69.0, 89.3, 0.854, 9.2]],
        [[49.5, 31.0, 32.9, 72.4, 89.4, 0.823, 4.6], [80.6, 22.1, 36.2, 71.0, 89.3, 0.840, 8.1]],
        [[50.8, 33.6, 43.4, 68.4, 91.1, 0.868, 28.9], [85.9, 25.7, 48.1, 67.3, 90.1, 0.867, 24.1]],
        [[44.7, 20.0, 47.5, 70.2, 77.7, 0.732, 17.6], [66.4, 17.7, 51.9, 68.9, 75.8, 0.732, 17.7]],
        [[30.2, 37.6, 31.2, 69.4, 81.5, 0.769, 5.4], [61.4, 26.5, 34.6, 67.1, 84.0, 0.821, 5.3]],
        [[38.9, 50.4, 26.1, 71.0, 85.9, 0.802, 0.1], [65.3, 35.0, 32.6, 69.7, 84.8, 0.808, 3.4]],
        [[33.0, 43.7, 38.7, 66.0, NA, NA, NA], [66.8, 36.1, 43.7, 65.2, 88.0, NA, 28.9]],
        [[26.0, 32.5, 49.1, 66.3, 67.8, NA, 9.1], [40.1, 30.1, 50.0, 65.8, NA, NA, NA]],
        [[14.8, 59.2, 23.1, 66.3, 77.7, 0.744, 4.4], [37.7, 36.1, 33.2, 64.6, 77.5, 0.778, -0.9]],
        [[28.1, 75.6, 15.3, 67.5, NA, NA, NA], [50.0, 57.4, 23.6, 67.0, 81.5, 0.806, 3.0]],
        [[14.2, 56.3, 29.3, 61.1, NA, NA, NA], [37.3, 46.8, 39.6, 61.6, NA, NA, NA]],
        [[6.6, 92.4, 6.1, 57.9, NA, NA, NA], [11.6, 79.5, 14.1, 57.9, NA, NA, NA]],
    ];
    println!("\n== Attack regions of the plate (heart / shadow one ball either side of the edge / chase / waste, by nearest edge): sim / MLB");
    for (g, label) in ["0-1 strikes", "two strikes"].iter().enumerate() {
        println!("  {label:<22} swing %       whiff %       foul %        bat speed     exit velo     efficiency      launch angle");
        for (i, name) in REGION_NAMES.iter().enumerate() {
            let h = &t.region_classes[i][g];
            let n: f64 = h.iter().sum();
            let sw = h[2] + h[3] + h[4] + h[6..].iter().sum::<f64>();
            let b = &t.region_bip[i][g];
            let q = &t.region_swing[i][g];
            let m = &REGION_MLB[i][g];
            println!(
                "    {name:<20} {:5.1}/{:5.1}   {:5.1}/{:5.1}   {:5.1}/{:5.1}   {:5.1}/{:5.1}   {:5.1}/{:5.1}   {:5.3}/{:5.3}   {:5.1}/{:5.1}",
                pct(sw, n),
                m[0],
                pct(h[2] + h[3], sw),
                m[1],
                pct(h[4], sw),
                m[2],
                q[1] / q[0],
                m[3],
                b[1] / b[0],
                m[4],
                q[3] / q[2],
                m[5],
                b[4] / b[0],
                m[6]
            );
        }
    }
    // MLB, all tracked swings by pitch height above the middle of the zone (4-in bins from
    // -24 in): intercept depth (in, relative to 30 in), attack angle (deg), bat speed (mph).
    const GEOM_MLB: [[f64; 3]; 12] = [
        [12.0, 19.1, 62.2],
        [10.5, 17.8, 66.6],
        [8.7, 16.1, 69.6],
        [6.0, 13.6, 71.1],
        [2.7, 10.7, 71.7],
        [0.0, 8.6, 71.6],
        [-2.8, 6.8, 71.0],
        [-5.3, 5.5, 69.5],
        [-7.5, 4.7, 67.7],
        [-10.3, 4.0, 64.7],
        [-12.6, 4.0, 61.1],
        [-13.6, 4.3, 58.6],
    ];
    println!("  swings by pitch height above the middle of the zone (in): sim / MLB contact depth (in; MLB intercept less 30 in), attack angle (deg), bat speed (mph)");
    for (i, (c, m)) in t.height_geom.iter().zip(&GEOM_MLB).enumerate() {
        println!(
            "    [{:3},{:3})  share {:4.1}   depth {:6.1}/{:6.1}   attack {:5.1}/{:5.1}   bat speed {:5.1}/{:5.1}",
            -24 + 4 * i as i32,
            -20 + 4 * i as i32,
            pct(c[0], t.height_geom.iter().map(|r| r[0]).sum()),
            c[1] / c[0],
            m[0],
            c[2] / c[0],
            m[1],
            c[3] / c[0],
            m[2]
        );
    }
}
