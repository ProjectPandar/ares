# Task 22O.39 — Detect grouped bridge directions

## Status and source boundary

Locally implemented, crate-private, inactive, and unreleased. Exact predecessor
O38 is released as implementation commit
`04920e061b9b7e3e780b0735fccd0610b52eb73c` and documentation commit
`2d6154d401c3c954bed69de6ba631a53af05f1a3`. Exact-SHA Tier-1 run
`31303115603` passed all five jobs and both browser executions at
`2d6154d401c3c954bed69de6ba631a53af05f1a3`; authoritative run JSON is
archived outside the repository at `/tmp/task22o38-tier1-exact-sha.json`.
O38 remains crate-private and inactive. Pinned Orca remains v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

The repaired fresh implementation cycle has an authoritative compiling stub
RED with 11 body-dependent failures and exactly two disclosed stub-equivalent
passes, followed by 14/14 focused debug/release GREEN. The pinned helper matrix
passes 12 assertions in both Debug and `NDEBUG` with byte-identical output and
covers two ordered bridges plus an unmatched boundary. Complete reviewed
literals, contour and every hole point-buffer identity, M01-M28 mutation
coverage, exact restoration, and both implementation rereviews pass.

Complete exact-final-byte native, WASM, static, and rollback verification also
passes: focused debug/release are 14/14, workspace Nextest is 6,094/6,094 with
two skipped, all-target check, warning-denying all-feature Clippy, rustfmt,
four wasm32 checks, two optimized WASM builds, both bindgen runs, export/npm/
JavaScript audits, exact allowlist audit, and exact-O38 rollback are green. Both
local Playwright attempts failed before test execution because Chromium lacks
`libglib-2.0.so.0`; neither is called a pass, and both exact-SHA CI browser
executions remain mandatory. O39 remains unreleased pending final reviews,
separate commits, push, and exact-SHA Tier-1. Public slicing still returns
`ProjectSlicingIncomplete`; this milestone is not KSR parity.

Port only `detect_bridge_directions` at
`OrcaSlicer/src/libslic3r/LayerRegion.cpp:262-308`. This translation-unit-local
helper composes the released O36 `WaveSeed`/`ExpansionZone` records, O37
`Bridge`, and O38 `detect_bridging_direction` helper. Its exact direct source
dependencies are:

- `to_polylines(const ExPolygon &)` at `ExPolygon.hpp:228-242`;
- `to_polygons(const ExPolygon &)` at `ExPolygon.hpp:300-307`;
- `to_lines(const Polylines &)` at `Polyline.hpp:180-193`;
- `EPSILON`, `scale_`, and `SCALED_EPSILON` at
  `libslic3r.h:52,93-96`;
- `DefaultJoinType == jtMiter` and `DefaultMiterLimit == 3.` at
  `ClipperUtils.hpp:19,23-27`;
- `expand(const Polygons &, float, ...)` at `ClipperUtils.hpp:373-376`;
- the non-recombining open-path kernel at `ClipperUtils.cpp:837-845`, the
  distinct recombining closed-polygon adapter at lines 848-903, and
  `diff_pl(const Polylines &, const Polygons &)` at
  `ClipperUtils.hpp:457` / `ClipperUtils.cpp:908-909` (contrasted with the
  `Polygons` overload at lines 916-917);
- the released O38 direct helper at `BridgeDetector.hpp:75-119`.

Ares already owns all corresponding platform-neutral primitives. O39 composes
those primitives through the existing indexed Clipper kernel; it adds no new
geometry engine or dependency.

Deferred: `merge_bridges` at `LayerRegion.cpp:310-351`, the remaining
multi-type/thickness behavior of `fill_surfaces_extract_expolygons` at lines
147-164 beyond O35's single-type extraction, `expand_bridges_detect_orientations`
at lines 398-437, and the active
`LayerRegion::process_external_surfaces` implementation at lines 486-623 with
declaration at `Layer.hpp:86`. Also deferred are lifecycle activation, flow-
derived expansion-zone construction, Options, public adapters, fill, toolpath,
seam, motion, serialization, G-code, post-processing, and normalized KSR
parity. The `if constexpr(false)` diagnostic-only SVG block at
`LayerRegion.cpp:295-306` is intentionally omitted: it is compile-time
unreachable behavior and O39 adds no debug filesystem/instrumentation path.

## Ares destination and API

Extend only the inactive private
`project_slice::prepare_infill::external_surfaces` module with:

```rust
pub(in crate::project_slice) fn detect_bridge_directions(
    bridge_anchors: &[WaveSeed],
    bridges: &mut [Bridge],
    expansion_zones: &[ExpansionZone],
    scale: CoordinateScale,
) -> Result<(), ClipperError>;
```

