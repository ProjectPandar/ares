# Spec: M302 PrintApply painted region config apply

## Goal

Port OrcaSlicer's color-painted region update-in-place diff, invalidate callback, and config-apply sequence from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:826-828`: compute config diff for a changed zero-ref painted region, call invalidate callback before applying, then apply only the diff with `ignore_nonexistent = false`.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:813-820`: M300 derives the painted config from parent config and painted extruder id.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:821-831`: M301 gates changed painted-region configs by ref count.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:801-803`: existing-region update-in-place branch performs the same diff/callback/apply sequence.
- `OrcaSlicer/src/libslic3r/Print.hpp:136-139`: `PrintRegion::config_apply_only(...)` delegates to config apply-only behavior.
- `OrcaSlicer/src/libslic3r/Config.cpp:461-500`: `ConfigBase::apply_only(...)` copies selected keys and honors `ignore_nonexistent`.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse `StagedExistingRegionUpdateAction`, `StagedConfigValue`, `StagedExistingRegionConfigDiff`, `StagedExistingRegionInvalidateEvent`, and `StagedExistingRegionConfigApply` where practical.
- Add painted-region wrappers/helpers that take the M301 action, current/derived config-key fingerprints for callback payloads, current/derived config value vectors for diff/apply, and the invalidate event `Option` used to gate apply.
- Emit diff keys only for `UpdateInPlace`; `Unchanged` and `RequiresReslice` must produce empty/no-op staged outputs.
- Preserve current-config key order when collecting diff keys.
- Ignore derived-missing keys during diff and apply, matching the existing staged apply-only boundary.
- Emit an invalidate event only for `UpdateInPlace`, before apply in the staged sequence; pass that event `Option` into apply so apply cannot occur without the staged callback event.
- Emit apply-only state only when the update-in-place invalidate event exists, with `ignore_nonexistent = false` and hash refresh recorded.
- Do not perform painted-region ref increment, fuzzy painted-region handling, real callbacks, real `PrintRegionConfig`, real `PrintObjectRegions`, real config hash calculation, vector `#` option handling, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Changed zero-ref painted-region action produces diff keys in current-config order.
- Unchanged painted-region action produces no diff and no invalidate/apply output.
- Requires-reslice painted-region action produces no diff and no invalidate/apply output.
- Update-in-place painted-region action emits invalidate event carrying current config key, derived config key, and diff keys before apply.
- Update-in-place painted-region action passes the invalidate event into apply and emits apply-only output with changed values, `ignore_nonexistent = false`, and hash refresh recorded.
- Apply-only output is absent when the invalidate event is absent.
- Missing derived keys are ignored during painted-region diff/apply.

## Migration note

This milestone stages `PrintApply.cpp:826-828` only. Later milestones must wire the painted-region ref increment at `PrintApply.cpp:834` and fuzzy painted regions at `PrintApply.cpp:837-856` as separate source-cited rewrite slices.
