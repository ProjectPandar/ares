# Applied process variant vectors

## Source boundary

This is a source-cited Rust rewrite slice of `libslic3r` at
`345efec64c7b95a05ce5cf96df0dde66a158835c`. It completes process-vector
materialization through the existing configuration adapters.

- `PrintConfig.cpp:10102–10148` defines `print_options_with_variant`.
- `PrintConfig.cpp:11661–11862`, `DynamicPrintConfig::update_values_to_printer_extruders`,
  resolves source variant indices and gathers the requested option vectors.
- `PrintApply.cpp:1423–1434` applies printer variants, then process variants with
  `print_options_with_variant`, `print_extruder_id` and `print_extruder_variant`.
  It retains the resulting pre-filament checkpoint in `m_ori_full_print_config`.
- `PresetBundle.cpp:5561–5582` uses the same process producer when
  `full_fff_config` applies extruder variants. `OrcaSlicer.cpp:6985` sends the
  prepared dynamic configuration through `Print::apply` for slicing.
- `GCode.cpp:8244–8247,8342–8352` serializes the applied full configuration.

The producer walks emitted extruder/nozzle-volume slots and stores their source
indices. Its type-specific gathers allocate `variant_count * stride` values.
Process selection uses stride one. `Config.hpp:676–680` returns an indexed source
value when present and the first value when the index is out of range.

Thus `[50]` selected for two slots becomes `[50,50]`, but different source
variants `[50,200]` must retain the selected `50` and `200`. Neither logical
filament count nor a constant two determines the output length. Full applied
configuration means the selected slots, not every unselected preset column.

## Existing Rust producer and missing coverage

The generic producer is already present in
`options/update_printer_extruders.rs::SliceOptions::update_values_to_printer_extruders_string_int_keys`.
Its multiple-filament counterpart is in
`options/update_printer_extruders/multiple_filament.rs`. Both resolve indices and
gather registered values. The multiple-filament overload gathers per logical
filament and is not the process-vector call at `PrintApply.cpp:1429`.

The missing process data has three concrete causes in the existing adapters:

1. `options/registry/key_lists.rs::PRINT_OPTIONS_WITH_VARIANT_KEYS` contains only
   `print_extruder_id` and `print_extruder_variant`. The exposed
   `print_options_with_variant()` therefore omits every value key in this slice.
   This is an incomplete port of the upstream key list.
2. The bounded slicing path calls
   `options/project_variants.rs::materialize_project_variants`, not the generic
   `SliceOptions` gather. Its `project_variants/printer.rs::materialize_process`
   selects an explicit subset of typed fields. That subset omits all 22 keys
   below, even though `project_variants/index.rs::resolve_process_indices`
   supplies the selected process indices. Expanding the registry list alone
   does not connect these values to the slicing result.
3. These fields still have scalar typed representations. For example,
   `options/region_fields.rs:14–15` declares scalar `bridge_speed` and
   `internal_bridge_speed`. `registry/definitions/table/early.rs` registers
   `bridge_speed` as scalar `Float`, while upstream `PrintConfig.cpp:2043–2054`
   registers `coFloats`. The generic producer dispatches numeric vectors through
   `Floats`/nullable-vector kinds and skips scalar `Float`. Adding a missing key
   without correcting its representation cannot preserve its source columns.

This is **key-list and typed-projection coverage**, not absence of a gather
algorithm. The generic public producer and the typed apply adapter must retain
one source contract. A second process optimizer or footer reconstruction is not
part of this boundary.

### The activation guard is not the missing two-head call

`project_variants/index.rs::resolve_activation` implements the extruder branch
as `physical_count > 1 || tokens.len() > 1`, using the printer's variant groups.
`materialize_project_variants` calls process-column synthesis, resolves process
indices, and invokes `materialize_process` inside that branch. Two physical
heads enter it regardless of whether their values are equal. The 22 fields are
absent from the called adapter, rather than excluded by its activation guard.

One-head, one-variant input keeps the inactive process path. Multiple preset columns
must not themselves force every column into the applied output. Existing
process-column synthesis and index selection remain the owners of that choice.

## Which configuration reaches the footer

`PrintBase.hpp:518` returns `const DynamicPrintConfig&` referring to
`PrintBase::m_full_print_config`. `GCode::append_full_config` copies that exact
object into local `cfg` at `GCode.cpp:8244–8246`. It does not serialize the static
`FullPrintConfig` defaults or directly serialize `Print::m_config`.

`PrintApply.cpp:1674–1676` assigns post-materialization `new_full_config` to
`m_full_print_config`. The `m_ori_full_print_config` checkpoint is distinct.
`Print.cpp:4722–4726` can rebuild the applied full snapshot from that checkpoint
and rematerialize filament variants after a filament-map update. The checkpoint
must already retain the selected process vectors.

