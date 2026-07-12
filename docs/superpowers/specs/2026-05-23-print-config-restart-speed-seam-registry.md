# M106 Spec: PrintConfig restart, retraction speed, M73, and seam registry slice

## Goal
Port the adjacent restart-extra, retraction/deretraction speed, firmware retraction, calibration mark, M73 disable, seam-position, staggered-inner-seam, and seam-gap option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1382`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5306-5312`: `retract_restart_extra` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1383`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5314-5320`: `retract_restart_extra_toolchange` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1384`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5322-5328`: `retraction_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1296`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5330-5336`: `deretraction_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1417`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5338-5343`: `use_firmware_retraction` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1424`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5345-5349`: `bbl_calib_mark_logo` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1425`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5351-5355`: `disable_m73` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:211-213`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:944`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:350-357`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5357-5373`: `seam_position` enum map and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:945`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5375-5380`: `staggered_inner_seams` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1182`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5382-5390`: `seam_gap` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode/category/cli metadata beyond the current registry boundary.
- Restart compensation, retraction/deretraction motion planning, firmware-retraction G10/G11 G-code generation, M73 generation suppression, Bambu calibration mark generation, seam placement, staggered inner seam geometry, and seam-gap geometry behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5392+`: `seam_slope_type`, scarf-seam conditional options, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `deretraction_speed` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add `bbl_calib_mark_logo` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `disable_m73` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `retract_restart_extra`, `retract_restart_extra_toolchange`, and `retraction_speed` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `seam_gap` and `seam_position` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add `staggered_inner_seams` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add `use_firmware_retraction` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all ten definitions.
- `docs/roadmap.md` and `docs/milestones/m106-print-config-restart-speed-seam-registry.md`: milestone sequencing docs.

## Included option definitions

- `retract_restart_extra` (`coFloats`, default `0`, field at `PrintConfig.hpp:1382`, definition lines 5306-5312, Ares kind `Floats`)
- `retract_restart_extra_toolchange` (`coFloats`, default `0`, field at `PrintConfig.hpp:1383`, definition lines 5314-5320, Ares kind `Floats`)
- `retraction_speed` (`coFloats`, default `30`, field at `PrintConfig.hpp:1384`, definition lines 5322-5328, Ares kind `Floats`)
- `deretraction_speed` (`coFloats`, default `0`, field at `PrintConfig.hpp:1296`, definition lines 5330-5336, Ares kind `Floats`)
- `use_firmware_retraction` (`coBool`, default `false`, field at `PrintConfig.hpp:1417`, definition lines 5338-5343, Ares kind `Bool`)
- `bbl_calib_mark_logo` (`coBool`, default `true`, field at `PrintConfig.hpp:1424`, definition lines 5345-5349, Ares kind `Bool`)
- `disable_m73` (`coBool`, default `false`, field at `PrintConfig.hpp:1425`, definition lines 5351-5355, Ares kind `Bool`)
- `seam_position` (`coEnum`, default `aligned`, enum at `PrintConfig.hpp:211-213`, field at `PrintConfig.hpp:944`, enum map at `PrintConfig.cpp:350-357`, definition lines 5357-5373, Ares kind `Enum`)
- `staggered_inner_seams` (`coBool`, default `false`, field at `PrintConfig.hpp:945`, definition lines 5375-5380, Ares kind `Bool`)
- `seam_gap` (`coFloatOrPercent`, default `10%`, field at `PrintConfig.hpp:1182`, definition lines 5382-5390, Ares kind `FloatOrPercent`)

## Functional requirements

1. Add the ten missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, restart/retraction behavior, firmware-retraction behavior, M73 behavior, calibration-mark behavior, seam placement behavior, staggered-seam behavior, seam-gap geometry behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add following scarf seam options from `PrintConfig.cpp:5392+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the ten new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all ten covered definitions.
- Plan/spec explicitly account for deferred UI metadata, restart/retraction runtime behavior, firmware/M73/calibration behavior, seam runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5392+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
