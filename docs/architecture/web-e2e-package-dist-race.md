# Web e2e: package builds racing package readers

## Scope

This is a test-harness repair. No `libslic3r` or `libvgcode` behaviour changes, and no
package, app or core code changes. It ports nothing from upstream. It fixes the order of
existing web e2e specs so that the specs which rebuild a package's `dist` never run while
other specs read that `dist`.

## Cause

Two specs pack a workspace package with `npm pack`, which runs its `prepack` script.

- `web/e2e/package.spec.ts` packs `web/packages/ares-wasm-js`. `prepack` runs `vite build`
  with `emptyOutDir: true` (`web/packages/ares-wasm-js/vite.config.ts`), which deletes
  `dist/` and writes it again.
- `web/e2e/profiles.spec.ts` packs `web/packages/ares-profiles`. `prepack` runs
  `scripts/build-profiles.mjs`, which calls `rmSync(dist)` and writes it again.

Four specs build a test page with `vite build` that imports those packages through
`web/node_modules/@ares-slicer/*`. The workspace links resolve to the same `dist/`
directories.

| spec | page | reads |
| --- | --- | --- |
| `configRoundTrip.spec.ts` | `e2e/config-page` | `ares-wasm-js/dist` |
| `ksrProject.spec.ts` | `e2e/ksr-project-page` | `ares-wasm-js/dist` |
| `platerSession.spec.ts` | `e2e/plater-session-page` | `ares-wasm-js/dist` |
| `profileSlice.spec.ts` | `e2e/profile-slice-page` | `ares-wasm-js/dist`, `ares-profiles/dist` |

With more than one Playwright worker, a reader's `vite build` can run while a pack spec has
emptied the `dist/` directory. A full local desktop run with default workers failed
`configRoundTrip.spec.ts` with
`ENOENT: no such file or directory, open '…/web/packages/ares-wasm-js/dist/ares_wasm_bg.wasm'`.
The same spec passed when run alone. CI runs the desktop project with `--workers 1`
(`.github/actions/web-e2e/action.yml`), so CI does not hit the race.

`configRoundTrip.spec.ts` and `platerSession.spec.ts` also both preview their page on port
4178 with `strictPort: true`. When they overlap, the second one fails to bind the port.

## Ways it can fail

1. A reader's `vite build` opens a `dist` file after the pack spec deleted it: ENOENT (seen).
2. A reader copies a `dist` file while the pack spec is still writing it: the page is built
   from truncated bytes. Nothing reports this; the page may still load and then misbehave.
3. A reader resolves `dist/index.js` from one build and `dist/worker.js` from the next. Both
   builds come from the same sources, so the bytes match, but only if no one edits the
   sources during the run.
4. Two readers both preview on port 4178 and overlap: `strictPort` refuses the second
   server.
5. A fix that moves these specs out of the `desktop` project makes `--project desktop` skip
   them silently. CI would lose coverage without failing.
6. A spec added later that also reads a package `dist`, or that also packs a package, is
   not covered by the fix, and the race comes back.

## Behaviour

- `web/playwright.config.ts` gets a `packages` project with `workers: 1` (Playwright
  `TestProject.workers`, available since 1.52; the repository uses 1.63). Its `testMatch`
  lists the six specs above, and the `desktop` project's `testIgnore` excludes them. The six
  specs run one at a time, in parallel with the rest of the desktop suite. No desktop spec
  reads a package `dist`, since the app under test is the prebuilt `web/app/dist` that the
  preview server serves.
- The project uses the desktop browser settings, so the six specs run exactly as before.
- `.github/actions/web-e2e/action.yml` passes `--project desktop --project packages`, so CI
  runs the six specs as before (failure mode 5).
- One serial project covers failure modes 1-4: no pack runs during a read, and no two
  readers share port 4178.
- A comment at the project lists the rule: a spec that packs a workspace package, or builds
  a page from one, belongs here (failure mode 6).

Not chosen:

- Running the pack specs as a project dependency of `desktop` would force them, and their
  rebuilds, before every single-spec desktop run.
- Packing with `--ignore-scripts` no longer tests the `prepack` script that `npm publish` runs.
- Turning off `emptyOutDir` leaves stale files to be packed, and it replaces failure mode 1
  with failure mode 2.

## Tests

The suite itself is the test. The six specs keep their assertions, and each run writes
Playwright's JSON report to `target/web-e2e/package-dist-race/`. Each report records every
test's start time and duration.

- Before the change: running the six specs plus the desktop suite with default workers shows
  at least one pack spec overlapping a reader in time.
- After the change: in the same run, no two of the six specs overlap, all six pass, and
  `--list` shows them under `packages` and not under `desktop`.

A small node script checks the overlap. It reads the JSON report, prints each of the six
specs' time intervals, and exits non-zero on any overlap. It is not committed; its output is
kept with the run's logs.

## Deferred

- `web/e2e/plateMove.spec.ts` and other specs that share one `target/web-e2e/<name>/`
  artifact directory across parallel workers.
