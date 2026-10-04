import { Outlet } from "@tanstack/react-router";
import { type DragEvent, useLayoutEffect, useState } from "react";
import { importFiles } from "../state/actions";
import { useApp } from "../state/store";
import { StatusBar } from "./StatusBar";
import { TabBar, TitleBar } from "./TopBar";

const hasFiles = (event: DragEvent) => event.dataTransfer.types.includes("Files");

export function Shell() {
  const theme = useApp((state) => state.theme);
  const engine = useApp((state) => state.engine);
  const [dragging, setDragging] = useState(false);
  // Layout effect: CSS theme variables must be live before the viewport reads them.
  useLayoutEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
  return (
    <div className="shell">
      <TitleBar />
      <TabBar />
      <main className="workspace" data-testid="workspace"
        onDragOver={(event) => {
          if (!hasFiles(event) || engine !== "ready") return;
          event.preventDefault();
          setDragging(true);
        }}
        onDragLeave={(event) => {
          if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setDragging(false);
        }}
        onDrop={(event) => {
          if (!hasFiles(event) || engine !== "ready") return;
          event.preventDefault();
          setDragging(false);
          importFiles([...event.dataTransfer.files]);
        }}>
        <Outlet />
        {dragging && <div className="drop-overlay" data-testid="drop-overlay">Drop a 3MF project or STL model to import</div>}
      </main>
      <StatusBar />
    </div>
  );
}
