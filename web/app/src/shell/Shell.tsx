import { Outlet } from "@tanstack/react-router";
import { useLayoutEffect } from "react";
import { useApp } from "../state/store";
import { StatusBar } from "./StatusBar";
import { TabBar, TitleBar } from "./TopBar";

export function Shell() {
  const theme = useApp((state) => state.theme);
  // Layout effect: CSS theme variables must be live before the viewport reads them.
  useLayoutEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
  return (
    <div className="shell">
      <TitleBar />
      <TabBar />
      <main className="workspace">
        <Outlet />
      </main>
      <StatusBar />
    </div>
  );
}
