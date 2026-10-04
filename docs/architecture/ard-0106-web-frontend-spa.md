# ARD-0106: Browser SPA frontend over the ares-wasm adapter

## Source and boundary

The web frontend in `web/` is a UI rewrite of OrcaSlicer's GUI layer
(pinned `8500fcdccaa10b5099ac20d252af3a7c560046f1`), not of `libslic3r`:

| Upstream (`src/slic3r/GUI/`) | Web destination |
| --- | --- |
| `MainFrame.cpp` top bar, Prepare / Preview tabs | `web/src/app/TopBar.tsx`, TanStack routes `/prepare`, `/preview` |
| `Plater.cpp` `Sidebar` (printer / filament / process presets), `GUI_ObjectList.cpp` | `web/src/sidebar/*` |
| `Tab.cpp` `TabPrint::build` page/group/option layout | `web/src/sidebar/processPages.ts` (layout only; values come from Rust) |
| `GLCanvas3D.cpp`, `3DBed.cpp` plate + model volumes | `web/src/viewport/*` (Three.js) |
| `GCodeViewer.cpp`, `IMSlider.cpp`, `LibVGCode/LibVGCodeWrapper.cpp` legend, layer slider | `web/src/preview/*` |

All slicing, project loading, option values and G-code logic stays in Rust.
TypeScript only renders data returned by `crates/ares-wasm`.

## Non-negotiable decisions

- **Stack:** Vite + React + TypeScript, TanStack Router (hash history so the
  static build works from any path), plain Three.js (no React renderer layer)
  owned by an imperative viewport class mounted from React. npm with a
  committed `web/package-lock.json`; `web/` is self-contained.
- **WASM boundary:** `ares-wasm` gains only thin byte-oriented bindings that
  call existing public `ares-core` APIs (`load_project`, `load_model`,
  `slice_project_plate`, `slice`). Project scene data crosses as one JSON
  metadata string plus flat `Float32Array`/`Uint32Array` buffers. No
  filesystem, terminal, or new pipeline code in the adapter. Volume/instance
  matrices are recovered through the public `Transform3d::transform_point`.
- **Threading:** all WASM calls run in a dedicated module Web Worker
  (`web/src/wasm/slicer.worker.ts`); the UI thread never instantiates the
  slicer. Results transfer `ArrayBuffer`s (zero-copy).
- **Build output:** `npm run build:wasm` runs
  `cargo build -p ares-wasm --target wasm32-unknown-unknown --release` and
  `wasm-bindgen --target web` into the git-ignored `web/src/wasm/pkg/`;
  `npm run build` emits a static SPA in `web/dist/`.
- **Look:** dense desktop layout with CSS custom-property theme tokens
  (dark/light), Orca teal accent, no UI framework.
- **E2E:** Playwright under `web/e2e/` drives the built SPA: import a
  sample 3MF, slice through the worker, assert G-code markers, screenshot
  Prepare and Preview, and write `target/web-e2e/` artifacts
  (screenshots + G-code SHA-256).

## Included / deferred

Included: project (3MF) and STL import, plate + model display, object list,
read-only preset/settings display from project settings, slicing in the
worker, G-code export download, Preview tab with slice summary.

Deferred, pending core APIs (reported to coordinator, not built in TS):

- Toolpath preview + layer slider + feature legend need upstream
  `GCodeProcessor::process_file` → `GCodeProcessorResult::moves` ported as a
  public `ares-core` API; the GUI-side `libvgcode::convert` then belongs in
  `ares-wasm`/`ares-vgcode` data, never in TypeScript.
- Editing settings and re-slicing needs a public
  `slice_project_plate` entry accepting option overrides
  (`DynamicPrintConfig::apply`).
- Arrange, orient, transform gizmos, multi-plate layout, Device/Project tabs.