The explicit `CoordinateScale` is Ares' request-local replacement for Orca's
mutable global `SCALING_FACTOR`. The same value must reach both the scaled-
epsilon calculation and O38. Ares returns `ClipperError` because its safe
indexed offset and open-path difference kernels expose coordinate failures.
No error may be remapped, retried, swallowed, or converted into a partial
result.

The source's unconditional `std::runtime_error` for an empty zone list becomes
an unconditional Rust assertion with the exact text
`At least one expansion zone must exist!`. It executes before bridge or anchor
inspection. Do not replace it with an empty shortcut, debug-only assertion,
new public error variant, or validation object.

Do not add a request object, generic/trait overload, public export, alternate
entry, lifecycle caller, or production test seam. The helper mutates only
`Bridge.angle`; it borrows all anchors, zone geometry/parameters, and bridge
ExPolygon geometry. O39 remains inactive and public slicing continues to
consume O26 before returning `ProjectSlicingIncomplete`.

## Frozen anchor cursor and boundary lookup

After the zone assertion, initialize one anchor cursor at the start of
`bridge_anchors`. Iterate `bridge_id` in increasing `u32` source order over the
narrowed bridge count, matching the source's
`bridge_id < uint32_t(bridges.size())`. For each bridge:

1. Start an empty ordered `anchor_areas` polygon vector and
   `last_anchor_id = -1i32`.
2. Consume consecutive anchors only while the current anchor's `src` equals
   this `bridge_id`. The `WaveSeed.path` geometry is never inspected; only
   `src` and `boundary` participate. Do not sort, regroup, search ahead,
   restart the cursor, or validate source ordering.
3. Convert each matching `boundary` to `i32` exactly like the source
   `int(...)` cast. Only when it differs from `last_anchor_id`, assign it and
   perform zone lookup. Consecutive duplicate boundary IDs are skipped; a
   later nonconsecutive duplicate is processed again if presented by the
   supplied order.
4. For every `ExpansionZone` in input order, maintain `start_index` and
   `end_index` as source-width unsigned `u32` values. Advance `end_index` by
   `zone.expolygons.len() as u32` with wrapping arithmetic. Compare the signed
   `last_anchor_id` widened to `i64` against `end_index` widened to `i64`.
5. On the first matching zone, compute the local index with the source's
   unsigned subtraction semantics:
   `(last_anchor_id as u32).wrapping_sub(start_index) as usize`. Trust that
   index, append a clone of the selected ExPolygon contour followed by clones
   of its holes, then stop scanning zones.
6. If no zone matches, append nothing. After each nonmatch, assign
   `start_index = start_index.wrapping_add(zone.expolygons.len() as u32)`.

The `i32` boundary cast, `u32` cumulative casts/additions, signed comparison,
and unsigned local subtraction are part of the frozen compatibility boundary.
Do not replace them with `usize` arithmetic, checked conversion, saturating
math, validation, or fallback. Malformed trusted IDs may panic on indexing, as
the source would leave the valid internal domain.

## Frozen geometry composition and angle assignment

For every bridge, after anchor consumption:

1. Materialize two source-shaped views in contour-before-holes order: an
   overhang polygon list containing contour/hole clones for O38, and an open
   polyline list produced by `split_at_first_point()` for each contour/hole,
   explicitly duplicating each ring's first point exactly once like upstream
   `to_polylines`.
2. Compute `scaled_epsilon` as `(1e-4_f64 / scale.factor()) as f32`, preserving
   the source double division followed by `float` cast, then execute
   `assert!(scaled_epsilon > 0.0)` as the Rust equivalent of upstream
   `expand`'s unconditional positive-delta assertion.
3. Expand the accumulated anchor polygons exactly once with existing
   `offset_paths(anchor_areas, scaled_epsilon, JoinType::Miter, 3.0)`. The
   assertion and Clipper error precede open-path difference. The raw offset's
   inherent union behavior is required; do not add a separate pre-union.
4. Difference the explicitly closed polyline list against expanded anchors
   exactly once through a new narrow crate-private helper:
   `difference_open_polylines(&[Polyline], &[Polygon]) ->
   Result<Vec<Polyline>, ClipperError>`. This helper must call the existing
   indexed Clipper open-subject path once with NonZero subject/clip rules and
   return `PolyTree::into_open_polylines()` directly. It must not call
   `recombine_polylines`; that stage belongs only to the source's distinct
   closed-`Polygons` overload and would change O39 fragment/order semantics.
5. Convert the returned non-recombined polylines to lines in polyline order and point-window
   order, emitting one `Line` for each adjacent point pair. Do not close an
   already open fragment or add/remove/reorder segments.
6. Call released O38 exactly once with those lines, the ordered bridge polygon
   list, and the unchanged `scale`. Ignore only its unsupported-distance
   result, as upstream does.
