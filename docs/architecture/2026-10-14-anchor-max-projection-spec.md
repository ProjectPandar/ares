# D-M: local infill_anchor_max projection admission

Docs only, no Native admission claimed. Against main
5a9687ac020962f254b6262f2184f3345025c70f. This narrows the earlier D proposal in
bridge-anchor-float-projection-admission.md to maximum projection only.
The C2-T/C2-S and option-matrix slices are already merged.

## Source boundary and observed remaining mismatch candidate

Upstream source: OrcaSlicer 8500fcdccaa10b5099ac20d252af3a7c560046f1,
paths below relative to src/libslic3r.

| Owner | Source behavior |
|---|---|
| PrintConfigDef::init_fff_params, PrintConfig.cpp:3137-3186 | infill_anchor/max are FloatOrPercent; max_literal=1000 is a GUI literal hint, separate from numeric bounds. |
| ConfigOptionDef, Config.hpp:2475-2482; numeric validation, PrintConfig.cpp:10521-10562 | Default bounds are +/-FLT_MAX; global loaded-config validation cannot prove acceptance of larger local values. |
| region_config_from_model_volume, PrintObject.cpp:3664-3709 | Merges object/part/range options; final normalization clamps density/filaments/fuzzy settings, without an anchor-finiteness rule. |
| group_fills, Fill/Fill.cpp:978-989 | Sparse anchor carriers narrow to float before percent projection; maximum is projected, then anchor=min(anchor, maximum). No finite-maximum rejection here. |
| Layer::generate_sparse_infill_polylines_for_anchoring, Fill/Fill.cpp:1477-1482 | Passes projected anchor and maximum to FillParams. |
| FillParams::dont_connect, Fill/FillBase.hpp:52 | Selects simple connection when maximum<0.05, rather than defining a new admissibility range. |

At this exact main, transaction/anchor_projection.rs:28-30 still returns
UnsupportedProjectFeature("infill_anchor_max") for a non-finite projected
maximum. sparse_anchoring/projected.rs already uses the upstream float
narrowing/projection order. Ares FloatOrPercent accepts finite f64 carriers
(options/config_types/scalar.rs:226-309); a finite carrier can overflow f32.
This is a source-level admission discrepancy candidate, not an established
Native output difference. The global bound above may reject a probe earlier.

## Rust destination and limits

Primary production boundary:
crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/
transaction/anchor_projection.rs, only the maximum-projection rejection.

Included: one finite local carrier reaching that exact check, if Native
accepts it and produces repeatable complete output; the existing float
conversion, percent projection, min ordering and real Fill consumers remain.
This transaction validation is temporary Ares admission scaffolding around
the cited upstream FillParams boundary, not an independent numeric policy.

Deferred: the adjacent infill_anchor finiteness gate (D-A), any scalar/schema
changes, saturation/clamping, Fill algorithm rewrites, arbitrary numeric
admission, counterbore, and changes to shared rotation/Grid/arm code.
If deleting the selected check exposes a Fill mismatch, report it and request
a separate source-cited repair PLAN; do not expand the implementation.

## Public RED plan

1. Use the existing positive-density rectilinear solid-ceiling fixture family
   (infill_anchor_negative.rs / infill_anchor_dont_connect.rs). Keep global
   anchors ordinary. Inject infill_anchor_max="1e39" in object metadata;
   probe model-part metadata separately. Test "1e39%" as a distinct input.
   These are candidate inputs, not predeclared legal Native cases.
2. Before treating a result as RED, establish exact applied local value,
   finite input carrier, completed Native stream/footer, active internal
   bridge and sparse roles, and an ordinary local max=20 paired control.
   Preserve original inputs, complete outputs, exit/log and existing receipts
   externally. Use the canonical official/seeded configurations.
3. For an admitted witness, write the public Native/API/CLI whole-output
   parity assertion first and commit RED separately. Expected baseline Ares
   failure is the exact infill_anchor_max unsupported result. Never call
   Native rejection, a crash, timeout, setup failure, or absent role a RED.
4. If Native rejects/crashes/times out or no repeatable active witness exists,
   STOP and report actual evidence. Do not delete the guard,
   infer unreachability, invent a numeric policy or weaken assertions.

Only after the RED: attempt the selected gate
removal, keep RED assertions unchanged, and run active/ordinary controls plus
existing bridge/anchor regressions in both modes and strict/fmt/wasm32.
Each file stays <=400 lines. Comparator, retries, exception predicates and
Native RNG remain unchanged; no Native products are committed.

## Scheduling

Official and seeded use a fresh root/cache/target per Native run. The SPEC
precedes probes or RED implementation; RED and GREEN remain separate commits.
No production, tests, Native or build runs were made for this proposal.
