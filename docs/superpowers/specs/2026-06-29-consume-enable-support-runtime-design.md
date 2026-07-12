# Consume enable_support runtime design

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948` declares `enable_support` as a `ConfigOptionBool` in the FFF support option group, immediately before `support_type`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5903-5908` registers `enable_support` as a support boolean with label `Enable support`, tooltip `Enable support generation.`, and default `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10228-10229` rejects `enable_support` when spiral-vase CLI validation is active.
- Representative downstream consumers are `Print.hpp:429`, `Slicing.cpp:124`, `Print.cpp:1456`, `Support/SupportMaterial.hpp:28`, `Support/TreeSupport3D.cpp:199`, `Support/TreeSupport.cpp:670`, `PrintApply.cpp:1134-1138`, and `PrintApply.cpp:1767`.

## Rust destination boundary

- Add `crates/ares-core/src/options/support_enable.rs`.
- Add `support_enable` to the existing support option module declaration in `crates/ares-core/src/options.rs`. `options.rs` is exactly 400 LOC, so this must be an edit to the existing `option_modules!(...)` line rather than a new line.
- Add a module-local `impl SliceOptions` in `support_enable.rs` with `support_enable_options() -> Result<SupportEnableOptions, SliceError>`.
- Add a small `SupportEnableOptions` value type with an `enabled()` accessor and `consume_runtime()` following the existing support runtime-state pattern.
- Consume `options.support_enable_options()?.consume_runtime()` in `crates/ares-core/src/pipeline.rs` after existing validation calls and before `support_type()`, `support_style()`, support placement, support threshold, and tree support option consumption.
- Add parser tests in `crates/ares-core/src/options/tests/support_enable.rs`.
- Add `support_enable` to the existing `#[rustfmt::skip] option_test_modules!(...)` line in `crates/ares-core/src/options/tests.rs`. `options/tests.rs` is exactly 400 LOC, so this must not add a new line.
- Add pipeline tests in `crates/ares-core/src/pipeline/tests/support_enable.rs` and register the module in `crates/ares-core/src/pipeline/tests.rs`.
- Update `docs/roadmap.md` after implementation with this source-cited runtime slice and deferred behavior.

## Existing Ares context

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs` already registers `enable_support` as a `Bool` with default `false` and source citations.
- `crates/ares-core/src/options/validation/spiral_vase.rs` already reads `enable_support` for CLI spiral-vase compatibility and must keep the same error-key behavior.
- Existing staged `PrintApply` support-used metadata docs and tests remain related prior work, not the runtime pipeline destination for this slice.

## Behavior to implement

- Parse `enable_support` as a boolean, defaulting to `false`.
- Accept only JSON booleans for configured `enable_support`.
- Reject strings, numbers, null, arrays, and objects with `SliceError::InvalidInput` containing `enable_support`.
- Make `run_slicing_pipeline()` reject invalid `enable_support` before model loading.
- Preserve current generated geometry, print paths, G-code, and diagnostics for `enable_support = true` and `enable_support = false` because this slice only consumes typed state.
- Preserve existing `validate_spiral_vase_cli_options()` behavior for `enable_support`.

## Out of scope

- Do not add new user-facing options.
- Do not implement support material generation, normal support generation, tree support generation, organic support generation, support enforcer/blocker routing, support layer-height synchronization, `Print::has_support()`, `SupportMaterial::has_support()`, `SlicingParameters` support-layer min/max coupling, prime-tower support-gap validation, `PrintApply` `m_support_used` mutation, raft/enforced-support composition, UI behavior, CLI behavior, WASM bindings, registry definitions, or legacy migration behavior.
- Do not use `enable_support` to change slicing output yet.
- Do not add dependencies or new crates.

## Acceptance criteria

- Missing `enable_support` produces `enabled() == false`, matching Orca and the current Ares registry.
- `enable_support` accepts `true` and `false` JSON booleans.
- `enable_support` rejects `"true"`, `"false"`, numeric values, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- `run_slicing_pipeline(b"not a model", &options)` with invalid `enable_support` returns the `enable_support` validation error before model parsing.
- A valid `enable_support = true` configuration remains a no-op for current Ares slicing output, proven by comparing generated print paths and G-code/output artifacts against an `enable_support = false` baseline.
- The existing spiral-vase CLI validation still reports `enable_support` when `spiral_mode` and `enable_support` are both true.
- Touched Rust files remain at or below 400 LOC.
- Fresh verification includes targeted option tests, targeted pipeline tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check`.
