# M193 Spec: PrintConfig extend_extruder_variant API

## Goal
Port OrcaSlicer's `extend_extruder_variant(DynamicPrintConfig&, unsigned int)` helper into Ares as an explicit `SliceOptions` API for UI/config consumers that need to materialize printer extruder variant/id arrays before later `set_num_extruders` resizing behavior is ported.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8558-8591`: `extend_extruder_variant` default `extruder_variant_list` creation, resizing to `num_extruders`, clearing `printer_extruder_id` and `printer_extruder_variant`, comma-splitting each extruder variant-list entry, and appending 1-based extruder ids plus variant strings.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8593-8596`: call-site context where `set_num_extruders` invokes `extend_extruder_variant` before generic option-vector resizing.
- Option-definition anchors: `PrintConfig.cpp:5239-5244` (`extruder_variant_list` default `"Direct Drive Standard"`), `PrintConfig.cpp:5252-5257` (`printer_extruder_id` default), and `PrintConfig.cpp:5259-5264` (`printer_extruder_variant` default).
- Declaration context: `PrintConfig.hpp:634` for `set_num_extruders`.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:8597-8610` generic `set_num_extruders` option-vector resizing through `print_config_def.extruder_option_keys()` and `FullPrintConfig::defaults`.
- `PrintConfig.cpp:8612-8627` `set_num_filaments`.
- `PrintConfig.cpp:8629+` validation, variant override resolution, preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/extruder_variant.rs`: add `SliceOptions::extend_extruder_variant(&mut self, num_extruders: usize) -> Result<(), SliceError>`.
- `crates/ares-core/src/options.rs`: register the new module.
- `crates/ares-core/src/options/tests/extruder_variant.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m193-print-config-extend-extruder-variant-api.md`: milestone sequencing docs.

## Functional requirements

1. Add a public explicit mutating API `SliceOptions::extend_extruder_variant(num_extruders)`.
2. If `extruder_variant_list` is absent, create it as a string array of length `num_extruders`, filled with `"Direct Drive Standard"`.
3. If `extruder_variant_list` is present, require it to be a non-empty string array when `num_extruders > 0`; reject non-array, non-string-member, or empty-present-list values with `SliceError::InvalidInput`.
4. Resize present `extruder_variant_list` to `num_extruders`: truncate extra entries when too long; extend missing entries by cloning the first existing entry, matching upstream's “use first option as default” behavior.
5. Clear/rebuild `printer_extruder_id` and `printer_extruder_variant` regardless of previous values.
6. For each resized `extruder_variant_list[i]`, split the string like `boost::split(..., boost::is_any_of(","), boost::token_compress_on)`: adjacent commas are compressed into one delimiter, but leading, trailing, and fully empty boundary tokens are preserved. Append one 1-based `i + 1` id per produced variant token to `printer_extruder_id`, and append the variant token to `printer_extruder_variant`.
7. Preserve token text exactly; do not trim whitespace and do not drop boundary empty tokens.
8. If `num_extruders == 0`, resize/create `extruder_variant_list` as an empty string array and set both generated arrays to empty arrays.
9. Preserve existing parameter-size API, registry APIs, legacy normalization, and FDM normalization behavior.
10. Do not add generic `set_num_extruders`, `set_num_filaments`, `FullPrintConfig::defaults`, option-vector resizing beyond the three source-boundary keys, UI runtime behavior, slicing, extrusion, G-code behavior, new crates, or dependencies.
11. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove missing `extruder_variant_list` with `num_extruders = 3` creates three `"Direct Drive Standard"` entries and generated id/variant arrays `[1, 2, 3]` / three default variants.
- Tests prove an existing shorter `extruder_variant_list` extends by cloning the first entry.
- Tests prove an existing longer `extruder_variant_list` truncates.
- Tests prove comma-separated variant entries generate repeated 1-based ids and flattened variants, including Boost-compatible edge cases for `""`, `","`, `",A"`, `"A,"`, and `"A,,B"`.
- Tests prove previous `printer_extruder_id` and `printer_extruder_variant` values are cleared/replaced.
- Tests prove `num_extruders = 0` produces empty arrays.
- Tests prove invalid present `extruder_variant_list` values return `SliceError::InvalidInput` without mutating unrelated keys or generated arrays.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8597+` generic resizing and `set_num_filaments` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
