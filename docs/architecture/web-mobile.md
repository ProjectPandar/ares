# Web app on phones and tablets

Status: in review. Branch `pi/web-mobile`. Task: `/tmp/ares-web-mobile-task.md`.

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
  for fine pointers). The settings mode switch widens to three 44px dots.
  Fine pointers keep the desktop sizes.
- **Hover only where a pointer hovers.** Every `:hover` rule sits in
  `@media (hover: hover)`, so a tapped control keeps no stuck hover style.
- **Tablet project.** `ipad` (WebKit, iPad gen 7, 810×1080) covers the
  641–1024px layout with the same flow and checks as the phones.
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
- Tablet preview overlays (slice 3b). At 810px the G-code window covers the
  3D centre and the summary, the legend sits under the moves slider, and the
  44px camera toolbar touches the summary card and the layer label.
- axe serious findings (color-contrast) and the ≥16px input font (slice 4);
  axe is report-only until then.
- `min(devicePixelRatio, 2)` and worker confirmation (slice 5); Lighthouse
  (slice 6).
