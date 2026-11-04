# Web initial JS gzip budget headroom

## Why this slice

After `c4572b368` the initial JS (entry + modulepreloads) is 124,955 B gzip,
45 B under the `bundle.spec.ts` budget `INITIAL_GZIP_BUDGET = 125_000`. The next
web slice cannot add anything to the first screen. The budget stays as it is;
headroom must come from moving non-first-screen bytes out of the initial chunk.

This is a web-shell packaging slice. It ports no `libslic3r`/`libvgcode`
behavior and changes no slicing, G-code, config or viewer-data output. The
rewrite gate in `AGENTS.md` applies to the domain slices and is not used here.

## Measurement (`c4572b368`)

The initial chunk was rebuilt with `vite build --sourcemap` into a scratch
outDir; it has the same content hash, `index-C3T7tT2x.js`. Its minified bytes
were attributed to sources through the sourcemap. The gzip column is each
group's text compressed alone, so it is approximate and the rows do not sum to
the total.

| gzip ≈ | raw | source |
|---:|---:|---|
| 63,731 | 206,992 | react-dom |
| 18,130 | 51,454 | @tanstack/router-core |
| **11,062** | **105,861** | **app/src/shell/icons.tsx** |
| 4,741 | 12,703 | @tanstack/react-router |
| 3,163 | 17,540 | app/src/sidebar/processLayout.ts |
| 3,089 | 8,226 | react |
| ≤ 2,068 each | | the rest of the app shell |

`icons.tsx` is 33 lines. Its weight is
`import.meta.glob("../assets/orca/*.svg", { eager: true, query: "?url" })`:
all 73 OrcaSlicer SVGs (87,762 B on disk) are below Vite's
`assetsInlineLimit`, so each becomes a base64 `data:image/svg+xml` URL inside
the entry chunk. The built chunk contains 73 such URLs and `dist/assets`
contains 0 `.svg` files. Only a few icons show on the first screen: the
TopBar's 3, the Sidebar's 4 and the ObjectList volume icons. The settings-page
group icons appear only when those pages open.

## Change

**Emit the Orca SVGs as asset files instead of inlining them.** In
`app/src/shell/icons.tsx`, the glob query becomes `"?url&no-inline"`; Vite
8.3.2 honors `no-inline` per import. `OrcaIcon` keeps rendering `<img src>`,
which now points at the hashed `assets/*.svg` file rather than a `data:` URL.

Measured with that one-line change (scratch build, reverted):

- Initial JS gzip is 115,515 B, a 9,476 B decrease (headroom 45 → 9,485 B).
- `dist/assets` gains 73 `.svg` files; nothing else moves between chunks.

Rejected or deferred alternatives:

- **Raising the budget:** not allowed.
- **Lazy Preview route (`PreviewPage`, `LayerSlider`, `layerRange`,
  `roles`):** about 3 kB gzip. This is a larger router change. It is deferred
  until headroom runs out again.
- **Lazy `processLayout.ts` with the settings pages:** about 3 kB, but the
  Process section of the first-screen sidebar reads it. Deferred.
- **Lazy `ShortcutsDialog`:** about 0.8 kB, too small to justify on its own.

## Failure modes, and the evidence that rules each out

1. **A first-screen icon no longer renders** (a broken URL from `base: "./"`,
   or a missing asset): a new assertion requires every `img.orca-icon` on the
   Prepare first screen to have loaded (`complete && naturalWidth > 0`). It
   runs in the same `bundle.spec.ts` test, after the shell is visible.
2. **Icons are inlined again later:** `bundle.spec.ts` asserts that the
   initial JS contains no `data:image/svg+xml`.
3. **Headroom is lost silently:** the budget constant is unchanged. The new
   initial gzip is recorded in `target/web-e2e/bundle/result.json`, as today.
4. **Screenshots captured before the icons arrive:** the visual specs
   only produce artifacts and contain no pixel assertions. Their screenshots
   are checked by eye.

## RED-first plan and acceptance evidence

1. **RED** (`test(web)`): in `bundle.spec.ts`, assert that no initial script
   contains `data:image/svg+xml`, plus the first-screen `img.orca-icon` load
   assertion. Expected failure: the `data:` assertion on the 73 inlined URLs;
   the icon load assertion passes both before and after. Filter
   `bundle.spec.ts`.
2. **GREEN** (`perf(web)`): the `?url&no-inline` query.
3. **Gates:** `npm run build:wasm && npm run build`, typecheck, lint. Then the
   full web E2E on the final HEAD. No Rust or G-code change is expected; fmt, clippy and
   `cargo nextest run --workspace` still run as a sanity gate.

## Open question

Is about 9.5 kB of headroom from the icon change alone enough, or should the
lazy Preview route also be scheduled in this slice?
