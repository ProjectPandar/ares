# Lint remediation: behavior-preserving module slices

## Non-negotiable boundaries

- Preserve existing runtime behavior, comparator thresholds, mutation rejection and every test assertion. This is lint remediation, not a new slicing milestone or upstream-behavior change.
- Fix excessive nesting and argument lists at their owning module boundaries; do not raise lint thresholds or blanket-suppress warnings.
- Work in an isolated worktree. Keep concurrent support-tip, Arachne-outline and tower/AdaptivePA/bridge work out of the current slice.
- For each slice: run full workspace nextest, clippy, fmt and WASM compilation; push the topic branch, print `REVIEW-REQUEST <commit-hash> <summary>` in the terminal, and stop for coordinator Claude's `APPROVED <hash>` or revision requests. Do not invoke external reviewers or start new changes on the same branch while waiting. Only after approval: fetch/rebase latest `origin/main`, rerun the gates, and fast-forward main without force pushing.
- The final workspace gate is `cargo clippy --workspace --all-targets --all-features -- -D warnings`. Intermediate module slices report remaining baseline warnings explicitly rather than claiming the workspace is warning-free.

## First slice: CLI semantic comparison

Destination: `crates/ares-cli/tests/ksr_fdmtest_v4/semantic/{controls,parser,depositions,lifecycle,travel}.rs` and `semantic/depositions/leftovers/geometry.rs`.

ARD-0081 owns the boundary: semantic comparison remains a diagnostic utility, never a fallback for strict ordered-byte acceptance. Preserve existing control canonicalization, fan parsing/error order, geometric sampling arithmetic, matching predicates, and diagnostic output. Extract existing nested control/diagnostic operations or use equivalent flatter branches. Do not add probes, tolerances, fixtures, or assertions.

Verification uses the existing semantic mutation regressions before and after the refactor, then full workspace gates and WASM compilation. The prior full baseline is commit `9a7b46fc`: 7,589 tests passed, nine skipped; existing lint debt is tracked separately from this slice's changed files.

First-slice results: 144 semantic regression cases passed before editing; after editing the full workspace passed all 7,589 cases (nine skipped). `cargo clippy -p ares-cli --test ksr_fdmtest_v4 --no-deps -- -D warnings` passed, eliminating the ten semantic nesting diagnostics. Full ordinary clippy, fmt and the `wasm32-unknown-unknown` build for both core and the WASM adapter passed. Remaining core/parity warnings are not suppressed and remain follow-up work.

## Scalar wire adapters

Destination: `ares-core/src/options/config_types/{scalar,scalar_bool_int}.rs`, the existing Rust serde wire adapters around `Config.hpp` scalar option concepts (`ConfigOptionFloat`, `ConfigOptionPercent`, `ConfigOptionFloatOrPercent`, `ConfigOptionBool`). No upstream behavior is added or deferred by this lint-only slice. A shared generic `drain_sequence` retains the same `IgnoredAny` reads after the first typed array element. Empty-array/first-element errors still occur first, malformed trailing input still propagates an error, and successful scalar values and integer-macro visitors are unchanged. Existing deserialization and full-workspace tests remain intact.

Scalar-slice validation: full workspace nextest passed 7,589 tests (nine skipped); ordinary workspace/all-targets/all-features clippy, fmt and the core/WASM target build passed. The four scalar diagnostics disappeared: core library warnings dropped from 253 to 249, and core test warnings from 269 to 265 including duplicates. Remaining workspace warnings are explicit follow-up work, not suppressed. Publication follows the coordinator approval protocol above.
