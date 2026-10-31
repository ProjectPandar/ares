# ARD: beta overhang sampling by effective printing state

Status: approved source slice; implementation pending. Later batch only, not
frozen B13. Authority: OrcaSlicer v2.5.0-beta,
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Owning upstream boundary

- `GCode/ExtrusionProcessor.hpp:50–355::estimate_points_properties`: the
  `<true,true,true,true,false>` scaled-input, intersection-adding,
  previous-boundary-offset, signed-distance, no-curvature instantiation.
- `ExtrusionProcessor.hpp:541–719::estimate_extrusion_quality`: retain the
  existing speed ladder/curl adapter; port the effective-speed callback and fan
  threshold transport into point discovery, and the source processed-point
  speed/overlap calculation.
- `GCode.cpp:9538–9643::_extrude`: detect overhang for speed **or fan**, preserve
  first-layer/over-raft/role guards, reference/effective speed caps, reset point
  speeds when slowdown is disabled, and distinguish segmented extrusion from a
  uniform path-wide overhang fan state.
- `GCode.cpp:9164–9187`, `9771–9777`, `9855–9914`, `10040–10084`: source threshold
  mapping, role-forced fan state, constant-path marker consumption and existing
  segmented-path marker consumption. These consume the newly transported state;
  they are not a separately designed cooling or motion pipeline.

The Rust destination is exclusively
`project_slice/gcode_emit/motion/overhang.rs` and new children under
`motion/overhang/`, with the necessary existing `motion/path` and `motion/fan`
consumers. Core remains platform-neutral; no terminal, filesystem or Native
instrumentation enters production code.

## Existing compatibility shell being replaced

The current `overhang/sections.rs` mixes the source speed ladder and boundary
intersection adapter with an older distance-only splitter. It segments long
lines by distance even when every piece prints identically and has no beta
interior discovery or ordered two-cut retention. `overhang::estimate` currently
returns points only when speed varies; `path.rs` treats that as the sole signal
for point-wise emission and loses fan-only transitions or uniform overlap-driven
fan state. Replace that splitter and result transport, not the underlying
boundary tree, ClipperZ, path preparation, or cooling buffer.

The earlier independent `ExtrusionLine` collinear slice is complete: RED
`906e6f34c`, GREEN `c1c52a8d7`. Its supported-layer witness remains strict and
must stay GREEN. The first-overhang witness remains independently RED.

## Included source behavior and exact data flow

### Printing-state predicates

Represent the source captured callback state in a borrowed internal context:
existing `[(f32,f32)]` distance/speed sections, original speed as `f32`, inverse
flow width as `f32`, and fan overlap threshold as `f32`. Do not allocate a boxed
callback or invent speed bands.

- Effective speed is `min(calculate_speed(distance), original_speed)`; distinguish
  section reference speed from the actual capped/resonance-avoided path speed.
- Fan-on is threshold>=0 and `1.f-distance*width_inv <= threshold`.
- Same printing state means effective speeds differ by at most1mm/s and equal
  fan state; slower/cooled means lower by more than1mm/s or newly cooled.
- Threshold mapping is source10/25/50/75/95 percent to .9/.75/.5/.25/.05 as
  **f32**; the zero-percent enum has threshold−1 and source external-role-wide
  cooling. Bridge and overhang-perimeter roles bypass overlap thresholding.

### Input distances and discovery

Keep existing signed previous-boundary distances, intersection ordering, 3D
positions and boundary-offset addition before narrowing to `f32`. Boundary boxes
are at Z0. The known-distance map is pure memoization and may be omitted by
recomputing against the same boundary; do not introduce another distance policy.

Run interior discovery only when smallest slowdown distance>0 or overlap fan
threshold>=0. The quality-estimator caller always supplies effective speed, so
its source callback-present branch is the actual port boundary, not a runtime
feature flag. Generic callback-absent callers are not routed through this API.

Use `max(2.,4*min_spacing)` discovery spacing and depth10 exactly. Per segment,
push left then right subspans onto the LIFO stack, sample midpoint distances,
sort by position along the span, then retain samples **in place** with the source
kept-prefix/next-unprocessed-neighbor semantics. Preserve the special treatment
of the segment's own two ends and lazy output construction. Do not discard
matching coarse samples before the retention pass.

