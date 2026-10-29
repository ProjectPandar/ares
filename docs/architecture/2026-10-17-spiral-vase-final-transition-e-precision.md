# SpiralVase final transition E precision

Status: SPEC proposal, docs only; no RED or production change.
Owner: worker6. Base: `origin/main` 41f2c90f7.

## Boundary

This prerequisite slice ports the numeric and formatting boundary of the
SpiralVase final `transition_out` pass. Upstream `GCode.cpp:4596-4612`
selects a body layer only when spiral mode is active, the layer is beyond the
bottom/skirt region, and the layer has at most one perimeter and no fill.
`SpiralVase.cpp:123-160` then stores `len`, the XY distance, progress and
`finishing_e_factor` as `float`; it parses each input `line.e()` as the already
formatted five-digit float, accumulates `len += dist_XY`, multiplies that float
E by the float finishing factor, and writes E with five decimal places. The
input E is therefore the previous G-code text value, rather than an internal
unrounded double.

The Ares compatibility shell is
`crates/ares-core/src/gcode_spiral_vase/project.rs:67-72,109-132`:
`transition_out` is selected for the final relative-E layer, `length` is
currently accumulated at `:109`, progress is formed at `:110-113`, and the
finishing factor and tapered E are applied at `:121-132`. The implementation
slice is limited to reproducing the upstream float input, float accumulation
order, float factor, and five-digit E formatting at this boundary. The
corresponding `transition_in` arithmetic at `:67-72,116-119` shares the same
parsed-E/progress/factor pattern; it is included only if the RED demonstrates
the same first-difference class, with its own witness. No geometry, path
ordering, layer scheduling, or ordinary extrusion calculation is included.

## Inputs and RED

The primary input is the existing two-island `two-islands-on.3mf` fixture
(`tests/parity/arachne-alternate-extra-wall/two-islands-on.3mf`, source SHA
`6fdbe58b898d27de0c39c67b679059ed0e31f4eb8ec2978b0566eaacf5ad6f8c`),
with only `spiral_mode=1`; its deterministic derived archive is already
manifested as SHA
`60154fac5cadb3cb49a1e3589317434a153856d5967edfa8734b885d8bce7382` in the
layer-range evidence. The RED will generate this Native input at runtime and
assert strict whole-file equality through both validated API and CLI.

The control uses the same derived archive and mesh with only `spiral_mode=0`;
its derived-input SHA will be recorded in the RED manifest. It must remain
whole-file equal in both official and seeded modes. The existing layer-range
no-range spiral E-only failure is the target; the regular control's PASS
establishes that this is not the ordinary E path. A failing control stops the
slice rather than weakening its assertion.

RED is a dedicated runtime-Native CLI E2E module with one test per target and
control, strict comparator unchanged, no committed Native output, no ignore,
and no tolerance. Runs are pinned to CPU11 with fresh roots and caches. The
first differing line and the parsed float E, accumulated float length, factor,
and formatted five-digit output will be retained in the evidence report.

## Deferred

Layer-config-range admission and its spiral/Regular versus
PositiveLargestContour scheduling remain deferred to the paused prerequisite
slice. Geometry producers, `SpiralVase.cpp` path gating beyond the cited
boundary, absolute-E behavior, and unrelated generic extrusion rounding are
out of scope. No production edit is authorized before a public RED is
reviewed.
