# Consume support_interface_not_for_body runtime design

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:961` declares `support_interface_not_for_body` as a `ConfigOptionBool` in the FFF support interface option group.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6036-6041` registers `support_interface_not_for_body` with label `Avoid interface filament for base`, support category, tooltip `Avoid using support interface filament to print support base if possible.`, and default `true`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1200` marks changes to `support_interface_not_for_body` as support-affecting invalidation input.
- Representative downstream consumers are `GCode/ToolOrdering.cpp:768`, `GCode/ToolOrdering.cpp:1703`, `GCode.cpp:4784` inside disabled BBS code, and `Preset.cpp:1086`.

## Rust destination boundary

- Add `crates/ares-core/src/options/support_interface_not_for_body.rs`.
- Add `support_interface_not_for_body` to the existing support option module declaration in `crates/ares-core/src/options.rs`. `options.rs` is exactly 400 LOC, so this must be an edit to the existing `option_modules!(...)` line rather than a new line.
- Add a module-local `impl SliceOptions` in `support_interface_not_for_body.rs` with `support_interface_not_for_body_options() -> Result<SupportInterfaceNotForBodyOptions, SliceError>`.
- Add a small `SupportInterfaceNotForBodyOptions` value type with a `not_for_body()` accessor and `consume_runtime()` following the existing support runtime-state pattern.
- Consume `options.support_interface_not_for_body_options()?.consume_runtime()` in `crates/ares-core/src/pipeline.rs` before model loading, after the existing support interface/style/placement validation calls and before tree support option consumption.
- Add parser tests in `crates/ares-core/src/options/tests/support_interface_not_for_body.rs`.
- Add `support_interface_not_for_body` to the existing `#[rustfmt::skip] option_test_modules!(...)` line in `crates/ares-core/src/options/tests.rs`. `options/tests.rs` is exactly 400 LOC, so this must not add a new line.
- Add pipeline tests in `crates/ares-core/src/pipeline/tests/support_interface_not_for_body.rs` and register the module in `crates/ares-core/src/pipeline/tests.rs`.
- Update `docs/roadmap.md` after implementation with this source-cited runtime slice and deferred behavior.

## Existing Ares context

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs` already registers `support_interface_not_for_body` as `Bool`, default `true`, with source citations.
- `crates/ares-core/src/options/tests/registry_lookup_support_interface_line_width.rs` already verifies registry lookup metadata for this option.
- Current Ares runtime has support filament, support line-width, support interface layers, support interface pattern, support interface spacing, support speed/flow, and support ironing behavior, but no non-metadata module for `support_interface_not_for_body`.

## Behavior to implement

- Parse `support_interface_not_for_body` as a boolean, defaulting to `true`.
- Accept only JSON booleans for configured `support_interface_not_for_body`.
- Reject strings, numbers, null, arrays, and objects with `SliceError::InvalidInput` containing `support_interface_not_for_body`.
- Make `run_slicing_pipeline()` reject invalid `support_interface_not_for_body` before model loading.
- Preserve current generated geometry, print paths, G-code, and diagnostics for `support_interface_not_for_body = true` and `support_interface_not_for_body = false` because this slice only consumes typed state.

## Out of scope

- Do not add new user-facing options.
- Do not implement support invalidation graph behavior, support base/interface extruder override behavior, support interface filament avoidance, wipe-volume selection, tool-ordering changes, `GCode.cpp` dontcare-extruder fallback behavior, preset migration behavior, support material generation changes, UI behavior, CLI behavior, WASM bindings, registry definitions, or legacy migration behavior.
- Do not use `support_interface_not_for_body` to change slicing output yet.
- Do not add dependencies or new crates.

## Acceptance criteria

- Missing `support_interface_not_for_body` produces `not_for_body() == true`, matching Orca and the current Ares registry.
- `support_interface_not_for_body` accepts `true` and `false` JSON booleans.
- `support_interface_not_for_body` rejects `"true"`, `"false"`, numeric values, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- `run_slicing_pipeline(b"not a model", &options)` with invalid `support_interface_not_for_body` returns the `support_interface_not_for_body` validation error before model parsing.
- A valid `support_interface_not_for_body = false` configuration remains a no-op for current Ares slicing output, proven by comparing generated print paths and G-code/output artifacts against a `support_interface_not_for_body = true` baseline.
- Touched Rust files remain at or below 400 LOC.
- Fresh verification includes targeted option tests, targeted pipeline tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check`.
