# Latched tool marker wait in `checkToolpathPreview` (SPEC)

Scope is the web E2E harness only. The
completion signal it relies on already exists; no `ares-core`, `ares-wasm` or
app code changes.

## Upstream behavior under test

`GCodeViewer::render` (`OrcaSlicer/src/slic3r/GUI/GCodeViewer.cpp:1594-1601`)
keeps drawing the tool marker once the moves thumb has left the end. A fresh
load does not latch it (`GCodeViewer.cpp:1388`). `web/e2e/toolpathPreview.ts:320-324`
checks this: after `End` on the top layer slider, the canvas must differ from
the unlatched full view `full`.

## Failure evidence (CI run 38614639446 attempt 1, main 48ff6b184)

Source: `web-e2e-desktop-artifacts`, trace `1-trace.trace`,
`error-context.md` and `ARES_WEB_FAILURE_DIAGNOSTICS`.

Observed:

1. `keyboardPress End` ran at t=486468 ms. The `Top layer` aria-valuenow check passed at
   486485. The `moves-slider[aria-busy=true]` count-0 check passed at 486497 and the
   `viewport[data-drawing]` count-0 check at 486505. The `canvas` `waitForSelector` returned at 486516.
2. The `canvasDigest` readback (`locator.evaluate`: `drawImage` +
   `getImageData` of the WebGL canvas) was **still pending when the 5000 ms
   `expect.poll` deadline hit at 491501**. The poll never received a single
   value, so this is not a case of the poll repeatedly seeing `full`.
   Playwright reports both cases as "Timeout 5000ms exceeded while waiting on the predicate".
3. The failure-diagnostics readback of the same canvas, with no new input, took
   **3095 ms** (491501 → 494597). Diagnostics showed `diffPixels: 0` and `contextLost: false`.
   The renderer was SwiftShader (Vulkan, Subzero).
4. Earlier `canvasDigest` readbacks in the same test took 18–357 ms. One
   took 1251 ms.
5. The DOM at failure is the expected latched state. The Moves slider is at its last stop (74),
   and the ToolPosition window shows the marker (X 113.318, Y 134.044, Z 10.000,
   Top surface).

Root cause: the poll at `:324` puts a GPU readback inside the
default 5 s `expect` budget (`playwright.config.ts` sets no `expect.timeout`).
With SwiftShader, `getImageData` on a WebGL canvas waits for the rasterizer to
finish the submitted frame, and on CI that took more than 5 s for the
full-50-layer view. The view itself was correct.

Not confirmed:
- Why SwiftShader was this slow at that moment. Runner load is the likely
  cause, but the job does not record it.
- Whether the final frame differs from `full`. No readback finished.

## Deterministic completion signal (already in the app)

After `End`, the chain is synchronous React state plus one posted render task:

1. `LayerSlider` (`aria-valuenow`, `LayerSlider.tsx:79`) and `MovesSlider` read
   the same `state.preview` (`PreviewPage.tsx:65,115,119`). The commit that shows
   the new top layer therefore also computes `busy = true`
   (`MovesSlider.tsx:137`, layers differ from the settled filter).
2. The worker's `movesHigher` result calls `onMoves` and `setApplied` in one
   callback (`MovesSlider.tsx:126-127`). They are batched into one commit, which
   clears `aria-busy` and hands `enabled` (with `marker`) to `ViewportView`.
3. `ViewportView`'s layout effect (`ViewportView.tsx:143-147`) calls
   `setToolMarker`, then `render()`. That sets `data-drawing`
   (`Viewport.ts:268-273`) within the same commit. The render task draws the
   frame and removes `data-drawing` (`Viewport.ts:66-69`).

So once `aria-valuenow` is the last layer, both checks inside `canvasDigest`
(`aria-busy` count 0, then `data-drawing` count 0; `toolpathPreview.ts:11-13`)
can only pass after the latched-marker frame has been drawn. The first digest
after them reads that frame. Re-polling adds nothing; its only effect is to
put the readback under a 5 s deadline.

## Change (test-only commit)

`web/e2e/toolpathPreview.ts:324-325`:

```ts
const marked = await canvasDigest(page);
expect(marked, "the latched tool marker draws at the last move").not.toBe(full);
```

- The assertion is unchanged: the same `not.toBe(full)` on the same digest.
- The wait is the existing `aria-busy`/`data-drawing` completion signal.
- The duplicate second readback at `:325` is reused instead of repeated.
- No timeout change, no retries, no thresholds.
- The readback runs under the test timeout (`playwright.config.ts:12`, 300 s),
  like the existing unpolled digests at `:298` and `:137`.

## Same hazard elsewhere (not changed here)

Every other `expect.poll(() => canvasDigest(page))` puts the readback under the
same 5 s budget:
- `toolpathPreview.ts:142,151,152,263,268,272,307,331,334`
- `colorRanges.ts:168,196`
- `filamentView.ts:117,121`
- `shortcuts.spec.ts:56,114,117`
- `colorChange.spec.ts:107`

Each one needs its own proof that a busy or drawing signal precedes it. For
example, camera zoom (`shortcuts.spec.ts:114`) passes through OrbitControls,
not `aria-busy`. This change fixes `:324` only; a follow-up
slice audits the rest call by call.

## Evidence plan

1. Base (main, unmodified) vs head on the desktop project, `presetSlice.spec.ts`,
   `--workers=1`. The browser process is pinned to a single CPU to slow
   SwiftShader; `--repeat-each` is not used. Expected: base fails at `:324`
   with the pending-readback timeout when the readback goes over 5 s; head passes.
   If base passes under this pinning, report that as "not reproduced" and keep
   the trace timing as the evidence. The CPU budget is not changed to force a failure.
2. Head on the normal two-CPU pinning: `presetSlice.spec.ts` and `slice.spec.ts` pass.
   The trace shows exactly one readback at `:324`.
3. `npm run lint` and `npm run typecheck`.

## Deferred

- The other polls listed above.
- Recording runner load in the CI web job.
