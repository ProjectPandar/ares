# max_print_z integer-ceiling binding

## Upstream source and audit

OrcaSlicer `src/libslic3r/GCode.cpp:2968-2973` computes the maximum object
layer print_z and stores `max_print_z` as ConfigOptionInt(ceil(maximum)).
This is the only max_print_z binding in GCode.cpp; later start/end templates
reuse shared parser state. End dynamic config :3470-3493 independently
sets floating max_layer_z, not max_print_z. These are distinct placeholders.

Rust has three writes to max_print_z:
- placeholders.rs::insert_runtime_placeholders via base_config computes the
  maximum object height across all objects but currently stores Number.
- machine/start_config.rs::self_start_config calls base_config then overwrites
  max_print_z with first object's raw height; not a distinct upstream binding.
- finish.rs::append calls base_config then overwrites max_print_z with floating
  max_layer_z; not a distinct upstream binding.

All three represent the ONE shared upstream binding, not three upstream
phases. Rust destination is the existing shared base_config/runtime-placeholder
helper; retain one max_print_z computation and insertion there, as Integer of
the ceiled height. Remove both redundant overwrite sites. No new helper or
parallel computation is needed because base_config is already the shared
helper used by start/end. Existing traversal-based height computation is a
temporary geometry compatibility shell; this slice changes binding type/ceil
only and does not redesign layer scheduling or geometry.

## Scope and E2E acceptance

Do not change max_layer_z or geometry/Z arithmetic. No deps, fallback or
output constants. Start/end retain all unrelated bindings.
Real official-exported INPUT3MF control comment `{ceil(max_print_z)}` and
positive `{max_print_z}` in GUI machine_start_gcode exercise the 1.2mm model.
Shared Native stream comparator must pass control and fail positive before
implementation (Native2/Ares1.2 line27), then pass both official/seeded modes.
Existing generated-by normalization only.
