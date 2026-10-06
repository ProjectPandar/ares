# Ares web

Browser frontend for Ares (ARD-0106). It is an npm workspace with two packages:

| Path | Package | Role |
| --- | --- | --- |
| `packages/ares-wasm-js` | `@ares-slicer/wasm` | Publishable library: the wasm-bindgen output of `crates/ares-wasm`, the typed `AresSlicer` client and the slicing Web Worker. No React/Three. |
| `app` | `ares-app` | Vite + React + TanStack Router + Three.js SPA (Prepare / Preview), modelled on the OrcaSlicer plater. |

All slicing, 3MF/STL parsing and G-code generation run in Rust (`ares-core`) through
the WASM worker. The TypeScript code only handles UI and rendering.

## Requirements

- Node.js 22+ and npm
- Rust with the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- `wasm-bindgen` CLI matching the workspace `wasm-bindgen` version (`cargo install wasm-bindgen-cli --version <ver>`)

## Build

```sh
cd web
npm ci
npm run build:wasm   # cargo wasm32 release + wasm-bindgen → packages/ares-wasm-js/src/wasm/
npm run build        # library (packages/ares-wasm-js/dist) then app (app/dist)
```

`app/dist/` is a static site; serve it with any static file server
(`npm run preview -w ares-app` serves it on http://127.0.0.1:4174).

### Bundle size

Measured at `314b224c`:

| Asset | Raw | gzip |
| --- | --- | --- |
| Initial JS: React, TanStack Router and app | 415 kB | 117 kB |
| Viewport chunk: Three.js, lazily loaded | 590 kB | 147 kB |
| `ares_wasm_bg.wasm`, loaded in the worker | 9.25 MB | 3.07 MB |

`e2e/bundle.spec.ts` enforces the initial-JS budget.

The WASM size comes from the Rust build settings, which belong to the Rust
side and not to `web/`:
- the workspace `[profile.release]`: `lto = true`, `codegen-units = 1`,
  `opt-level = 3`, `strip = "symbols"`;
- `scripts/build-wasm.mjs`: `cargo --release`, then `wasm-bindgen`, with no
  `wasm-opt` pass.

## Develop

```sh
npm run dev          # Vite dev server for the app
npm run lint         # oxlint
npm run typecheck    # tsc --noEmit for library, app and e2e
```

Regenerate `app/src/sidebar/processLayout.ts` and `presetLayout.ts` (process, filament
and printer tab pages, groups, lines and labels from OrcaSlicer `Tab.cpp` /
`PrintConfig.cpp`) with:

```sh
node scripts/extract-process-layout.mjs /path/to/OrcaSlicer
```

## Known differences from OrcaSlicer

- Process edits in the sidebar are kept when another printer or process preset is
  selected, and apply on top of the new presets. OrcaSlicer asks first, with
  `UnsavedChangesDialog` from `Tab::may_discard_current_dirty_preset`
  (`Tab.cpp:6278-6282`), whether to transfer or discard them; that dialog is
  not ported.

## End-to-end test

```sh
npx playwright install chromium   # once
npm run test:e2e
```

The Playwright test loads the built SPA in Chromium, imports
`tests/parity/orca_cli_ender3.3mf`, slices it in the WASM worker, downloads the
G-code, and checks that it is byte-identical to native `ares slice --plate 1`.
Artifacts are written to `target/web-e2e/`:

- `prepare.png`, `preview.png`: screenshots
- `orca_cli_ender3_plate_1.gcode`, `native.gcode`: browser and native G-code
- `gcode.sha256`, `result.json`: size and digest of the browser G-code
- `settings/`: one screenshot per process / printer / filament settings page, and
  `result.json` with the per-page field coverage and the unedited slice digest
- `profile-slice/`: `sliceWithProfiles` on `@ares-slicer/profiles` system presets
  (`browser.gcode`), the native OrcaSlicer `--load-settings` oracle run (`native/`),
  and `result.json` with catalog load and slice timings, with and without the BBL vendor.
  It needs `ORCA_APPDIR` set to an official OrcaSlicer 2.4.2 AppImage extraction
  (see `scripts/orca-parity.sh`).

The oracles are external binaries named by environment variables: `ORCA_APPDIR`
above, and `ARES_PROCGCODE_BIN` for the OrcaSlicer `GCodeProcessor` build
(`tools/chain-probe/procgcode.nix`) that checks the toolpath preview. A variable
set to a missing path fails the test. An unset variable fails it too, except
under `CI`, which does not provision the oracles and reports the check as
skipped.

If Playwright's bundled Chromium cannot run on the host (for example on NixOS),
set `ARES_E2E_CHROME` to a system Chrome/Chromium binary.
