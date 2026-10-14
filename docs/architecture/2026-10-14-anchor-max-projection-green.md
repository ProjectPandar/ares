# D-M: local anchor maximum projection GREEN

Approved SPEC 90c43e984; approved independent public RED c2fd57b60.
Base main 5a9687ac0. Only production change: anchor_projection.rs deletes
its non-finite maximum rejection and discards the now-unused tuple maximum.
The existing projected-anchor finiteness check (D-A), Fill/schema and
sparse_anchoring/linear.rs are unchanged. RED tests/assertions are unchanged.

Source: Fill.cpp::group_fills:978-989 narrows local FloatOrPercent carriers,
projects percentage values and clamps anchor to maximum without this gate;
Layer::generate_sparse_infill_polylines_for_anchoring:1477-1482 passes the
projected lengths to FillParams. PrintObject.cpp::
region_config_from_model_volume:3664-3709 owns local effective-region merging.
This removes temporary Ares admission scaffolding around those upstream
operations; it adds no new numeric policy or fallback.

## Final directed evidence

Driver /tmp/worker2-dm-green-leg.sh, official CPU13 / seeded CPU14,
jobs2/threads2, independent fresh target/cache/root, outer taskset.
Filter includes solid-pattern/D-M, top-pattern, bridge_over_infill and
infill_anchor regressions, following their existing test contracts.

| Mode | Root | Result |
|---|---|---|
| official | /tmp/worker2-dm-green-official.wC1NmK | 25/25 PASS, exit0 |
| seeded | /tmp/worker2-dm-green-seeded.mJm0UR | 25/25 PASS, exit0;1 slow, no timeout |

The four unchanged D-M tests each compare ordinary local max=20 followed
by object/part absolute1e39 or percentage1e39%. Each mode has8 strict whole-
stream Native/API/CLI comparisons PASS, including4 formerly rejected extremes.
All have complete Native footer,52 Internal Bridge/378 Sparse infill positive
extrusion moves, finite public f64 carriers and ordinary global anchors.
The extrema still narrow to non-finite f32; no carrier/projection was clamped.

Independent offline reread using the existing generator/date normalization
verified all16 Native/API/CLI comparisons and raw API/CLI equality. It also
verified input/native hashes, untouched ZIP members and ordinary global
anchors. Each mode's4 ordinary control raw Ares/CLI streams are byte-identical
to the approved RED controls, without normalization. Full hashes remain in
/tmp/worker2-dm-green-evidence.json; verifier /tmp/worker2-dm-green-verify.py.
The source snapshot remained unchanged throughout execution.

## Engineering gates

All exit0, CPU13/14 jobs2; /tmp/worker2-dm-green-gates.sh:
- cargo clippy --workspace --all-targets --all-features -- -D warnings
- cargo fmt --all -- --check
- cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown --all-features

Gates root: /tmp/worker2-dm-green-gates.w7ytcJ.
Production diff is exactly the approved gate/binding change. No test,
comparator, retry, exception or adjacent Fill change; all files <=400 lines.
The proven admission scope is the finite positive local absolute/percent
witnesses above, not arbitrary numeric acceptance or the deferred D-A domain.
No Native products committed. START/END and REVIEW posted to inbox.
Full/workspace/CI allocation and merge remain coordinator-owned; no push.
