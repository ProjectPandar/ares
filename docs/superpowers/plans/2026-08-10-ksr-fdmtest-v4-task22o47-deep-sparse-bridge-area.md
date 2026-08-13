# Task 22O.47 deep sparse bridge area implementation plan

## Status

Implementation and focused/real-KSR tests are complete. Final focused 9/9,
dependency 590/590, workspace 6,250/6,250, rustfmt, Clippy, wasm32, diff, LOC,
and structural gates pass. The first independent review's flat-result,
arithmetic, closing, and deferred multi-region findings were repaired. The
same independent review thread re-verified both repair rounds and returned
`VERDICT: APPROVE`; all plan steps are complete.

## Objective

Implement the exact borrowed deep sparse-area operation from pinned
`PrintObject.cpp:2819-2846` as the next dependency of the real
bridge-over-infill transaction. Do not activate a public lifecycle successor.

## Plan

### 1. Freeze RED focused behavior

Create
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/deep_sparse_area/tests.rs`
and register a private `deep_sparse_area` module.

Use literal source-shaped layers to assert traversal threshold semantics,
per-layer density classification, unconditional `InternalVoid`, non-sparse
subtraction, union/closing with holes and overlaps, exact target-height
arithmetic, empty results, error atomicity, and owned before/after snapshots.
Run the focused filter and retain the expected unresolved-module/compiler RED.

### 2. Implement the source operation

Create
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/deep_sparse_area.rs`.
Borrow planned layers, post-O42 fill surfaces, and per-layer effective density.
Traverse downward in source order, gather ephemeral sparse/non-sparse
ExPolygons, flatten contour-before-holes, reuse the one-pass flat path
difference operation, and
return the first error. Add only a missing exact shared geometry overload if
existing helpers cannot express the source operation.

Do not introduce a Flow type, map, sort, thread pool, callback, generic stage,
public API, fallback, or filesystem behavior.

### 3. Add the real-KSR regression

Create
`crates/ares-core/src/project_slice/tests/prepare_infill/deep_sparse_area.rs`
and register it in the existing test module. Build aligned borrowed views from
the retained project graph and embedded effective region options. Exercise all
O43 candidate-layer keys, assert repeatability and complete input preservation,
and freeze ordered polygon/point counts and hashes in the Rust test itself.

Do not read the reference G-code or temporary files and do not invoke or inspect
Orca at test runtime.

### 4. Record completion evidence

Update the ADR, specification, this plan, `docs/roadmap.md`, and
`docs/architecture/option-parity-v4.md` with actual behavior, test counts, and
gate results. Keep all downstream bridge transaction, extrusion, G-code, and
CLI work explicitly pending.

### 5. Verify and independently review

Run:

```bash
cargo nextest run -p ares-core -E 'test(/task22o47/)' --no-fail-fast
cargo nextest run -p ares-core -E 'test(/task22o4[3-7]|clipper|flow/)' --no-fail-fast
cargo nextest run --workspace --no-fail-fast
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown
git diff --check
```

Audit changed Rust files for the 400-LOC limit, forbidden source-splitting
macros, fixture/reference reads, fixture-name/hash branches, fallback, platform
branches, and accidental lifecycle activation.

Start an independent read-only reviewer and require findings across requirement
completeness, logic correctness, edge cases, code quality, test coverage, and
actual runtime results. Apply repairs only in the main thread, rerun affected
gates, and request re-review until approval.

## Exit criteria

- The cited source semantics are represented by one private borrowed operation.
- Focused and real-KSR tests pass and inputs remain unchanged.
- Public slicing still terminates at O43 with `ProjectSlicingIncomplete`.
- All validation gates pass.
- Independent review approves with no blockers.
