# Proposed slice: FillGrid multiline trapezoidal mode0

Status: approved boundary; public Native-backed RED accepted, GREEN in validation.
Base: `296b060186fc90590e23238145a27fa4805c59e7`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Selection and ownership

Choose the deferred Grid multiline dispatch finding from
[the multiline union boundary](fillbase-multiline-clipper2-union.md), rather
than reopening the closed stock matrix. Historical G2 is a real whole-output
DIVERGENT, not a guard or an attributable union RED. Its old observation is
Native/Ares line18 `M73 P0 R45` / `M73 P0 R46`, retained at
`/tmp/worker6-union-numerics-trial.yr9Ms7/control-audit.json`. It is not fresh
acceptance evidence for this base.

Offline inbox audit found no other active claim to `FillGrid::fill_surface`
mode0 or its Grid generator. Previous worker6 notices explicitly deferred it
until the shared multiline union and Stars slices merged; both are now main.
Audit: `/tmp/worker6-grid-mode0-owner-audit.json`.

Do not touch W2 bridge-anchor C2 projection/admission, W3 centering C2/model
assembly, W4 counterbore, W5 interface_shells, or web/CI. The one adjacent
W2-area target is the **Grid arm only** in sparse_anchoring/arms.rs, required
because upstream calls the same FillGrid generator for anchoring and emission.
This is not authorization to alter transaction/anchor_projection.rs, bridge
classification, loaded-config admission or any other pattern arm. Ask the
coordinator before implementation if an active owner also needs that Grid arm.

Per-object skirts were considered but not selected: their new per-instance
emission hook would intersect the ongoing centering consumer work.

## Upstream boundary

| Source | Included behavior |
| --- | --- |
| `PrintConfig.cpp:2996-3002` | fill_multiline integer1..10, default1. Use existing typed option; no schema/inventory change. |
| `Fill/Fill.cpp:926` | Multiple lines belong to internal sparse infill; other roles retain count1. |
| `Fill/FillRectilinear.hpp:67-78` | FillGrid derives FillRectilinear; `_layer_angle` returns0. |
| `Fill/FillRectilinear.cpp:3422-3446` | Count>1 invokes fill_surface_trapezoidal mode0; count1 retains two sweep families. Odd-layer path reversal is confined to the count1 branch. |
| `Fill/FillRectilinear.cpp:3048-3085` | Shared setup: truncate scaled spacing before multiplication by count; mode0 period truncates `2*d1/density*sqrt(2)`; float infill direction then double pi/4; rotate about source reference; read extended object bbox and effective infill layer id. |
| `Fill/FillRectilinear.cpp:3087-3162` | Grid rows: d2=`coord_t(0.5*period-d1)`, minimum bbox grid alignment, five points per cell, vertically mirrored alternate half-period rows; transpose XY on odd effective infill layers. Preserve casts, integer division, row/point order. |
| `Fill/FillRectilinear.cpp:3344-3384` | Existing multiline expansion, half-spacing domain contraction and original-domain fallback, ordered clipping, short-segment removal, connect-or-chain, and inverse rotation. No coordinate sorting after generation. |
| `Fill/FillBase.cpp:275-319,1569-1578` | Source float direction/reference and PrintObject extended bbox transpose/scale; use the object box, not a per-slice box. |
| `Fill/Fill.cpp:1224-1237` | Layer::make_fills installs the object bbox before normal fill generation. |
| `Fill/Fill.cpp:1394-1442,1477-1498` | generate_sparse_infill_polylines_for_anchoring installs the same bbox and multiline params, then calls FillGrid::fill_surface. Both consumers must agree. |

The existing round/Positive union and its Clipper2 numeric policy were ported
in the earlier prerequisite. This proposal does not reopen their numerical,
event-order, cleanup or union semantics.

## Rust destination and temporary scaffold

Current `project_slice/fill_entities/grid.rs::grid_polylines_inner` always
uses ordinary `fill::multiline::fill_surface` with the two Grid sweeps,
including count>1, and reverses odd-layer paths for both counts. It is a
compatibility scaffold for the count1 source branch, not a mode0 port.
The Grid arm in
`project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs`
also uses the ordinary two-sweep generator for every count.

Proposed changes after approval and fresh RED:

- New private `fill/grid_trapezoidal.rs` with a separate rows file if needed;
  each file <=400 LOC. Port mode0 setup and rows literally. No new crate,
  public API, general pattern framework or heuristic generator.
- Reuse the existing multiline_offset Positive helper and ordered clipping/
  connection primitives. If extracting a shared post-row tail from
  `fill/stars_trapezoidal.rs` avoids duplicating the source tail, preserve its
  expressions/order exactly and separately prove Stars raw bytes unchanged.
  Do not change the Stars mode2 row builder.
- `fill_entities/grid.rs` + `fill_entities/generate.rs`: dispatch only Grid
  count>1 to mode0, retain count1 expressions and its odd-layer reversal.
