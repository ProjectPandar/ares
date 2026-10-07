# Spec: `estimate_points_properties` annotation inside `estimate_malformations` collect

Status: revised after upstream junction probe (was: "Arachne
junction-level parity of stored perimeter entities")
Origin: the C1 curled-perimeter residual
(`docs/superpowers/specs/2026-10-07-curl-artificial-distance-residual.md`)
was localized outside the curl formula — `artificial_distance` is
proven at parity by `artificial_distance_matches_the_sermoon_v1_
oracle_records` (commit `0786b86a9`, C1 pre-evidence riding this
slice's RED chain, not merging separately).

## Verified facts

Case: stock matrix Sermoon V1 cube10
(`/tmp/ares-stock-matrix-028c758-seeded/case-6qvP0t/input.3mf`).

- **Upstream's stored `ExtrusionLine` is IDENTICAL to ares'**: a
  dedicated oracle probe (`/tmp/junctdump.nix`, nix postPatch on
  `WallToolPaths::simplifyToolPaths`, built at
  `/tmp/junct-orca`) dumps upstream's pre/post-simplify junction
  lists: `PRE nj=9` (4 corners + 4 edge midpoints, all w=567079)
  → `POST nj=5` — the exact same 9→5 sequence ares produces. So
  Arachne junction generation/simplify is at parity; the earlier
  junction-density hypothesis is DISPROVEN.
- The 1.1612mm-spaced points in orca's CURL `seg=` records are
  therefore **not** stored polyline vertices — they are the
  `ExtendedPoint` list produced by
  `estimate_points_properties<true,true,false,false>` inside
  `estimate_malformations` (`SupportSpotsGenerator.cpp:159-175`):
  the annotation inserts boundary-intersection points and
  prev-layer-distance segmentation points (ExtrusionProcessor.hpp
  ~95-160: `min_distance`/`line_len` splits where
  `distance + 3·boundary_offset` crosses zero), and the
  `ExtrusionLine`s handed to `curled_up_height` are consecutive
  annotated-point pairs (~19 segments vs ares' 5).
- Ares' `CurlTracker::collect` (`motion/overhang/curl.rs:116-180`)
  consumes the raw 5-point list — no annotation, no segmentation —
  producing 5 whole-edge CurledLines including the degenerate
  corner line at h=0.4761 (orca: 0.0561/0.1503). That line drives
  `art=0.1179` vs orca's `0.0743` → band-edge flip →
  F6120 vs F6818.82 (and the F4800 vs F5580 sibling), matching
  every observed F-diff.
- Other Arachne cases pass byte-identical because their
  annotated point lists coincide with the raw polyline (flat
  walls fully supported → no boundary-crossing or segmentation
  inserts). The branch that differs here: **segments whose
  prev-layer distance crosses the slowdown band or
  `boundary_offset` on a partially-supported (chamfer) layer** —
  the C1 cube's layer-1 overhang exactly.

## Upstream anchors (source-cited boundary)

- `src/libslic3r/Support/SupportSpotsGenerator.cpp:150-196`:
  `estimate_malformations` — `extrusion->collect_points` →
  `estimate_points_properties<true,true,false,false>` →
  per-annotated-pair `ExtrusionLine` → `curled_up_height` →
  `l->curled_lines`.
- `src/libslic3r/GCode/ExtrusionProcessor.hpp:95-160`:
  `estimate_points_properties` — `boundary_offset` distance
  annotation, `intersections_with_line` insertion, and the
  `min_distance`/`line_len >= 2` segmentation that produces the
  ~1.16mm splits on this wall. The `<true,true,false,false>`
  instantiation vs the emit path's `<true,true,true,true>`:
  confirm which annotation stages each flag enables during
  implementation.
- Ares port: `crates/ares-core/src/project_slice/gcode_emit/
  motion/overhang/curl.rs::CurlTracker::{collect,collect_entity}`
  and `curl/geometry.rs` — the missing annotation step must be
  added on the collect path (sharing the `extended`-point
  construction used by `overhang::estimate` where upstream shares
  code, or porting `estimate_points_properties` faithfully).

## The open delta (what this slice must resolve)

Make `collect` build its `ExtrusionLine` inputs from the upstream
annotation result for `<true,true,false,false>`:

1. Distance annotation with `boundary_offset` (the `0.5·flow_width`
   convention used at `ExtrusionProcessor.hpp:95-140`).
2. `intersections_with_line` insertion where the distance sign
   flips.
3. The `line_len >= 2` / `min_distance` segmentation that produces
   the observed ~1.16mm vertices on this case.
4. `curvatures` must run over the ANNOTATED list (upstream computes
   curvature inside the same `estimate_points_properties` walk).

## RED tests (required order)

1. `ares-core` white-box RED: drive `CurlTracker::collect` (or the
   factored annotation step) on the fixture point list — the
   Sermoon-V1 layer-1 outer-wall entities (three `erExternalPerimeter`
   paths: `(4.597,4.645)→…→(-4.645,1.935)`,
   `(-4.645,1.935)→…→(4.645,4.645)`,
   `(4.645,4.645)→…→(-4.645,1.983)`; raw = 5-pt loop) — assert the
   produced `CurledLine` set equals orca's (the fixture
   `oracle_artd_sermoon_v1.txt` layer-1 sets: 13 lines with
   h ∈ {0.2057, 0.2613, 0.3054, 0.2980, 0.3003, 0.3129} pattern).
   Fails today (5 coarse lines, corner h=0.4761).
   The `prev_layer_lines`/`prev_layer_boundary` inputs for the
   annotation come from the L0 chamfer layer — fixture holds the
   minimal boundary lines needed.
2. E2E whole-file byte compare on `case-6qvP0t` (Sermoon V1),
   normalize only `; generated by` — the C1 acceptance case —
   must go green with the fix, including M73/E hunks.

## Blast radius / gate requirements

The change is confined to `collect` (the malformation
annotation path): emitted vertex sequences, `estimate()`'s own
`<true,true,true,true>` path, and all other consumers are
untouched. Affected surface: `CurlTracker` output on
partially-supported walls (the `detect_overhang_wall` Arachne
cases where annotation inserts points).

GREEN bar: full v2 control set (seeded + official) on the
submission hash name-for-name non-regressing vs base `d789b0750`;
`cargo nextest run --workspace`, clippy, fmt, wasm32 check.

## Scope

In: the `estimate_points_properties` annotation inside
`CurlTracker::collect` (boundary-offset distance, boundary
intersections, min_distance segmentation, annotated curvature);
the fixture white-box test; the C1 byte-compare flip.
Out: Arachne junction generation (verified at parity via the
junct-orca probe), `artificial_distance` (verified at parity via
`0786b86a9`), emitted-move paths, other matrix cases.
