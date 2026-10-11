import type { ActualSpeedProfile as Profile } from "@ares-slicer/wasm";
import { useLayoutEffect, useRef, useState } from "react";
import { printf } from "./toolPositionFormat";
import "./actualSpeed.css";

/** Width of the level strip left of the profile (`offset`, `GCodeViewer.cpp:181`). */
const OFFSET = 10;
const saturate = (value: number) => Math.min(Math.max(value, 0), 1);
const rgba = ([r, g, b, a]: number[]) => `rgb(${r * 255} ${g * 255} ${b * 255} / ${a})`;

/**
 * `ActualSpeedImguiWidget::plot` (`GCodeViewer.cpp:152-230`): level marks in
 * the left strip, a grid line per point, the profile segments. Reports the
 * hovered segment, or -1.
 */
function Plot({ profile, hovered, onHover }: { profile: Profile; hovered: number; onHover: (index: number) => void }) {
  const { data, levels, yRange: [y0, y1] } = profile;
  const sizeY = y1 - y0;
  const sizeX = data.length > 0 ? data.at(-1)!.pos - data[0].pos : 0;
  const drawn = sizeX > 0 && data.length >= 2;
  const invScaleY = sizeY === 0 ? 0 : 1 / sizeY;
  const x = (pos: number) => saturate((pos - data[0].pos) / sizeX);
  const y = (speed: number) => 1 - saturate((speed - y0) * invScaleY);
  const hover = (event: React.PointerEvent<HTMLDivElement>) => {
    if (!drawn) return;
    const box = event.currentTarget.getBoundingClientRect();
    const t = Math.min(Math.max((event.clientX - box.left - OFFSET) / (box.width - OFFSET), 0), 0.9999);
    onHover(data.findIndex((item, n) => n + 1 < data.length && x(item.pos) < t && t < x(data[n + 1].pos)));
  };
  return (
    <div className="actual-speed-plot" data-testid="actual-speed-plot" onPointerMove={hover} onPointerLeave={() => onHover(-1)}>
      {drawn && <>
        <svg className="actual-speed-levels" viewBox="0 0 1 1" preserveAspectRatio="none" aria-hidden="true">
          {levels.map(({ value, color }) => (
            <line key={value} className="actual-speed-level" x1={0.1} x2={0.9} y1={y(value)} y2={y(value)} stroke={rgba(color)} strokeWidth={3} vectorEffect="non-scaling-stroke" />
          ))}
        </svg>
        <svg className="actual-speed-profile" viewBox="0 0 1 1" preserveAspectRatio="none" aria-hidden="true">
          {data.slice(0, -1).map((item, n) => (
            <line key={n} x1={x(item.pos)} x2={x(item.pos)} y1={0} y2={1} className={item.internal ? "actual-speed-grid is-internal" : "actual-speed-grid"} vectorEffect="non-scaling-stroke" />
          ))}
          <line x1={1} x2={1} y1={0} y2={1} className="actual-speed-grid" vectorEffect="non-scaling-stroke" />
          {data.slice(0, -1).map((item, n) => (
            <line key={n} x1={x(item.pos)} y1={y(item.speed)} x2={x(data[n + 1].pos)} y2={y(data[n + 1].speed)}
              className={n === hovered ? "actual-speed-segment is-hovered" : "actual-speed-segment"} vectorEffect="non-scaling-stroke" />
          ))}
        </svg>
      </>}
    </div>
  );
}

/**
 * The external window of `Marker::render_position_window` (`GCodeViewer.cpp:534-611`):
 * plot over the Position/Speed table, at least as tall as the position window
 * and kept inside the canvas, the table scrolling under its header.
 */
export function ActualSpeedWindow({ profile }: { profile: Profile }) {
  const [hovered, setHovered] = useState(-1);
  const ref = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const window = ref.current!;
    const host = window.parentElement!;
    const phone = matchMedia("(max-width: 640px)");
    // Desktop: from the window bottom up to 8px below the canvas top; phone: above the position window.
    const fit = () => {
      window.style.maxHeight = `${phone.matches ? host.offsetTop - 5 - 8 : host.offsetTop + host.offsetHeight - 8}px`;
    };
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(host);
    observer.observe(host.offsetParent!);
    return () => observer.disconnect();
  }, []);
  return (
    <div className="actual-speed-window" data-testid="actual-speed-profile" ref={ref}>
      <Plot profile={profile} hovered={hovered} onHover={setHovered} />
      <div className="actual-speed-table">
        <table>
          <thead><tr><th>Position (mm)</th><th>Speed (mm/s)</th></tr></thead>
          <tbody>
            {profile.data.map((item, index) => {
              const highlight = hovered >= 0 && (index === hovered || index === hovered + 1);
              const className = [item.internal && "is-internal", highlight && "is-hovered"].filter(Boolean).join(" ");
              return (
                <tr key={index} className={className || undefined}>
                  <td>{printf(item.pos, 3)}</td>
                  <td>{printf(item.speed, 1)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
