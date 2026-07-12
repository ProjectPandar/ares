# Consume `support_style` runtime design

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:179-181` defines `SupportMaterialStyle` with `smsDefault`, `smsGrid`, `smsSnug`, `smsTreeOrganic`, `smsTreeSlim`, `smsTreeStrong`, and `smsTreeHybrid`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:202-204` defines `is_tree_slim(SupportType, SupportMaterialStyle)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:975` stores `support_style` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:322-331` maps serialized enum strings to `SupportMaterialStyle`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6204-6230` registers `support_style`, accepted enum values, labels, tooltip, and default `smsDefault`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:183-195` resolves `smsDefault` and mismatched explicit styles against `support_type`.
- Representative downstream consumers are `Support/TreeSupport.cpp:642-650`, `Support/SupportMaterial.cpp:619-656`, `Support/SupportCommon.cpp:64-65,1591,1624,1791`, and `Print.cpp:1408-1410,1582-1617`.

## Rust destination boundary

- Add `crates/ares-core/src/options/support_style.rs`.
- Add a `SliceOptions::support_style()` typed accessor as a module-local `impl SliceOptions` inside `support_style.rs`, following existing files such as `parameter_size.rs`, `tree_support_options.rs`, and `support_different_extruders.rs`.
- Reuse `crates/ares-core/src/options/support_type.rs` for the upstream `is_tree_slim()` relationship and support-style resolution helper input.
- Keep LOC compliance explicit: `options.rs` is exactly 400 LOC after the prior slice, so add `support_style` to the existing `option_modules!(...)` line and put the accessor implementation in `support_style.rs` instead of adding lines to `options.rs`. `options/tests.rs` is exactly 400 LOC, so add the new test module name to the existing `#[rustfmt::skip] option_test_modules!(..., support_type, support_z_distance, ...)` line near the top of the file instead of adding a new `mod` line.
- Consume `support_style` in the current early runtime option-validation phase of `crates/ares-core/src/pipeline.rs`, after `support_type` validation and before model loading.
- Add parser and helper tests in `crates/ares-core/src/options/tests/support_style.rs`.
- Add a pipeline guard/no-op test in `crates/ares-core/src/pipeline/tests/support_style.rs`.
- Update `docs/roadmap.md` after implementation with this slice and deferred behavior.

## Behavior to implement

- Parse the canonical Orca strings:
  - `default` -> `SupportStyle::Default`
  - `grid` -> `SupportStyle::Grid`
  - `snug` -> `SupportStyle::Snug`
  - `organic` -> `SupportStyle::TreeOrganic`
  - `tree_slim` -> `SupportStyle::TreeSlim`
  - `tree_strong` -> `SupportStyle::TreeStrong`
  - `tree_hybrid` -> `SupportStyle::TreeHybrid`
- Default missing `support_style` to `SupportStyle::Default`, matching Orca's `smsDefault`.
- Reject non-string values and unknown strings with `SliceError::InvalidInput` messages containing `support_style`.
- Expose helper behavior matching the cited upstream boundary:
  - `is_tree_style()` is true for `TreeOrganic`, `TreeSlim`, `TreeStrong`, and `TreeHybrid`.
  - `resolve_for_support_type()` follows `SupportParameters.hpp:183-195`: default resolves to `TreeOrganic` for any tree support type and to `Grid` for normal support; explicit `Grid`/`Snug` with a tree support type fall back through default to `TreeOrganic`; explicit tree styles with a non-tree support type fall back through default to `Grid`; explicit tree styles with `TreeAuto` or `TreeManual` are preserved.
  - `SupportType::is_tree_slim(style)` matches `PrintConfig.hpp:202-204`: true only when the support type is tree and the resolved style is `TreeSlim`.
- Make `run_slicing_pipeline()` reject invalid `support_style` before model loading.
- Preserve all current generated geometry, print paths, G-code, and diagnostics for every valid value.

## Out of scope

- Do not add new user-facing options.
- Do not implement normal support generation, tree support generation, organic support generation, support blockers, support enforcers, or support material fill behavior.
- Do not route `grid`, `snug`, `organic`, `tree_slim`, `tree_strong`, or `tree_hybrid` into support geometry yet.
- Do not implement `support_base_pattern` or `support_interface_pattern`; they remain separate existing registered options.
- Do not change the existing `support_type` parser beyond adding the `is_tree_slim()` helper needed by this slice.
- Do not add dependencies or new crates.

## Acceptance criteria

- All seven canonical `support_style` strings parse to typed variants.
- Missing `support_style` returns `Default`.
- Invalid strings and non-string JSON values reject with `SliceError::InvalidInput` containing `support_style`.
- The support-style resolution matrix matches `SupportParameters.hpp:183-195` for default style, normal styles on tree support, tree styles on normal support, explicit `TreeSlim` on `TreeAuto`, and explicit `TreeSlim` on `TreeManual`.
- `SupportType::is_tree_slim()` returns true only for tree support types paired with resolved `TreeSlim`.
- `run_slicing_pipeline(b"not a model", &options)` with invalid `support_style` returns the `support_style` validation error before model parsing.
- All seven valid `support_style` values remain no-ops for current Ares slicing output in this slice, proven by comparing generated print paths and G-code/output artifacts for each value against the default baseline.
- Touched Rust files remain at or below 400 LOC.
- Fresh verification includes targeted tests for the new parser/helper and pipeline guard, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, and `cargo check -p ares-core --target wasm32-unknown-unknown`.
