import { Link } from "@tanstack/react-router";
import { EXTRUSION_ROLES, type Toolpaths, type ViewColors } from "@ares-slicer/wasm";
import { lazy, Suspense, useEffect, useMemo, useState } from "react";
import { Sidebar } from "../sidebar/Sidebar";
import { cmdDown, isShortcut } from "../shell/keys";
import { setPreview, slicePlate, stepLayers, toggleGcodeWindow, toggleOneLayer } from "../state/actions";
import { slicer } from "../state/slicer";
import { useApp } from "../state/store";
import { usedOptions, usedRoles } from "../viewport/roles";
import { ViewportView } from "../viewport/ViewportView";
import { LayerSlider } from "./LayerSlider";

// Fetched once a slice has move vertices to show.
const Legend = lazy(() => import("./Legend").then((module) => ({ default: module.Legend })));
const GCodeWindow = lazy(() => import("./GCodeWindow").then((module) => ({ default: module.GCodeWindow })));

/**
 * Preview keys: C toggles the G-code window (GLCanvas3D::on_char), L one layer
 * mode (Preview::update_layers_slider_from_canvas, no modifiers), Up/Down move
 * the active layer thumb, x5 with Shift or Ctrl/Cmd (GLCanvas3D::on_key).
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
  else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();
    const step = cmdDown(event) || event.shiftKey ? 5 : 1;
    stepLayers(event.key === "ArrowUp" ? step : -step, false);
  }
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
  if (lineType) return { colors: toolpaths.colors, range: null };
  return state?.toolpaths === toolpaths ? state.colors : null;
}

export function PreviewPage() {
  const scene = useApp((state) => state.scene);
  const slice = useApp((state) => state.slice);
  const view = useApp((state) => state.preview);
  const toolpaths = slice.status === "done" ? slice.toolpaths : null;
  const gcodeWindow = useApp((state) => state.gcodeWindow);
  useEffect(() => {
    window.addEventListener("keydown", onPreviewKey);
    return () => window.removeEventListener("keydown", onPreviewKey);
  }, []);
  const roles = useMemo(() => (toolpaths ? usedRoles(toolpaths) : []), [toolpaths]);
  const options = useMemo(() => (toolpaths ? usedOptions(toolpaths) : []), [toolpaths]);
  const colors = useViewColors(toolpaths, view.viewType, view.hiddenRoles, view.visibleOptions);
  return (
    <div className="page">
      <Sidebar />
      <ViewportView toolpaths={toolpaths} colors={colors?.colors ?? null} view={view} fit="objects">
        <div className="preview-card" data-testid="slice-summary">
          <div className="preview-card-title">Slice result</div>
          {slice.status === "done" ? (
            <dl className="summary">
              <dt>G-code size</dt><dd data-testid="gcode-bytes">{slice.gcode.length.toLocaleString()} bytes</dd>
              <dt>Slicing time</dt><dd>{(slice.elapsedMs / 1000).toFixed(2)} s</dd>
              <dt>Plate</dt><dd>{scene?.plates[0]?.id ?? 1}</dd>
              {toolpaths && <><dt>Layers</dt><dd data-testid="layer-count">{toolpaths.layerZs.length}</dd></>}
            </dl>
          ) : slice.status === "slicing" ? (
            <div className="summary-empty"><span className="spinner" /> Slicing…</div>
          ) : !scene ? (
            <div className="summary-empty">Nothing to preview. <Link to="/prepare">Import a model</Link>.</div>
          ) : (
            <div className="summary-empty">
              Plate not sliced. <button className="link-button" onClick={() => void slicePlate()}>Slice plate</button>
            </div>
          )}
          {slice.status === "done" && !toolpaths && (
            <div className="preview-note" data-testid="toolpaths-pending">
              Toolpaths appear once the Ares engine exports GCodeProcessor move vertices.
            </div>
          )}
        </div>
        {toolpaths && (
          <Suspense>
            <Legend roles={roles} options={options} toolpaths={toolpaths}
              range={colors?.range ?? null} view={view} onChange={setPreview} />
          </Suspense>
        )}
        {toolpaths && toolpaths.layerZs.length > 0 && (
          <LayerSlider zs={toolpaths.layerZs} view={view} />
        )}
        {slice.status === "done" && gcodeWindow && toolpaths && (
          <Suspense>
            <GCodeWindow gcode={slice.gcode} toolpaths={toolpaths} view={view} />
          </Suspense>
        )}
      </ViewportView>
    </div>
  );
}
