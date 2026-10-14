# D-A: remaining local projected-anchor admission

Status: SPEC-REVIEW-REQUEST; source reading/docs only. No Native probe run.
Docs branch base: main 5a9687ac020962f254b6262f2184f3345025c70f.
Approved D-M dcbd3a084 is frozen awaiting stacked full/CI in its own worktree.
This proposal narrows the remaining half of the earlier D SPEC99a7b9373.
Before implementation, carry onto the main containing D-M or a base explicitly
assigned by the coordinator; this docs branch does not duplicate its repair.

## Upstream owner and unproved runtime discrepancy

Orca source8500fcdccaa10b5099ac20d252af3a7c560046f1, paths under src/libslic3r:

| File/function | Relevant behavior |
|---|---|
| PrintConfig.cpp::PrintConfigDef::init_fff_params:3137-3163 | infill_anchor is FloatOrPercent with no dedicated numeric min/max; max_literal1000 is a distinct GUI hint. |
| Config.hpp::ConfigOptionFloatOrPercent:1259-1264,1303-1309 | Double-valued carrier, percentage flag and stream extraction; acceptance must still be checked through the public application. |
| Config.hpp:2475-2482; PrintConfig.cpp:10521-10562 | Global validation uses bounds defaulting to +/-FLT_MAX; a local override cannot be assumed legal from schema presence. |
| PrintObject.cpp::region_config_from_model_volume:3664-3709 | Local region merge; final normalization does not add anchor sign/finiteness checks. |
| Fill/Fill.cpp::group_fills:983-989 | Narrows anchor carrier to float before percentage projection and min(anchor, maximum); does not reject a non-finite projected anchor here. |
| Fill/Fill.cpp::Layer::generate_sparse_infill_polylines_for_anchoring:1477-1482 | Passes projected lengths into FillParams. |
| Fill/FillBase.cpp::Fill::connect_infill:1580-1587; connect_base_support:2247-2252 | Nonnegative-length assertions do not establish shipped release-oracle acceptance/completion of extreme inputs. |

Ares transaction/anchor_projection.rs still rejects !anchor.is_finite()
with UnsupportedProjectFeature("infill_anchor"). Its existing projected.rs
narrows/projects/mins in the upstream order. Finite local -1e39 or -1e39%
can remain negative infinity after min with an ordinary finite maximum.
A positive overflow carrier instead clamps to finite max and cannot witness
this guard. D-M's accepted positive maximum cases prove nothing about D-A.
This is a source-level candidate, not a claimed admitted Native discrepancy.

## Rust boundary / included and deferred

Primary production boundary: crates/ares-core/src/project_slice/prepare_infill/
bridge_over_infill/transaction/anchor_projection.rs, only the remaining
projected-anchor rejection and the resulting unused tuple calculation/import
and private unsupported helper. The projection helper used by real Fill stays.
This validation is temporary Ares scaffolding around the upstream FillParams
boundary; it must not become a new numeric policy.

Included only after an admitted public RED: finite local negative carriers
with ordinary global options and a finite local/effective maximum. Preserve
projection/min order, real sparse generation and all other admission behavior.
Initial repair would remove only that rejection; if insufficient, STOP and
request a source-cited repair PLAN before touching another consumer.

Deferred: FloatOrPercent/schema changes, global extreme acceptance, new
clamping/defaults, Fill algorithms, max semantics beyond D-M, W6 linear.rs,
W4 counterbore, rotation/Grid/arms and region/layer-range redesign.
No inf/NaN literal strings, fallback, silently skipped geometry or comparator
change can substitute for an accepted finite public input.

## Native-first preflight and independent public RED

After approval and a directed window, use the admitted solid-ceiling family
with sparse rectilinear20%, ordinary global anchors, finite max=20, and local
object/model-part anchor=-1e39. Probe -1e39% independently. Pair absolute and
percentage cases with finite local -1 / -50% controls already supported by
the negative-anchor port. Check exact local XML and untouched ZIP members;
ordinary global settings must remain ordinary.

First require successful Native exit, full footer and active Internal Bridge
and Sparse infill extrusion, then repeat with fresh cache/root to establish
complete reproducible output under the existing generator/date rules. Retain
original inputs, streams and exit logs. If either oracle rejects/crashes,
times out, lacks active roles or cannot establish repeatability, STOP and
report evidence; do not infer unreachable behavior or invent numeric policy.

Only an admitted completed witness may become RED. The public E2E test must
require existing strict whole-stream Native/API/CLI parity, recording the exact
Ares failure rather than expecting Unsupported as a passing contract. Commit
RED separately and report before repair. Reuse the existing solid-ceiling
fixture through its owning module; no duplicate fixture module or new generic
fixture framework. Intended test file: runner/compatible_regions/
bridge_anchor_length_projection.rs, child of the existing solid-pattern tests.

After RED review/repair approval: run finite controls/new witnesses and existing
bridge/anchor regressions in both modes, prove ordinary raw controls unchanged,
and pass strict/fmt/wasm. Files <=400 lines. Official CPU13 / seeded CPU14,
outer taskset on cargo/nextest/Native, jobs2/threads2, fresh root/cache/target.
Full/workspace only with coordinator GO; coordinator owns CI/merge/push.
No code, tests, builds or Native runs are included in this docs-only proposal.
