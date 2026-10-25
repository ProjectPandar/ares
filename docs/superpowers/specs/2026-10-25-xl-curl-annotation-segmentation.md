# Spec: curl-annotation point pipeline for long walls — XL block-hole witness

Status: GREEN implemented on branch `w6/xl-curl-annotation-red-4af042def`
(RED frozen at `f33e5ab6c`, coordinator-approved scope; witness,
controls and the C1 Sermoon regression pass in both modes).
Provenance: machine-matrix discovery at main `4af042def` (Prusa XL, 2nd
machine), two-sided GDB localization in
`/home/indexyz/ares-evidence/w6-machine-matrix-blockhole-p8n4d/`
(`SPEC-PROPOSAL-v2.md`); extends the C1 curled-perimeter spec family
(`2026-10-06-curled-perimeter-slowdown.md`, which already anticipated
"`bridge_distance` 16 passed as `max_line_length` changes segmentation if
edges were longer" — the Sermoon C1 cube10 edges are 10 mm, below the
threshold, which is why `2026-10-07-sermoon-c1-verification.md` correctly
observed "five annotated vertices remain five").

## Verified residual

Stock Prusa XL 0.4 nozzle defaults + `hole-to-polyhole` block mesh
(29.55 mm outer-wall edges), `slowdown_for_curled_perimeters=1`,
elephant-foot default 0.2. Dual-mode complete stream: exactly 2 hunks in
10581 lines, official and seeded identical:

1. layer 3 inner wall: 4 lines with E 5th-decimal `.97122→.97119`
   (same segment byte-equal on layer 2 and every layer ≥ 4);
2. one inserted `G1 F10140` (169 mm/s) at `;TYPE:Outer wall` start.

Root cause (GDB-verified): ares layer-2 external square registers 5
CurledLines with heights 0.21–0.34 (distance 0.318/0.2249994 to the
compensated first-layer lines, curvature π/3..π/12); native registers
none. The layer-3 artificial curl distance then forces 169/167/162 mm/s,
`variable=true`, and the processed emission path emits
`processed[0].speed*60 = 10140` plus quantized per-segment E. Native keeps
raw-path emission (speeds all 170, `variable=false`). Single-key proof:
`slowdown_for_curled_perimeters=0` → ares == native, 0 hunks (both change
vs curled=1). Retracted hypotheses: `speeds/` band shell (v1), estimator
replacement (v1), `boundary_sign` zero-boundary float flip (inputs are
bit-identical f32 middle + `0.5f·width`; sign=+1 both sides).

## Upstream boundary (source-cited, complete instantiation)

`estimate_malformations` (`Support/SupportSpotsGenerator.cpp:141-196`)
feeds every `erExternalPerimeter` entity through
`estimate_points_properties<SCALED_INPUT=true, ADD_INTERSECTIONS=true,
PREV_LAYER_BOUNDARY_OFFSET=false, SIGNED_DISTANCE=false>`
(`:170`) with `max_line_length = params.bridge_distance = 16.0`
(`SupportSpotsGenerator.hpp:38`). With `boundary_offset = 0` the full
point-annotation pipeline (`GCode/ExtrusionProcessor.hpp`) is:

1. start/next point annotation — unsigned distance to the previous
   layer's external `ExtrusionLine`s, `+0` offset (`hpp:92-105`);
2. **intersections block — EXECUTES** (`ADD_INTERSECTIONS=true`,
   `hpp:106-121`): when consecutive annotated points straddle
   `boundary_offset + EPSILON` (= EPSILON), insert every
   `intersections_with_line<true>` crossing point with
   `distance = boundary_offset = 0`, filtered by
   `min_spacing = 0.25·flow_width`;
3. segmentation block — SKIPPED (requires
   `PREV_LAYER_BOUNDARY_OFFSET && ADD_INTERSECTIONS`, `hpp:124`);
4. **max_line_length block — EXECUTES** (`hpp:189-215`):
   `len = squaredNorm`, `t = sqrt(16²/len)`, `count = 1/t` (truncated),
   insert points at `curr·(1−j·t) + next·(j·t)` for `j = 1..=count`,
   distance re-queried unsigned, `min_spacing` filter;
5. **curvature — on the FINAL annotated sequence** (`hpp:219-278`):
   windows 3/9/16 mm, `looped` from the raw input's front==back,
   backward/forward walks over annotated points.

Per-segment curl (`SupportSpotsGenerator.cpp:176-184`): middle-distance
re-queried unsigned at the segment midpoint (`:177`), sign via
`prev_layer_boundary.distance_from_lines<true>(middle) + 0.5f·flow_width`
(`:179`), `estimate_curled_up_height(dist·sign, 0.5·(a.curvature +
b.curvature), …)` (`:102-135`), tolerance 0.1 (`hpp:50`).

## Ares destination boundary and segment-by-segment status

`crates/ares-core/src/project_slice/gcode_emit/motion/overhang/curl.rs`
(`CurlTracker::collect`, `:148-186`; entity collection `:97-131`):

- start/next annotation + midpoint re-query + sign + curl formula +
  chained heights + f32 middle rounding + degenerate `i == 0` line +
  `previous`/`previous_tree` rollover: **ported and C1-verified** — keep;
- intersections block (item 2): **missing**. The existing
  `overhang/sections.rs::add_boundary_intersections` is the
  `<true,true,true,true>` instantiation (offset `0.5·width` threshold, z
  crossing check, `ExtendedPoint` in the estimator's units) and is NOT
  the curl instantiation's semantics (threshold `EPSILON`,
  `distance = 0`, 2D lines) — GREEN must implement the curl instance's
  block, not reuse the estimator one;
- segmentation block (item 3): correctly absent in both;
- max_line_length block (item 4): **missing** — the primary defect;
- curvature (item 5): `curl/geometry.rs::curvatures` already mirrors
  `hpp:219-278` (windows, `looped`, walk interpolation), but runs on the
  RAW entity points — GREEN must run it on the annotated sequence
  (items 2+4 applied first).

GREEN shape: `collect` gains the two insertion passes in upstream order
(intersections first, then max_line_length, both with unsigned distance
re-query and `0.25·width` min-spacing), then curvature, then the existing
per-segment loop over annotated neighbors. No estimator (`overhang.rs`,
`sections.rs`) changes; no `speeds/` changes. The C1 Sermoon tests must
keep passing: 10 mm edges give `count = 10/16 → 0` inserted points, so
their "five annotated vertices" behavior is preserved by the same
truncation semantics.

Included: the two insertion passes + curvature resequencing for the curl
instantiation. Deferred: the estimator instantiation's pipeline (already
ported), machines 3-12 of the discovery matrix (resume after GREEN).

## RED witnesses (tests-only, runtime Native dual mode)

`tests/parity/xl-curl-segmentation/` (input 3MFs committed; Native
outputs are runtime-only, never committed):

- `xl-witness.3mf` — stock XL export, RED: strict whole-file equality
  fails on exactly the two hunks above;
- `xl-curl-off.3mf` — same project with
  `slowdown_for_curled_perimeters=0`, control: must PASS (curl chain
  bypassed) in both modes;
- `mini-control.3mf` — stock Prusa MINI export of the same mesh,
  control: must PASS in both modes.

Tests: `crates/ares-cli/tests/orca_parity/xl_curl_segmentation.rs`
(fixture SHA pins, Native admission, strict ordered compare for API and
CLI, raw API==CLI).
