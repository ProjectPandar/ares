# M298: PrintApply verify-update existing region config apply-only

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `region.region->config_apply_only(cfg, diff, false);` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:803`, with `PrintRegion::config_apply_only(...)` context from `OrcaSlicer/src/libslic3r/Print.hpp:136-139`, `ConfigBase::apply_only(...)` behavior from `OrcaSlicer/src/libslic3r/Config.cpp:461-500`, M296 diff-key context from `PrintApply.cpp:801`, and M297 callback-before-apply context from `PrintApply.cpp:802`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned config pipeline.

## Exit criteria

- Add private staged config apply-only state for the existing-region update-in-place branch.
- Preserve that apply-only runs only after the M297 staged invalidate event exists.
- Preserve applying diff keys in order by copying matching values from derived config into current config.
- Preserve `ConfigBase::apply_only(...)` behavior that keys missing from the derived config leave current config unchanged.
- Preserve `ignore_nonexistent = false` as staged metadata for this source line.
- Preserve `PrintRegion::config_apply_only(...)` hash-refresh context as a staged `hash_refreshed` marker.
- Add tests for value replacement, diff-key order with duplicate keys, missing-derived-key no-op, no apply without invalidate event, `ignore_nonexistent = false`, and hash-refresh marker.
- Defer real `ConfigBase` / `PrintRegionConfig`, vector `#` option handling, unknown-option exceptions, real config hash calculation, real `PrintRegion` mutation, ref-count increment from `PrintApply.cpp:809`, derived config source selection, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
