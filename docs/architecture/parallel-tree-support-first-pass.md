# Parallel polygon checks in tree support first-pass detection

Follow OrcaSlicer v2.5.0 at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Port the independent geometry checks in the first-pass
`TreeSupport::detect_overhangs` loop
(`src/libslic3r/Support/TreeSupport.cpp:899–1080`). Keep Ares's ordered
cutoff decisions, support geometry and warnings.

## Measured candidate

On an earlier revision, a five-run four-thread KSR release profile measured
**192.702 ms** median in the serial first pass, excluding the extrudable
layer preparation that precedes it. KSR exercises support necessity on a
support-disabled object. The measurement includes raw clipping, tail checks
and cantilever checks; it is not a measurement of the parallelizable share.
The baseline for this change is `8f7acd26d6fffd0172d5052ba5e36cf1cf5c70c6`.
Measure the narrower intervals on that baseline before claiming a gain.

## Rewrite boundary

Orca starts one clock before the layer loop (`TreeSupport.cpp:894–899`),
checks cancellation and manual-layer eligibility, and handles the first
layer separately (`:900–919`). Each normal layer offsets the previous
extrudable layer and computes a raw difference (`:921–926,985–987`). It
then reads the clock and raw polygon count (`:989–996`). The loop checks
each current polygon for floating sharp tails (`:997–1052`) and each raw
overhang polygon for cantilevers (`:1055–1077`). The necessity path returns
at `:1083–1084`; later sharp-tail propagation starts at `:1086`. The
layer-indexed `tbb::parallel_for` is the upstream owner of this work, but
Ares does not adopt its cross-layer scheduling: a layer's elapsed-time
sample and flag changes must stay ahead of every later layer's work.

In `crates/ares-core/src/project_slice/support_material/tree/overhangs/first_pass.rs`,
keep `SmartOverhangs::detect_layer` and the physical layer traversal ordered.
Port only the independent polygon checks **within the current layer**:

- For floating sharp tails, map over the current extrudable polygons. Each
  job uses the same expansion, overlap test against the unoffset preceding
  layer, and conditional erosion. Return the classification; append the
  original polygons and zero heights in source order, and set
  `has_sharp_tails` only if at least one qualifies.
- Finish the tail pass before constructing the cantilever base. Map over
  raw overhang polygons using the existing intersection, contour point
  traversal and distance calculation. Keep the strict scaled 3 mm predicate;
  append qualifying polygons and fold the maximum in original order. Do not
  parallelize the min/max calculations inside a polygon or change how points
  and boundaries are enumerated.

Use the existing indexed dispatcher in `crates/ares-core/src/parallel.rs`.
Workers read immutable slices and return indexed results; the caller owns
all writes. Geometry errors retain their current public error mapping and
first-in-order precedence: raw clipping, tails in input order, cantilever
base offset, then cantilevers in input order. WASM and native builds without
`parallel` run the same checks serially in the same order. Do not add a pool,
feature, or geometry algorithm. Do not dispatch singleton or empty ranges
to Rayon.

## Ordered decisions and cancellation

- Keep the object-local time origin immediately after extrudable preparation
  and before the layer traversal. In each eligible non-first layer, install
  raw overhangs, then sample elapsed time exactly once, before either
  optional check. Keep strict `> 30.0` seconds and `> 100` raw polygons, in
  that order. If either trips, keep the raw result, clear `detect_sharp_tails`
  and `remove_small_overhangs`, and skip both checks for that layer. Later
  layers still compute raw overhangs and sample the clock; cantilevers can
  resume if the guard permits, but the tail flag is not restored. Keep
  earlier results and the final warning precedence and strict 6 mm threshold.
- Do not precompute future layers or defer classification until a layer batch
  has completed. Raw clipping for layer N+1 must follow the join and ordered
  merge for layer N. No worker samples the clock, owns a cutoff flag or
  publishes warnings. A faster implementation can reach a real wall-clock
  cutoff on a different layer; the clock, checkpoint and decisions for a
  given elapsed/count sequence must not change. Do not replace the clock
  with a count, deterministic budget or estimated serial duration.
- Keep manual eligibility (`index > enforced_layers`), the distinct strict
  `< enforced_layers` offset rule, first-layer bounding-box checks and
  height recording, coordinate conversions, clipping parameters, empty
  layers and error messages. Leave the preceding extrudable preparation
  (`:880–892`), adjacent belt/tilt geometry, propagation, clustering and
  finalization out of this change.
- Share the same check in `tree/necessity.rs`, `tree/organic/generation.rs::detect`
  and `tree/non_organic/generation.rs::generate`; keep their eligibility and
  caller-visible status intact. Pass the existing cancellation token from the
  necessity stage and both generation callers. Check at first-pass entry,
  before each layer and each polygon job, and after each join before merging.
  Cancellation stays `SliceError::Canceled` and publishes no support,
  warning or sink bytes from the canceled operation. Do not add callbacks or
  move any status callback to a worker.

## Verification

- Test public behavior before changing implementation. Cover support-disabled
  necessity warnings, enabled organic and non-organic support, and a synthetic
  many-island layer that crosses the raw-count cutoff, with later geometry
  that distinguishes the disabled tail flag from continued cantilever
  detection. Preserve complete warning order and confirm the fixtures produce
  real support where enabled. Do not make functional tests wait for the
  30-second timeout or depend on thread scheduling. Cancel through the public
  support phase callback; require cancellation, an empty sink, and identical
  bytes and warnings on a fresh slice.
- Retain all existing full G-code byte witnesses. Compare complete G-code
  and ordered warning contents on clean witness and implementation revisions,
  in fresh processes at Rayon 1/2/7/16 and in a native feature-off build.
  Performance-only preservation witnesses may pass unchanged geometry.
- Run focused necessity and tree-mode parity and paired complete Official and
  Seeded Orca v2.5.0 lanes at exact base and tip. Require unchanged selected
  inventories, zero pass-to-fail and zero selected skips. Run canonical
  workspace nonparity, formatting, diff checks, native all-targets Clippy,
  wasm32 build checks and all-targets Clippy excluding the CLI, and pre-push
  checks.
- Compare matched release builds at one and four threads with fixed affinity,
  five interleaved base/tip pairs per fixture, and byte-identical outputs for
  KSR, Arachne, toolchange and an enabled tree-support fixture. Separate raw
  clipping from the two mapped checks, and report wall and stage medians with
  paired deltas and ranges. A gain in isolated checks is not a wall-time
  claim; report one-thread overhead or an inconclusive result. Keep profiling
  instrumentation out of commits.
