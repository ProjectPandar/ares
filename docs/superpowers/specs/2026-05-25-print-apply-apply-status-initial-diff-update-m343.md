# Spec: M343 PrintApply apply status initial diff update

## Goal

Port the initial `apply_status` setup and non-empty-diff update from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1231-1239` into `ares-core` private staged state.

## Upstream source mapping

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

The Rust staging must model:

- unchanged initial status,
- changed status for non-invalidating updates,
- invalidated status for invalidating updates,
- max-based status aggregation,
- no initial status change when all three diff lengths are zero,
- initial changed status when any of the three diff lengths is non-zero,
- staged log metadata containing print/object/region diff lengths when the initial update fires.

## Non-goals / deferred behavior

- Do not implement lock acquisition from `PrintApply.cpp:1241-1242`.
- Do not implement later `print_diff`, `object_diff`, or `region_diff` invalidation behavior.
- Do not emit real logs.
- Do not perform real `DynamicPrintConfig` or `PrintConfig` lookup/mutation.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Empty print/object/region diffs keep status unchanged and produce no staged log.
- Non-empty print diff updates status to changed and records print/object/region sizes.
- Non-empty object diff updates status to changed and records print/object/region sizes.
- Non-empty region diff updates status to changed and records print/object/region sizes.
- Direct helper update with `invalidated = true` updates status to invalidated.
- Direct helper update with `invalidated = false` does not downgrade an already invalidated status.
- Status discriminants preserve max ordering: unchanged = 0, changed = 1, invalidated = 2.
- All new symbols stay private to `ares-core` staged modules.
