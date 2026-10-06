import { EXTRUSION_ROLES, type MovesSlider as Stops, type MovesView, type Toolpaths } from "@ares-slicer/wasm";
import { type Dispatch, type SetStateAction, useEffect, useRef, useState } from "react";
import { cmdDown, isShortcut } from "../shell/keys";
import { slicer } from "../state/slicer";
import { type PreviewView, useApp } from "../state/store";
import type { ToolpathFilter } from "../viewport/toolpathMesh";

const ALL_ROLES = (1 << EXTRUSION_ROLES.length) - 1;

type SliderState = { toolpaths: Toolpaths; stops: Stops; high: number; request: number; filter: ToolpathFilter };
type Applied = { slider: SliderState; viewType: string; high: number };
type Moves = { high: number; max: number; request: (high: number) => void };
/** `stepLayers` and `stepTopLayer` of the app actions. */
export type LayerSteps = { stepLayers: (delta: number, wheel: boolean) => void; stepTopLayer: (delta: number) => void };

/** The engine's view between the moves slider thumbs for `toolpaths`, and the filter it was made for. */
export type MovesState = { toolpaths: Toolpaths; view: MovesView; filter: ToolpathFilter };

/**
 * DoubleSlider navigation in Preview (GLCanvas3D::on_key,
 * `GLCanvas3D.cpp:3844-3885`): Up/Down move the active layer thumb and, for the
 * upper one, set the moves thumb to the max (`:3851-3857`); Left/Right move the
 * moves thumb, past its first (last) stop the upper layer thumb down (up) with
 * the moves thumb at the max (min) (`:3862-3875`); Home/End jump to the ends.
 * x5 with Shift or Ctrl/Cmd. `moves` is the engine's applied moves thumb and
 * last stop, `null` while a rebuild or request is pending; the engine clamps
 * the requested thumb.
 */
function onNavigationKey(event: KeyboardEvent, moves: Moves | null, { stepLayers, stepTopLayer }: LayerSteps) {
  if (!isShortcut(event) || event.altKey) return;
  const increment = cmdDown(event) || event.shiftKey ? 5 : 1;
  const { preview, slice } = useApp.getState();
  if (slice.status !== "done" || !slice.toolpaths) return;
  const [, top] = preview.layers;
  const last = slice.toolpaths.layerZs.length - 1;
  switch (event.key) {
    case "ArrowUp":
    case "ArrowDown":
      if (preview.oneLayer || preview.selection === 1) moves?.request(moves.max);
      stepLayers(event.key === "ArrowUp" ? increment : -increment, false);
      break;
    case "ArrowLeft":
      if (!moves) return;
      if (moves.high === 0 && top > 0) {
        moves.request(moves.max);
        stepTopLayer(-1);
      } else moves.request(moves.high - increment);
      break;
    case "ArrowRight":
      if (!moves) return;
      if (moves.high === moves.max && top < last) {
        moves.request(0);
        stepTopLayer(1);
      } else moves.request(moves.high + increment);
      break;
    case "Home":
    case "End":
      if (!moves) return;
      moves.request(event.key === "Home" ? 0 : moves.max);
      break;
    default:
      return;
  }
  event.preventDefault();
}

/**
 * Horizontal moves slider under the preview (GCodeViewer `m_moves_slider`,
 * the `IMSlider` higher thumb): stop `i` shows the shown layers' moves up to
 * G-code line `stops.gcodeIds[i]`. The engine rebuilds the stops and places the
 * thumbs for the shown layers and legend (`update_moves_slider`,
 * `GCodeViewer.cpp:1765-1821`), starting from the thumb last requested here;
 * reports the engine's view between the thumbs (`set_view_visible_range`) to
 * `onMoves`; `aria-busy` while that view is behind the slider, layers or legend.
 */
export function MovesSlider({ toolpaths, view, onMoves, steps }: {
  toolpaths: Toolpaths;
  view: PreviewView;
  onMoves: Dispatch<SetStateAction<MovesState | null>>;
  steps: LayerSteps;
}) {
  const [slider, setSlider] = useState<SliderState | null>(null);
  const [applied, setApplied] = useState<Applied | null>(null);
  const latest = useRef(0);
  // The thumb the next rebuild starts from: requests reach the engine after the rebuild a key also causes.
  const held = useRef<{ toolpaths: Toolpaths; high: number } | null>(null);
  const { layers, hiddenRoles, visibleOptions, viewType } = view;
  useEffect(() => {
    const request = ++latest.current;
    const high = held.current?.toolpaths === toolpaths ? held.current.high : undefined;
    void slicer.movesSlider(layers, ALL_ROLES & ~hiddenRoles, visibleOptions, high).then((stops) => {
      const filter = { layers, hiddenRoles, visibleOptions };
      if (request !== latest.current) return;
      held.current = { toolpaths, high: stops.thumbs[1] };
      setSlider({ toolpaths, stops, high: stops.thumbs[1], request, filter });
    });
  }, [toolpaths, layers, hiddenRoles, visibleOptions]);
  useEffect(() => {
    if (!slider) return;
    // The engine keeps the latest stops only: a newer `movesSlider` request supersedes these.
    const current = () => slider.request === latest.current;
    const { filter } = slider;
    slicer.movesHigher(slider.high, viewType, ALL_ROLES & ~filter.hiddenRoles, filter.visibleOptions).then(
      (result) => {
        if (!current()) return;
        onMoves({ toolpaths: slider.toolpaths, view: result, filter });
        setApplied({ slider, viewType, high: result.thumbs[1] });
      },
      (error: unknown) => {
        if (current()) throw error;
      },
    );
  }, [slider, viewType, onMoves]);
  const live = slider?.toolpaths === toolpaths ? slider : null;
  const settled = live && applied?.slider === live ? applied : null;
  const high = settled ? settled.high : live?.high ?? 0;
  useEffect(() => {
    const request = settled && ((value: number) => {
      held.current = { toolpaths: settled.slider.toolpaths, high: value };
      setSlider({ ...settled.slider, high: value });
    });
    const moves = request && { high, max: settled.slider.stops.gcodeIds.length - 1, request };
    const onKey = (event: KeyboardEvent) => onNavigationKey(event, moves, steps);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [settled, high, steps]);
  if (!live) return null;
  const { stops, filter } = live;
  const busy = applied?.slider !== live || applied.viewType !== viewType || filter.layers !== layers
    || filter.hiddenRoles !== hiddenRoles || filter.visibleOptions !== visibleOptions;
  return (
    <div className="moves-slider" data-testid="moves-slider" aria-busy={busy}>
      <input type="range" aria-label="Moves" min={0} max={stops.gcodeIds.length - 1} step={1} value={high}
        aria-valuemax={stops.gcodeIds.length - 1}
        aria-valuetext={`G-code line ${stops.gcodeIds[high]}`}
        onChange={(event) => {
          const value = Number(event.currentTarget.value);
          held.current = { toolpaths: live.toolpaths, high: value };
          setSlider({ ...live, high: value });
        }} />
      <span className="moves-slider-label">{stops.gcodeIds[high]}</span>
    </div>
  );
}
