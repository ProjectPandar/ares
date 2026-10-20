# Consistent-grid object bounds: FillRectilinear source slice

Status: SPEC-REVIEW-REQUEST, Native-first discovery only. Base/current main:
`a5eb0f0356057fe9c62ec3319834f82b40a7731a`. No public RED or production edit.
Queue: main is frozen for W1 Voronoi, then W4 ironing; this RR queues behind
those workers and requests neither a merge window nor a FULL run.

## Owning upstream boundary

OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `PrintObject.cpp:95-112` / `Print.hpp:347`: transformed raw model-size,
  scaled/truncated then integer-halved symmetric object box, independent of
  compensated slice surfaces.
- `Fill.cpp:1224,1237,1398,1442`: emitted and projected sparse anchoring
  fillers receive `object()->bounding_box()`.
- `FillRectilinear.cpp:2741-2750`,
  `FillRectilinear::extended_object_bounding_box`: transpose-merge the
  supplied box, then scale about zero by sqrt2.
- `FillRectilinear.cpp:2782-2807`, `fill_surface_by_lines`: for
  `has_consistent_pattern()`, use that extended object frame; sparse
  `align_to_grid` then snaps its minimum to the reference/line-spacing grid.
- `FillRectilinear.cpp:1401-1403,1489,2874`: existing graph traversal uses
  vertical-column parity (`i_vline2 % 2`) to maintain serpentine direction.
  `FillRectilinear.hpp:198,207,219` makes exactly FillZigZag/FillCrossZag/
  FillLockedZag consistent. Their common frame is the owning boundary.

One missing concept, not three new fill features: Rust still substitutes
compensated contour extents for the raw object frame at both consistent-grid
adapters. The PlanePath slice already corrected its separate consumers and
exposed the source-cited raw object helper; it explicitly deferred this
remaining compatibility shell. This slice replaces that shell only for the
three existing consistent-pattern classes. It does not design an Ares
pipeline or rewrite graph traversal, crosszag phase, or locking behavior.

## Source-driven independent Native witness

Keep the approved authored10mm object/profile ZIP bundle unchanged outside
settings. Classic, single object/instance, direction45, sparse20%, multiline1,
layer0.2, top6, ZAA0/0. Only sparse pattern and XY contour compensation vary.
No mesh/geometry search. +0.8 is chosen from the existing source grid-snapping
boundary below, not by searching arbitrary shapes. +0.1 is a below-boundary
control; the same generic compensation on Rectilinear is another control.

All8 inputs use two fresh canonical Native references per mode, CPU12:
32 Native streams total, repeat/cross-mode normalized bytes identical.
Effective Native pattern/compensation/multiline/top/ZAA footers are admitted.

| Pattern | XY compensation | both official and seeded |
|---|---|---|
| ZigZag | 0 | PASS |
| ZigZag | +0.1 | PASS |
| ZigZag | +0.8 | DIVERGENT target |
| CrossZag | 0 | PASS |
| CrossZag | +0.8 | DIVERGENT sibling target |
| LockedZag | 0 | PASS |
| LockedZag | +0.8 | DIVERGENT sibling target |
| Rectilinear | +0.8 | PASS |

The sibling targets are included because they own the exact same upstream
consistent bbox selection; treating observed sibling failures as passing
controls would hide the shared boundary. No pattern outside the three
consistent classes gains new behavior.

First strict deltas, equal in both modes:

| Target | byte / line | Native | Ares | normalized sizes |
|---|---|---|---|---|
| ZigZag+0.8 |14905 /625|`M73 P19 R7`|`G1 X115.125 Y104.875 E.34001`|100682/102683|
| CrossZag+0.8 |2096 /94|`G1 X117.989 Y115.8 E.35644`|`M73 P9 R8`|102291/102439|
| LockedZag+0.8 |1362 /63|`M73 P8 R9`|`G1 Z.2`|135852/136023|

