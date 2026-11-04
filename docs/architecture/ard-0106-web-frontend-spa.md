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
  the app builds a static SPA in `web/app/dist/`. There is no `wasm-opt`
  pass: binaryen does not shrink the transferred module (see below), so WASM
  size work belongs on the Rust side, starting from an analysis of where the
  code size comes from.
- **Look:** dense desktop layout with CSS custom-property theme tokens
  (dark/light), Orca teal accent, no UI framework.
- **E2E:** Playwright under `web/e2e/` drives the built SPA: import a
  sample 3MF, slice through the worker, assert G-code markers, screenshot
  Prepare and Preview, and write `target/web-e2e/` artifacts
  (screenshots + G-code SHA-256).

## wasm-opt evaluation (2026-10-05)

binaryen 132 `wasm-opt` on the same `ares_wasm_bg.wasm` (rustc 1.96.0,
wasm-bindgen 0.2.121, workspace `[profile.release]` with `lto`,
`codegen-units = 1`, `opt-level = 3`). Sizes are bytes with `gzip -9` and
brotli 11. Times come from `web/e2e/slice.spec.ts` in Chrome on
`tests/parity/orca_cli_ender3.3mf`, the mean of five separate runs: engine
ready, the first slice as the status bar reports it, and the median of 20
repeat slices.

| wasm-opt | Raw | gzip | brotli | Engine ready | First slice | Repeat slice | wasm-opt time |
| --- | --- | --- | --- | --- | --- | --- | --- |
| none | 9,267,490 | 3,074,493 | 2,029,353 | 1.49 s | 0.39 s | 123.1 ms | — |
| `-O1` | 9,002,761 | 3,043,860 | 2,017,965 | 1.51 s | 0.40 s | 123.4 ms | 53 s |
| `-O3` | 8,807,611 | 3,090,939 | 2,116,517 | 1.46 s | 0.77 s | 129.2 ms | 399 s |
| `-Os` | 8,798,033 | 3,090,352 | — | — | — | — | 368 s |
| `-Oz` | 8,635,680 | 3,089,317 | 2,116,198 | 1.48 s | 0.76 s | 124.3 ms | 417 s |

The G-code sha256 is identical at every level. `-O3`, `-Os` and `-Oz` grow
the gzip and brotli transfer and `-O3`/`-Oz` double the first slice; `-O1`
saves only about 1% gzip at the cost of a binaryen dependency, 53 s per
build and a hand-kept feature list (Rust emits no `target_features`
section). Decision: no `wasm-opt`; WASM size reduction goes back to the
Rust side. `slice.spec.ts` keeps writing `engineReadyMs`, `firstSliceS` and
`sliceMedianMs` to `target/web-e2e/result.json` as the performance baseline.

## Included / deferred

Included: project (3MF) and STL import, plate + model display, object list,
read-only preset/settings display from project settings, slicing in the
worker, G-code export download, Preview tab with slice summary.

Deferred, pending core APIs (not built in TS):

- Toolpath preview + layer slider + feature legend need upstream
  `GCodeProcessor::process_file` → `GCodeProcessorResult::moves` ported as a
  public `ares-core` API; the GUI-side `libvgcode::convert` then belongs in
  `ares-wasm`/`ares-vgcode` data, never in TypeScript.
- Editing settings and re-slicing needs a public
  `slice_project_plate` entry accepting option overrides
  (`DynamicPrintConfig::apply`).
- Arrange, orient, transform gizmos, multi-plate layout, Device/Project tabs.
