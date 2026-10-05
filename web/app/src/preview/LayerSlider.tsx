import { type KeyboardEvent, type PointerEvent, useEffect, useRef } from "react";
import { moveLayerThumb, setPreview, stepLayers } from "../state/actions";
import type { PreviewView, Thumb } from "../state/store";

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
 * upper thumb picks the top visible layer, the lower one the bottom layer; in
 * one layer mode a single thumb picks the only visible layer.
 */
export function LayerSlider({ zs, view }: { zs: Float32Array; view: PreviewView }) {
  const root = useRef<HTMLDivElement>(null);
  const track = useRef<HTMLDivElement>(null);
  const dragging = useRef<Thumb | null>(null);
  const { layers: value, oneLayer } = view;
  const last = zs.length - 1;
  const fraction = (layer: number) => (last === 0 ? 0 : layer / last);
  const thumbs: Thumb[] = oneLayer ? [1] : [1, 0];

  // IMSlider::on_mouse_wheel: one layer per notch, x5 with Shift or Ctrl/Cmd.
  // Non-passive so Ctrl+wheel does not zoom the page.
  useEffect(() => {
    const onWheel = (event: WheelEvent) => {
      const notch = -Math.sign(event.deltaY || event.deltaX);
      if (!notch) return;
      event.preventDefault();
      stepLayers(notch * (event.shiftKey || event.ctrlKey || event.metaKey ? 5 : 1), true);
    };
    const element = root.current!;
    element.addEventListener("wheel", onWheel, { passive: false });
    return () => element.removeEventListener("wheel", onWheel);
  }, []);

  const layerAt = (clientY: number) => {
    const rect = track.current!.getBoundingClientRect();
    return ((rect.bottom - clientY) / rect.height) * last;
  };
  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    const layer = layerAt(event.clientY);
    const thumb: Thumb = oneLayer || Math.abs(layer - value[1]) <= Math.abs(layer - value[0]) ? 1 : 0;
    dragging.current = thumb;
    event.currentTarget.setPointerCapture(event.pointerId);
    moveLayerThumb(thumb, layer);
  };
  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (dragging.current !== null) moveLayerThumb(dragging.current, layerAt(event.clientY));
  };
  const onKeyDown = (thumb: Thumb) => (event: KeyboardEvent) => {
    const step = STEP_KEYS[event.key];
    if (!step) return;
    event.preventDefault();
    moveLayerThumb(thumb, step(value[thumb], last));
  };

  return (
    <div className="layer-slider" data-testid="layer-slider" ref={root}>
      <div className="layer-slider-label top">
        <span>{value[1] + 1}</span>
        <span className="layer-slider-z">{zs[value[1]].toFixed(2)}</span>
      </div>
      <div className="layer-slider-track" ref={track}
        onPointerDown={onPointerDown} onPointerMove={onPointerMove}
        onPointerUp={() => (dragging.current = null)}>
        <div className="layer-slider-fill"
          style={{ bottom: `${fraction(value[0]) * 100}%`, top: `${(1 - fraction(value[1])) * 100}%` }} />
        {thumbs.map((thumb) => (
          <div key={thumb} className="layer-slider-thumb" role="slider" tabIndex={0}
            aria-label={thumb === 1 ? "Top layer" : "Bottom layer"} aria-orientation="vertical"
            aria-valuemin={1} aria-valuemax={last + 1} aria-valuenow={value[thumb] + 1}
            aria-valuetext={`Layer ${value[thumb] + 1}, ${zs[value[thumb]].toFixed(2)} mm`}
            style={{ bottom: `${fraction(value[thumb]) * 100}%` }}
            onFocus={() => setPreview({ selection: thumb })} onKeyDown={onKeyDown(thumb)} />
        ))}
      </div>
      {!oneLayer && (
        <div className="layer-slider-label bottom">
          <span>{value[0] + 1}</span>
          <span className="layer-slider-z">{zs[value[0]].toFixed(2)}</span>
        </div>
      )}
    </div>
  );
}
