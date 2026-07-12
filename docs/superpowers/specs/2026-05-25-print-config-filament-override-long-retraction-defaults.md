# compute_filament_override_value long-retraction override defaults Spec

## Goal
Port OrcaSlicer's long-retraction special-case input preparation from `compute_filament_override_value(...)` into `ares-core` as a private helper for later filament override resolution milestones.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10051-10071`: function signature, `is_nil` local context, and the two long-retraction special cases that substitute nil/default nullable vectors before `apply_override`.

Context only:
- `PrintConfig.hpp:690-691`: function declaration.
- `PrintConfig.hpp:183-188`: `LongRectrationLevel::{Disabled=0, EnableMachine=1, EnableFilament=2}`.
- `PrintConfig.cpp:5077-5090`: relevant option definitions for `enable_long_retraction_when_cut`, `long_retractions_when_cut`, and `retraction_distances_when_cut`.
- `PrintConfig.cpp:10073-10082`: later clone/apply_override/changed-key behavior deferred to later milestones.

## Requirements
- Add a private helper in `crates/ares-core/src/options/filament_override.rs` or the nearest existing options module if a better local fit exists.
- Wire the new module from `crates/ares-core/src/options.rs` or the existing `options` module root without changing public API.
- The helper must accept an option key, a filament-provided JSON value, and the full/new config value for `enable_long_retraction_when_cut`.
- If the option key is `long_retractions_when_cut` and `enable_long_retraction_when_cut != 2`, return a JSON array of `"nil"` entries with the same length as the provided filament array.
- If the option key is `retraction_distances_when_cut` and `enable_long_retraction_when_cut != 2`, return a JSON array of `"nil"` entries with the same length as the provided filament array. This intentionally follows the apparent upstream intent of filling `opt_retraction_distance_default`; `PrintConfig.cpp:10069` pushes to `opt_long_retraction_default`, but that object is not assigned to `opt_new_filament` in the float branch.
- If `enable_long_retraction_when_cut == 2`, return the original filament-provided value unchanged for both special keys.
- For any other key, return the original filament-provided value unchanged.
- Reject non-integer or missing `enable_long_retraction_when_cut` with `SliceError::InvalidInput`.
- Reject non-array filament values only when a special key needs nil/default substitution; unchanged keys may pass through any JSON value unchanged.
- Preserve no-partial-pipeline scope: this helper only prepares the override input and does not apply it to machine values or decide changed keys.
- Add tests for disabled/machine modes producing nil arrays for both special keys, enable-filament mode preserving both special keys, non-special key passthrough, and invalid enable/special-array input errors.

## Non-goals
- Do not implement `ConfigOptionVector::apply_override`, `diff_keys.emplace_back`, `filament_overrides.set_key_value`, full `compute_filament_override_value`, public APIs, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
