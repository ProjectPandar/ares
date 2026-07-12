# M302: PrintApply painted region config apply

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the color-painted region update-in-place diff, invalidate, and apply sequence in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:826-828`: compute `region.region->config().diff(cfg)`, call `callback_invalidate(region.region->config(), cfg, diff)`, then call `region.region->config_apply_only(cfg, diff, false)`. Required context comes from the M300 painted config derivation at `PrintApply.cpp:813-820`, the M301 changed/zero-ref update gate at `PrintApply.cpp:821-831`, the existing-region diff/callback/apply sequence at `PrintApply.cpp:801-803`, `PrintRegion::config_apply_only(...)` in `OrcaSlicer/src/libslic3r/Print.hpp:136-139`, and `ConfigBase::apply_only(...)` behavior in `OrcaSlicer/src/libslic3r/Config.cpp:461-500`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned painted-region or slicing pipeline.

## Exit criteria

- Add private staged painted-region config diff behavior for `UpdateInPlace` actions only.
- Preserve that unchanged and requires-reslice painted-region actions produce no diff, invalidate event, or config apply.
- Preserve diff-key ordering by current config order and compare against derived config values.
- Preserve callback-before-apply sequencing for painted-region update-in-place actions, including no apply output when the invalidate event is absent.
- Preserve `config_apply_only(..., false)` semantics by recording `ignore_nonexistent = false` and staged hash refresh metadata through the existing staged apply vocabulary.
- Reuse existing private config diff, invalidate event, and apply vocabulary where possible instead of introducing an Ares-owned pipeline state.
- Add tests for changed zero-ref diff keys, unchanged no-op, requires-reslice no-op, callback-before-apply event creation, apply-only output, apply suppression without an invalidate event, and missing-derived-key ignore behavior.
- Defer painted-region ref increment from `PrintApply.cpp:834`, fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegionConfig`, real `PrintObjectRegions`, real callback execution, real config hash calculation, vector `#` option handling, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
