import type { PreviewView, Thumb } from "../state/store";

/**
 * IMSlider::SetLowerValue / SetHigherValue: clamps to `[0, last]`; the other
 * thumb follows when crossed (correct_lower_value / correct_higher_value) and
 * always in one layer mode.
 */
export function moveThumb(view: PreviewView, thumb: Thumb, layer: number, last: number): PreviewView {
  const value = Math.round(Math.min(last, Math.max(0, layer)));
  const [lower, higher] = view.layers;
  const layers: [number, number] = view.oneLayer
    ? [value, value]
    : thumb === 1 ? [Math.min(lower, value), value] : [value, Math.max(higher, value)];
  return { ...view, layers };
}

/**
 * IMSlider::switch_one_layer_mode. Leaving saves the upper layer and shows all
 * layers; entering keeps a moved upper thumb, restores the saved layer when the
 * thumb is at the top, and falls back to the middle layer when nothing (or
 * layer 0, as `!m_one_layer_value`) is saved.
 */
export function switchOneLayer(view: PreviewView, last: number): PreviewView {
  const higher = view.layers[1];
  if (view.oneLayer) return { ...view, oneLayer: false, oneLayerValue: higher, layers: [0, last] };
  let saved = view.oneLayerValue;
  let value = higher;
  if (!saved || saved > last) value = saved = Math.floor(last / 2);
  else if (higher === last) value = saved;
  return { ...view, oneLayer: true, oneLayerValue: saved, layers: [value, value] };
}
