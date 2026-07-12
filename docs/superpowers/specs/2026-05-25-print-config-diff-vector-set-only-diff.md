# DynamicPrintConfig diff vector set_only_diff branch Spec

## Goal
Port OrcaSlicer's vector `set_only_diff` branch from `DynamicPrintConfig::update_diff_values_to_child_config(...)` into `ares-core` as private helpers for later full diff update assembly.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10038-10045`: vector branch, stride selection, and `set_only_diff` call.
- `OrcaSlicer/src/libslic3r/Config.hpp:561-580`: concrete `ConfigOptionVector<T>::set_only_diff(...)` semantics.

Context only:
- `PrintConfig.cpp:9972-10022`: already-ported M254 variant-index setup.
- `PrintConfig.cpp:10024-10037`: already-ported M255 direct-set branch.
- `PrintConfig.hpp:668`: owning function declaration.

## Requirements
- Add private helpers in `crates/ares-core/src/options/update_diff_values_to_child_config.rs`.
- Select stride `2` when the key appears in `key_set2`, otherwise use stride `1`.
- Apply set-only-diff semantics to a mutable source vector and nullable target vector representation.
- Reject invalid source size when `source.len() != diff_index.len() * stride` using the upstream error message.
- For `diff_index[i] == -1`, leave the source stride segment unchanged.
- For non-`-1` diff indexes, copy `stride` values from target offset `diff_index[i] * stride` to source offset `i * stride`.
- If the target value at `diff_index[i] * stride` is nil, skip copying the whole stride segment.
- Add tests for stride selection, stride-1 copying, `-1` no-op, invalid source size, stride-2 pair copying, and nil target skip.

## Non-goals
- Do not wire the helper into full `update_diff_values_to_child_config`.
- Do not implement JSON option type dispatch, concrete nullable option types, profile loading, UI, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
