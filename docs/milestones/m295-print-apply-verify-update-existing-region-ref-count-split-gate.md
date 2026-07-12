# M295: PrintApply verify-update existing region ref-count split gate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the existing changed-region ref-count decision in `OrcaSlicer/src/libslic3r/PrintApply.cpp:798-806`, with the preceding config-change predicate from `PrintApply.cpp:796`, M287 `print_region_ref_cnt(...)` context from `PrintApply.cpp:729-731`, and `PrintRegion::m_ref_cnt` context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper that maps an existing-region config-change result plus current staged print-region ref count to one of three outcomes: unchanged, update existing region in place, or requires reslice because the changed region is already referenced.
- Preserve that unchanged configs do not enter the update/split branch regardless of ref count.
- Preserve that changed configs with `print_region_ref_cnt(...) == 0` are eligible for in-place region parameter update.
- Preserve that changed configs with nonzero ref count require reslice for the split branch.
- Add tests for unchanged config, changed zero-ref update-in-place, changed positive-ref reslice, and unchanged positive-ref behavior.
- Defer `t_config_option_keys diff = region.region->config().diff(cfg)` from `PrintApply.cpp:801`, `callback_invalidate(...)` from `PrintApply.cpp:802`, `config_apply_only(...)` from `PrintApply.cpp:803`, `print_region_ref_inc(...)` from `PrintApply.cpp:809`, derived config source selection, real config merge internals, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