Retain all M73/time and E bytes. Whole-file duration explains why progress
placement may diverge before the first changed Sparse layer; progress is not
the owning implementation boundary. All first five solid-only G0/G1 layers
match, and Rectilinear+0.8 matches completely, controlling general surface
compensation/perimeter behavior.

All targets have positive Sparse geometry differences at physical6:
ZigZag507/507, CrossZag502/518, LockedZag1572/1582 positive extrusion rows.
First positive Sparse endpoints swap XY, for example ZigZag Native
`G1 X113.816 Y114.82 E.05569`, Ares
`G1 X114.82 Y113.816 E.05569`. CrossZag similarly112.684/114.82 and LockedZag
113.26/114.82. This is not an M73-only witness; equal row counts do not imply
parity. Entry moves may retain the preceding Outer-wall TYPE label; they are
not evidence of a separate perimeter-generation defect.

## First inconsistent frame argument

Read-only GDB on actual seeded Native + existing matching DWARF and the
verified mainA5 CLI, ZigZag+0.8 at layer43:

- Native `fill_surface_by_lines` spacing0.40707963705062866,
  density0.20000000298023224; `this->bounding_box` is exactly
  `[-5000000,5000000]` on both axes. `extended_object_bounding_box` receives
  that same raw box. Backtraces identify both projected anchoring and emitted
  calls. No post-compensation object-frame update occurs.
- Ares common rectilinear kernel receives
  `extended_object_min_x=-8202438`, max=8202438 in both its anchoring linear
  adapter and emitted monotonic adapter, with the same spacing/density and
  consistent_pattern=true. This frame derives from the11.6mm contour envelope.
- Native source extension of its observed raw frame derives ±7071067.
  With the source-truncated line spacing2035398, the corresponding grid minima
  derive to Native-8141592 versus old Ares-10176990, one column apart.
  These extended/snapped numbers are source calculations, not claimed GDB
  measurements of return values or post-alignment locals.
- At compensation0 and+0.1, old frames derive ±7071067 and±7212489 but both
  snap to-8141592. This explains why the smaller passing control masked the
  wrong provenance. The old comment inferring contour bounds from the
  observed-8.14mm grid anchor confuses the raw bbox with its later grid snap.

Successful entry probes completed normally, with no debugger mutation,
diagnostic build, nix invocation or gcroot. Earlier source-line breakpoints
that did not hit are retained but not used as proof. Canonical fresh streams
alone supply the strict verdict. A bbox-only port's closure is not yet claimed.

## Exact Rust destination, included and deferred behavior

Reuse the already ported `fill_entities/stars_bounds.rs::{object,extended}`
model-size boundary from the approved PlanePath chain. Do not duplicate its
mesh/ModelPart/float/no-offset-transform/truncate/halve calculations.

Included behavior: the raw model frame for all three consistent classes in
ordinary emission and projected sparse anchoring, followed by the existing
transpose/sqrt2 extension and grid snap. Planned destination:

1. `fill_entities/generate.rs` lazy `extended_object_bounds` closure: obtain
   the raw source object box with owning print-object transform, not
   `plane_path::object_bounding_box(object_slices)`. Keep the existing
   extension formula and all caller parameters. Non-consistent rectilinear/
   monotonic classes continue to ignore these object-frame fields.
2. `prepare_infill/bridge_over_infill/sparse_anchoring/mod.rs`: source the
   object bbox passed to its dispatch from that same owning model/transform,
   rather than projected contour extents. `linear.rs` retains its
   `has_consistent_pattern()` gate, overlap0, layer/direction/phase fields
   and identical extension formula. Existing PlanePath raw-box dispatch and
   all non-linear generators remain unchanged.
3. Remove only contour-bounds helpers/re-exports made unreferenced by these
   two replacements (`fill_entities/plane_path.rs::object_bounding_box` and
   `sparse_anchoring/projected.rs::object_bounding_box`); retain adjacent
   density/PlanePath behavior. Update only newly stale contour-provenance
   comments in `generate.rs`, `flow.rs`, `sparse_anchoring/{mod,linear}.rs`.

