# Bambu gap-fill interleaving: fixed-order residual collections

Status: fresh public RED qualified on the accepted spiral source port;
awaiting the RED RR before implementation. No interleaving implementation
or GREEN claim belongs to this RED commit.

## Upstream owner and Rust boundary

Port OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`Fill/FillBase.cpp::insert_gap_fill:232-302` and its `out->no_sort` call
at353-356. The upstream comment names Bambu Studio's gap-fill interleaving
for fills printed in a fixed order. `FillRectilinear.hpp:57-74` supplies
the monotonic no-sort premise of the selected witness.

Rust destination: `project_slice/fill_entities/gap_residual.rs`, with a
private insertion module under it if needed for the module-size limit.
`ResidualInput` must carry the existing collection's no-sort decision
before conversion; `fill_entities/monotonic.rs:196-210` currently assigns
that flag only after the residual pass. Preserve other callers' existing
collection flags, including the calibration-order and concentric paths;
do not introduce another configuration policy or force every gap to reverse.

The current `gap_residual.rs:132-135` converts all residual thick polylines
and appends them unconditionally. It is an incomplete compatibility shell
for the cited upstream decision and must be replaced, not kept as a fallback.
No public API, new crate or independent Ares travel planner is proposed.

## Included literal behavior

- Empty fill entities convert gaps directly. Otherwise initialize items
  from the existing entity entry/exit points, preserving their order.
- Visit raw gaps in their current order. Try every insertion slot from
  zero through the current item count. For an open gap consider forward
  before reverse; replace the best candidate only on strict `<`.
- Measure the exact added Euclidean travel in scaled coordinates using
  the upstream subtraction/multiplication/square-root order. Do not use
  a new global-nozzle heuristic, sorting, epsilon tie bias or path rotation.
- Closed-gap candidates use the raw thick polyline's closest point to
  the preceding exit, or following entry for the first slot. The final
  sequence travel comparison recomputes closed entries from its live
  preceding point. Do not rotate the emitted loop here: upstream leaves
  its nearest-nozzle entry behavior to loop extrusion.
- Preserve the original append sequence whenever its total travel is
  `<=` the interleaved sequence's travel. This fallback is an upstream
  tie decision, not a retained Ares implementation.
- Materialize the chosen sequence without reordering existing entities.
  Reverse a selected raw `ThickPolyline` before `variable_width` converts
  it. Reuse the existing variable-width conversion/role implementation;
  do not reverse already-converted extrusions or patch WIDTH/feed/E text.
- At353-356 only no-sort collections select insertion; sortable collections
  retain the upstream direct variable-width append branch.

Adjacent residual-area construction, medial-axis geometry, length filters,
Flow math, classic `PerimeterGenerator` gap production, global entity
chaining and G-code formatting are not new rewrite targets. A Gap-infill
label alone does not prove a shared producer or every raw operand.
Any newly exposed earlier boundary is reported, not silently absorbed.

## Real public RED and strict acceptance

Base: accepted literal spiral source port
`b83400af103301442d570833842287bda700e964`, explicitly NON-GREEN.
Use the unchanged velocity project, SHA256
`d8f74deff1274800b04f021ba812c5fb84c1629e45981653218963e2cc3a1e37`,
at `tests/parity/bambu-gap-fill-interleaving/velocity-limit.3mf`.
Its authored settings and Native footer are monotonic internal-solid fill,
gap target everywhere, Classic walls, relative E and disabled arc fitting.
The preceding spiral now matches. Retained evidence identifies Native
entry `G1 X105.806 Y113.63 F30000` at436 versus API/CLI
`G1 X106.37 Y114.194 F30000` at435, followed by Gap infill extruding to
the opposite endpoint. These are observations, not test coordinate goldens.

The dedicated option-coverage test runs two fresh stable ordinary Native
exports, `ares_core::slice_project_plate_validated` and CLI `slice` once.
The full ordered comparator keeps M73, XYZ/E, travel/feed, TYPE/WIDTH,
wipe/retract, labels, terminals, whitespace and trailing bytes; only the
existing canonical generator line is normalized. No partial-stream oracle,
count pin, tolerance, cyclic normalization or filtered acceptance.

M73 remains separately owned by the Klipper estimator rewrite. An external
M73-only diagnostic may locate this first remaining endpoint difference,
with complete exports, removed records and original-line maps retained.
It does not alter the strict failing test or establish a timing-only cause.
Any estimator prerequisite stack must be explicitly approved; no permanent
mask or independent estimator/cooling patch is part of interleaving.

## Fresh qualification

One selected ordinary public run completed with the expected strict
failure: two fresh stable Native exports, one validated API export and
one CLI export, raw API/CLI equal. The first complete-output difference
remains byte583, line19, column9: Native `M73 P0 R8` versus API/CLI
`M73 P0 R7`. This remains a failure under the unchanged comparator.

The separately retained external diagnostic's first non-M73 difference
is the selected Gap infill entry at Native436/API435, with the observations
above and extrusion back to the other stream's starting endpoint. Its
removed records, original-line maps and full unfiltered contexts are in
the external evidence bundle. No other command or byte class was removed.
The public RED contains no fixed endpoint/count assertion.

Fmt and strict workspace/all-targets Clippy pass. The exact parent-plus-
staged execution tree and frozen binary are retained externally. Only
this result note changed after execution; no postcommit rerun is claimed.
Raw insertion operands/closed-gap choices remain unobserved, and this
RED does not turn the printed reversal into a general producer proof.

## Exit and deferrals

Submit ARD plus the fresh public RED RR before implementation. This RED
phase has one selected run: Native2/API1/CLI1, no retry, observer, operand
dump, input variant or family run. Failed attempts remain immutable.
The exact execution tree and repeatable artifacts belong to the external
evidence bundle, not committed Native G-code.

A later authorized literal port requires the complete public output to
pass, M73 included, and a frozen before/after fill-family inventory with
zero prior-PASS to FAIL. Preserve any remaining failure and report its
owning source instead of declaring local direction-only GREEN.
After the gap-direction port, rerun E5 as planned: if its feed mismatch
vanishes, no cooling slice; if it persists, request separate attribution.
No E5 run or cooling change occurs in this RED. Core stays platform-neutral
on browser WASM, Windows, macOS and Linux without filesystem/UI behavior.
