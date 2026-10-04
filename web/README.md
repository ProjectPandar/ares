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

## Develop

```sh
npm run dev          # Vite dev server for the app
npm run lint         # oxlint
npm run typecheck    # tsc --noEmit for library, app and e2e
```

Regenerate `app/src/sidebar/processLayout.ts` (option pages, groups and labels from
OrcaSlicer `Tab.cpp` / `PrintConfig.cpp`) with:

```sh
node scripts/extract-process-layout.mjs /path/to/OrcaSlicer
```

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

If Playwright's bundled Chromium cannot run on the host (for example on NixOS),
set `ARES_E2E_CHROME` to a system Chrome/Chromium binary.