### Segmentation and ordered two-cut retention

For source near-boundary spans, split only if length>=2mm and either end is
slower/cooled relative to a supported distance0. Preserve completed `f32`
distance-plus-three-offset expressions, promotion to `f64` length arithmetic,
clamping, sorted cut parameters and each cut's own signed distance.

Sample both source candidates in order with the asymmetric parameter-end guards;
apply strict minimum endpoint spacing. Update keep[0] first using still-current
keep[1], then keep[1] using the **updated** keep[0]. Finally discard the second
cut if both survive within minimum spacing, and append surviving cuts in source
order. No emitted-move filtering, endpoint canonicalization or alternate ties.

### Quality estimator and G-code consumer

Preserve source per-segment endpoint speed minimum, actual original-speed cap,
existing artificial curl contribution and minimum endpoint overlap. Disable
speed changes after estimation when the nozzle slowdown option is off, without
disabling the fan discovery path.

Return the processed points plus source `requires_segmented_extrusion` and
`path_overhang_fan_active` results. Require segmentation for speed deviation>1
or adjacent overlap fan-state transitions; a uniform fan state is path-wide.
Only segmented emission consumes quantized processed points. Constant emission
retains original prepared path points, raw lengths, arc fitting and wipe data,
while receiving the uniform fan flag. Preserve source forced cooling for bridge,
overhang-perimeter and zero-threshold external roles. Existing variable fan
markers use the source previous/current conjunction and initial internal-bridge
state, with f32 threshold comparisons.

The current normal path adapter always passes its extrusion through the cooling
buffer, corresponding to `m_enable_cooling_markers=true`. Source disabling of
markers during sequential transition **travel** remains in the travel owner;
no new global cooling option or invented extrusion fallback is introduced.

## Explicit deferrals / unchanged neighbors

- Generic max-line-length resampling and curvature calculation are inactive in
  this cited instantiation (`max_line_length=-1`, `CURVATURE=false`); do not port
  unrelated template modes or expose configuration for them here.
- Known-distance and ordering caches are pure memoization, not mandatory mutable
  state. No Native-internal cache instrumentation.
- Existing curled-line geometry/quality approximation is reused, not redesigned.
  Configuration speed identities and existing cap/flow helpers remain owned by
  their source adapters; no footer, statistics, general progress or shared-feed
  workaround is included.
- The adjacent Arachne outline-simplifier tolerance gap remains deferred without
  its own sensitive witness. No additional Clipper2 or offset changes.
- No changes to original seven full-stream comparators, parity ROOT, fixtures,
  feature enums, or Native expected byte fragments.

## Test-first acceptance and execution

The approved immutable input SHA
`7688c5eaac8bb610fae4e92863474c85262d9f83f64737ba17f5574c6b635866`
already activates the source boundary. At the collinear GREEN checkpoint,
first Native overhang object block ordinal3/5 is498 bytes versus API571, with
stable official Native replay and unchanged same-ordinal full-record comparator.
That existing RED is the acceptance witness; **all498 bytes must match**,
including initial travel, E, feeds, M73, fan and role/width records. Keep full API
before/after captures and Native replay proof. Do not replace it with individual
coordinate assertions or masked diagnostics.

1. Read/audit source and write this ARD before implementation.
2. Replace old sampling/result shell with source printing-state contexts and
   child modules; preserve scan, mutation and numeric domains described above.
3. Run the first-overhang witness and independent supported-layer witness; only
   complete-block equality establishes their GREEN. Record unresolved strict
   full-stream dependencies honestly.
4. Run owned directed geometry/tour cases, fmt and strict workspace/all-target
   Clippy. No FULL, seeded/broad sweep or baseline-ledger rerun.

Every new or growing file stays below400 LOC; helper arguments<=5 via actual
source context structs/tuples, nesting<=4, complexity<=15, function lines<=100.
No `#[allow]` or `#[expect]` in any intermediate commit. New implementation
commits retain the required coauthor trailer and belong only to the later batch.