7. Assign `bridge.angle = Some(PI + atan2(direction.y, direction.x))` only after
   both fallible Clipper operations and O38 complete.

The bridge polygon clones are source-shaped copies performed by upstream
`to_polylines`/`to_polygons`; no clone of the owned `Bridge.expolygon` may
replace or reallocate its stored point buffers. Preserve contour-before-holes,
zone order, anchor order, Clipper path/point order, exact f32/f64 cast points,
and the written `PI + atan2(y, x)` association. Do not normalize the angle,
consume the unsupported distance, sort lines, union anchors, add a safety
offset, use a different join/miter setting, substitute O38's polygon overload,
or introduce an extra/pre-union or an early empty-anchor/empty-bridge shortcut
before the mandatory zone assertion. The union inherent in `offset_paths` is
source behavior and remains required.

## Mutation, error, and ownership semantics

The function is sequential and commits bridge angles one at a time. If offset
or difference fails for bridge `n`:

- angles assigned to earlier bridges remain committed;
- the failing bridge retains its entry angle;
- later bridges retain entry angles and are not visited;
- borrowed anchors, zones, parameters, ExPolygons, and all stored bridge point
  buffers remain unchanged;
- no output value other than the direct error escapes.

The empty-zone assertion precedes every Clipper failure and all mutation. With
at least one zone, zero bridges returns `Ok(())` without reading anchors.
Empty anchors still run the complete per-bridge geometry pipeline: expansion of
an empty polygon list, open-path difference, O38, and angle assignment.

Do not add preflight, transaction staging, rollback of prior angles, source-ID
validation, boundary validation, coordinate validation, retry, partial-result
container, or fallback. These are internal source-shaped inputs, not a public
boundary.

## Tests, original Orca oracle, and chronological TDD

Use ordinary split test modules. Prefix every focused test with `task22o39_` so
`cargo nextest run -p ares-core task22o39` is exact. Commit only manually
reviewed behavior-named Rust literals; raw helper source/output, generated
G-code, serialized blobs, and mutation logs stay under `/tmp`.

Capture a real compiling RED against a temporary production stub that performs
the mandatory empty-zone assertion and otherwise returns `Ok(())` without
mutating angles. The assertion and function shape are stub-equivalent and must
be disclosed, not called RED failures. Do not reconstruct chronology after the
body is installed.

Historical evidence is retained truthfully: the first attempted repaired RED
still contained two stale Linux-host tie-oracle failures before O39 and is not
promoted. O39 was reset to the exact assertion/`Ok(())` stub for a genuine fresh
body cycle; corrected released-O38 literals and reviewed repeated/multi/pointer
witnesses were installed before the body. The authoritative fresh-cycle RED is
`/tmp/task22o39-fresh-cycle-authoritative-red.txt`: 13 tests, 11 failures that
reach and contradict the stub, and exactly two stub-equivalent passes. Only
after that run was the frozen body reinstalled. Later no-restart and no-sort
recurrence witnesses are explicitly post-hoc mutation coverage, not
chronological RED.

Focused tests must cover:

- empty-zone panic with exact text, including zero bridges and otherwise
  invalid geometry to prove precedence;
- nonempty zones with zero bridges and unread/unconsumed anchors;
- empty anchors with complete contour/hole floating-edge line order and exact
  angle bits;
- one bridge with repeated boundary IDs and deliberately unrelated seed paths,
  proving paths are ignored; contour-plus-hole anchor extraction, complete
  expanded-anchor/difference line literals, O38 manual-pipeline equality, and
  final angle bits;
- one pinned vector for which the existing recombining closed-polygon `diff_pl`
  would merge returned fragments, proving O39 instead retains the upstream
  non-recombined fragment, point, line, direction, and angle order;
- multiple bridges and multiple zones, including leading empty zones, a
  boundary with no matching zone, cumulative global-to-local rebasing, ordered
  cursor consumption, and complete final angle vector;
- intentionally unsorted or source-skipping anchors proving there is no sort,
  search-ahead, or cursor restart;
- Normal and LargeBed scale vectors exposing the f64-to-f32 scaled epsilon and
  unchanged O38 scale forwarding where behavior is observable; structural
  source/diff audit fixes forwarding when a bounded vector is equivalent;
- at least one direct offset coordinate failure and one direct non-recombining
  open-difference coordinate failure; collectively prove first-bridge no-
  commit, later-bridge earlier-angle commit, exact `ClipperError` identity,
  failing/later angle retention, and unchanged borrowed/stored geometry;
- trusted signed-boundary/local-index panic behavior where constructible;
- exact API/result/visibility shape and stored contour/hole pointer identity.

