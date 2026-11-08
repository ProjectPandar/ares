# Project CLI preset-supply parity census

The API project helper and runtime Native receive the supplied system presets.
Every matching Ares CLI invocation must use the same catalog via the existing
`runner::application::CliCommand` helper. Its `output` method adds
`--project-preset-catalog` only when a catalog is supplied and no explicit
preset-loading mode was requested. Comparator and expected bytes are unchanged.

## Direct spawn sites corrected

All 46 sites in these 44 files formerly bypassed that helper. Counts include
the negative override control and the early validation branch. Every test in
each listed module belongs to the runtime control inventory.

| Path relative to `crates/ares-cli/tests/orca_parity/` | Sites |
| --- | ---: |
| `runner/stock_project_native_stream.rs` | 1 |
| `runner/stock_project_native_stream/config_block.rs` | 1 |
| `arachne_fuzzy_f2/public.rs` | 1 |
| `arachne_fuzzy_f1/public.rs` | 1 |
| `wipe_tower2_rib.rs` | 1 |
| `wipe_zero_retraction_comment.rs` | 1 |
| `wipe_retraction_epsilon.rs` | 1 |
| `wipe_before_override.rs` | 1 |
| `support_touching_grid.rs` | 1 |
| `option_coverage/filament_override_range.rs` | 3 |
| `brim_physical_layer.rs` | 1 |
| `brim_consumes_efc_region_polygons.rs` | 1 |
| `shared_mesh_centering.rs` | 1 |
| `arachne_alternate_extra_wall.rs` | 1 |
| `bridge_flow_admission.rs` | 1 |
| `print_object_centering.rs` | 1 |
| `option_coverage/ironing_pattern.rs` | 1 |
| `option_coverage/support_ironing_pattern_domain.rs` | 1 |
| `option_coverage/printer_physical_identity.rs` | 1 |
| `option_coverage/interface_shells_alignment.rs` | 1 |
| `option_coverage/semm_ramming.rs` | 1 |
| `option_coverage/max_travel_detour_distance.rs` | 1 |
| `option_coverage/hole_to_polyhole.rs` | 1 |
| `option_coverage/nozzle_dataset.rs` | 1 |
| `option_coverage/legacy_brim_ears.rs` | 1 |
| `option_coverage/region_speed_emission.rs` | 1 |
| `print_validation/c1_thresholds/run.rs` | 1 |
| `inner_only_brim.rs` | 1 |
| `filament_flow_admission.rs` | 1 |
| `per_object_skirt_min_length.rs` | 1 |
| `gcode_flavor_admission.rs` | 1 |
| `per_object_skirt_min_brim.rs` | 1 |
| `clearance_admission.rs` | 1 |
| `custom_gcode_motion_state.rs` | 1 |
| `per_object_skirt_groups.rs` | 1 |
| `per_object_skirt_combine.rs` | 1 |
| `per_object_skirt_brim.rs` | 1 |
| `per_object_draft_triple.rs` | 1 |
| `outer_inner_brim.rs` | 1 |
| `per_object_draft_shield.rs` | 1 |
| `per_object_draft_compositions.rs` | 1 |
| `outer_only_brim.rs` | 1 |
| `runner/g29_time.rs` | 1 |
| `runner/spiral_layer_acceleration.rs` | 1 |

## Other command paths

- `runner/application/cli_command.rs` already supplies the catalog; retain its
  internal `std::process::Command` constructor. Existing `cli_command()` and
  `CliCommand::new()` callers already inherit the same rule.
- `runner/loaded_system_inherits_group.rs` passes explicit `--load-settings`
  and `--load-filaments`; adding the catalog would conflict with that entry.
- `runner/native_travel_reference_migration.rs` already uses `CliCommand`.
- `runner/scarf_conditional.rs` already uses `cli_command()`; its serialized
  command description is not another process spawn.
- `cli_master_extruder_id.rs` already uses the same helper. Its direct `bin`
  command launches Native, not Ares.
- Native reference, Python/GDB/git, current-test-process replay and taskset
  commands are not Ares parity slice commands. Standalone affinity and dump
  isolation tests compare Ares with itself, without a supplied-profile API or
  Native comparison. Their input premise is unchanged.
- `gcode_moves_oracle.rs` gives Native the G-code already produced by Ares
  for parser comparison; Native does not independently slice the project or
  select system presets. Plain CLI option/output tests have no catalog-fed
  Native/API peer.

## Failure modes

A missing catalog recreates absent preset-owned footer keys. A conflicting
explicit-load argument would change entry semantics. A change to normalization,
assertions, geometry or Native inputs would conceal rather than repair the
premise. The single command-wrapper substitution must alter none of those.
