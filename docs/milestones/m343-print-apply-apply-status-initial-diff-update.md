# M343: PrintApply apply status initial diff update

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1231-1239`: after `print_diff`, `object_diff`, and `region_diff` are known, `Print::apply(...)` initializes `apply_status` to `APPLY_STATUS_UNCHANGED`, defines `update_apply_status` as a max operation that maps `invalidated = false` to `APPLY_STATUS_CHANGED` and `invalidated = true` to `APPLY_STATUS_INVALIDATED`, and performs the initial non-empty-diff update with `invalidated = false` when any of the three diff vectors is non-empty.

```cpp
// Do not use the ApplyStatus as we will use the max function when updating apply_status.
unsigned int apply_status = APPLY_STATUS_UNCHANGED;
auto update_apply_status = [&apply_status](bool invalidated)
    { apply_status = std::max<unsigned int>(apply_status, invalidated ? APPLY_STATUS_INVALIDATED : APPLY_STATUS_CHANGED); };
if (! (print_diff.empty() && object_diff.empty() && region_diff.empty())) {
    update_apply_status(false);
    //BBS: add more logs
    BOOST_LOG_TRIVIAL(info) << __FUNCTION__ << boost::format(", got print_diff %1%, object_diff %2%, region_diff %3%, set status to APPLY_STATUS_CHANGED")%print_diff.size() %object_diff.size() %region_diff.size();
}
```

This milestone models only the numeric status initialization, max-update helper semantics, the any-diff gate, and the staged changed-status log metadata as private staged data.

## Exit criteria

- Preserve `apply_status` initialization as unchanged.
- Preserve numeric ordering required by max update: unchanged < changed < invalidated.
- Preserve `update_apply_status(false)` mapping to changed.
- Preserve `update_apply_status(true)` mapping to invalidated.
- Preserve max semantics so invalidated is never downgraded by a later changed update.
- Preserve the initial update gate: update to changed only when at least one of print/object/region diff lengths is non-zero.
- Preserve staged log metadata containing print/object/region diff sizes only when the initial update gate fires.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer lock acquisition from `PrintApply.cpp:1241-1242`, later print/object/region invalidation and status updates, real logging, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
