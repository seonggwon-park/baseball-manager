import { useLabels } from "../labels";
import { useUi } from "../store";
import type { PitchResult } from "../types";

const CLASS: Record<PitchResult, string> = {
  ball: "ball",
  called_strike: "strike",
  swinging_strike: "strike",
  foul_tip: "strike",
  foul: "foul",
  in_play: "inplay",
  hit_by_pitch: "ball",
};

/** Pitch locations of the current plate appearance, seen from behind the plate. */
export function ZonePanel() {
  const ui = useUi();
  const { t } = useLabels(ui.view);
  const pitches = ui.paPitches;
  const last = pitches[pitches.length - 1];
  const zone = last?.zone ?? { top: 1.05, bottom: 0.49, half_width: 0.216 };
  // View box in metres: x from -0.75 to 0.75, height from 0 to 1.6 (drawn upside down).
  const X = (x: number) => 75 + x * 100;
  const Y = (z: number) => 160 - z * 100;
  return (
    <section className="panel zone">
      <h3>
        {t("zone.title")} <span className="muted small">{t("zone.catcherView")}</span>
      </h3>
      <svg viewBox="0 0 150 172" role="img" aria-label={t("zone.title")}>
        <rect x={X(-zone.half_width)} y={Y(zone.top)} width={zone.half_width * 200} height={(zone.top - zone.bottom) * 100} className="zone-box" />
        {[1, 2].map((i) => (
          <line key={`v${i}`} x1={X(-zone.half_width + (i * zone.half_width * 2) / 3)} x2={X(-zone.half_width + (i * zone.half_width * 2) / 3)} y1={Y(zone.top)} y2={Y(zone.bottom)} className="zone-grid" />
        ))}
        {[1, 2].map((i) => (
          <line key={`h${i}`} y1={Y(zone.bottom + (i * (zone.top - zone.bottom)) / 3)} y2={Y(zone.bottom + (i * (zone.top - zone.bottom)) / 3)} x1={X(-zone.half_width)} x2={X(zone.half_width)} className="zone-grid" />
        ))}
        <path d={`M ${X(-0.216)} 162 L ${X(0.216)} 162 L ${X(0.216)} 166 L ${X(0)} 171 L ${X(-0.216)} 166 Z`} className="plate" />
        {pitches.map((p, i) => {
          const x = Math.max(-0.72, Math.min(0.72, p.plate[0]));
          const z = Math.max(0.03, Math.min(1.57, p.plate[2]));
          return (
            <g key={i} className={`pitch ${CLASS[p.result]} ${i === pitches.length - 1 ? "last" : ""}`}>
              <circle cx={X(x)} cy={Y(z)} r="5.2" />
              <text x={X(x)} y={Y(z) + 2.6}>
                {i + 1}
              </text>
            </g>
          );
        })}
      </svg>
      <div className="zone-last">
        {last ? (
          <>
            <b>{t(`pitch.${last.kind}`)}</b> {Math.round(last.speed_kmh)} km/h · {Math.round(last.spin_rpm)} rpm · {t(`result.${last.result}`)}
          </>
        ) : (
          <span className="muted">{t("zone.noPitch")}</span>
        )}
      </div>
    </section>
  );
}
