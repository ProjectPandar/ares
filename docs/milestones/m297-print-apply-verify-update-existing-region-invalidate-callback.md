# M297: PrintApply verify-update existing region invalidate callback

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `callback_invalidate(region.region->config(), cfg, diff);` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:802`, with callback signature context from `PrintApply.cpp:734-741`, M296 diff-key context from `PrintApply.cpp:801`, and update-in-place branch context from `PrintApply.cpp:798-803`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned invalidation pipeline.

## Exit criteria

- Add private staged invalidation callback event data for the existing-region update-in-place branch.
- Preserve callback argument order: current config first, derived config second, diff keys third.
- Preserve that the callback is emitted only for the M295 update-in-place action after M296 diff keys are available.
- Preserve that unchanged and requires-reslice actions do not emit callback events.
- Preserve that an empty diff-key vector is still passed through when the staged action is update-in-place.
- Add tests for argument order, diff-key preservation, empty update-in-place diff callback, unchanged action suppression, and requires-reslice action suppression.
- Defer `config_apply_only(...)` from `PrintApply.cpp:803`, real callback invocation, background-process cancellation, real `ConfigBase` / `PrintRegionConfig`, config mutation, hashing, ref-count increment, derived config source selection, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
