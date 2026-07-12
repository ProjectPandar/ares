# Consume `support_type` runtime design

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:195-209` defines `SupportType` with `stNormalAuto`, `stTreeAuto`, `stNormal`, and `stTree`, plus `is_tree`, `is_tree_slim`, and `is_auto` helper predicates.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:950` stores `support_type` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:342-348` maps serialized enum strings to `SupportType`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5910-5925` registers `support_type`, accepted enum values, labels, tooltip, and default `stNormalAuto`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7925-7929` migrates legacy serialized values `normal`, `tree`, and `hybrid(auto)`.
- Representative downstream consumers are `PrintObject.cpp:1523-1526`, `Support/TreeSupport3D.cpp:199`, `Support/TreeSupport.cpp:640-681,790-819,854,910,1003,1086,1737`, `Support/SupportMaterial.cpp:1387,1572,2106,2155`, and `Support/SupportParameters.hpp:185-191`.

## Rust destination boundary

- Add `crates/ares-core/src/options/support_type.rs`.
- Add a `SliceOptions::support_type()` typed accessor in `crates/ares-core/src/options.rs`.
- Keep LOC compliance explicit: `options.rs` is already 399 LOC, so wire `support_type` by moving one existing standalone `mod` declaration into the existing `option_modules!(...)` line that will also include `support_type`, then add the one-line accessor without taking the file over 400 LOC. `options/tests.rs` is already 400 LOC, so add the new test module name to the existing `option_test_modules!(...)` line instead of adding a new `mod` line.
- Consume `support_type` in the current early runtime option-validation phase of `crates/ares-core/src/pipeline.rs`, after FDM normalization and before model loading.
- Add parser tests in `crates/ares-core/src/options/tests/support_type.rs`.
- Add a pipeline guard test in `crates/ares-core/src/pipeline/tests/support_type.rs`.
- Update `docs/roadmap.md` after implementation with this slice and deferred behavior.

## Behavior to implement

- Parse the canonical Orca strings:
  - `normal(auto)` -> `SupportType::NormalAuto`
  - `tree(auto)` -> `SupportType::TreeAuto`
  - `normal(manual)` -> `SupportType::NormalManual`
  - `tree(manual)` -> `SupportType::TreeManual`
- Default missing `support_type` to `SupportType::NormalAuto`, matching Orca's `stNormalAuto` default.
- Reject non-string values and unknown strings with `SliceError::InvalidInput` messages containing `support_type`.
- Expose helper predicates matching `PrintConfig.hpp`:
  - `is_tree()` is true for `TreeAuto` and `TreeManual`.
  - `is_auto()` is true for `NormalAuto` and `TreeAuto`.
- Exercise the existing Ares legacy migration by asserting deserialized `normal`, `tree`, and `hybrid(auto)` values resolve through the typed accessor to the matching canonical variants.
- Make `run_slicing_pipeline()` reject invalid `support_type` before model loading, consistent with the existing tree-support option validation guard.
- Preserve all current generated geometry, print paths, G-code, and diagnostics for every valid value.

## Out of scope

- Do not add new user-facing options.
- Do not implement normal support generation, tree support generation, organic support generation, support enforcers, or support blockers.
- Do not route `normal(auto)` vs `normal(manual)` into support material geometry yet.
- Do not route `tree(auto)` vs `tree(manual)` into tree support geometry yet.
- Do not implement `is_tree_slim()` until `SupportMaterialStyle` runtime state is in scope.
- Do not change legacy migration behavior beyond consuming the existing migrated values.
- Do not add dependencies or new crates.

## Acceptance criteria

- All four canonical `support_type` strings parse to typed variants and helper truth tables match the upstream helpers.
- Missing `support_type` returns `NormalAuto`.
- Legacy `normal`, `tree`, and `hybrid(auto)` inputs still deserialize successfully and resolve to `NormalManual`, `TreeManual`, and `TreeAuto` respectively.
- Invalid strings and non-string JSON values reject with `SliceError::InvalidInput` containing `support_type`.
- `run_slicing_pipeline(b"not a model", &options)` with invalid `support_type` returns the `support_type` validation error before model parsing.
- All four valid `support_type` values remain no-ops for current Ares slicing output in this slice, proven by comparing generated rectangular-pipeline diagnostics or G-code/output artifacts for each value against the default `normal(auto)` baseline.
- Touched Rust files remain at or below 400 LOC.
- Fresh verification includes targeted tests for the new parser and pipeline guard, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, and `cargo check -p ares-core --target wasm32-unknown-unknown`.
