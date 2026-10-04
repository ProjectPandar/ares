import type { Toolpaths } from "@ares-slicer/wasm";
import { type ReactNode, useEffect, useRef } from "react";
import { selectObject } from "../state/actions";
import { type PreviewView, useApp } from "../state/store";
import { firstString } from "../sidebar/options";
import { bedFromSettings } from "./bed";
import { type ViewPreset, Viewport, type ViewportColors } from "./Viewport";

function themeColors(): ViewportColors {
  const style = getComputedStyle(document.documentElement);
  const color = (name: string) => parseInt(style.getPropertyValue(name).trim().slice(1), 16);
  return {
    background: color("--canvas-bg"),
    bed: color("--bed"),
    gridMinor: color("--bed-grid"),
    gridMajor: color("--bed-grid-major"),
    outline: color("--bed-outline"),
  };
}

const VIEWS: [ViewPreset, string][] = [["iso", "Iso"], ["top", "Top"], ["front", "Front"], ["left", "Left"]];

export function ViewportView({ toolpaths = null, view, children }: {
  toolpaths?: Toolpaths | null;
  view?: PreviewView;
  children?: ReactNode;
}) {
  const host = useRef<HTMLDivElement>(null);
  const viewport = useRef<Viewport | null>(null);
  const theme = useApp((state) => state.theme);
  const scene = useApp((state) => state.scene);
  const selected = useApp((state) => state.selectedObject);

  useEffect(() => {
    const instance = new Viewport(host.current!, themeColors());
    instance.onPick = selectObject;
    viewport.current = instance;
    return () => {
      instance.dispose();
      viewport.current = null;
    };
  }, []);

  // `theme` switches the CSS variables that themeColors() reads.
  // oxlint-disable-next-line react/exhaustive-effect-dependencies
  useEffect(() => viewport.current?.setColors(themeColors()), [theme]);

  useEffect(() => {
    const instance = viewport.current!;
    instance.setBed(bedFromSettings(scene?.settings ?? null));
    instance.setScene(scene, firstString(scene?.settings?.filament, "filament_colour") ?? "#29b2a3");
    instance.setView("iso");
  }, [scene]);

  useEffect(() => viewport.current?.setSelected(selected), [selected]);
  useEffect(() => {
    viewport.current?.setToolpaths(toolpaths);
    viewport.current?.setModelsVisible(!toolpaths);
  }, [toolpaths]);
  useEffect(() => {
    if (view) viewport.current?.setPreviewView(view);
  }, [view]);

  return (
    <div className="viewport">
      <div className="viewport-canvas" ref={host} data-testid="viewport" />
      <div className="view-toolbar" role="toolbar" aria-label="Camera view">
        {VIEWS.map(([preset, label]) => (
          <button key={preset} className="view-button" onClick={() => viewport.current?.setView(preset)}>
            {label}
          </button>
        ))}
      </div>
      {children}
    </div>
  );
}
