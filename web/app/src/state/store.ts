import type { ConfigOverrides, Scene, Toolpaths } from "@ares-slicer/wasm";
import { create } from "zustand";

export type Theme = "dark" | "light";

export type SliceState =
  | { status: "idle" }
  | { status: "slicing"; startedAt: number }
  | { status: "done"; gcode: Uint8Array<ArrayBuffer>; toolpaths: Toolpaths | null; elapsedMs: number }
  | { status: "error"; error: string };

/** Layer slider thumb: 0 lower, 1 upper (IMSlider `ssLower` / `ssHigher`). */
export type Thumb = 0 | 1;

/** GCodeViewer visibility state: layer range, hidden roles and options. */
export interface PreviewView {
  /** Inclusive `[bottom, top]` layer indices. */
  layers: [number, number];
  /** Thumb moved by canvas arrow keys and the wheel (IMSlider `m_selection`). */
  selection: Thumb;
  /** One layer mode (IMSlider `m_is_one_layer`) and its saved layer (`m_one_layer_value`). */
  oneLayer: boolean;
  oneLayerValue: number;
  /** Bit `i` hides `EXTRUSION_ROLES[i]`. */
  hiddenRoles: number;
  travel: boolean;
  wipe: boolean;
}

export interface AppState {
  theme: Theme;
  engine: "loading" | "ready" | "error";
  engineError?: string;
  file: { name: string; bytes: Uint8Array } | null;
  scene: Scene | null;
  loadingFile: boolean;
  loadError?: string;
  selectedObject: number | null;
  /** Process option edits on top of the project config. */
  overrides: ConfigOverrides;
  slice: SliceState;
  preview: PreviewView;
  /** G-code window visibility (GUI_App `show_gcode_window`). */
  gcodeWindow: boolean;
}

export const DEFAULT_PREVIEW: PreviewView = {
  layers: [0, 0],
  selection: 1,
  oneLayer: false,
  oneLayerValue: 0,
  hiddenRoles: 0,
  travel: false,
  wipe: false,
};

export const useApp = create<AppState>(() => ({
  theme: (localStorage.getItem("ares.theme") as Theme | null) ?? "dark",
  engine: "loading",
  file: null,
  scene: null,
  loadingFile: false,
  selectedObject: null,
  overrides: {},
  slice: { status: "idle" },
  preview: DEFAULT_PREVIEW,
  gcodeWindow: localStorage.getItem("ares.gcodeWindow") !== "false",
}));