No new bbox cache/state/framework/API, capability, validation/unsupported
condition, compensation-specific branch, hardcoded fixture extent or legacy
fallback. No kernel/graph/line-spacing/align-to-grid/column-parity changes.
No crosszag shift/symmetry, LockedZag skin/skeleton density/flow/depth changes,
base offset, clipping/chaining/order, E/flow, bridge-angle, M73/time/cooling,
support/raft, or complex-transform/modifier geometry expansion. Existing
Stars/Grid/Triangles multiline and PlanePath raw bounds must stay unchanged.
The observed classic single-instance witness is not a new support guard.

## Separate RED then bounded GREEN

After independent SPEC approval only, freeze these8 input-only ZIPs with
SPEC SHA pins (external matrix index holds exact identities). Reuse shared
public sparse-fill repeats/producers/strict acceptance and existing bbox
activity checks as applicable; no cloned harness and no weakened old Sparse
invariant. Public load/effective compensation and pattern, fresh Native/API
repeats, raw API/CLI identity, positive Sparse geometry/control activity,
initial solid-motion identity and strict complete Native output are required.
Discovery currently covers CLI, not new public API RED acceptance.

Expected RED per mode:3 targets FAIL,5 controls PASS; existing19 shared tests
remain PASS, including all PlanePath controls. No fixed Native golden,
filter/tolerance, inverted Ares-divergence assertion, ignore/allow/expect or
UnsupportedProjectFeature addition. Separate RED approval precedes production.

GREEN changes only frame provenance at the two adapters and necessary orphan
helper removal. Tests/harness/inputs are frozen; preserve every existing
passing control. **Any strict residual means STOP/report**, not expanding into
parity/traversal, locking, E formatting, generic compensation or M73 fixes.
Exact current-main final hash must pass fmt, locked workspace/all-target clippy
`-D warnings`, core/wasm wasm32, both-mode targeted cases and canonical fresh
FULL CPU20-23/threads4 with0 FAIL/skipped. CPU12 compilation/probes.
Rebase/range-diff and repeat final-hash FULL if main advances. W1 then W4
retain merge priority; this SPEC makes no claim to their window.

## External evidence

Exact discovery input identities to freeze only after SPEC approval:

| Pattern | XY compensation | SHA256 |
|---|---|---|
| zigzag | 0 | `e413f331e008147446c4ef63ddda32d5c8c1b8694eedace21948c2df1829bb76` |
| zigzag | 0.1 | `b9bba1156611846a0e1aef3efc56b8520403cf4307e2129fb1c0cc9b23bfd7fa` |
| zigzag | 0.8 | `2764c00429118e2285aeac74e36242c93fb0f6e3b2841d6576527a11abc3f8c2` |
| rectilinear | 0.8 | `851a673b2a5db02a5f184fcf6c6f5053eb0ca262b78bee68a31f801b34715a03` |
| crosszag | 0 | `1531d7401eb1f9dfa7d99beda680da61ddd37ed124baef6350ebf418fde0ffd8` |
| crosszag | 0.8 | `2f1e3004772244b83ed0aa8df94858f9e2e7d95ec225bfdca471e1a503ccd077` |
| lockedzag | 0 | `1ff705f79f4b97bdc6e8e8d80bb56d7412e7bbc3c4ad2293a2d0af3b4988cf03` |
| lockedzag | 0.8 | `e4d5d41399bdd49f59dee8942b84d0d72802f05651158eb9ec3268a1c76b7d6e` |


`/home/indexyz/ares-evidence/w3-consistent-bounds-{zigzag,crosszag,lockedzag,rect}`
and matching `-seeded` directories contain commands, producer hashes, exact
inputs, effective requests, repeats and complete-output comparisons.
`w3-consistent-bounds-localize/{native-entry,ares}-gdb.log` contains successful
first-argument probes. `w3-consistent-bounds-spec-review/{matrix-audit,
activity-audit,derived-grid-threshold}.json` and external
`w3-consistent-bounds-audit.py` provide repeatable checks. No public fixture,
RED, production edit or FULL is included in this SPEC stage.
