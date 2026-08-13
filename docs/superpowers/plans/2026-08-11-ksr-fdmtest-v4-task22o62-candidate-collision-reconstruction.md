# Task 22O.62 candidate collision reconstruction implementation plan

## Status

Implementation and all runtime/static gates are complete; final independent six-axis implementation review approved unconditionally.

## Objective

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3274-3288`, into exact Rust destination
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/candidate_collision_reconstruction.rs`.
The private operation composes existing source-cited upstream rewrite
dependencies O43 `CandidateSurface`, O48 Flow, O53 anchored construction, and
O61 initial bridge output; it is not a new Ares-owned pipeline.

Included behavior is exact initial-area expansion, prior-completed surface
traversal in future-composer append order, first-collision angle selection and
break, and one conditional O53 reconstruction. Caller-provided
`new_polygons` must be postprocessed at source lines `3292-3297` and appended
at `3304-3305`, never raw O43 candidate geometry; producing that history remains
deferred. O62 behavior freezes exact use of supplied completed-surface polygons;
raw-versus-postprocessed provenance remains a static contract whose integration
mutation is deferred to the future composer.

Direct citations are `Flow.hpp:62-69::scaled_spacing`,
`libslic3r.h:60-94::scale_`, flat polygon `expand`/`intersection` in
`ClipperUtils.hpp/.cpp`, O43 `CandidateSurface`, O53
`construct_anchored_polygon`, and O61 Polyline-to-Line conversion.

Deferred behavior is `PrintObject.cpp:3292-3298` opening/closing and
limiting/total-fill/top-area postprocessing; expansion-area mutation; candidate
append and per-layer replacement through `3304+`; history-producing cluster
composer; prepared successor/lifecycle; second bridge pass; extrusion, motion,
G-code, CLI, and full golden parity.

## Plan

1. **Approve the source boundary**
   - Review the full pinned path/lines, Flow scaling, Clipper flat
     offset/intersection, completed-surface provenance, O43 candidate shape, O53
     call contract, and O61 line conversion.
   - Independently review ADR/spec/plan and repair until approved for RED.

2. **Write behavioral RED tests**
   - Register ordinary private module/test children.
   - Add literal no-collision and first-collision outputs, composer-order/break,
     empty-history and empty-initial cases, exact both-scale arithmetic, exact
     offset input and intersection operand roles, discarded intersection output,
     forwarding, allocation identity, repeatability, and nonmutation snapshots.
   - Add injected operation trace and competing errors plus real Clipper/O53
     success and natural range failures. Confirm focused RED fails for missing
     behavior rather than compilation.

3. **Implement the minimum source slice**
   - Consume O61 owned output and borrow original area/prior candidates.
   - Narrow O53's source-exact scaling helper and O61's Polyline-to-Line helper
     to `pub(super)` and reuse them without duplicating arithmetic.
   - Offset exact initial area once; intersect completed surfaces in composer
     append order with exact subject/clip roles; discard intersection geometry;
     break at first collision; rerun O53 once conditionally; return exact owned
     boundaries/pre-postprocessing area/angle. Preserve no-collision initial
     allocations and boundary allocation on collision; errors return no partial
     owned result.
   - Add no option lookup, validation fallback, postprocess, commit, successor,
     lifecycle, filesystem, platform branch, or output sorting.

4. **Verify discrimination and restoration**
   - Run focused and exact dependency bands.
   - Reversibly mutate each arithmetic, offset input, intersection operand,
     discarded-result, operation, supplied-history use/traversal, ownership,
     forwarding, error, and output-order requirement; require every mutation to
     fail and restore production byte-exact.
   - Run workspace Nextest, strict Clippy, rustfmt, wasm32, x86_64/aarch64
     Windows and macOS checks, diff/LOC/static, clean pinned Orca, and no staged
     files.

5. **Independent final review**
   - Launch a fresh read-only reviewer across completeness, source logic,
     boundaries/errors, architecture/quality, test discrimination, and runtime
     evidence.
   - Return its repair list to the main thread, fix, rerun affected/full gates,
     and re-review until unconditional approval.

## Exit criteria

- Exact pinned collision reconstruction behavior is frozen and implemented.
- No-collision ownership and first-collision composer-append-order semantics are
  exact.
- Tests/mutations discriminate arithmetic, kernels, traversal, forwarding,
  errors, and output order.
- Private/unwired architecture, ordinary modules, and 399-line cap hold.
- Runtime, portability, static, mutation, and independent review gates pass.

## Completion record

Focused 8/8, dependency 2,371/2,371, workspace 6,402/6,402 with two skipped,
strict Clippy, five portability builds, format/static, clean/no-staged gates,
and 26/26 reversible mutation kills pass. Mutation-audit and restored-source
SHA-256 values are `cb35772b...` and `ad7143d6...`; final independent six-axis
review approved unconditionally.
