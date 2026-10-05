import type { Toolpaths } from "@ares-slicer/wasm";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { cmdDown, isShortcut } from "../shell/keys";
import { selectObject } from "../state/actions";
import { type PreviewView, useApp } from "../state/store";
import { firstString } from "../sidebar/options";
import { bedFromSettings } from "./bedShape";
import type { ViewPreset, ZoomTarget } from "./camera";
import type { Viewport, ViewportColors } from "./Viewport";

// Three.js lives in this chunk; requesting it with the entry lets the shell render while it downloads.
const viewportModule = import("./Viewport");

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

/** GLCanvas3D::on_char with Ctrl/Cmd: 0 plate + zoom_to_bed, 1-6 axis views. */
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
  const [viewport, setViewport] = useState<Viewport | null>(null);
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
    let instance: Viewport | null = null;
    const onKey = (event: KeyboardEvent) => {
      if (!instance || !isShortcut(event) || event.altKey) return;
      const key = event.key.toLowerCase();
      // GLCanvas3D::on_char without Ctrl: I / O zoom in / out.
      if (!cmdDown(event) && (key === "i" || key === "o")) {
        instance.zoomBy(key === "i" ? 1 : -1);
        return;
      }
      const preset = VIEW_KEYS[event.key];
      if (!preset || !cmdDown(event)) return;
      event.preventDefault();
      instance.setView(preset);
      if (preset === "plate") instance.zoomTo("bed");
      setActive(preset);
    };
    let disposed = false;
    void viewportModule.then(({ Viewport: ViewportClass }) => {
      if (disposed) return;
      instance = new ViewportClass(host.current!, themeColors());
      instance.onPick = selectObject;
      instance.onOrbit = () => setActive(null);
      window.addEventListener("keydown", onKey);
      setViewport(instance);
    });
    return () => {
      disposed = true;
      window.removeEventListener("keydown", onKey);
      instance?.dispose();
    };
  }, []);

  // `theme` switches the CSS variables that themeColors() reads.
  // oxlint-disable-next-line react/exhaustive-effect-dependencies
  useEffect(() => viewport?.setColors(themeColors()), [viewport, theme]);

  useEffect(() => {
    viewport?.setBed(bedFromSettings(scene?.settings ?? null));
    viewport?.setScene(scene, firstString(scene?.settings?.filament, "filament_colour") ?? "#29b2a3");
  }, [viewport, scene]);

  useEffect(() => viewport?.setSelected(selected), [viewport, selected]);
  // A new scene or slice result is re-framed: Prepare the plate, Preview the print.
  useEffect(() => {
    if (!viewport) return;
    viewport.setToolpaths(toolpaths);
    viewport.setModelsVisible(!toolpaths);
    viewport.setView("plate");
    viewport.zoomTo(scene ? fit : "plate");
  }, [viewport, scene, toolpaths, fit]);
  useEffect(() => {
    if (view) viewport?.setPreviewView(view);
  }, [viewport, view]);

  return (
    <div className="viewport">
      <div className="viewport-canvas" ref={host} data-testid="viewport" />
      <div className="view-toolbar" role="toolbar" aria-label="Camera view" data-testid="view-toolbar">
        {VIEWS.map(([preset, label]) => (
          <button key={preset} className="view-button" aria-pressed={active === preset}
            onClick={() => {
              viewport?.setView(preset);
              setActive(preset);
            }}>
            {label}
          </button>
        ))}
        <span className="view-separator" />
        <button className="view-button" aria-label="Fit to plate" title="Fit to plate"
          onClick={() => {
            viewport?.setView("plate");
            viewport?.zoomTo("plate");
            setActive("plate");
          }}>Plate</button>
        <button className="view-button" aria-label="Fit to objects" title="Fit to objects" disabled={!scene}
          onClick={() => viewport?.zoomTo("objects")}>Fit</button>
      </div>
      {children}
    </div>
  );
}
