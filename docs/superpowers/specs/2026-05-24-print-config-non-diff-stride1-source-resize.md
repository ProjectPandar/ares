# DynamicPrintConfig non-diff stride-1 source vector resize Spec

## Goal
Port OrcaSlicer's non-stride-2 source-vector resize slice from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a private helper for later full restore assembly.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9952-9953`: conditionally resize `opt_vec_src` to `expected_size` using `opt_target`.
- `OrcaSlicer/src/libslic3r/Config.hpp:341-362`: vector resize declaration and default-copy contract.
- `OrcaSlicer/src/libslic3r/Config.hpp:632-664`: concrete resize behavior.

Context only:
- `PrintConfig.cpp:9943-9950`: prior size mismatch branch.
- `PrintConfig.cpp:9955-9963`: deferred target clone/resize and restore call.
- `PrintConfig.hpp:666-668`: owning function declaration.

## Requirements
- Add a private helper in `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs`.
- Preserve upstream resize behavior for the source vector:
  - equal size: unchanged;
  - expected size zero: clear;
  - oversized source: truncate;
  - undersized non-empty source: extend with the first source value;
  - empty source: extend with the first target/default value.
- Add focused unit tests for each behavior.
- Keep changed Rust files at or below 400 LOC.

## Non-goals
- Do not implement target clone/resize normalization from `PrintConfig.cpp:9955-9961`.
- Do not implement stride-1 `set_with_restore` from `PrintConfig.cpp:9963`.
- Do not implement logging, full non-diff assembly, diff update, profile loading, UI, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
