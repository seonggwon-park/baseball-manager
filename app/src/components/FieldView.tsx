import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FieldScene } from "../field/scene";
import { store, useUi } from "../store";
import { POSITION_CODES } from "../types";

/** The PixiJS canvas with the result banner and the last batted ball's numbers over it. */
export function FieldView() {
  const { t, i18n } = useTranslation();
  const ui = useUi();
  const wrap = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const scene = useRef<FieldScene | null>(null);
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    const el = wrap.current!;
    const s = new FieldScene();
    scene.current = s;
    let cancelled = false;
    s.init(canvas.current!, el.clientWidth, el.clientHeight)
      .then(() => {
        if (cancelled) return;
        s.setPositionLabels(POSITION_CODES.map((c) => t(`posShort.${c}`)));
        store.attachScene(s);
      })
      .catch((e: unknown) => {
        if (!cancelled) setFailure(String(e));
      });
    const ro = new ResizeObserver(() => s.resize(el.clientWidth, el.clientHeight));
    ro.observe(el);
    return () => {
      cancelled = true;
      ro.disconnect();
      store.attachScene(null);
      s.destroy();
      scene.current = null;
    };
    // The scene is created once; labels follow the language in the effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    scene.current?.setPositionLabels(POSITION_CODES.map((c) => t(`posShort.${c}`)));
  }, [i18n.language, t]);

  const last = ui.paPitches[ui.paPitches.length - 1];
  const batted = last?.result === "in_play" ? last.batted : null;
  return (
    <div className="field" ref={wrap}>
      <canvas ref={canvas} />
      {failure && (
        <div className="field-error">
          {t("field.failed")}
          <small>{failure}</small>
        </div>
      )}
      {ui.banner && <div className="banner">{t(ui.banner)}</div>}
      {batted && !batted.bunt && (
        <div className="batted">
          <b>{Math.round(batted.exit_speed_kmh)}</b> km/h · <b>{Math.round(batted.launch_angle_deg)}</b>° · <b>{Math.round(batted.distance)}</b> m
        </div>
      )}
    </div>
  );
}
