# M263: PrintApply full print-config diff branch

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `full_print_config_diffs(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:269-294`, with return context from `PrintApply.cpp:294`, comment/context from `PrintApply.cpp:267-268`, and wipe tower option-definition context from `PrintConfig.cpp:6694-6708`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add an internal `ares-core` helper that stages `full_print_config_diffs(...)` over JSON option maps.
- Preserve upstream loop behavior: iterate new full config keys in caller-provided order, append keys when the old full config lacks the key, suppress equal old/new values, append ordinary changed keys, and apply the same `wipe_tower_x` / `wipe_tower_y` plate-index comparison semantics as M262 when old values exist.
- Add focused tests for missing-old insertion, equal-value suppression, ordinary changed-key insertion, wipe-tower indexed comparison, wipe-tower one-sided index presence, and wipe-tower missing-old insertion.
- Do not implement public `full_print_config_diffs` wiring, `PrintApply::print_config_diffs` public wiring, print config mutation, config apply/apply_only, placeholder parser updates, profile loading, public API wiring, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
