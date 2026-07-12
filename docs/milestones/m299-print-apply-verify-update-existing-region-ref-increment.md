# M299: PrintApply verify-update existing region ref increment

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `print_region_ref_inc(*region.region);` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:809`, with helper context from `PrintApply.cpp:729`, `PrintRegion::m_ref_cnt` / friend-helper context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, changed-config branch context from `PrintApply.cpp:796-806`, M295 update-action context, and M298 config-apply-before-increment context from `PrintApply.cpp:803`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region lifecycle pipeline.

## Exit criteria

- Add private staged ref-increment sequencing for existing regions after config verification/update completes.
- Preserve that unchanged existing regions increment their ref count.
- Preserve that update-in-place existing regions increment only after the staged config apply result exists.
- Preserve that requires-reslice existing regions do not increment because upstream returns `false` before `PrintApply.cpp:809`.
- Preserve use of the existing M287 `staged_print_region_ref_inc(...)` helper for the actual count mutation.
- Add tests for unchanged increment, update-in-place increment after apply, update-in-place suppression without apply, requires-reslice no-op, and accumulated count behavior.
- Defer real `PrintRegion`, real `PrintObjectRegions`, loop integration, missing-override region creation, painted/fuzzy painted regions, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
