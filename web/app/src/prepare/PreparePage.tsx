import { Sidebar } from "../sidebar/Sidebar";
import { useApp } from "../state/store";
import { ViewportView } from "../viewport/ViewportView";

export function PreparePage() {
  const scene = useApp((state) => state.scene);
  return (
    <div className="page">
      <Sidebar />
      <ViewportView editable>
        {!scene && (
          <div className="canvas-hint">
            <div className="canvas-hint-title">Drag a 3MF project or STL model here</div>
            <div className="canvas-hint-sub">or use Import in the top bar</div>
          </div>
        )}
      </ViewportView>
    </div>
  );
}
