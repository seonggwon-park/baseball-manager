import { innings, useLabels } from "../labels";
import { store, useUi } from "../store";
import type { PlayerView, RatingScale } from "../types";

/** Colour tier of a rating: 13 on the 1-20 scale and 50 on the 20-80 scale are the major-league average. */
export function tier(value: number, scale: RatingScale): string {
  const [hi, mid, low] = scale === "20" ? [16, 12, 8] : [60, 45, 35];
  return value >= hi ? "hi" : value >= mid ? "mid" : value >= low ? "low" : "poor";
}

export function Rating({ label, value, scale }: { label: string; value: number; scale: RatingScale }) {
  return (
    <div className="rating">
      <span>{label}</span>
      <b className={tier(value, scale)}>{value}</b>
    </div>
  );
}

const PITCHING_KEYS = ["velocity", "stuff", "control", "command", "stamina"];
const BATTING_KEYS = ["contact", "timing", "power", "discipline", "speed"];

/** The handful of ratings that matter most for the job at hand. */
export function PlayerRatings({ p, pitching }: { p: PlayerView; pitching: boolean }) {
  const { t } = useLabels(null);
  const scale = useUi().scale;
  const keys = pitching && p.repertoire.length > 0 ? PITCHING_KEYS : BATTING_KEYS;
  return (
    <div className="ratings">
      {keys.map((key) => {
        const a = p.attrs.find((x) => x.key === key);
        return a ? <Rating key={key} label={t(`attr.${key}`)} value={scale === "20" ? a.r20 : a.r80} scale={scale} /> : null;
      })}
    </div>
  );
}

/** The pitcher and the batter facing each other, with today's lines. */
export function Matchup() {
  const ui = useUi();
  const { t, loc, player } = useLabels(ui.view);
  const state = ui.state!;
  const live = ui.live!;
  const s = state.situation;
  const pitcher = player(s.fielding, live.pitcher);
  const batter = player(s.batting, live.batter);
  const onDeck = player(s.batting, s.on_deck);
  const pLine = state.boxes[s.fielding].pitching.find((l) => l.player === pitcher.idx);
  const bLine = state.boxes[s.batting].batting.find((l) => l.player === batter.idx);
  const order = state.lineups[s.batting].indexOf(batter.idx) + 1;
  return (
    <section className="panel matchup">
      <div className="who">
        <div className="role">{t("matchup.pitcher")}</div>
        <button className="name link" onClick={() => store.showCard([s.fielding, pitcher.idx])} title={t("card.open")}>
          <span className="num">{pitcher.number}</span> {loc(pitcher.name)}
        </button>
        <div className="muted small">
          {t("hand.throws", { hand: t(`hand.${pitcher.throws}`) })} · {t("score.pitchCount", { n: s.pitch_count })}
          {pLine && <> · {t("matchup.todayPitch", { ip: innings(pLine.outs), r: pLine.r, so: pLine.so })}</>}
        </div>
        <PlayerRatings p={pitcher} pitching />
        {pitcher.repertoire.length > 0 && (
          <div className="chips">
            {pitcher.repertoire.map((r) => (
              <span key={r.kind} className="chip-plain" title={`${Math.round(r.usage * 100)}%`}>
                {t(`pitch.${r.kind}`)} {Math.round(r.speed_kmh)}
              </span>
            ))}
          </div>
        )}
      </div>
      <div className="who">
        <div className="role">
          {t("matchup.batter")} · {order > 0 ? `${order}` : "-"}
        </div>
        <button className="name link" onClick={() => store.showCard([s.batting, batter.idx])} title={t("card.open")}>
          <span className="num">{batter.number}</span> {loc(batter.name)}
        </button>
        <div className="muted small">
          {t(`pos.${batter.position}`)} · {t("hand.bats", { hand: t(`hand.${batter.bats}`) })}
          {bLine && <> · {t("matchup.todayBat", { ab: bLine.ab, h: bLine.h })}</>}
        </div>
        <PlayerRatings p={batter} pitching={false} />
        <div className="muted small">
          {t("matchup.onDeck")}: {loc(onDeck.name)}
        </div>
      </div>
    </section>
  );
}
