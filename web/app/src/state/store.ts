import type { ConfigOverrides, Scene, Toolpaths } from "@ares-slicer/wasm";
import { create } from "zustand";

export type Theme = "dark" | "light";

export type SliceState =
  | { status: "idle" }
  | { status: "slicing"; startedAt: number }
  | { status: "done"; gcode: Uint8Array<ArrayBuffer>; toolpaths: Toolpaths | null; elapsedMs: number }
  | { status: "error"; error: string };

/** GCodeViewer visibility state: layer range, hidden roles and options. */
export interface PreviewView {
  /** Inclusive `[bottom, top]` layer indices. */
  layers: [number, number];
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
}

export const DEFAULT_PREVIEW: PreviewView = { layers: [0, 0], hiddenRoles: 0, travel: false, wipe: false };

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
}));