Run the pinned original Orca CLI on the KSR project in a disposable environment.
Require success metadata and a nonzero generated G-code size, then delete the
G-code without reading its content. Build one disposable helper from the exact
pinned O39 function and dependencies in Debug and `NDEBUG`; require byte-
identical complete anchor-selection, floating-line, direction, and angle output
for the committed valid vectors. The helper may expose intermediate values only
under `/tmp`; production receives no oracle/test seam. Rust-specific Clipper
error witnesses are verified directly against the explicit Rust pipeline.

Apply post-hoc mutations one at a time and restore exact bytes. Include empty-
zone shortcut/assertion changes, anchor cursor restart/search/sort, duplicate
handling, signed/unsigned cast or zone-count changes, contour/hole order,
wrong epsilon width/scale, join/miter substitution, omitted offset or diff,
line closure/order changes, hard-coded O38 scale, unsupported-distance use,
angle sign/order/normalization, early angle assignment, swallowed errors,
signature, or visibility. Classify runtime kills, compiler rejections, and
behaviorally equivalent survivors truthfully; do not add production
instrumentation only to force observability.

## Files, limits, and prohibitions

Allowed Rust edits only:

1. `crates/ares-core/src/geometry/clipper/polyline.rs` — add only the narrow
   non-recombining `difference_open_polylines` wrapper over the existing
   private open-path kernel and its shape assertion;
2. `crates/ares-core/src/geometry/clipper.rs` — crate-private reexport and shape
   assertion only;
3. `crates/ares-core/src/geometry.rs` — crate-private facade reexport and shape
   assertion only;
4. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs` —
   ordinary module registration, private reexport, exact shape assertion;
5. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/detect_bridge_directions.rs`
   — sole O39 body, at most 220 physical lines;
6. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests.rs`
   — one ordinary test registration and exact shape constant;
7. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions.rs`
   — focused helpers and ordinary submodule registrations, at most 180 lines;
8. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions/anchors.rs`
   — anchor/cursor/oracle witnesses, at most 300 lines;
9. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions/geometry.rs`
   — geometry/scale/angle and non-recombining topology witnesses, at most 300
   lines;
10. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions/errors.rs`
   — assertion/error/ownership witnesses, at most 300 lines.

No existing O35-O38 Rust body/test, geometry Clipper engine/state-machine body,
`types.rs`, manifest/lock/dependency, lifecycle/stage/predecessor, adapter,
workflow, golden, fixture expectation, or G-code path may change. The three
geometry edits are limited to exposing the already implemented open-path
operation without recombination; no other kernel behavior may change. Allowed documentation
is the O38 spec/plan release-state correction, this O39 spec/plan,
`docs/roadmap.md`, and `docs/architecture/option-parity-v4.md`.

Every Rust file remains below 400 physical lines. No broad lint allowance,
`unsafe`, FFI, filesystem/native thread, platform branch, public API/hook,
hard-coded fixture identity/name/hash/layer-count/geometry branch, reference-
G-code read, binary oracle, legacy fallback, source concatenation,
source-pinning test, second clipping engine, dependency change, or host-random
ordering.

## Verification, review, release, and rollback

Require focused debug/release O39, complete external-surface tests, O38/O37/O36/
O35, O28/O30, RegionExpansion, complete geometry, PolyTree/boolean-paths/
offset, O26 lifecycle, workspace Nextest, all-target check, warning-denying
Clippy, rustfmt, four WASM checks, two optimized builds, bindgen/export/
JavaScript audit, and two Playwright runs. If local Chromium lacks
`libglib-2.0.so.0`, record each failure exactly and require both exact-SHA CI
executions; never label it a pass.

Static-audit the exact allowlist, ordinary modules, LOC, private visibility,
source operation/cast/error order, absence of forbidden patterns, empty staging,
and no generated artifact. Rehearse disposable rollback to exact released O38
`2d6154d401c3c954bed69de6ba631a53af05f1a3` and prove the primary candidate
unchanged.

Fresh independent six-dimensional and default-model OpenCode reviewers must
approve spec, plan, implementation, and final documentation. Every accepted
repair invalidates stale evidence: rerun affected and complete exact-byte gates,
refresh static/rollback evidence, and repeat both reviews.

Use separate Conventional Commits for implementation and documentation, push
only approved files, and require Tier-1 `headSha` to equal the pushed
documentation SHA with exactly five successful jobs and both browser executions.
No tracked O39 release-state edit follows that run; O40 records released state.

The next bounded source candidate after O39 is `merge_bridges` at
`LayerRegion.cpp:310-351`. `fill_surfaces_extract_expolygons`,
`expand_bridges_detect_orientations`, lifecycle integration, Options,
fill/toolpath/motion/G-code, and full KSR parity remain deferred.
