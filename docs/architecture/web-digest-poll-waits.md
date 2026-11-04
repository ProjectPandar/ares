# Canvas digest polls on the draw completion signal (SPEC)

This is the follow-up to
`web-latched-marker-wait.md`, which converted the poll at `toolpathPreview.ts:324`.
The scope is the web E2E harness only. No `ares-core`, `ares-wasm` or app code
changes, and no timeout, retry or threshold changes.

## Problem

17 checks still wrap `canvasDigest` in `expect.poll`. `canvasDigest`
(`web/e2e/toolpathPreview.ts:11-22`) first waits for
`moves-slider[aria-busy=true]` count 0 and `viewport[data-drawing]` count 0, then
reads back the WebGL canvas. Under `expect.poll`, that readback runs inside the
default 5 s `expect` budget (`playwright.config.ts` sets no `expect.timeout`).
CI run 38614639446 showed that one SwiftShader readback can take longer than
5 s. The poll then fails with "Timeout 5000ms exceeded while waiting on the
predicate" even though the view is correct. A single-CPU run
reproduced this at `:324`. All 17 polls
have the same exposure.

The poll only helps if the frame could still change after `canvasDigest`'s
waits return. Each conversion below therefore names the commit that the
preceding DOM assertion observes, and shows that the frame under test is either
pending behind `aria-busy` or already queued behind `data-drawing` in that
commit.

## Signal chain (app code, unchanged)

- **One store.** `Legend` `aria-pressed` (`Legend.tsx:92,127`), `LayerSlider`
  `aria-valuenow` (`LayerSlider.tsx:79`), the view-type select
  (`Legend.tsx:104`) and `MovesSlider` all read the same `state.preview`
  (`PreviewPage.tsx:65,111,115,119`). A commit
  that shows a changed layer range, legend toggle or view type also computes
  `busy = true` (`MovesSlider.tsx:137-138`), because the settled filter or view
  type differs.
- **Moves thumb.** `onChange` sets a new `slider` object
  (`MovesSlider.tsx:174-178`). `settled` is null until `movesHigher` applies,
  so `busy` stays true until then. `expectMarkedLine` reads
  `gcode-panel .current`, which derives from the applied view
  (`PreviewPage.tsx:79`). It passes only after the applied commit.
- **Draw.** The applied commit gives the viewport its colors and enabled ranges
  in layout effects (`ViewportView.tsx:137-147`). Each setter calls `render()`,
  which sets `data-drawing` synchronously (`Viewport.ts:268-273`). The posted
  render task draws the frame and clears it (`Viewport.ts:65-69`). Viewport
  colors come from the applied moves view (`moves?.colors`,
  `PreviewPage.tsx:83`), not from the async `useViewColors`.
- **Camera keys.** `I` and `O` are handled in the `keydown` listener
  (`ViewportView.tsx:85-93`), not by OrbitControls. `zoomBy` calls `render()`
  in that same handler (`Viewport.ts:240-246`), so `data-drawing` is set before
  the key event returns. This relies on `keyboard.press` resolving after the
  page has handled `keydown`. That is the CDP `Input.dispatchKeyEvent` ack, an
  assumption the evidence must confirm.

## Conversions

Each poll becomes a plain read followed by the same comparison with the same
message:
`expect(await canvasDigest(page), msg).not.toBe(prev)`, or `.toBe(...)` for
the polls that have no message. Where the next line reads the same digest again,
it reuses this value instead, as `:325` did in 622464bc1.

Line numbers are at 622464bc1.

