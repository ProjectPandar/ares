import { Link } from "@tanstack/react-router";
import { EXTRUSION_ROLES, type Scene, type Toolpaths, type ViewColors } from "@ares-slicer/wasm";
import { lazy, Suspense, useEffect, useMemo, useState } from "react";
import { cmdDown, isShortcut } from "../shell/keys";
import { setPreview, slicePlate, stepLayers, stepTopLayer, toggleGcodeWindow, toggleOneLayer } from "../state/actions";
import { slicer } from "../state/slicer";
import { useApp } from "../state/store";
import { usedOptions } from "../viewport/roles";
import { ViewportView } from "../viewport/ViewportView";
import { LayerSlider } from "./LayerSlider";
import { PlateStrip } from "./PlateStrip";
import type { LayerSteps, MovesState } from "./MovesSlider";

// Fetched once a slice has move vertices to show.
const Legend = lazy(() => import("./Legend").then((module) => ({ default: module.Legend })));
const LAYER_STEPS: LayerSteps = { stepLayers, stepTopLayer };
const MovesSlider = lazy(() => import("./MovesSlider").then((module) => ({ default: module.MovesSlider })));
const GCodeWindow = lazy(() => import("./GCodeWindow").then((module) => ({ default: module.GCodeWindow })));

/**
 * Preview keys: C toggles the G-code window (GLCanvas3D::on_char), L one layer
 * mode (Preview::update_layers_slider_from_canvas, no modifiers); the moves
 * slider handles the slider navigation keys.
 */
function onPreviewKey(event: KeyboardEvent) {
  if (!isShortcut(event) || event.altKey) return;
  const key = event.key.toLowerCase();
  if (key === "c" && !cmdDown(event)) {
    toggleGcodeWindow();
    return;
  }
  const { slice } = useApp.getState();
  if (slice.status !== "done" || !slice.toolpaths) return;
  if (key === "l" && !cmdDown(event) && !event.shiftKey) toggleOneLayer();
}

const ALL_ROLES = (1 << EXTRUSION_ROLES.length) - 1;

/**
 * Engine toolpath colors and color range of the legend's view type
 * (GCodeViewer `refresh` after the view type combo or a legend toggle). The
 * line type view is `Toolpaths.colors`, which has no range to follow toggles.
 */
function useViewColors(toolpaths: Toolpaths | null, viewType: string, hiddenRoles: number, visibleOptions: number) {
  const [state, setState] = useState<{ toolpaths: Toolpaths; colors: ViewColors } | null>(null);
  const lineType = viewType === "FeatureType";
  useEffect(() => {
    if (!toolpaths || lineType) return;
    let stale = false;
    void slicer.viewColors(viewType, ALL_ROLES & ~hiddenRoles, visibleOptions).then((colors) => {
      if (!stale) setState({ toolpaths, colors });
    });
    return () => {
      stale = true;
    };
  }, [toolpaths, lineType, viewType, hiddenRoles, visibleOptions]);
  if (!toolpaths) return null;
  if (lineType) return { colors: toolpaths.colors, range: null, colorsProbe: toolpaths.textures?.probes?.colors };
  return state?.toolpaths === toolpaths ? state.colors : null;
}

