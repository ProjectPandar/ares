# M48 Spec: PrintConfig cooling and default profile option registry slice

## Goal
Port the adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` cooling/default-profile option-definition slice into `ares-core` option registry metadata, covering layer-cooling slowdown, default acceleration/profile fields, air-filtration toggles, exhaust-fan speeds, and first-layer fan close count without changing cooling, acceleration, profile selection, slicing, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1772-1845`: `PrintConfigDef::init_fff_params()` option definitions for this slice.

Related upstream behavior explicitly deferred:

- Cooling slowdown behavior and layer-time calculations.
- Acceleration planning and G-code acceleration emission.
- Default filament/process profile selection behavior.
- Air-filtration and exhaust-fan G-code behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1847-1861`: `bridge_no_support` and `thick_bridges` are already registered and not changed by this milestone.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1863+`: `thick_internal_bridges` and following options.
- UI labels, mode behavior, CLI no-CLI enforcement, filesystem/network integrations, slicing, extrusion, and G-code behavior.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definitions for keys that sort into the early shard, including `complete_print_exhaust_fan_speed` after the `compatible_*` keys and before `cool_plate_temp`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definition for `slow_down_for_layer_cooling`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/profile.rs`: focused metadata assertions for cooling/default-profile keys.
- `crates/ares-core/src/options/tests.rs` plus `options/tests/registry_helpers.rs` and a new focused lookup test file: keep each modified Rust file under 400 LOC while preserving public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `slow_down_for_layer_cooling` (`coBools`, default `true`, lines 1772-1777)
- `default_acceleration` (`coFloat`, default `500`, lines 1779-1786)
- `default_filament_profile` (`coStrings`, default empty strings list, lines 1788-1792)
- `default_print_profile` (`coString`, default empty string, lines 1794-1798)
- `activate_air_filtration` (`coBools`, default `false`, lines 1800-1804)
- `activate_air_filtration_during_print` (`coBools`, default `true`, lines 1807-1811)
- `activate_air_filtration_on_completion` (`coBools`, default `true`, lines 1813-1817)
- `during_print_exhaust_fan_speed` (`coInts`, default `60`, lines 1819-1826)
- `complete_print_exhaust_fan_speed` (`coInts`, default `80`, lines 1828-1835)
- `close_fan_the_first_x_layers` (`coInts`, default `1`, lines 1837-1845)

The current `OptionDefinition` default representation for one-element bool/int vectors uses the existing scalar string default (`"true"`, `"false"`, `"60"`, `"80"`, `"1"`), matching existing `Bools`/`Ints` registry conventions.

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::Bools`, `Float`, `Strings`, `String`, and `Ints`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Keep modified Rust files under 400 LOC; if adding lookup cases would push `registry_helpers.rs` over the limit, move the public lookup test to a new focused `registry_lookup.rs` module before adding M48 cases.
7. Do not add typed parsing/accessors, cooling slowdown behavior, acceleration planning, default-profile selection behavior, air-filtration behavior, exhaust-fan G-code behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not alter existing `bridge_no_support` or `thick_bridges` metadata and do not add `thick_internal_bridges` or following options from `PrintConfig.cpp:1863+`.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M49, or verify those docs if the rename already exists in the current worktree.

## Deferred behavior

- Upstream label/category/tooltip/sidetext/min/max/mode/CLI metadata from `PrintConfig.cpp:1772-1845` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Cooling slowdown, acceleration planning, default profile selection, air-filtration fan control, exhaust fan G-code behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- Existing `bridge_no_support` and `thick_bridges` metadata remains unchanged.
- `thick_internal_bridges` and following options from `PrintConfig.cpp:1863+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all ten new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all ten new keys.
- Plan/spec explicitly account for deferred upstream UI metadata, CLI no-CLI enforcement, cooling behavior, acceleration behavior, default profile behavior, air-filtration/exhaust fan G-code behavior, slicing/extrusion/G-code behavior, existing bridge options, and following `thick_internal_bridges` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