- `fill_entities.rs` and `fill_entities/stars_bounds.rs`: reuse the existing
  once-per-object box producer for Grid count>1 as well as Stars count>1;
  use a source-owned name if its ownership widens. Other patterns/count1
  keep their existing path and do not compute this additional box.
- Sparse anchoring: only the Grid arm in arms.rs and the existing bbox
  eligibility predicate in transaction/candidate_expansion/layers.rs:65-78.
  The coordinator approved this actual call-site correction; sparse_anchoring/mod.rs
  already threads the box and layer id and remains unchanged. Feed the actual current
  object, layer id and representative thickness_layers to the same mode0
  generator as emission. Do not recalculate a regional/surface bbox or use
  regions[0]. Preserve the count1 Grid arm.

Any additional dependency, bbox widening or unrelated existing output change
must be reported before extending the production scope. No new Unsupported
rejection, silent skip, fallback scaffold or comparator adjustment is allowed.

## Included and deferred

Included acceptance domain: one printable object/instance, one model part,
single0.4mm nozzle/PLA, by_layer, uniform0.2mm layers, classic walls, positive
anchors, Grid sparse infill. Rectangular prism and vertical through-hole
prism exercise ordinary and clipped domains; solid/top/bottom fill stays on
existing rectilinear count1. Finite count1/2/3/10, density20/30%, angle0/45,
and one count2 infill-combination witness cover layer parity and division by
thickness_layers.

Deferred: Rectilinear's separate R2 main-dispatch finding; Triangles mode1;
other pattern ports; changes to minuscule/negative anchor admission;
new bridge projection or interface-shell behavior; ZAA, arbitrary transform/
shared mesh centering, modifiers/ranges, multi-object/tool, SEMM/towers,
support/raft/brim, browser/CLI admission and producer/CI changes. Existing
behavior is retained outside the selected dispatch; deferred is not a new
runtime guard and not a claim of global parity.

## Public E2E RED before GREEN

Proposed CLI module: orca_parity/runner/grid_multiline_mode0.rs, with only
its registration in runner.rs. Use existing OrcaRunner/required-optional
policy and cache; no private geometry-only tests or second cache mechanism.
Commit only authored input geometry/3MFs, generator and provenance. Historical
G2 requests derive from tracked `tests/parity/multiline-union/prism.stl`;
its SHA256 is `1d50f639d940e2a95c324b8f7ef1a33ba9bcf7e83706ba3cf0669f67f6feae7e`.
Use the existing preset/fixture adapter, record request/effective option
agreement from runtime export, and freeze the final input hashes before RED.

Nine finite controls: G1 count1, G2 count2, G3 count3, G10 count10 (all30%,45);
G2-A0 (angle0); G2-D20 (20%); H1/H2 (same through-hole geometry, count1/2);
G2-C2 (count2 with actual infill combination2). Defaults keep tower/support/
brim/skirt disabled and templates fixed. No arrange-based change to hide a
mismatch. Assert G2-C2 really reaches combined sparse layers; if it does not,
report before altering or dropping that witness.

For each official and seeded run, use fresh cache/root, taskset CPU11, explicit
jobs2/threads2, and the current canonical producers. Native success/output
must exist before asserting public validated Ares API and CLI success. Compare
complete ordered bytes/line count with the unchanged generated-by-only
comparator, layer Z/HEIGHT and meaningful sparse extrusion presence. Retain
whole first difference plus first sparse-layer diagnostic externally; M73 is
never removed from acceptance. Count1 controls must be whole-output PASS;
active controls must show Native sparse motion changed, not only CONFIG_BLOCK.

A separate RED commit must record a main296 failure against an accepted Native
stream in both modes. Do not reuse the historical G2 observation as current
RED. If all nine are already equal, report NO RED FOUND and stop. Any Native
refusal, request rewriting or unrelated deterministic Ares difference is a
blocking observation, not a reason to weaken controls or change context.
GREEN is a separate production commit. Existing assertion reversals/retirements
require advance approval; this proposal requests none.

## Exit and coordination

After fresh finite GREEN: complete option_coverage group in both modes,
including non-Native tests, existing Stars seven + union ten controls both
modes, and existing Grid count1 controls. Audit raw bytes for every non-RED
caller; any change stops for coordinator review. Strict Clippy, fmt, wasm32,
and default-environment workspace must pass in allocated slots. Files stay
<=400 LOC, no allow/expect/ignore/probes or Native output commits.

Request explicit FULL-GO only after the engineering/directed prerequisites;
canonical candidate-only official+seeded full uses the unchanged control script,
fresh per-leg resources, jobs2/threads2 and exact START/END inbox markers.
Report the exact reviewed HEAD and all evidence; merge only after APPROVED.
This document defines the approved boundary, not final acceptance evidence.
