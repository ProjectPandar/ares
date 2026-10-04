# Bridge direction port audit plan

1. Compare WIP `a53bc00c` against current main `c3f61645` at the source/Rust
   boundaries in the matching spec. Do not overwrite current behavior.
2. Run existing equal-cost, rehash and affine public complete-output regressions
   with explicit reference gates enabled, official CPU0 and separate seeded
   artifact roots. Retain failures rather than choosing a favorable oracle.
3. Record workspace Nextest, Clippy, fmt and WASM results; only then close the
   already-integrated slice audit. No unrelated geometry changes are imported.

## Inspection

`git diff a53bc00c HEAD -- crates/ares-core/src/geometry/bridge_direction.rs`
contains only a shortened comment. Both implementations use the same prime
bucket growth, double hash, front insertion and reverse-group rehash.
`project/transform.rs` retains the identical separate f32 product/sum operations;
main additionally has `linear_determinant`, which must not be removed.
Fresh execution of all four existing bridge/affine cases passes with the
original official CPU0 binary and separately with the initialized seeded
reference (seed 0 / CPU 0). Logs:
`/tmp/ares-pi-tower-pa-bridge-final-official.log` and
`/tmp/ares-pi-tower-pa-bridge-seeded.log`. Both seven-test runs have five PASS
and two Adaptive PA failures; do not report either aggregate as all green.
No bridge production changes are required by this audit. After rebasing the
checkpoint onto `9a7b46fc`, all four cases pass again in the two independent
runs `/tmp/ares-pi-tower-checkpoint-official.log` and
`/tmp/ares-pi-tower-checkpoint-seeded.log`. The same snapshot passes 7592
workspace tests, Clippy (exit 0 with warnings), fmt and WASM build; detailed
commands and logs are in the Adaptive PA checkpoint plan. Remaining Adaptive
PA REDs are not bridge failures and still prohibit main integration of the
combined checkpoint.
