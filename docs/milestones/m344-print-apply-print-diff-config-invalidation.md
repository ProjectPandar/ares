# M344: PrintApply print_diff config invalidation

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1241-1246`: after initial apply-status handling, `Print::apply(...)` grabs `this->state_mutex()` and, only when `print_diff` is non-empty, calls `invalidate_state_by_config_options(new_full_config, print_diff)` and passes that invalidation result into `update_apply_status(...)`.

```cpp
// Grab the lock for the Print / PrintObject milestones.
std::scoped_lock<std::mutex> lock(this->state_mutex());

// The following call may stop the background processing.
if (! print_diff.empty())
    update_apply_status(this->invalidate_state_by_config_options(new_full_config, print_diff));
```

Supporting context is M343's private staged max-based `update_apply_status(...)` behavior from `PrintApply.cpp:1231-1234`. This milestone models only lock acquisition ordering metadata, the non-empty `print_diff` gate, the staged invalidate-state call, and status aggregation from its boolean invalidation result.

## Exit criteria

- Preserve staged lock acquisition before the print-diff invalidation gate.
- Preserve no invalidate-state call when `print_diff` is empty.
- Preserve no status change from this block when `print_diff` is empty.
- Preserve calling staged `invalidate_state_by_config_options(new_full_config, print_diff)` when `print_diff` is non-empty.
- Preserve passing the staged invalidation boolean into max-based status update.
- Preserve changed status when non-empty `print_diff` invalidation returns false and prior status is unchanged.
- Preserve invalidated status when non-empty `print_diff` invalidation returns true.
- Preserve no downgrade when prior status is already invalidated and invalidation returns false.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer placeholder parser/full-config handling from `PrintApply.cpp:1248-1265`, real mutex locking, real background processing stop, real `Print::invalidate_state_by_config_options`, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
