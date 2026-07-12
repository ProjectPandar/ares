# Spec: M344 PrintApply print_diff config invalidation

## Goal

Port the lock-ordered, print-diff-gated config invalidation block from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1241-1246` into `ares-core` private staged state.

## Upstream source mapping

```cpp
// Grab the lock for the Print / PrintObject milestones.
std::scoped_lock<std::mutex> lock(this->state_mutex());

// The following call may stop the background processing.
if (! print_diff.empty())
    update_apply_status(this->invalidate_state_by_config_options(new_full_config, print_diff));
```

The Rust staging must model:

- lock acquisition occurs before the print-diff invalidation decision,
- no invalidation call when `print_diff` is empty,
- one invalidation call using `new_full_config` and `print_diff` when `print_diff` is non-empty,
- the invalidation boolean result feeds the same max-based status update semantics staged in M343,
- no downgrade from invalidated to changed.

## Non-goals / deferred behavior

- Do not implement real mutex locking or concurrent mutation.
- Do not implement real `Print::invalidate_state_by_config_options`.
- Do not implement placeholder parser/full-config handling from `PrintApply.cpp:1248-1265`.
- Do not emit real logs or stop background processing.
- Do not perform real `DynamicPrintConfig` or `PrintConfig` lookup/mutation.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Empty `print_diff` records lock acquisition but no staged invalidation call and leaves an unchanged prior status unchanged.
- Empty `print_diff` records lock acquisition but no staged invalidation call and leaves an invalidated prior status invalidated.
- Non-empty `print_diff` records a staged invalidation call with receiver `this`, config source `new_full_config`, and the input diff keys.
- Non-empty `print_diff` with invalidation result false updates unchanged prior status to changed.
- Non-empty `print_diff` with invalidation result true updates status to invalidated.
- Non-empty `print_diff` with invalidation result false does not downgrade an already invalidated status.
- The staged event order records lock acquisition before invalidation call.
- All new symbols stay private to `ares-core` staged modules.
