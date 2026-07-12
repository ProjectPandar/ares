# M305: PrintApply fuzzy painted region update/apply

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the fuzzy-skin painted-region configuration comparison and update/apply block in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:843-853`: compare the derived fuzzy-painted `PrintRegionConfig` with `region.region->config()`, update in place only when `print_region_ref_cnt(*region.region) == 0`, emit `diff`, invoke `callback_invalidate(...)`, call `region.region->config_apply_only(cfg, diff, false)`, and return `false` when the destination region is already referenced. Required prefix context comes from M304's `PrintApply.cpp:837-842` derivation, `PrintObjectRegions::FuzzySkinPaintedRegion` in `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, the destination `PrintRegion *region` field in that structure, `PrintRegion::config()` / `config_apply_only(...)` and `m_ref_cnt` context in `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, and the shared config diff / invalidation / apply pattern already ported for volume and color-painted regions from `PrintApply.cpp:786-800` and `PrintApply.cpp:821-833`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned fuzzy-skin or slicing pipeline.

## Exit criteria

- Add private staged fuzzy-painted config change detection comparing the current destination region config with the M304 derived config.
- Preserve the upstream gate: unchanged configs do nothing, changed zero-ref regions update in place, and changed referenced regions require reslice.
- Preserve callback/update sequencing for update-in-place: compute diff, stage invalidation payload from current to derived config, then stage apply-only state with `false`/no full invalidation behavior inherited from the shared staged helper.
- Preserve diff key behavior through the existing staged config diff/apply helper instead of introducing fuzzy-specific diff semantics.
- Preserve fuzzy painted region id, parent reference, and destination region metadata when carrying a change result.
- Add tests in a separate focused fuzzy-painted update module for unchanged zero-ref and referenced regions, changed zero-ref update-in-place, changed referenced requires-reslice, metadata/config preservation, diff key order, skipped apply for unchanged/requires-reslice, invalidation payload before apply, and apply-only payload.
- Defer fuzzy-painted `print_region_ref_inc(*region.region)` from `PrintApply.cpp:856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
