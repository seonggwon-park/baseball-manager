//! Side-by-side comparison of simulated aggregates with the measured 2025 MLB values.
//!
//! Targets come from two public sources:
//! * MLB Stats API, 2025 regular-season team totals (league-wide rates), and
//! * a 30-day, 126,266-pitch sample of 2025 Statcast pitch data from Baseball
//!   Savant (plate discipline, batted-ball and bat-tracking distributions).

// The tables below pair simulated arrays with constant target arrays by index.
#![allow(clippy::needless_range_loop)]

use crate::tally::{Tally, CALL_EDGES, DIST_EDGES, EFF_BAT_EDGES, EFF_LA_EDGES, NB, NC};
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
    println!("\n== Launch angle 20-35 deg by exit velocity: HR rate and landing distance; sim / MLB");
    let ht: [[f64; 2]; 7] = [[0.000, 281.0], [0.002, 303.0], [0.021, 328.0], [0.102, 354.0], [0.382, 377.0], [0.680, 398.0], [0.859, 413.0]];
    for (i, tg) in ht.iter().enumerate() {
        println!(
            "  EV [{:3},{:3})  HR {:.3}/{:.3}  dist {:4.0}/{:4.0}   (n = {:.0})",
            80 + 5 * i,
            85 + 5 * i,
            t.hr_ev_hr[i] / t.hr_ev_n[i],
            tg[0],
            t.hr_ev_dist[i].mean(),
            tg[1],
            t.hr_ev_n[i]
        );
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
