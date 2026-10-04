import { Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { Sidebar } from "../sidebar/Sidebar";
import { setPreview, slicePlate } from "../state/actions";
import { useApp } from "../state/store";
import { usedRoles } from "../viewport/toolpathMesh";
import { ViewportView } from "../viewport/ViewportView";
import { GCodeText } from "./GCodeText";
import { LayerSlider } from "./LayerSlider";
import { Legend } from "./Legend";

export function PreviewPage() {
  const scene = useApp((state) => state.scene);
  const slice = useApp((state) => state.slice);
  const view = useApp((state) => state.preview);
  const toolpaths = slice.status === "done" ? slice.toolpaths : null;
  const roles = useMemo(() => (toolpaths ? usedRoles(toolpaths) : []), [toolpaths]);
  return (
    <div className="page">
      <Sidebar />
      <ViewportView toolpaths={toolpaths} view={view}>
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
        {toolpaths && <Legend roles={roles} view={view} onChange={setPreview} />}
        {toolpaths && toolpaths.layerZs.length > 0 && (
          <LayerSlider zs={toolpaths.layerZs} value={view.layers} onChange={(layers) => setPreview({ layers })} />
        )}
      </ViewportView>
      {slice.status === "done" && <GCodeText gcode={slice.gcode} />}
    </div>
  );
}
