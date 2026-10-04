import { Link } from "@tanstack/react-router";
import { Sidebar } from "../sidebar/Sidebar";
import { slicePlate } from "../state/actions";
import { useApp } from "../state/store";
import { ViewportView } from "../viewport/ViewportView";
import { GCodeText } from "./GCodeText";

export function PreviewPage() {
  const scene = useApp((state) => state.scene);
  const slice = useApp((state) => state.slice);
  return (
    <div className="page">
      <Sidebar />
      <ViewportView>
        <div className="preview-card" data-testid="slice-summary">
          <div className="preview-card-title">Slice result</div>
          {slice.status === "done" ? (
            <dl className="summary">
              <dt>G-code size</dt><dd data-testid="gcode-bytes">{slice.gcode.length.toLocaleString()} bytes</dd>
              <dt>Slicing time</dt><dd>{(slice.elapsedMs / 1000).toFixed(2)} s</dd>
              <dt>Plate</dt><dd>{scene?.plates[0]?.id ?? 1}</dd>
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
          <div className="preview-note">
            Toolpath rendering, layer slider and feature legend wait on the
            GCodeProcessor move export from ares-core.
          </div>
        </div>
      </ViewportView>
      {slice.status === "done" && <GCodeText gcode={slice.gcode} />}
    </div>
  );
}