Ares has the corresponding applied-snapshot role, but incomplete values:

- `project/effective_config/phases.rs::resolve_with_observer` takes
  `MaterializedProjectVariants.full` into `first` before resolving
  `ProjectConfigViews`. It does not pass the pre-update input to the writer.
- `options/project_config_views.rs::resolve_project_config_views` retains that
  object as `views.full`. It clones a separate `runtime` and applies retraction
  overrides there. `views.runtime` is not the full-config export source.
- `options/config_export/writer.rs::write_config_block` transforms `views.full`,
  then calls `collect_config_entries` and `full_config::write_entries`.
- `config_export/collector.rs` collects typed process values.
  `process_options/region_source/wire.rs` serializes the scalar bridge field.
  `config_export/full_config.rs::write_entries` emits that token and skips raw
  reinsertion of an already emitted key.

The footer therefore reads the correct post-update **view role**, not a stale
pre-update view. Its `ProjectSettings` projection is still a temporary
compatibility shell for upstream's dynamic applied snapshot. Scalar values
left untouched by process materialization cannot become vectors by choosing a
different view or by changing full-config membership.

## Included fields and Rust destination

| Family | Keys |
|---|---|
| Speeds | `bridge_speed`, `gap_infill_speed`, `initial_layer_infill_speed`, `initial_layer_speed`, `initial_layer_travel_speed`, `internal_bridge_speed`, `internal_solid_infill_speed`, `overhang_1_4_speed`, `overhang_2_4_speed`, `overhang_3_4_speed`, `overhang_4_4_speed`, `small_perimeter_speed`, `sparse_infill_speed`, `support_interface_speed`, `support_speed`, `top_surface_speed`, `travel_speed`, `travel_speed_z` |
| Jerk/junction | `default_junction_deviation`, `initial_layer_travel_jerk` |
| Threshold | `small_perimeter_threshold` |
| Flow | `top_solid_infill_flow_ratio` |

The code destination is the existing `ares-core` configuration boundary:

- Restore source membership in `options/registry/key_lists.rs`, preserving the
  existing public `print_options_with_variant()` authority.
- Replace the included scalar compatibility fields in `options/region_fields.rs`,
  `object_fields.rs` and `gcode_fields.rs` with source-appropriate vector values.
  Their existing process-option wire/assembly and registry kind declarations
  must preserve numeric versus percentage values and nullable semantics.
- Complete `options/project_variants/printer.rs::materialize_process` for those
  fields using the indices from `project_variants/index.rs` and the existing
  `project_variants/select.rs::select_stride`. Preserve actual selected values
  in both the full result and the pre-filament checkpoint.
- Keep `project/effective_config/phases.rs`, `project_config_views.rs` and
  `config_export/writer.rs` as the existing handoff and consumer boundaries.
  No additional retained view, copied geometry, decoder or export state is needed.

The generic producer is not reimplemented. Its existing vector dispatch and
index/gather semantics remain the reference for the typed adapter. Existing
acceleration/jerk vectors, wall speeds, toggles, selector columns and filament
materialization stay intact. Registry kind correction is distinct from the
`full_config_default` membership flags, which remain unchanged.

Related boundaries are [process-column synthesis](process-variant-columns.md),
[acceleration and jerk](ard-0111-acceleration-jerk-variant-materialization.md), and
[applied full-config membership](full-config-residual-membership.md).

## Invariants and verification boundary

- Preserve per-variant values. The discriminating two-slot High Flow output is
  `bridge_speed = 50,200`, not `50,50` or `50`.
- Preserve shared-value gathering. Two selected slots from one source value
  serialize both values. A single selected slot does not gain extra entries.
- Preserve percentage tokens and nullable entries through gathering. In
  particular, `initial_layer_travel_speed` and `initial_layer_travel_jerk` are
  upstream `coFloatsOrPercents`, not ordinary numeric scalars.
- Preserve the selected process vectors when filament variants are refreshed.
  Filament ids, physical extruders, process columns and nozzle slots stay distinct.
- Public Native CLI, validated core API and CLI inputs must expose each included
  field at the applied config boundary before implementation changes. Retain
  inputs, selected profiles, raw outputs and complete-output comparisons.
- Check one-head inactive controls, equal two-head values, nonuniform variants,
  explicit multi-column source values and existing vectorized-family controls.
  Body comparisons must retain unrelated differences rather than mask them.

Dynamic grouping, physical multi-nozzle selection, loaded inventory, timing,
statistics, geometry and motion/planner changes are outside this slice.
`default_junction_deviation` vector preservation does not change the established
JD planner. Matrix reconstruction, footer ordering, banned-key/nil rules,
static/default membership and raw-input serialization policies remain unchanged.
