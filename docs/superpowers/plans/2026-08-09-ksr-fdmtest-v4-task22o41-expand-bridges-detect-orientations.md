# Task 22O.41 — Implementation plan

## Status

Implementation and verification are complete for this bounded slice. The
initial independent review requested an observable zone-major sorting witness
and first/later error mutation ledgers; all three tests were added. The same
reviewer approved the repaired candidate after 6/6 focused, 53/53 external-
surface, and 6,107/6,107 workspace tests passed with two skipped, together with
workspace Clippy, rustfmt, diff, LOC, include, and expected-RED KSR checks.

## Outcome

Implement the source-cited boundary in the matching O41 spec without activating
`process_external_surfaces` or changing public adapters.

## Red-green sequence

1. Register a compiling orchestration stub and add one public-behavior test for
   a nonempty bottom bridge; verify the focused test fails.
2. Compose O36/O37/O39/O40 in upstream order and make that tracer test pass.
3. Add one behavior at a time for empty input, source extraction/preservation,
   ordering, selective zone clipping, defaults, and error propagation.
4. Run all external-surface regressions, workspace Nextest, rustfmt, Clippy,
   the LOC/include audit, and the ignored normalized KSR progress probe.
5. Start a fresh read-only reviewer for the required six-dimensional review;
   repair findings and ask the same reviewer to revalidate until approval.

## Implementation shape

Move matching `BottomBridge` ExPolygons from the surface slice, run the four
released helpers, sort with stable Rust tuple keys matching the upstream strict
comparators, clone only the final merged ExPolygons needed as the clip operand,
and replace each marked zone with `difference_ex`. No validation is added for
internal invariants and no fallback geometry is emitted.

## Verification

```bash
cargo nextest run -p ares-core task22o41
cargo nextest run -p ares-core external_surfaces
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Also require `git diff --check`, every Rust source/test file below 400 LOC, no
source-splitting include macros, and the expected-red normalized KSR probe.
