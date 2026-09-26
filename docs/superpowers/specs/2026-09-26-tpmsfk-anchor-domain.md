# TPMS-FK sparse-anchor surface boundary

Port the `FillBase.cpp::fill_surface` offset reached by
`Fill.cpp::generate_sparse_infill_polylines_for_anchoring` into the TpmsFk
arm in `sparse_anchoring/arms_tail.rs`. `FillTpmsFK::_fill_surface_single`
already has a Rust generator and emission already supplies the half-spacing
inset. Anchoring must do the same at its zero-overlap boundary. Existing
raw-surface dispatch is a temporary compatibility shell being corrected,
not a newly designed bridge pipeline.

Included: offset each anchoring surface by negative half spacing (float
coordinate delta, Miter/3), dispatch each resulting component; empty results
naturally produce no arms. Deferred: TPMS field generation, unrelated
patterns, bridge-angle scoring and generic geometry-engine parity.

Official source-output evidence `/tmp/parity-line-full/case-80nQL0` first
differs in the internal-bridge angle at line 4037, while preceding sparse
TPMS-FK emission matches. Three official references reproduce that same
geometry difference. The emission caller already applies this inset,
whereas sparse anchoring directly calls the raw-surface generator.

Plan: red test through actual sparse dispatch checks all resulting arm
vertices against the half-spacing-inset bounds; correct the dispatch,
then run the official option comparison, workspace/KSR, fmt/clippy and
independent review. Keep complete artifacts and strict acceptance. No
causal attribution to randomness is made for this deterministic mismatch.

The initial bounds test used a rotated TPMS field; rotate/unrotate integer
quantization moved a clipped endpoint by one coordinate unit. The final
fixture cancels TPMS-FK's -45-degree correction with a +45-degree input,
so its bounds assertions test the exact clipping seam without loosening
equality or changing output acceptance. A zero-inset mutation is rerun
against that corrected fixture before final validation.

## Verification

`/tmp/tpmsfk-anchor-mutation-red.log` reproduces missing inset with a
vertex at y=4223233; the corrected fixture and existing Lightning test both
pass in final workspace. `/tmp/tpmsfk-anchor-final-workspace.log` records
7331 passed / 10 skipped; `/tmp/tpmsfk-anchor-ksr.log` records 84 passed /
2 skipped. Final clippy completes with existing warnings; fmt/diff checks
pass. Source production is unchanged since the official full sweep in
`/tmp/tpmsfk-anchor-oracle.log`: 23 PASS / 3 DIVERGENT. TPMS-FK now matches
strict ordered bytes; crosszag, Lightning and lockedzag remain unresolved.
Complete paired outputs are retained under `/tmp/parity-tpmsfk-anchor`.

Independent six-axis reviewer 48500adb found no issues and approved the
scoped correction; the parent verified final workspace/clippy completion
after review. This is not all-printer/all-option acceptance.