/** The Preview tab pane: mounted for the application, hidden while Prepare is shown. */
export function PreviewPage({ shown, placed }: { shown: boolean; placed: Scene | null }) {
  const scene = useApp((state) => state.scene);
  const currentPlate = useApp((state) => state.currentPlate);
  const slice = useApp((state) => state.slice);
  const view = useApp((state) => state.preview);
  const toolpaths = slice.status === "done" ? slice.toolpaths : null;
  const gcodeWindow = useApp((state) => state.gcodeWindow);
  useEffect(() => {
    if (!shown) return;
    window.addEventListener("keydown", onPreviewKey);
    return () => window.removeEventListener("keydown", onPreviewKey);
  }, [shown]);
  const options = useMemo(() => (toolpaths ? usedOptions(toolpaths) : []), [toolpaths]);
  const colors = useViewColors(toolpaths, view.viewType, view.hiddenRoles, view.visibleOptions);
  const [movesState, setMoves] = useState<MovesState | null>(null);
  const live = movesState?.toolpaths === toolpaths ? movesState : null;
  const moves = live?.view ?? null;
  // GCodeViewer::render: the window frames the current vertex's line (`GCodeViewer.cpp:1601`).
  const currentLine = toolpaths && moves ? toolpaths.ids[2 * moves.range[2]] : 0;
  // Tablets and phones start with the slicing result collapsed, keeping the 3D view clear.
  const [expanded, setExpanded] = useState(() => !window.matchMedia("(max-width: 1024px)").matches);
  const summary = slice.status === "done" && (
    <section>
      <h3>G-code</h3>
      <dl className="summary">
        <dt>G-code size</dt>
        <dd data-testid="gcode-bytes">{slice.gcode.length.toLocaleString()} bytes</dd>
        <dt>Slicing time</dt>
        <dd>{(slice.elapsedMs / 1000).toFixed(2)} s</dd>
        <dt>Plate</dt>
        <dd data-testid="slice-plate">{scene?.plates[currentPlate]?.id ?? 1}</dd>
        {toolpaths && (
          <>
            <dt>Layers</dt>
            <dd data-testid="layer-count">{toolpaths.layerZs.length}</dd>
          </>
        )}
      </dl>
    </section>
  );
  return (
    <div className="tab-pane" hidden={!shown}>
      <ViewportView
        scene={placed}
        shown={shown}
        toolpaths={toolpaths}
        colors={moves ?? colors}
        enabled={moves}
        view={live?.filter ?? view}
        fit="objects"
      >
        {(scene?.plates.length ?? 0) > 1 && <PlateStrip />}
        <div className="preview-right">
          <section className="slice-result" data-testid="slice-summary">
            <header className="slice-result-header">
              <span>Slicing Result</span>
              <button
                className="slice-result-collapse"
                aria-expanded={expanded}
                aria-label={expanded ? "Collapse slicing result" : "Expand slicing result"}
                title={expanded ? "Collapse" : "Expand"}
                onClick={() => setExpanded(!expanded)}
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <path d="M3.5 10l4.5-4.5 4.5 4.5" />
                </svg>
              </button>
            </header>
            {toolpaths ? (
              <Suspense>
                <Legend
                  options={options}
                  toolpaths={toolpaths}
                  range={colors?.range ?? null}
                  view={view}
                  onChange={setPreview}
                  summary={summary}
                />
              </Suspense>
            ) : slice.status === "done" ? (
              <>
                <div className="slice-result-stats">{summary}</div>
                <div className="preview-note" data-testid="toolpaths-pending">
                  Toolpaths appear once the Ares engine exports GCodeProcessor move vertices.
                </div>
              </>
            ) : slice.status === "slicing" ? (
              <div className="summary-empty">
                <span className="spinner" /> Slicing…
              </div>
            ) : !scene ? (
              <div className="summary-empty">
                Nothing to preview. <Link to="/prepare">Import a model</Link>.
              </div>
            ) : (
              <div className="summary-empty">
                Plate not sliced.{" "}
                <button className="link-button" onClick={() => void slicePlate()}>
                  Slice plate
                </button>
              </div>
            )}
          </section>
          {slice.status === "done" && gcodeWindow && currentLine !== 0 && (
            <Suspense>
              <GCodeWindow gcode={slice.gcode} current={currentLine} />
            </Suspense>
          )}
        </div>
        {toolpaths && toolpaths.layerZs.length > 0 && <LayerSlider zs={toolpaths.layerZs} view={view} />}
        {toolpaths && (
          <Suspense>
            <MovesSlider toolpaths={toolpaths} view={view} onMoves={setMoves} steps={LAYER_STEPS} shown={shown} />
          </Suspense>
        )}
      </ViewportView>
    </div>
  );
}
