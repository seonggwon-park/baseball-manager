import { innings, useLabels } from "../labels";
import { useUi } from "../store";
import type { PlayerView } from "../types";

function Rating({ label, value }: { label: string; value: number }) {
  const tier = value >= 15 ? "hi" : value >= 11 ? "mid" : value >= 7 ? "low" : "poor";
  return (
    <div className="rating">
      <span>{label}</span>
      <b className={tier}>{value}</b>
    </div>
  );
}

export function PlayerRatings({ p, pitching }: { p: PlayerView; pitching: boolean }) {
  const { t } = useLabels(null);
  if (pitching && p.pitching) {
    return (
      <div className="ratings">
        <Rating label={t("rating.velocity")} value={p.pitching.velocity} />
        <Rating label={t("rating.control")} value={p.pitching.control} />
        <Rating label={t("rating.stamina")} value={p.pitching.stamina} />
      </div>
    );
  }
  return (
    <div className="ratings">
      <Rating label={t("rating.contact")} value={p.batting.contact} />
      <Rating label={t("rating.power")} value={p.batting.power} />
      <Rating label={t("rating.eye")} value={p.batting.eye} />
      <Rating label={t("rating.speed")} value={p.batting.speed} />
      <Rating label={t("rating.bunt")} value={p.batting.bunt} />
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
        <div className="name">
          <span className="num">{pitcher.number}</span> {loc(pitcher.name)}
        </div>
        <div className="muted small">
          {t("hand.throws", { hand: t(`hand.${pitcher.throws ?? "R"}`) })} · {t("score.pitchCount", { n: s.pitch_count })}
          {pLine && <> · {t("matchup.todayPitch", { ip: innings(pLine.outs), r: pLine.r, so: pLine.so })}</>}
        </div>
        <PlayerRatings p={pitcher} pitching />
        {pitcher.pitching && (
          <div className="chips">
            {pitcher.pitching.repertoire.map((r) => (
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
        <div className="name">
          <span className="num">{batter.number}</span> {loc(batter.name)}
        </div>
        <div className="muted small">
          {t(`pos.${batter.position}`)} · {t("hand.bats", { hand: t(`hand.${batter.bats}`) })}
          {bLine && <> · {t("matchup.todayBat", { ab: bLine.ab, h: bLine.h })}</>}
        </div>
        <PlayerRatings p={batter} pitching={false} />
        <div className="muted small">
          {t("matchup.onDeck")}: {loc(onDeck.name)}
        </div>
      </div>
      <div className="muted tiny provisional">{t("common.provisional")}</div>
    </section>
  );
}
