# Lateral sparse multiline overlap removal

Status: SPEC proposal; docs only. No Native discovery or RED has run for this slice.
Base: `3ee8f272c6d8e2db22dde9003a01afeb7201e36a` (current main).
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Owning upstream boundary

`src/libslic3r/Fill/FillRectilinear.cpp:2996-3045`,
`FillRectilinear::fill_surface_by_multilines`, generates the sweep pool,
performs multiline offset, and intersects with the contracted surface.
At **3032-3033**, exclusively for `ipLateralLattice` or
`ipLateralHoneycomb` with `params.multiline > 1`, it calls
`remove_overlapped(fill_lines, line_width)` **before** connection at 3035-3042.
`line_width` at 3009 is `coord_t(scale_(this->spacing))`.

The complete owning helper is `remove_overlapped`, 2960-2994:

- `tolerance = coord_t(0.75 * line_width)`;
- each line's representative point is the integer midpoint of its first
  and last points (2965-2968), not its arc-length midpoint;
- visit input lines in order and compare against already retained lines;
- the early skip at 2979-2981 uses **both** coordinate differences greater
  than tolerance; the final rejection uses Euclidean distance **strictly
  less than** tolerance (2983), with `Point.hpp:259` casting the integer
  difference to double before the norm;
- retain the first qualifying line, preserving order. No endpoint reversal,
  post-sort, geometric union, or general near-line deduplication is implied.

LateralLattice enters this routine at 3449-3463 with two parallel shifted
families. LateralHoneycomb enters at 3550-3607, choosing one/two families
by height; its two-family calls are 3591-3599. Odd-layer reversal remains
in each caller after the shared routine returns.

## Ares destination and static discrepancy

`crates/ares-core/src/fill/multiline.rs:105-126` currently performs multiline
offset, intersection, and connection consecutively. It has no counterpart
of 3032-3033. Both `project_slice/fill_entities/lateral_lattice.rs::polylines`
and `lateral_honeycomb.rs::polylines` call this shared entry. The pattern's
identity is currently absent from `MultilineFillParams`.

Proposed port: a private helper under `fill/multiline/` with the exact
retained-prefix algorithm, plus minimal typed caller selection for these
**two** patterns only and only when count > 1. Preserve the current shared
entry behavior for all other callers; do not infer the pattern from sweep
count or angle. Exact interface and any constructor-only adaptations will
be reported before changes to existing tests. No public API/configuration
option or second fill generation loop is needed.

This is a source omission, **not yet a demonstrated output defect**. An
earlier multiline union may already eliminate every relevant near pair
in a bounded fixture. Native-first discovery must establish reachability.
The current Ares dispatch is a compatibility shell for the cited upstream
routine, not an independently designed cleanup pipeline.

## Included and deferred

Include only sparse LateralLattice/LateralHoneycomb count > 1, post-clip
pre-connect overlap removal. Preserve spacing, density, sweep arithmetic,
frame, existing multiline offset, connection, extrusion flow, and ordering.

Defer regular Honeycomb and TPMS-D anchoring (W3), SupportCubic E (W3),
Adaptive/touching, ironing and support ironing (W5), all other pattern
algorithms, shared union changes, and numerical fixes outside this helper.
The ordinary Honeycomb pattern is distinct from LateralHoneycomb. This
proposal does not claim any ownership of its anchoring or an ironing arm.
If discovery identifies a different first unequal intermediate, STOP and
report; do not expand production scope to make the candidate pass.

## Native-first discovery and RED plan

After approval, use an authored centered rectangular prism, one object,
one region/no modifier, one filament/no tower, by-layer, sparse density 30%,
0.2 mm layers and explicit 0.45 mm infill width. Keep ordinary walls and
solid caps. Only input 3MF/settings and test source enter Git.

Bounded requests (each official and seeded, CPU11, jobs1/threads2, fresh
cache/artifact root per run, canonical environment):

| Requests | Pattern and settings | Purpose |
| --- | --- | --- |
| L2/L3 | LateralLattice, count 2/3, angles -45/+45 | Candidate near-family overlap |
| LN2/LN3 | LateralLattice, count 2/3, angles 40/45 | Candidate closely shifted families |
| H2/H3 | LateralHoneycomb, count 2/3, overhang angle 45 | Candidate double-family phase |
| L1/H1 | Same lateral patterns, count 1 | Branch-excluded controls |
| C2/Q2 | Cubic/QuarterCubic, count 2 | Shared entry, excluded-pattern controls |

First prove Native admission and actual sparse-path consumption. Do not
replace rejected settings silently. Record full generated files externally,
first byte/line/physical layer/role, and strict whole-byte results, using
only the existing generated-by normalization. If all six candidates match,
report **NO RED FOUND / STOP**, with no production edit. Native timeout is
inconclusive, never an accepted witness.

For any true witness, localize the first unequal stage: pre-intersection
pool, post-intersection pool, retained-prefix output, then connected paths.
Require evidence that upstream removes a line here and Ares retains it;
if stages already differ before this helper, report and stop for a ruling.
Use only the existing Native binary if read-only gdb is necessary.

Commit the public RED separately and request review before GREEN. Exercise
`slice_project_plate_validated` and CLI with runtime `OrcaRunner` reference;
assert complete normalized byte equality and API == CLI. Missing Native
uses shared required/optional policy. No guards, ignore, weakened assertions,
Native artifacts, or fabricated expected geometry. All existing controls
must pass first and remain raw-byte unchanged after any port.

## Acceptance after a separately approved RED

Exact final clean hash: frozen RED cases and four controls in both modes;
non-RED shared multiline callers raw-byte unchanged; full option_coverage
(including non-Native tests) in both modes with visible skips reported;
strict clippy, rustfmt, wasm32, files <= 400 LOC. Any residual is STOP,
not GREEN. Full/workspace remain coordinator-owned and require allocation.
Rebase onto stack-17's merged main before runtime work if it has advanced.
