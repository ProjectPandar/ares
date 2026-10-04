import type { Toolpaths } from "@ares-slicer/wasm";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { selectObject } from "../state/actions";
import { type PreviewView, useApp } from "../state/store";
import { firstString } from "../sidebar/options";
import { bedFromSettings } from "./bed";
import type { ViewPreset, ZoomTarget } from "./camera";
import { Viewport, type ViewportColors } from "./Viewport";

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

const VIEWS: [ViewPreset, string][] = [
  ["iso", "Iso"], ["top", "Top"], ["bottom", "Bottom"], ["front", "Front"],
  ["rear", "Rear"], ["left", "Left"], ["right", "Right"],
];

/** GLCanvas3D::on_char: 0 plate + zoom_to_bed, 1-6 axis views. */
const VIEW_KEYS: Record<string, ViewPreset> = {
  "0": "plate", "1": "top", "2": "bottom", "3": "front", "4": "rear", "5": "left", "6": "right",
};

export function ViewportView({ toolpaths = null, view, fit = "plate", children }: {
  toolpaths?: Toolpaths | null;
  view?: PreviewView;
  /** Initial framing when the scene loads. */
  fit?: Exclude<ZoomTarget, "bed">;
  children?: ReactNode;
}) {
  const host = useRef<HTMLDivElement>(null);
  const viewport = useRef<Viewport | null>(null);
  const [active, setActive] = useState<ViewPreset | null>("plate");
  const theme = useApp((state) => state.theme);
  const scene = useApp((state) => state.scene);
  const selected = useApp((state) => state.selectedObject);
  // A new scene or slice result resets the camera to the plate view.
  const [framed, setFramed] = useState([scene, toolpaths]);
  if (framed[0] !== scene || framed[1] !== toolpaths) {
    setFramed([scene, toolpaths]);
    setActive("plate");
  }

  useEffect(() => {
    const instance = new Viewport(host.current!, themeColors());
    instance.onPick = selectObject;
    instance.onOrbit = () => setActive(null);
    viewport.current = instance;
    const onKey = (event: KeyboardEvent) => {
      const preset = VIEW_KEYS[event.key];
      if (!preset || event.ctrlKey || event.metaKey || event.altKey) return;
      if ((event.target as HTMLElement).closest("input, textarea, select")) return;
      instance.setView(preset);
      if (preset === "plate") instance.zoomTo("bed");
      setActive(preset);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
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
  }, [scene]);

  useEffect(() => viewport.current?.setSelected(selected), [selected]);
  // A new scene or slice result is re-framed: Prepare the plate, Preview the print.
  useEffect(() => {
    const instance = viewport.current!;
    instance.setToolpaths(toolpaths);
    instance.setModelsVisible(!toolpaths);
    instance.setView("plate");
    instance.zoomTo(scene ? fit : "plate");
  }, [scene, toolpaths, fit]);
  useEffect(() => {
    if (view) viewport.current?.setPreviewView(view);
  }, [view]);

  return (
    <div className="viewport">
      <div className="viewport-canvas" ref={host} data-testid="viewport" />
      <div className="view-toolbar" role="toolbar" aria-label="Camera view" data-testid="view-toolbar">
        {VIEWS.map(([preset, label]) => (
          <button key={preset} className="view-button" aria-pressed={active === preset}
            onClick={() => {
              viewport.current?.setView(preset);
              setActive(preset);
            }}>
            {label}
          </button>
        ))}
        <span className="view-separator" />
        <button className="view-button" aria-label="Fit to plate" title="Fit to plate"
          onClick={() => {
            viewport.current?.setView("plate");
            viewport.current?.zoomTo("plate");
            setActive("plate");
          }}>Plate</button>
        <button className="view-button" aria-label="Fit to objects" title="Fit to objects" disabled={!scene}
          onClick={() => viewport.current?.zoomTo("objects")}>Fit</button>
      </div>
      {children}
    </div>
  );
}
