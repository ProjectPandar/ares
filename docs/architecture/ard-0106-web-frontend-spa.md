# ARD-0106: Browser SPA frontend over the ares-wasm adapter

## Source and boundary

The web frontend in `web/` is a UI rewrite of OrcaSlicer's GUI layer
(pinned `8500fcdccaa10b5099ac20d252af3a7c560046f1`), not of `libslic3r`:

| Upstream (`src/slic3r/GUI/`) | Web destination |
| --- | --- |
| `MainFrame.cpp` top bar, Prepare / Preview tabs | `web/app/src/shell/*`, TanStack routes `/prepare`, `/preview` |
| `Plater.cpp` `Sidebar` (printer / filament / process presets), `GUI_ObjectList.cpp` | `web/app/src/sidebar/*` |
| `Tab.cpp` `TabPrint::build` page/group/option layout | `web/app/src/sidebar/processPages.ts` (layout only; values come from Rust) |
| `GLCanvas3D.cpp`, `3DBed.cpp` plate + model volumes | `web/app/src/viewport/*` (Three.js) |
| `GCodeViewer.cpp`, `IMSlider.cpp`, `LibVGCode/LibVGCodeWrapper.cpp` legend, layer slider | `web/app/src/preview/*` |
| `BackgroundSlicingProcess.cpp` (slicing off the UI thread) | `web/packages/ares-wasm-js` worker + client |

All slicing, project loading, option values and G-code logic stays in Rust.
TypeScript only renders data returned by `crates/ares-wasm`.

## Non-negotiable decisions

- **Stack:** Vite + React + TypeScript, TanStack Router (hash history so the
  static build works from any path), plain Three.js (no React renderer layer)
  owned by an imperative viewport class mounted from React. npm workspaces
  with one committed `web/package-lock.json`; `web/` is self-contained.
- **Toolchain:** Vite (Rolldown + oxc transform/minify) for both packages,
  oxlint for lint, `tsc` only for type checking and `.d.ts` emission. No
  ESLint, no Babel.
- **Library package:** `web/packages/ares-wasm-js` (`@ares-slicer/wasm`) owns
  every WASM call: the wasm-bindgen output, the Web Worker, and a typed
  promise client (`AresSlicer`). It is publishable on its own (own
  `package.json`, `exports` map, emitted types, Vite library build) and has
  no React/Three.js dependency. The SPA in `web/app` consumes it as a
  workspace dependency and never imports wasm-bindgen glue directly.
- **WASM boundary:** `ares-wasm` gains only thin byte-oriented bindings that
  call existing public `ares-core` APIs (`load_project`, `load_model`,
  `slice_project_plate`, `slice`). Project scene data crosses as one JSON
  metadata string plus flat `Float32Array`/`Uint32Array` buffers. No
  filesystem, terminal, or new pipeline code in the adapter. Volume/instance
  matrices are recovered through the public `Transform3d::transform_point`.
- **Threading:** all WASM calls run in a dedicated module Web Worker shipped
  by the library (`src/worker.ts`); the UI thread never instantiates the
  slicer. Results transfer `ArrayBuffer`s (zero-copy).
- **Build output:** `npm run build:wasm` runs
  `cargo build -p ares-wasm --target wasm32-unknown-unknown --release` and
  `wasm-bindgen --target web` into the git-ignored
  `web/packages/ares-wasm-js/wasm/`; the library builds to its `dist/`, and
  the app builds a static SPA in `web/app/dist/`.
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
