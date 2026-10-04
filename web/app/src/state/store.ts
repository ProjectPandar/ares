import type { Scene } from "@ares-slicer/wasm";
import { create } from "zustand";

export type Theme = "dark" | "light";

export type SliceState =
  | { status: "idle" }
  | { status: "slicing"; startedAt: number }
  | { status: "done"; gcode: Uint8Array; elapsedMs: number }
  | { status: "error"; error: string };

export interface AppState {
  theme: Theme;
  engine: "loading" | "ready" | "error";
  engineError?: string;
  file: { name: string; bytes: Uint8Array } | null;
  scene: Scene | null;
  loadingFile: boolean;
  loadError?: string;
  selectedObject: number | null;
  slice: SliceState;
}

export const useApp = create<AppState>(() => ({
  theme: (localStorage.getItem("ares.theme") as Theme | null) ?? "dark",
  engine: "loading",
  file: null,
  scene: null,
  loadingFile: false,
  selectedObject: null,
  slice: { status: "idle" },
}));
