# Upstream v2.5.0-beta inventory — Config and profiles

Part 1 of the [baseline migration ARD](2026-10-30-upstream-baseline-v2.5.0-beta.md).
Planning base: Ares `e4a12268782154587e164a11bb67f09701599d3b`. Target: upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`; source baseline `v2.4.2` -> `v2.5.0-beta`.

All upstream paths below have prefix `src/libslic3r/`; statuses are Git `M`/`A`/`D`.
`C:` = `crates/ares-core/src/`; `V:` = `crates/ares-vgcode/src/`.
`partial port` identifies an existing bounded 2.4.2 rewrite/counterpart, NOT full-file
coverage or 2.5.0 qualification. `not ported` includes new files, native-only APIs
and helpers without an identified source-port owner; a similarly named type or
comment alone is not evidence of a port. Deleted files retain their old owner
when present. The config owner is the proposed sole upstream-file owner; deferred rows
are tracking ownership, not approval to implement those features.

Rows: **23**. Every listed Rust path was checked at the planning base.

| Status | Upstream file | Current Ares port/counterpart | Scope / gap |
|---|---|---|---|
| M | `AppConfig.cpp` | **not ported** | native desktop application configuration not ported |
| M | `AppConfig.hpp` | **not ported** | native desktop application configuration not ported |
| M | `Config.cpp` | partial port — `C:options/config_types.rs`, `C:options/json_deserialize.rs`, `C:options/config_export.rs` | bounded 2.4.2 slice |
| M | `Config.hpp` | partial port — `C:options/config_types.rs`, `C:options/json_deserialize.rs`, `C:options/config_export.rs` | bounded 2.4.2 slice |
| M | `MaterialType.cpp` | partial port — `C:project_slice/gcode_emit/brim/auto.rs` | bounded material-category consumer |
| M | `ParameterUtils.cpp` | **not ported** | no identified current port |
| M | `ParameterUtils.hpp` | **not ported** | no identified current port |
| M | `Preset.cpp` | partial port — `C:profiles/catalog.rs`, `C:profiles/compatible.rs` | bounded 2.4.2 slice |
| M | `Preset.hpp` | partial port — `C:profiles/catalog.rs`, `C:profiles/compatible.rs` | bounded 2.4.2 slice |
| M | `PresetBundle.cpp` | partial port — `C:profiles/composition.rs`, `C:profiles/catalog.rs`, `C:profiles/import.rs` | bounded 2.4.2 slice |
| M | `PresetBundle.hpp` | partial port — `C:profiles/composition.rs`, `C:profiles/catalog.rs`, `C:profiles/import.rs` | bounded 2.4.2 slice |
| A | `PresetCacheFormat.cpp` | **not ported** | new beta file |
| A | `PresetCacheFormat.hpp` | **not ported** | new beta file |
| M | `PrintApply.cpp` | partial port — `C:project/effective_config/phases.rs`, `C:options/project_variants.rs` | bounded 2.4.2 slice |
| M | `PrintConfig.cpp` | partial port — `C:options.rs`, `C:options/registry.rs`, `C:options/project_fdm_normalization.rs` | bounded 2.4.2 slice |
| M | `PrintConfig.hpp` | partial port — `C:options.rs`, `C:options/registry.rs`, `C:options/project_fdm_normalization.rs` | bounded 2.4.2 slice |
| M | `PrintRegion.cpp` | partial port — `C:print/validate/extrusion_width.rs`, `C:project/region_graph.rs` | bounded 2.4.2 slice |
| A | `PrinterBedAssets.hpp` | **not ported** | new beta file |
| A | `PublishSettings.cpp` | **not ported** | new beta file |
| A | `PublishSettings.hpp` | **not ported** | new beta file |
| M | `Semver.cpp` | **not ported** | no identified current port |
| M | `Semver.hpp` | **not ported** | no identified current port |
| M | `TextConfiguration.hpp` | **not ported** | no identified current port |
