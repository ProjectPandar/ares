# Spec: M342 PrintApply apply print_diff set reassignment

## Goal

Port the conditional `print_diff` reassignment from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1227-1228` into `ares-core` private staged state.

## Upstream source mapping

```cpp
if (print_diff_set.size() != print_diff.size())
    print_diff.assign(print_diff_set.begin(), print_diff_set.end());
```

The Rust staging must model:

- original `print_diff` length,
- staged duplicate-suppressed `print_diff_set` length,
- the size-difference gate,
- conditional reassignment from set contents,
- no reassignment when sizes are equal.

The upstream source assigns from `std::unordered_set`; output order must be treated as unspecified. Tests may compare set membership or a canonicalized staged representation, but must not encode original-order preservation after reassignment.

## Non-goals / deferred behavior

- Do not implement apply-status update from `PrintApply.cpp:1231-1239`.
- Do not implement lock acquisition or later invalidation/update behavior from `PrintApply.cpp:1241+`.
- Do not perform real `DynamicPrintConfig` or `PrintConfig` lookup/mutation.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- If staged set size equals original `print_diff` length, the staged result records no reassignment and returns the original `print_diff` unchanged.
- If staged set size differs because a key was erased, the staged result records reassignment and returns duplicate-suppressed staged set contents.
- If staged set size differs because original `print_diff` contained duplicate keys, the staged result records reassignment and returns duplicate-suppressed staged set contents.
- If staged set membership differs but size is equal, no reassignment occurs, matching the upstream size-only guard.
- Reassigned output order is not asserted as original-order preservation.
- All new symbols stay private to `ares-core` staged modules.
