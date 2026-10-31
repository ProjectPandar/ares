import type { MovesView, Scene, SceneInstance, Toolpaths } from "@ares-slicer/wasm";
import { type ReactNode, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { cmdDown, isShortcut } from "../shell/keys";
import type { ToolpathFilter } from "./toolpathMesh";
import { selectObject } from "../state/actions";
import { moveInstance, rotateInstance } from "../state/plates";
import { openStlSession, placedScene, presetSliceBlocker } from "../state/presets";
import { useApp } from "../state/store";
import { firstString } from "../sidebar/options";
import { bedFromSettings } from "./bedShape";
import type { ViewPreset, ZoomTarget } from "./camera";
import { ArrangeMenu } from "./ArrangeMenu";
import type { GizmoMode } from "./gizmos";
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
    current: color("--accent"),
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

/**
 * The instances the canvas draws: per object in scene order its name and each instance's printable
 * flag and column-major matrix, and per plate its `[object, instance]` positions in that order.
 */
function instancesProbe(scene: Scene | null) {
  if (!scene) return undefined;
  const objects = scene.objects.map(({ name, instances }) => ({ name, instances: instances.map(({ printable, matrix }) => ({ printable, matrix })) }));
  const position = ([objectId, instanceId]: readonly number[]) => {
    const at = (instances: SceneInstance[]) => instances.findIndex(({ id }) => id[0] === objectId && id[1] === instanceId);
    const object = scene.objects.findIndex(({ instances }) => at(instances) >= 0);
    return [object, at(scene.objects[object].instances)];
  };
  return JSON.stringify({ objects, plates: scene.plates.map((plate) => plate.instances.map(position)) });
}

export function ViewportView({ toolpaths = null, colors = null, enabled = null, view, fit = "plate", editable = false, children }: {
  toolpaths?: Toolpaths | null;
  /** Per-vertex RGB of `toolpaths` (`ViewColors.colors`); `null` keeps `Toolpaths.colors`. */
  colors?: Uint8Array | null;
  /** Enabled segments and option markers of `toolpaths`; `null` draws all. */
  enabled?: MovesView | null;
  view?: ToolpathFilter;
  /** Initial framing when the scene loads. */
  fit?: Exclude<ZoomTarget, "bed" | "plates">;
  /** Instances move by drag and the move gizmo (Prepare). */
  editable?: boolean;
  children?: ReactNode;
}) {
  const host = useRef<HTMLDivElement>(null);
  const [viewport, setViewport] = useState<Viewport | null>(null);
  const [active, setActive] = useState<ViewPreset | null>("plate");
  const [gizmo, setGizmo] = useState<GizmoMode>("move");
  const theme = useApp((state) => state.theme);
  const loaded = useApp((state) => state.scene);
  const file = useApp((state) => state.file);
  const selection = useApp((state) => state.presetSelection);
  const vendors = useApp((state) => state.vendors);
  const presetImports = useApp((state) => state.presetImports);
  const selected = useApp((state) => state.selectedObject);
  const currentPlate = useApp((state) => state.currentPlate);
  const plater = useApp((state) => state.plater);
  // With presets selected the plate shows the model and bed the preset slice uses, re-placed on
  // every printer switch (on_select_preset -> on_config_change -> set_bed_shape, Plater.cpp:9576-9580,
  // 16754-16755). An STL becomes a plater session there; plater edits keep the placement.
  const [placed, setPlaced] = useState<{ inputs: unknown[]; scene: Scene } | null>(null);
  const inputs = [file, selection, vendors, presetImports];
  const current = placed && placed.inputs.every((input, at) => input === inputs[at]) ? placed.scene : null;
  // A plater session keeps its edited objects and plates; the placement supplies the composed bed.
  const scene = useMemo(
    () => (current && plater && loaded ? { ...loaded, settings: current.settings } : current ?? loaded),
    [current, loaded, plater],
  );
  useEffect(() => {
    const model = useApp.getState().scene;
    if (!file || !model || !selection || presetSliceBlocker(selection, model)) return;
    let stale = false;
    const sources = { vendors, presetImports };
    // A selection the engine cannot compose keeps the loaded plate; the process tab shows the
    // same composition error (usePresetValues).
    (model.format === "stl" ? openStlSession(file, model, selection, sources) : placedScene(file, model, selection, sources)).then(
      (result) => !stale && setPlaced({ inputs: [file, selection, vendors, presetImports], scene: result }),
      () => !stale && setPlaced(null),
    );
    return () => {
      stale = true;
    };
  }, [file, selection, vendors, presetImports]);
  // A new scene or slice result resets the camera to the plate view; plater edits keep the
  // session's settings and the camera.
  const frame = plater ? scene?.settings : scene;
  const [framed, setFramed] = useState([frame, toolpaths]);
  if (framed[0] !== frame || framed[1] !== toolpaths) {
    setFramed([frame, toolpaths]);
    setActive("plate");
  }
  const plateCount = scene?.plates.length ?? 0;
  const counted = useRef({ frame, plateCount });

  useEffect(() => {
    let instance: Viewport | null = null;
    const onKey = (event: KeyboardEvent) => {
      if (!instance || !isShortcut(event) || event.altKey) return;
      const key = event.key.toLowerCase();
      // GLCanvas3D::on_char without Ctrl: I / O zoom in / out; GLGizmosManager: M move, R rotate.
      if (!cmdDown(event) && (key === "i" || key === "o")) {
        instance.zoomBy(key === "i" ? 1 : -1);
        return;
      }
      if (!cmdDown(event) && (key === "m" || key === "r")) {
        setGizmo(key === "m" ? "move" : "rotate");
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
    const origins = scene?.plates.map(({ origin }): [number, number] => origin ?? [0, 0]);
    viewport?.setBed(bedFromSettings(scene?.settings ?? null), origins?.length ? origins : undefined, currentPlate);
  }, [viewport, scene, currentPlate]);
  useEffect(() => {
    viewport?.setScene(scene, firstString(loaded?.settings?.filament, "filament_colour") ?? "#29b2a3");
  }, [viewport, scene, loaded]);

  // A press on an instance selects it (GLCanvas3D::on_mouse LeftDown) and drags it.
  useEffect(() => {
    if (!viewport) return;
    const select = (object: number) => {
      const id = useApp.getState().scene?.objects[object]?.id;
      if (id !== undefined && useApp.getState().selectedObject !== id) selectObject(id);
    };
    viewport.setEditHandlers(editable && plater ? {
      move: (target, displacement) => {
        select(target.object);
        void moveInstance(target, displacement);
      },
      rotate: (target, radians) => void rotateInstance(target, radians),
    } : undefined, gizmo);
  }, [viewport, editable, plater, gizmo]);
  useEffect(() => viewport?.setSelected(selected), [viewport, selected]);
  // A new scene or slice result is re-framed: Prepare the plate, Preview the print.
  useEffect(() => {
    if (!viewport) return;
    viewport.setToolpaths(toolpaths);
    viewport.setModelsVisible(!toolpaths);
    viewport.setView("plate");
    viewport.zoomTo(frame ? fit : "plate");
  }, [viewport, frame, toolpaths, fit]);
  // An added plate frames every plate (Plater::priv::on_action_add_plate, REQUIRES_ZOOM_TO_ALL_PLATE).
  useEffect(() => {
    if (viewport && counted.current.frame === frame && plateCount > counted.current.plateCount) viewport.zoomTo("plates");
    counted.current = { frame, plateCount };
  }, [viewport, frame, plateCount]);
  // Layout effects: the commit that shows a preview state already has its draw queued.
  useLayoutEffect(() => {
    if (view) viewport?.setPreviewView(view);
  }, [viewport, view]);
  useLayoutEffect(() => {
    if (colors) viewport?.setToolpathColors(colors);
  }, [viewport, colors]);
  useLayoutEffect(() => {
    if (!enabled || !toolpaths) return;
    viewport?.setEnabled(enabled.segments, enabled.options, toolpaths.vertexCount);
    viewport?.setToolMarker(toolpaths, enabled.marker?.vertex ?? null);
  }, [viewport, enabled, toolpaths]);

  return (
    <div className="viewport" data-instances={instancesProbe(scene)}>
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
      {editable && plater && (
        <div className="gizmo-toolbar" role="toolbar" aria-label="Gizmos">
          {(["move", "rotate"] as const).map((mode) => (
            <button key={mode} className="view-button" aria-pressed={gizmo === mode} title={mode === "move" ? "Move (M)" : "Rotate (R)"}
              onClick={() => setGizmo(mode)}>{mode === "move" ? "Move" : "Rotate"}</button>
          ))}
          <span className="view-separator" />
          <ArrangeMenu />
        </div>
      )}
      {children}
    </div>
  );
}
