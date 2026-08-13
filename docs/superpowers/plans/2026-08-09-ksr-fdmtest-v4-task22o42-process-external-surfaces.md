# Task 22O.42 — Implementation plan

## Outcome

Implement and activate the complete source-cited O42 boundary defined in the
matching spec. Public project slicing will advance past horizontal-shell
propagation and still terminate explicitly at the next unimplemented upstream
stage.

## Red-green sequence

1. Add a compiling record-transform stub and one nonempty reconstruction test;
   run `cargo nextest run -p ares-core task22o42` and retain the genuine RED.
2. Implement only parameter derivation, extraction, zones, O41/O35 calls,
   sparse promotion, and final reconstruction needed for the tracer to pass.
3. Add one behavior at a time for angle modes, both scales, wall-loop branches,
   sparse gates/equality, metadata/order, and direct errors.
4. Add the consuming O26 lifecycle adapter and public KSR lifecycle test, then
   make that vertical slice GREEN without adding a fixture branch.
5. Run focused predecessor regressions, workspace gates, the normalized golden
   progress probe, LOC/include/source-pinning audits, and independent review.

## Implementation shape

- `external_surfaces/process.rs`: destructive owned-record transform;
- `external_surfaces/parameters.rs`: source-order scaled parameter derivation;
- `external_surfaces/stage.rs`: O26 alignment, per-record dispatch, and the
  `PreparedPostExternalSurfaces` wrapper;
- `external_surfaces/tests/process*.rs`: behavior tests split before 400 LOC;
- `project_slice.rs`: insert O42 after O26 and consume the new terminal type.

Use existing scaled `ClassicPreludeRecord` values instead of resolving flows a
second time. Consume the O26 graph directly; a failed owned stage is dropped,
so no full-project defensive clone or rollback graph is added. Add only the
narrow `RegionSurface` construction/angle operation required by the upstream
surface semantics.

## Verification

```bash
cargo nextest run -p ares-core task22o42
cargo nextest run -p ares-core external_surfaces
cargo nextest run -p ares-core -E 'test(/task22o(2[4-6]|4[0-2])/)' # dependency slice
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run -p ares-cli --test ksr_fdmtest_v4 \
  -E 'test(project_matches_orca_242_except_generator_line)' \
  --run-ignored ignored-only
```

Also require `git diff --check`, every Rust file below 400 physical lines, no
production source-splitting include macro, and no fixture/hash/reference-G-code
read or implementation/source-text pin in production.

## Completion record

The planned stub first failed on an observable KSR `InternalVoid`, then the
full record transform and O26 successor made the lifecycle green. After review
repairs, final focused results are O42 19/19, external surfaces 72/72, and
O24-O26/O40-O42 119/119. Workspace Nextest passes 6,126/6,126 with 27 slow and
two skipped; workspace Clippy with warnings denied, rustfmt, WASM checks, diff,
LOC, and include audits pass. The normalized golden probe remains the expected
CLI `--options` RED. The final independent standards, specification, and
upstream-parity re-review returned unconditional approval with no findings.
