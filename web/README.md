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

### Cloudflare Pages

`npm run build:pages` produces `app/dist/`, including the WASM, module worker,
lazy profile assets, Pages `_headers`, existing `THIRD_PARTY_NOTICES.md` and
`LICENSES/` texts. It requires the pinned OrcaSlicer
profiles checkout used by CI. `npm run test:pages` runs Chromium against
Wrangler's local Pages server and writes screenshots/traces to the runner's
`target/pages-e2e/` workspace only; they are not uploaded as GitHub artifacts.
Install Chromium first with `npx playwright install chromium`.
The headers enable cross-origin isolation and SharedArrayBuffer; all app assets
are served from the same origin. The app uses `/#/prepare` and `/#/preview`, so
no custom SPA redirect is required.

The Cloudflare Pages workflow uses the existing `Default` self-hosted runner
group (the current fleet has no `self-hosted` label) and
builds and checks every push and pull request,
independently of the native oracle jobs. Bundles, screenshots and traces stay in
the runner workspace; diagnostic text stays in job logs. The workflow does not
upload GitHub artifacts or consume artifact storage quota. Workspace files are
not guaranteed to remain available after the job ends. PRs and branch pushes
never deploy.
Production deploys only on `main` pushes when repository variable
`CLOUDFLARE_PAGES_ENABLED` is `true`; that variable is now enabled.

Public hosting has been approved in the existing **Indexyz Cloudflare** account.
The direct Pages project is **`ares`**, at <https://ares-cdh.pages.dev>, with
`main` as its production branch. It is a Pages project, not a Workers deployment.
GitHub is already configured with:

- repository variable `CLOUDFLARE_PAGES_PROJECT=ares`;
- repository variable `CLOUDFLARE_PAGES_ENABLED=true`;
- environment `cloudflare-pages-production`, restricted to deployments from `main`;
- environment secret `CLOUDFLARE_ACCOUNT_ID`.

The only remaining user setup is the environment secret `CLOUDFLARE_API_TOKEN`.
Keep its value out of files, command arguments, logs and chat. The API token
needs only **Account / Cloudflare Pages / Edit**, restricted to the selected
account, with an explicitly approved expiry. This permission covers Pages
projects in that account; it is not restricted to one project. Transfer a newly
created token directly into GitHub secrets through an approved secure handoff;
do not reuse or read existing secret values.

Both web library packages declare `AGPL-3.0-or-later`; npm's `prepack` license
copy does not run during a Vite build. The Pages package preserves the existing
notices and license texts, but does not publish private source or establish
license compliance. Public-hosting approval does not replace review of any
required corresponding-source availability and disclosure obligations.

By user decision, both build and deployment jobs run on the self-hosted
`Default` group. Deployment still requires the build job's checks to pass via
`needs: build`, but does not depend on uploaded artifacts. It checks out the same
commit, repeats the build job's pinned toolchain and profile setup, and runs
`npm ci` and `npm run build:pages` before deploying `web/app/dist` directly.
It uses `cloudflare/wrangler-action@v4` with the locked Wrangler version and
references deployment credentials only in the final action, not in the build
commands. PR builds perform no preview deployment or Cloudflare
account changes. See Cloudflare's [Direct Upload CI guide](https://developers.cloudflare.com/pages/how-to/use-direct-upload-with-continuous-integration/).

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

## Breaking processed-palette API change

`sliceProjectPlateProcessed(input, plate, overridesJson?)` no longer accepts
`colorPrintColors`. Profile slicing options no longer accept
`color_print_colors` (unknown fields are rejected). Remove these arguments/keys;
there is no compatibility fallback. Rust derives both palettes from processed
filament colors and custom G-code items, including the pause/custom gray entry.

## Known differences from OrcaSlicer

- The Preview Filament (`ColorPrint`) view uses the processed tool colors and
  metric usage columns and result-derived palettes. Source-selected ColorChange
  ranges and partial records are available; absent native volume intervals make
  partial usage unavailable rather than inferred. Imperial units remain deferred.
  Its Display
  header has no visibility control: the upstream row callback is empty
  (`GCodeViewer.cpp:4097`); this slice does not invent one.

- Process edits in the sidebar are kept when another printer or process preset is
  selected, and apply on top of the new presets. OrcaSlicer asks first, with
  `UnsavedChangesDialog` from `Tab::may_discard_current_dirty_preset`
  (`Tab.cpp:6278-6282`), whether to transfer or discard them; that dialog is
  not ported.
- The Prepare plate switches to the selected printer's bed, with an STL centered
  on it, once a process and every filament are selected too: `placementScene`
  composes the full selection. OrcaSlicer updates the bed as soon as the
  printer changes (`Plater::priv::on_select_preset` -> `on_config_change` ->
  `set_bed_shape`, `Plater.cpp:9576-9580, 16754-16755`).
- The Preview moves slider labels its higher thumb with the G-code line of
  the stop. Upstream `GCodeViewer.cpp:1765-1821` also supplies G-code IDs as
  `alternate_values`; this is not an elapsed-time/distance feature. The
  separate tool-position Time property uses the Rust-owned inclusive f32 prefix;
  see [the marker-time record](../docs/architecture/web-marker-time.md).
