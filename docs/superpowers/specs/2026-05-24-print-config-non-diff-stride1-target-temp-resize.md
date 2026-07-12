# DynamicPrintConfig non-diff stride-1 target temporary resize Spec

## Goal
Port OrcaSlicer's non-stride-2 cloned target temporary normalization slice from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a private helper for later full restore assembly.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9955-9961`: clone `opt_target`, require a vector temporary, and resize it to `expected_size` when needed.
- `OrcaSlicer/src/libslic3r/Config.hpp:341-362`: vector resize declaration and default-copy contract.
- `OrcaSlicer/src/libslic3r/Config.hpp:632-664`: concrete resize behavior.

Context only:
- `PrintConfig.cpp:9952-9953`: already-ported source resize.
- `PrintConfig.cpp:9963`: deferred `set_with_restore` call.
- `PrintConfig.hpp:666-668`: owning function declaration.

## Requirements
- Add a private helper in `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` that returns a resized clone/temporary of the target vector.
- Preserve upstream target temporary behavior:
  - equal size: clone is returned unchanged;
  - expected size zero: returned temporary is empty;
  - oversized target: returned temporary is truncated;
  - undersized non-empty target: returned temporary extends with the first target value.
- Ensure the original target input is not mutated.
- Add focused unit tests for each behavior.
- Keep changed Rust files at or below 400 LOC.

## Non-goals
- Do not implement stride-1 `set_with_restore` from `PrintConfig.cpp:9963`.
- Do not implement logging, full non-diff assembly, diff update, profile loading, UI, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