| # | Site | Preceding commit observed | Signal |
|---|------|---------------------------|--------|
| 1 | `toolpathPreview.ts:142` middle stop | `expectMarkedLine(stops[middle])` | applied → drawing |
| 2 | `:151` latched end `toBe(full)` | `expectMarkedLine(last)` at `:147`, marker count 1 | applied → drawing |
| 3 | `:152` unlatched end | same as 2 | applied → drawing |
| 4 | `:263` hide seams (reuse at `:264`) | `aria-pressed=false` | busy (options) |
| 5 | `:268` show retractions | `aria-pressed=true` | busy (options) |
| 6 | `:272` restore `toBe(shown)` | two clicks; **no assertion follows them** | see below |
| 7 | `:307` first layer | `aria-valuenow=1`, current line | busy (layers) |
| 8 | `:332` hide outer wall | `aria-pressed=false` | busy (roles) |
| 9 | `:335` restore `toBe(marked)` | one click; **no assertion follows it** | see below |
| 10 | `colorRanges.ts:168` range view | `expectRows` of the new view | busy (view type) |
| 11 | `colorRanges.ts:196` Line Type `toBe` | range rows count 0 | busy (view type) |
| 12 | `filamentView.ts:117` Filament | filament legend rows | busy (view type) |
| 13 | `filamentView.ts:121` Line Type `toBe` | filament legend count 0 | busy (view type) |
| 14 | `shortcuts.spec.ts:56` one-layer follow | `aria-valuenow=44` | busy (layers) |
| 15 | `shortcuts.spec.ts:114` `I` (reuse at `:115`) | none; `keydown` handler | drawing (camera keys) |
| 16 | `shortcuts.spec.ts:117` `O` | none; `keydown` handler | drawing (camera keys) |
| 17 | `colorChange.spec.ts:107` ColorPrint `toBe` | `changes` visible | busy (view type) |

**Sites 6 and 9.** After the last click there is no DOM assertion, so the
single read could run before the click's commit. That is only safe if
`locator.click()` resolves after the `click` handler, and React flushes discrete
events synchronously, so `busy` is already true on return. To avoid depending on
that, the conversion adds the missing state assertion before the read. The
existing locator gets `toHaveAttribute("aria-pressed", ...)`:
`seams`/`"true"` at `:272` and `outerWall`/`"true"` at `:335`. This matches how
sites 4, 5 and 8 already work. It adds an assertion and changes no comparison,
input or expected value.

## Ways the conversion could fail (checked before the code)

1. **Stale read.** The preceding assertion observes a commit without the change.
   A stale frame equals the previous digest, so every `not.toBe` site fails
   loudly. The `toBe` restores (2, 6, 9, 11, 13, 17) all follow a state with a
   different frame, so a stale read fails too. No site can pass by accident on
   a stale frame.
2. **Late second frame.** Something redraws after the waits return. `render()`
   coalesces through `#dirty`, and any later `render()` sets `data-drawing`
   again, but the read does not re-wait. Known late sources:
   - `ResizeObserver` → `#fitCanvas` (`Viewport.ts:94,287-293`), if a legend or
     overlay change resizes the viewport container.
   - OrbitControls `change` (`Viewport.ts:72`). The tests produce no pointer
     input, and `zoomBy` calls `controls.update()` in the handler.

   The poll would absorb a late frame; a single read would not. The evidence
   must show these sites pass on two CPUs and on a single CPU. If one
   fails, that site stays a poll and gets reported. No fallback is added.
3. **Async colors.** `useViewColors` feeds the viewport only while `moves` is
   null, which does not happen at these sites after the first apply. Only the
   legend ranges use it, and `expectRows` asserts those separately.
4. **Camera-key assumption is false.** If `keyboard.press` returns before the
   handler runs, sites 15 and 16 can read the pre-zoom frame. That fails loudly
   (stale `not.toBe`), so site 15/16 would stay a poll and be reported. No
   sleep or retry is added.
5. **The readback is slow in its own right.** It now runs under the 300 s test
   timeout (`playwright.config.ts:12`) instead of the 5 s `expect` budget. That
   is the intended change, not a raised limit: nothing is retried, and the
   comparison runs once.

## Out of scope

- Non-digest `expect.poll` calls, such as `filamentView.ts:107` legend headers.
  These are DOM reads with no GPU readback.
- `mobile`/`worker.mobile` specs. They have no digest polls.
- App code, timeouts and pixel thresholds.

## Evidence plan

1. **RED-free test commit.** This is a flake fix with no new behavior, so there
   is no RED. Each site's comparison and message stay the same; the only
   assertion additions are the two `aria-pressed` checks for sites 6 and 9.
2. **Head, desktop, `--workers=1`, tier1 web env, two CPUs, `--trace=on`.** Run
   `presetSlice`, `slice`, `shortcuts` and `colorChange`. Each converted site
   must show exactly one canvas `evaluate` in the trace.
3. **Same specs with the whole browser tree on a single CPU.** Head must pass.
   Base gets one run per spec, reported as observed with no repeats; a base
   pass is reported as "not reproduced".
4. **Checks.** `npm run lint`, `npm run typecheck` and the web build.
