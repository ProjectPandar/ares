# DynamicPrintConfig non-diff stride-1 set_with_restore Spec

## Goal
Port OrcaSlicer's non-stride-2 `set_with_restore` mutation slice from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a private helper for later full restore assembly.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9963`: `opt_vec_src->set_with_restore(rhs_vec, variant_index, stride)` in the non-stride-2 branch.
- `OrcaSlicer/src/libslic3r/Config.hpp:488-504`: concrete `ConfigOptionVector<T>::set_with_restore(...)` semantics.

Context only:
- `PrintConfig.cpp:9943-9961`: already-ported stride-1/general size, source resize, and target temporary normalization context.
- `PrintConfig.hpp:666-668`: owning function declaration.
- `Config.hpp:341-360`: vector base operation context.

## Requirements
- Add a private helper in `crates/ares-core/src/options/update_non_diff_values_to_base_config` for stride-1/general vector restore mapping.
- Preserve upstream operation order:
  - back up original source values;
  - replace source values with target temporary values;
  - reject invalid target size when `target.len() != restore_index.len()` after replacement;
  - for each restore index not equal to `-1`, restore the backed-up source element at that index into the corresponding target position.
- Work for generic cloneable element types, not only floats.
- Keep changed Rust files at or below 400 LOC; split a helper module if necessary without changing existing behavior.
- Add focused unit tests for selected restore, all-negative/no restore, duplicate restore indexes, invalid target-size mutation-before-error ordering, and generic element handling.

## Non-goals
- Do not implement logging, full non-diff assembly, diff update, profile loading, UI, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
