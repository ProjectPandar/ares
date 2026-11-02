# Cubic multiline trapezoidal rewrite

## Source and destination

Rewrite Orca beta `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `FillRectilinear.cpp:3603–3618`, `FillCubic::fill_surface`: count greater
  than one dispatches `fill_surface_trapezoidal(..., 3)` and returns; count one
  retains the three z-shifted `fill_surface_by_multilines` sweeps.
- `FillRectilinear.cpp:3057–3098`, `cubic_upper_level`: the five-line lower
  envelope, crossing candidates, cut, sorting/unique and collinear removal.
- `FillRectilinear.cpp:3100+`, common trapezoidal frame, scaled thickness,
  period, rotated surface coverage and thickness-adjusted layer rotation.
- `FillRectilinear.cpp:3379–3411`, mode 3: z phase, two upper levels,
  staggered rows and source `std::round` conversion.
- `FillRectilinear.cpp:3420–3485`, common frame restoration and tail. Reuse
  the existing `fill/trapezoidal_tail.rs` port rather than fork it.

Rust destinations are `project_slice/fill_entities/cubic.rs` for dispatch and
layer context, and `fill/cubic_trapezoidal.rs` with separate row/envelope
modules as necessary. Existing cubic sweeps are a temporary compatibility
shell for count greater than one and must stop serving that route. Count one
is the actual upstream path, not a legacy fallback.

This slice starts at `9579c373f`, which supplies the shared trapezoidal tail.
Do not change the approved parent or the independently approved bridge port.

## Non-negotiable arithmetic and ownership

Preserve operation order: truncate scaled spacing before multiplying by
count; evaluate period in double and truncate; evaluate z phase and positive
remainders in double. Keep the envelope's five lines and candidate insertion
order, strict comparisons and first-match behavior. Sort candidate x values,
remove adjacent values with distance below one, and remove collinear points
using source EPSILON. Preserve the row loop bounds, parity-dependent half
period, first row-point versus subsequent tile indexing, integer `j * period`
before conversion, and round coordinates before integer conversion.

Use the surface in the rotated frame for local coverage, translating by the
source reference and rotating by the negative layer phase before computing
bounds and expanding by one period. Restore the layer phase and reference
before the shared tail. Do not substitute the object bounds for this coverage.
Do not change count-one sweep arithmetic, connection, flow, writer, or shared
geometry wrappers. If a primitive is implicated, coordinate its owner instead
of creating a local replacement.

## Included and deferred behavior

Include cubic count greater than one with zero sparse infill smoothing, the
mode-3 envelope and rows, common frame and the existing shared tail. Frozen
fixtures have `sparse_infill_smooth_factor=0%`, so the upstream smoother is a
source-defined no-op here. Nonzero smoothing remains a separately owned source
slice; do not add a stub, configuration projection, or claim coverage of it.
Quarter-cubic, adaptive cubic, bridge preparation, other infill patterns and
configuration footer serialization are not part of this port.

## RED and GREEN contract

Before production changes, freeze independent exported C2, C3 and HC2 inputs
with SHA assertions. Keep three public Native-double-replay, API and CLI
complete-output tests enabled under the unchanged ordered comparator. Add a
count-one control whose explicit assertion boundary is exact ordered raw
`G0/G1/G2/G3` records plus `M73`, with byte-equal API and CLI output. Record its
complete-output comparison separately; do not represent the control as full
parity while the footer prerequisite is missing.

The authorized GREEN ruling is the same as the parent's: the three target
raw motion-plus-M73 sequences must exactly match Native and API must equal
CLI byte for byte. Exhaustive whole-output differences may contain only the
permitted generator line and these missing configuration records:

- `; compatible_process_expression_group = ""`
- `; pellet_flow_coefficient = 0.4157`

The three complete-output tests remain failing, enabled and unmodified until
the configuration owner's prerequisite lands. No overlay, ignored test,
normalization, reordered records or comparator weakening is allowed. Full
GREEN qualification belongs to the later integrated prerequisite batch.
Record this distinction in the port commit and this ARD. Require count-one
raw-control PASS before and after, same-inventory fill breadth with zero
PASS-to-FAIL, rustfmt and all-target Clippy with warnings denied. Intermediate
Native observation budget remains zero; ordinary public replay is unchanged.
