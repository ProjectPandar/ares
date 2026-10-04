import { useState } from "react";
import { Sidebar } from "../sidebar/Sidebar";
import { importFile } from "../state/actions";
import { useApp } from "../state/store";
import { ViewportView } from "../viewport/ViewportView";

export function PreparePage() {
  const scene = useApp((state) => state.scene);
  const engine = useApp((state) => state.engine);
  const [dragging, setDragging] = useState(false);
  return (
    <div className="page"
      onDragOver={(event) => {
        event.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={(event) => {
        event.preventDefault();
        setDragging(false);
        const file = event.dataTransfer.files[0];
        if (file && engine === "ready") void importFile(file);
      }}>
      <Sidebar />
      <ViewportView>
        {!scene && (
          <div className="canvas-hint">
            <div className="canvas-hint-title">Drag a 3MF project or STL model here</div>
            <div className="canvas-hint-sub">or use Import in the top bar</div>
          </div>
        )}
        {dragging && <div className="drop-overlay">Drop to import</div>}
      </ViewportView>
    </div>
  );
}
