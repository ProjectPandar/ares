# Task 22O.43 — Implementation plan

## Outcome

Implement and activate the source-cited candidate-gathering seam defined in the
matching spec. Public project slicing will retain the owned candidate inventory
in an O43 successor and still terminate explicitly before anchor generation.

## Red-green sequence

1. Add the compiling candidate types and an empty-returning gather stub. Add a
   nonempty synthetic candidate test and run `cargo nextest run -p ares-core
   task22o43` to retain the genuine behavioral RED.
2. Implement the exact flat-path construction, Miter-3 morphology, area gates,
   policy branches, stable identities, ordering, and direct error propagation;
   rerun the same filter GREEN.
3. Add threshold, density, policy, scale, hole, ordering, no-op, and error
   cases one behavior at a time through the same module interface.
4. Add the consuming O42 lifecycle adapter. Retain candidates in the successor,
   prove the real KSR project produces a nonempty inventory from composed
   options, and move the public terminal/disposal path to O43.
5. Run focused predecessor regressions, workspace/Tier-1 gates, the normalized
   golden progress probe, LOC/include/source-pinning audits, and independent
   review. Repair and re-review until unconditional approval or a concrete
   blocker.

## Implementation shape

- `prepare_infill/bridge_over_infill.rs`: module root and successor interface;
- `bridge_over_infill/candidates.rs`: pure in-process gather implementation;
- `bridge_over_infill/types.rs`: stable candidate identity and owned inventory;
- `bridge_over_infill/stage.rs`: O42 graph adapter, composed option provenance,
  lifecycle ownership, and exhaustive geometry-error mapping;
- `bridge_over_infill/tests.rs` plus shards: module-interface behavior tests;
- `project_slice/tests/prepare_infill/bridge_over_infill.rs`: real-project
  lifecycle/provenance tests;
- `project_slice.rs`: call and dispose O43 after O42.

Do not add an adapter for the pinned-disabled `clip_fill_surfaces` body. Do not
reuse the approximate legacy CrossHatch infill scaffold, store references into
`fill_surfaces`, clone the full predecessor for rollback, add a fixture branch,
or activate the CLI project-only contract.

## Verification

```bash
cargo nextest run -p ares-core task22o43
cargo nextest run -p ares-core bridge_over_infill
cargo nextest run -p ares-core \
  -E 'test(/task22o(2[4-6]|4[0-3])/)' # focused predecessor band
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown
cargo nextest run -p ares-cli --test ksr_fdmtest_v4 \
  -E 'test(project_matches_orca_242_except_generator_line)' \
  --run-ignored ignored-only --no-fail-fast
```

Also require `git diff --check`, every Rust file below 400 physical lines, no
production source-splitting `include!`/`include_bytes!`, and no fixture/hash/
reference-G-code read or upstream-source-text pin in production.

## Completion record

The plan was executed through the active O42 successor. The initial empty stub
gave the intended nonempty-candidate RED. Review-driven REDs also exposed and
then fixed object-wide Lightning provenance and aligned-empty-lower handling.
The final implementation retains owned candidates by stable indices, maps the
first geometry error through the O43 lifecycle, and leaves the public terminal
at `ProjectSlicingIncomplete` before anchor generation.

Final verification is O43 35/35, predecessor band 154/154, and workspace
Nextest 6,161/6,161 with 27 slow and two skipped. Warning-denying workspace
Clippy, rustfmt, ares-core/ares-wasm wasm32 check, and all specified static
audits pass. The normalized golden remains the expected CLI-contract RED; the
broader KSR pipeline and every behavior listed as deferred in the spec remain
incomplete. Both final independent review tracks approve the repaired slice
unconditionally with no remaining findings.
