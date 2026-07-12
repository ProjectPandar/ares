# M296: PrintApply verify-update existing region config diff keys

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `t_config_option_keys diff = region.region->config().diff(cfg);` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:801`, with `ConfigBase::diff(...)` behavior from `OrcaSlicer/src/libslic3r/Config.cpp:518-528`, `t_config_option_keys` alias context from `OrcaSlicer/src/libslic3r/Config.hpp:73-75`, and M295 update-in-place branch context from `PrintApply.cpp:798-803`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned config pipeline.

## Exit criteria

- Add private staged config-diff-key data for the existing-region update-in-place branch.
- Preserve that diff keys are produced only for the M295 update-in-place action.
- Preserve `ConfigBase::diff(...)` key order by using the current config's key order.
- Preserve duplicate current keys without sorting or deduplication.
- Preserve `ConfigBase::diff(...)` intersection semantics: keys absent from either config are ignored.
- Preserve that equal values do not produce diff keys.
- Add tests for current-key order, duplicate current-key preservation, ignored missing keys on either side, equal-value suppression, and no diff keys for unchanged/reslice actions.
- Defer `callback_invalidate(...)` from `PrintApply.cpp:802`, `config_apply_only(...)` from `PrintApply.cpp:803`, real `ConfigBase` / `PrintRegionConfig` storage, config option value typing, hashing, ref-count increment, derived config source selection, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
