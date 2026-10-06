import { EXTRUSION_ROLES, type MovesSlider as Stops, type MovesView, type Toolpaths } from "@ares-slicer/wasm";
import { type Dispatch, type SetStateAction, useEffect, useRef, useState } from "react";
import { slicer } from "../state/slicer";
import type { PreviewView } from "../state/store";
import type { ToolpathFilter } from "../viewport/toolpathMesh";

const ALL_ROLES = (1 << EXTRUSION_ROLES.length) - 1;

type SliderState = { toolpaths: Toolpaths; stops: Stops; high: number; request: number; filter: ToolpathFilter };
type Applied = { slider: SliderState; viewType: string };

/** The engine's view between the moves slider thumbs for `toolpaths`, and the filter it was made for. */
export type MovesState = { toolpaths: Toolpaths; view: MovesView; filter: ToolpathFilter };

/**
 * Horizontal moves slider under the preview (GCodeViewer `m_moves_slider`,
 * `IMSlider` without its lower thumb): stop `i` shows the shown layers' moves
 * up to G-code line `stops.gcodeIds[i]`. Holds the engine's stops for the
 * shown layers (`update_moves_slider` after `set_layers_z_range`,
 * `GCodeViewer.cpp:1864-1868`) and the higher thumb, which a layer or
 * visibility change resets to the visible range end; reports the engine's
 * view between the thumbs (`set_view_visible_range`) to `onMoves`; `aria-busy`
 * while that view is behind the slider, layers or legend.
 */
export function MovesSlider({ toolpaths, view, onMoves }: {
  toolpaths: Toolpaths;
  view: PreviewView;
  onMoves: Dispatch<SetStateAction<MovesState | null>>;
}) {
  const [slider, setSlider] = useState<SliderState | null>(null);
  const [applied, setApplied] = useState<Applied | null>(null);
  const latest = useRef(0);
  const { layers, hiddenRoles, visibleOptions, viewType } = view;
  useEffect(() => {
    const request = ++latest.current;
    void slicer.movesSlider(layers, ALL_ROLES & ~hiddenRoles, visibleOptions).then((stops) => {
      const filter = { layers, hiddenRoles, visibleOptions };
      if (request === latest.current) setSlider({ toolpaths, stops, high: stops.span[1], request, filter });
    });
  }, [toolpaths, layers, hiddenRoles, visibleOptions]);
  useEffect(() => {
    if (!slider) return;
    // The engine keeps the latest stops only: a newer `movesSlider` request supersedes these.
    const current = () => slider.request === latest.current;
    const { filter } = slider;
    slicer.movesRange([slider.stops.span[0], slider.high], viewType, ALL_ROLES & ~filter.hiddenRoles, filter.visibleOptions).then(
      (result) => {
        if (!current()) return;
        onMoves({ toolpaths: slider.toolpaths, view: result, filter });
        setApplied({ slider, viewType });
      },
      (error: unknown) => {
        if (current()) throw error;
      },
    );
  }, [slider, viewType, onMoves]);
  if (slider?.toolpaths !== toolpaths) return null;
  const { stops, high } = slider;
  const { filter } = slider;
  const busy = applied?.slider !== slider || applied.viewType !== viewType || filter.layers !== layers
    || filter.hiddenRoles !== hiddenRoles || filter.visibleOptions !== visibleOptions;
  return (
    <div className="moves-slider" data-testid="moves-slider" aria-busy={busy}>
      <input type="range" aria-label="Moves" min={0} max={stops.gcodeIds.length - 1} step={1} value={high}
        aria-valuemax={stops.gcodeIds.length - 1}
        aria-valuetext={`G-code line ${stops.gcodeIds[high]}`}
        onChange={(event) => setSlider({ ...slider, high: Number(event.currentTarget.value) })} />
      <span className="moves-slider-label">{stops.gcodeIds[high]}</span>
    </div>
  );
}
