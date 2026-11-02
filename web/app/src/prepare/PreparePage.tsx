import { useEffect } from "react";
import { cmdDown, isShortcut } from "../shell/keys";
import { Sidebar } from "../sidebar/Sidebar";
import { deleteAllObjects } from "../state/plates";
import { useApp } from "../state/store";
import { ViewportView } from "../viewport/ViewportView";

export function PreparePage() {
  const scene = useApp((state) => state.scene);
  // "Delete All" (Shortcut::DeleteAll, Ctrl+D, Shortcuts.cpp:80), enabled on the Prepare tab while the
  // model has objects (MainFrame::can_delete_all, MainFrame.cpp:2128-2131).
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!isShortcut(event) || !cmdDown(event) || event.shiftKey || event.altKey || event.key.toLowerCase() !== "d") return;
      const { scene: current, slice } = useApp.getState();
      if (!current?.objects.length || slice.status === "slicing") return;
      event.preventDefault();
      void deleteAllObjects();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
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
