# ARD: literal upstream support-base extension priority

Status: accepted bounded SPEC -> RED -> GREEN.

- Rewrite owner: upstream8500fcdcc `FillBase.cpp:1834–1948::base_support_extend_infill_lines`, invoked by `Fill::connect_base_support:2326` through shared FillSupportBase. Rust boundary: existing `ares-core/src/fill/base_support/extend.rs`; not a new support pipeline.
- `FillBase.cpp:1915` parses as `cond ? prev : (next = -1)`. Previous is never cleared; valid previous wins the subsequent dispatch. Preserve literal behavior, not the comment-derived longest-side interpretation currently used by the incompatible Rust scaffold.
- Change only that candidate-elimination block, obsolete previous mutability and affected comment/module description. No bounds, geometry, marking, trimming, linking, costs, rotation or writer changes. All adjacent behavior is deferred.
- Native output is the sole independent E2E reference. Use unchanged native_stream_case and comparator, full streams and raw API==CLI, all cases/modes Native-first. No internal-helper tests, output masking, fallback or oracle replacement.
- Stack from frozen ccc79c314; preserve qualified reset/spiral changes.

See [SPEC](support-base-extension-priority-spec.md).