- The Preview tool position window (`Marker::render_position_window`,
  `GCodeViewer.cpp:325-690`) lists Type, Line Type, Width, Height, Layer, Speed,
  Flow rate, Fan speed, Temperature and Time (Normal mode). The Time row follows
  the selected property-window vertex, including seam substitution, rather
  than independently selecting an arrow vertex. Acceleration, Jerk, Pressure
  Advance, the actual speed profile and Stealth UI remain deferred. The marker arrow is a
  Three.js cone on a cylinder with the `stilized_arrow(16, 1.5, 3.0, 0.8, 3.0)`
  dimensions (`GLModel.cpp:934`), lit by the viewport lights instead of the
  `gouraud_light` shader.

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
  The preset and profile slice specs run the `ares-cli` parity tests with
  `ARES_REQUIRE_ORCA=1` and the Native binary and resources of that extraction, so
  a missing Native fails them instead of skipping the comparison.
- `ksr-project/`: `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` sliced twice
  through the `AresSlicer` worker API (`browser.gcode`, `result.json`). The
  `ares-cli` parity test compares it as ordered bytes with the KSR initialized
  OrcaSlicer 2.4.2 named by `ARES_KSR_INITIALIZED_BIN`,
  `ARES_KSR_INITIALIZED_NATIVE_BIN` and `ARES_KSR_INITIALIZED_RESOURCES_DIR`.

The oracles are external binaries named by environment variables: `ORCA_APPDIR`
above, and `ARES_PROCGCODE_BIN` for the OrcaSlicer `GCodeProcessor` build
(`tools/chain-probe/procgcode.nix`) that checks the toolpath preview.
`ARES_MARKER_TIME_STABLE_BIN` selects the approved marker-only variant of the
real-conversion/native source-kernel probe
(`tools/chain-probe/marker-time-stable-options.nix`) for exact Time text. It is not full native
viewer execution. A variable
set to a missing path fails the test. An unset variable fails it too, except
under `CI`, which does not provision the oracles and reports the check as
skipped.

If Playwright's bundled Chromium cannot run on the host (for example on NixOS),
set `ARES_E2E_CHROME` to a system Chrome/Chromium binary.

### Phones and tablets

The `desktop` project (Chromium, 1600x900) runs every spec except
`e2e/mobile.spec.ts`, which runs in three touch-emulated projects: the phones
`iphone` (Playwright WebKit, iPhone 15) and `pixel` (Chromium, Pixel 7), and
the tablet `ipad` (Playwright WebKit, iPad gen 7, 810x1080). The
spec imports `tests/parity/cube10.stl`, installs the LH system presets, slices,
opens the preview, orbits and pinch-zooms the view, drags the layer slider and
downloads the G-code by touch, checking that every visible control is at
least 44x44, that a tapped control keeps no hover style, that fields use at
least 16px text and that nothing animates under `prefers-reduced-motion`; the
downloaded G-code must have the sha256 of the same flow with desktop layout and
mouse input. That desktop flow also tabs through each step and requires a
visible focus on every stop. Both flows run axe (WCAG 2.1 A/AA) at the prepare
and preview steps in the dark and the light theme and fail on any serious or
critical violation. Artifacts go to
`target/web-e2e/mobile/<project>/` (step screenshots, axe reports,
`gcode.sha256`, `result.json`).
`e2e/worker.mobile.spec.ts` checks, on the same devices, that the 3D view draws
at `min(devicePixelRatio, 2)` and that slicing a generated large model in the
Web Worker never stalls the main thread for 200 ms; its report goes to
`target/web-e2e/mobile-worker/<project>/result.json`.

```sh
npx playwright install webkit     # once
npx playwright test --project=pixel --project=iphone --project=ipad
```

Playwright's WebKit needs system libraries that NixOS does not provide on a
library path. `e2e/webkit-env.sh` builds them from `e2e/webkit-libs.nix` with
the nixpkgs named by `ARES_E2E_NIXPKGS` and points the WebKit projects at
`e2e/webkit-run.sh`, which starts the bundled headless MiniBrowser with them:

```sh
# same pinned nixpkgs as scripts/orca-replay/build.nix
ARES_E2E_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
  e2e/webkit-env.sh npx playwright test --project=iphone --project=ipad
```

The library list matches the WebKit revision of the pinned `@playwright/test`
(1.63.0, `webkit-2359`); a Playwright upgrade rechecks it with `ldd` on
`minibrowser-wpe/bin/*` and `lib/*.so*`.

### Lighthouse

`npm run lighthouse` audits the production build (run `npm run build` first)
served by `vite preview` on `127.0.0.1:4174`: Lighthouse's mobile (default)
and desktop presets, three runs each, in a headless Chrome (`ARES_E2E_CHROME`
selects the binary). Every score and metric is the median of the three runs.
It writes each run's JSON and HTML report and `summary.json`/`summary.md`
(the four category scores, LCP, TBT, CLS, FCP and total transfer size) to
`target/web-lighthouse/` or the directory given as its argument, and exits
non-zero when a median score misses its target: Accessibility 95, Best
Practices 95, SEO 90. Performance is recorded, not gated.
