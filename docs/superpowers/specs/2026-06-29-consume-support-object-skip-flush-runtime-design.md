# Consume support_object_skip_flush runtime design

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1339` declares `support_object_skip_flush` as a `ConfigOptionBool` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2500-2501` registers `support_object_skip_flush` as `coBool` with default `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3265` reads `print.config().support_object_skip_flush.value` with `m_enable_exclude_object` before object-sequential toolchange priming.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5116` reads `print.config().support_object_skip_flush.value` with `PrintSequence::ByLayer` and `m_enable_exclude_object` before layer toolchange handling.
- `OrcaSlicer/src/libslic3r/Preset.cpp:1345` includes `support_object_skip_flush` in the preset key list.

## Rust destination boundary

- Add `crates/ares-core/src/options/support_object_skip_flush.rs`.
- Add `support_object_skip_flush` to the existing support option module declaration in `crates/ares-core/src/options.rs`. `options.rs` is exactly 400 LOC, so this must be an edit to the existing `option_modules!(...)` line rather than a new line.
- Add a module-local `impl SliceOptions` with `support_object_skip_flush_options() -> Result<SupportObjectSkipFlushOptions, SliceError>`.
- Add a small `SupportObjectSkipFlushOptions` value type with a `skip_flush()` accessor and `consume_runtime()` following the existing support runtime-state pattern.
- Consume `options.support_object_skip_flush_options()?.consume_runtime()` from `crates/ares-core/src/gcode_object_labels.rs` inside `ObjectLabelConfig::from_options()`, which is Ares' existing `exclude_object`/object-label G-code configuration boundary.
- Add parser tests in `crates/ares-core/src/options/tests/support_object_skip_flush.rs`.
- Add `support_object_skip_flush` to the existing `#[rustfmt::skip] option_test_modules!(...)` line in `crates/ares-core/src/options/tests.rs`. `options/tests.rs` is exactly 400 LOC, so this must not add a new line.
- Extend `crates/ares-core/src/tests/gcode_label_objects.rs` with G-code-time validation/no-output-change coverage.
- Update `docs/roadmap.md` after implementation with this source-cited runtime slice and deferred behavior.

## Existing Ares context

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs` already registers `support_object_skip_flush` as `Bool`, default `false`, with source citations.
- `crates/ares-core/src/options/tests/registry_lookup_support.rs` already verifies registry lookup metadata for this option.
- The former source-line-only `GCodeConfig` tuple module was removed by the Option pinning cleanup; the retained registry and cited upstream consumer define this slice.
- `crates/ares-core/src/gcode_object_labels.rs` already parses `exclude_object` and `gcode_label_objects`, emits object labels/exclusion commands, and is the nearest Ares boundary to Orca's guarded `support_object_skip_flush` reads.
- `crates/ares-core/src/gcode.rs` is 399 LOC and must not be expanded for this validation-only slice.

## Behavior to implement

- Parse `support_object_skip_flush` as a boolean, defaulting to `false`.
- Accept only JSON booleans for configured `support_object_skip_flush`.
- Reject strings, numbers, null, arrays, and objects with `SliceError::InvalidInput` containing `support_object_skip_flush`.
- Make G-code formatting reject invalid `support_object_skip_flush` through the existing object-label configuration path.
- Preserve current generated G-code for valid `support_object_skip_flush = true` and `support_object_skip_flush = false` because this slice only consumes typed state.

## Out of scope

- Do not add new user-facing options.
- Do not implement support-object skip-flush output behavior, object-specific filament instance labels, wipe/purge behavior, sequential object toolchange priming behavior, by-layer toolchange behavior, multi-object semantics, support generation, support geometry, UI behavior, CLI behavior, WASM bindings, registry definitions, or legacy migration behavior.
- Do not emit `support_object_skip_flush` in the G-code config header.
- Do not use `support_object_skip_flush` to change generated G-code yet.
- Do not add dependencies or new crates.

## Acceptance criteria

- Missing `support_object_skip_flush` produces `skip_flush() == false`, matching Orca and the current Ares registry.
- `support_object_skip_flush` accepts `true` and `false` JSON booleans.
- `support_object_skip_flush` rejects `"true"`, `"false"`, numeric values, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- `slice()`/G-code formatting with invalid `support_object_skip_flush` returns the validation error through `ObjectLabelConfig::from_options()`.
- Valid `support_object_skip_flush = true` and `support_object_skip_flush = false` configurations preserve current object-label/exclude-object G-code output.
- Touched Rust files remain at or below 400 LOC.
- Fresh verification includes targeted option/G-code tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check`.
