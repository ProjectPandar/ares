# Web app on phones and tablets

This is a web-shell slice. It ports no `libslic3r`/`libvgcode` behavior and
changes no slicing, G-code, config or viewer-data output. The phone G-code is
byte-identical to the desktop G-code for the same input (E2E hash check). The
rewrite gate in `AGENTS.md` applies to domain slices and is not used here.

## Decisions

- **Projects.** `desktop` (Chromium 1600×900, every spec except
  `mobile.spec.ts`), `iphone` (`devices["iPhone 15"]`, real WebKit) and `pixel`
  (`devices["Pixel 7"]`, Chromium). iPhone never falls back to Chromium; on
  NixOS WebKit runs through `web/e2e/webkit-env.sh` (README "Phones").
- **Breakpoints.** `≤1024px` narrows the sidebar to 280px. `≤640px` is the
  phone layout: the sidebar becomes a drawer behind a `Settings panel` toggle
  (`aria-expanded`, `aria-controls`), the notebook tabs keep their row, the
  plate actions (`Slice plate`, `Export G-code file`) move to a bottom bar, and
  `env(safe-area-inset-*)` pads the shell (`viewport-fit=cover`). Above 640px
  the layout is unchanged.
- **3D view centre stays free.** Gestures start at the canvas centre, so on
  phones the summary card and the 5-line G-code window sit above it and the
  legend sits below it, above the moves slider. The camera toolbar scrolls
  sideways in one row instead of wrapping over the view.
- **Touch targets.** `@media (pointer: coarse)` raises every control to
  44×44 CSS px (WCAG 2.5.5); the touch E2E sweeps all visible controls at each
  flow step. A checkbox's target is its wrapping label (`display: contents`
  for fine pointers). The settings mode switch widens to three 44px stops.
  Fine pointers keep the desktop sizes.
- **Mode switch is one widget.** As upstream `ModeSwitchButton`
  (`SwitchButton.cpp:293`, `:423-428`), it is a single `role="slider"` whose
  click picks the mode under its x third; arrows and Home/End step it. The
  48×18 track is unchanged; 3px of transparent padding above and below make
  it a 48×24 target (WCAG 2.5.8), 44px high on touch. The settings E2E runs
  axe `target-size` on the sidebar.
- **Hover only where a pointer hovers.** Every `:hover` rule sits in
  `@media (hover: hover)`, so a tapped control keeps no stuck hover style.
- **Tablet project.** `ipad` (WebKit, iPad gen 7, 810×1080) covers the
  641–1024px layout with the same flow and checks as the phones.
- **Tablet preview overlays stack.** At 641–1024px the 3D view is about 530px
  wide, so the overlays take the phone positions: camera toolbar left-aligned
  and scrolling, summary and layer slider below it, legend above the Moves
  slider, and the G-code window as the five-line band over the summary card
  (upstream stacks its windows, `GCodeViewer.cpp:957-972`; a 52-column side
  column would cut every line at this width). The touch E2E requires the
  overlays inside the view, pairwise apart and the centre free on every device.
- **WCAG AA contrast without moving the brand colour.** `#009688` stays the
  accent for borders, sliders, swatches and the logo mark (non-text, 3:1).
  Text on the accent uses `--accent-fill` (`#00796b`, 5.28:1 with white;
  hover `#00695c`, pressed `#004d40`). Accent-coloured text uses `--accent-ink`
  (dark `#2bb3a6`, light `#00796b`). `--text-faint` becomes `#9e9ea0` (dark)
  and `#6e6e6e` (light). axe asserts zero serious or critical findings in
  both themes at the desktop and touch viewports.
- **16px fields on touch** (no iOS focus zoom) and **reduced motion**: under
  `prefers-reduced-motion: reduce` every animation and transition is off; the
  spinner stands still beside its status text.
- **Keyboard.** The E2E Tab sweep starts on a control: a plain Tab with the
  body focused switches Prepare/Preview (GLCanvas3D::on_key), as in Orca.
- **Pixel ratio capped at 2.** The WebGL renderer uses
  `min(devicePixelRatio, 2)`: iPhone (3) and Pixel (2.625) draw at 2x, which
  cuts fill cost by up to 56% with no visible loss at phone sizes.
- **Slicing stays off the main thread.** `e2e/worker.mobile.spec.ts` slices a
  generated 512-facet cylinder (about 10 s) on each device and requires a
  16 ms main-thread heartbeat to never stall 200 ms or more, and a theme tap to
  apply while the plate is still slicing.
- **Touch input is OrbitControls' own mapping** (one finger rotate, two
  fingers dolly/pan). No custom gesture math.
- **Synthetic WebKit touches.** Playwright has no WebKit touch-move, so the
  harness dispatches `touch` PointerEvents. Their ids are not active browser
  pointers, and WebKit's `setPointerCapture` throws `NotFoundError` for them
  (probe), so capture is a no-op for those ids while a gesture plays.
  `touch-action` is only exercised on Chromium (CDP touches).

## Deferred to later mobile slices

- Option tooltips (`title`, from the option definitions) have no touch
  alternative yet; they are the remaining hover-only information.
- Lighthouse targets. `web/e2e/lighthouse.mjs` gates the median
  Accessibility >= 95, Best Practices >= 95 and SEO >= 90 of three runs per
  preset. All three are met: median Accessibility, Best Practices and SEO are
  100 on both presets (desktop Accessibility was 96 until the mode switch
  became one 48×24 target). Performance (median 65 mobile, 71 desktop) is
  recorded, not optimised for the score: observed mobile LCP equals FCP
  (461 ms, the canvas-hint text in the first frame), and Lighthouse's
  simulated LCP is set by the three.js Viewport chunk (149 KB gzip) that
  `ViewportView.tsx` starts downloading at about 80 ms so it overlaps the
  shell. Deferring that download would only make the 3D view ready later on
  real devices, so it is not done. Pre-linking the bed shaders
  (`compileAsync`) gave no measurable gain: the first synchronous WebGL call
  pays the GPU-process start-up wait wherever it happens.
- Total Blocking Time (simulated 1338-1802 ms mobile) is not JavaScript CPU
  work. Unthrottled, the page runs 0.8-1 s of main-thread tasks, and one of
  them (572-671 ms) is the `WebGLRenderer` context creation in the Viewport
  mount, which waits for the GPU process to start. A DevTools profile under
  4x CPU throttle shows the same wait at about 590 ms, so it does not scale
  with CPU speed. Lighthouse's simulated throttling multiplies it by 4 anyway
  and charges it to the `index` chunk, whose React scheduler task mounts the
  Viewport, and to the Viewport chunk. The `index` chunk itself is ordinary:
  373 KB, of which react-dom is 207 KB, the router 69 KB and app code about
  90 KB, with no catalog built at start when no vendors are installed.
  Trimming or splitting it would not change TBT.
- Rendering the viewport in a worker (`transferControlToOffscreen` with
  three.js in a Worker) is the only change that would take that wait off the
  main thread. It is deferred: Viewport input, picking, resize and
  screenshots would all move behind messages, and iOS needs OffscreenCanvas
  WebGL (Safari 17+).
