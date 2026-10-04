import { type KeyboardEvent, type PointerEvent, useRef } from "react";

type Thumb = 0 | 1;

const STEP_KEYS: Record<string, (value: number, last: number) => number> = {
  ArrowUp: (value) => value + 1,
  ArrowRight: (value) => value + 1,
  ArrowDown: (value) => value - 1,
  ArrowLeft: (value) => value - 1,
  PageUp: (value) => value + 10,
  PageDown: (value) => value - 10,
  Home: () => 0,
  End: (_, last) => last,
};

/**
 * Vertical layer range slider (OrcaSlicer `IMSlider` in the preview): the
 * upper thumb picks the top visible layer, the lower one the bottom layer.
 */
export function LayerSlider({ zs, value, onChange }: {
  zs: Float32Array;
  value: [number, number];
  onChange: (value: [number, number]) => void;
}) {
  const track = useRef<HTMLDivElement>(null);
  const dragging = useRef<Thumb | null>(null);
  const last = zs.length - 1;
  const fraction = (layer: number) => (last === 0 ? 0 : layer / last);

  const move = (thumb: Thumb, layer: number) => {
    const clamped = Math.round(Math.min(last, Math.max(0, layer)));
    onChange(thumb === 1 ? [Math.min(value[0], clamped), clamped] : [clamped, Math.max(value[1], clamped)]);
  };
  const layerAt = (clientY: number) => {
    const rect = track.current!.getBoundingClientRect();
    return ((rect.bottom - clientY) / rect.height) * last;
  };
  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    const layer = layerAt(event.clientY);
    const thumb: Thumb = Math.abs(layer - value[1]) <= Math.abs(layer - value[0]) ? 1 : 0;
    dragging.current = thumb;
    event.currentTarget.setPointerCapture(event.pointerId);
    move(thumb, layer);
  };
  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (dragging.current !== null) move(dragging.current, layerAt(event.clientY));
  };
  const onKeyDown = (thumb: Thumb) => (event: KeyboardEvent) => {
    const step = STEP_KEYS[event.key];
    if (!step) return;
    event.preventDefault();
    move(thumb, step(value[thumb], last));
  };

  return (
    <div className="layer-slider" data-testid="layer-slider">
      <div className="layer-slider-label top">
        <span>{value[1] + 1}</span>
        <span className="layer-slider-z">{zs[value[1]].toFixed(2)}</span>
      </div>
      <div className="layer-slider-track" ref={track}
        onPointerDown={onPointerDown} onPointerMove={onPointerMove}
        onPointerUp={() => (dragging.current = null)}>
        <div className="layer-slider-fill"
          style={{ bottom: `${fraction(value[0]) * 100}%`, top: `${(1 - fraction(value[1])) * 100}%` }} />
        {([1, 0] as const).map((thumb) => (
          <div key={thumb} className="layer-slider-thumb" role="slider" tabIndex={0}
            aria-label={thumb === 1 ? "Top layer" : "Bottom layer"} aria-orientation="vertical"
            aria-valuemin={1} aria-valuemax={last + 1} aria-valuenow={value[thumb] + 1}
            aria-valuetext={`Layer ${value[thumb] + 1}, ${zs[value[thumb]].toFixed(2)} mm`}
            style={{ bottom: `${fraction(value[thumb]) * 100}%` }}
            onKeyDown={onKeyDown(thumb)} />
        ))}
      </div>
      <div className="layer-slider-label bottom">
        <span>{value[0] + 1}</span>
        <span className="layer-slider-z">{zs[value[0]].toFixed(2)}</span>
      </div>
    </div>
  );
}
