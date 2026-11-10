# Profile application membership and vector shape

## Source and Rust boundary

This slice rewrites applied configuration construction from OrcaSlicer revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`, preserving distinct embedded,
retained-preset, replacement-preset, and final-default stages. It does not add
an independent slicing or configuration pipeline.

The Rust boundary is `project/load.rs`, `project/domain/profile_input.rs`,
`project/domain/profile_input/stl.rs`, `profiles/composition`, and the raw
settings view returned by `profiles/entry.rs::profile_settings`. Existing typed
profile composition is a temporary representation of the upstream dynamic
configuration, not an authority for inventing applied membership or shape.

## Retained versus replacement construction

`Preset.cpp:1059-1088` constructs retained presets from collection defaults,
then applies project values and resolves unchanged values through the system
preset. `OrcaSlicer.cpp:3167-3200` performs this only for printer/process kinds
without replacement settings. The `update_full_config` replacement path
applies the keys actually supplied by the loaded configuration instead.

The Rust selected-profile 3MF entry currently invokes the direct loader before
its selection-aware retained constructor. That direct loader inserts both
kinds' defaults before replacement membership is known; the later constructor
cannot undo those members. Load the embedded raw without retained defaults in
this entry, then use the existing per-kind retained/replaced selection at the
profile-overlay boundary. The direct no-catalog entry retains its construction
order. Catalog-derived refresh continues to resolve system values before
collection defaults. Preserve explicit project values, connection exclusions,
wire defaults, and the separate static `FullPrintConfig` membership contract.

## Complete applied settings view

`OrcaSlicer.cpp` imports embedded configuration before selected file overlays,
maintains compatibility/identity groups separately, and completes missing
static values before slicing. `PresetBundle::full_fff_config` similarly starts
with static defaults and overlays applied preset values. `GCode.cpp:8244-8246,
8338-8352` serializes actual completed dynamic members, omitting banned and nil
values; serialization does not construct a default preset.

`profile_settings` exposes the applied raw configuration consumed by its
selected-profile slice. Do not infer presence from typed defaults or remove
dynamic print-host values through static export membership filtering.

A rewritten 3MF sliced without replacement settings takes Native's retained
path. `Preset.cpp:1059-1088,1787-1792` constructs full default-preset membership;
`OrcaSlicer.cpp:3167-3200` applies that construction when a kind is not replaced.
By contrast, `OrcaSlicer.cpp:3100-3162` applies the actual keys of loaded
replacement configurations, and `:3257-3301` composes loaded filament metadata
and slots. `PresetBundle.cpp:194-213,300-341` likewise distinguishes an applied
configuration from the fields supplied by individual filament presets.

The settings-slice comparisons use the same original input and replacement
inputs on both sides. Native slices the original 3MF or STL with the original
machine, process, and filament files, plus CLI options generated generically
from the existing overrides JSON. `OrcaSlicer.cpp:4138-4221` records changed command-line keys in
`different_settings_to_system` before applying `m_extra_config`. This is the
Native counterpart of Ares `ConfigOverrides` bookkeeping; overlaying values
into a process file would omit that override operation. Ares keeps its original
selection and override call. The ordered-byte comparator and assertions stay
unchanged. Rename the tests to state this application-path contract instead of
suggesting a general settings round trip. Do not rewrite an archive as a proxy
for the original input: Native can change dirty-column membership for that
carrier even under identical replacement inputs.

The comparisons do not independently prove view completeness. That remains
deferred, and the STL view stays sparse. A CONFIG key inventory illustrates the
representation gap: the 3MF view lacks four CONFIG aliases and has 43 additional
members; the sparse STL view has 338 members versus 722 CONFIG members. A future
completeness contract must address final static-default completion, aliases,
nil values, hidden options, and profile metadata using their upstream-owned
boundaries. No additional output keys are filtered here.

Remove the old STL carrier-only `upward_compatible_machine` exception, including
its guard assertions and filtering. `OrcaSlicer.cpp:1769` creates that member
while reading a 3MF; original STL replacement loading does not perform that
read. Compare complete, unfiltered STL outputs with the same ordered-byte
comparator. Delete the archive-rewrite helper once neither comparison uses it.

## Filament vector defaults are not palette cardinality

`PrintConfig.cpp:2369-2379` defines
`activate_air_filtration_during_print` and
`activate_air_filtration_on_completion` as `coBools`, each with a singleton true
default. `Preset.cpp:1508` includes them in `s_Preset_filament_options`;
`PrintConfig.cpp:10150-10220` also includes them in
`filament_options_with_variant`. These lists define ownership and variant
mapping, not an unconditional resize to the selected filament count.

Native composes loaded filament configurations by their actual members using
`ConfigOptionVector::set_at` semantics. A field absent from every supplied
filament remains absent until final static-default completion; its default is
then a singleton, not one independently appended default per loaded filament.
`PrintConfigDef::filament_option_keys` is a distinct list and must not be
replaced by the broader variant-membership list.

`OrcaSlicer.cpp:3851-3866` grows only a short `filament_colour` palette, keeping
its prefix and padding white. The subsequent matrix construction resizes
`flush_volumes_matrix`; it does not resize air-filtration vectors. Final
`GCode.cpp` config serialization preserves their constructed shape.

Correct the typed composition boundary so that missing loaded values do not
become explicit repeated defaults. Reuse source-owned option membership and
canonical defaults rather than matching particular air-filtration names.
Preserve explicitly supplied vectors, per-slot insertion behavior, variant
selection, palette growth, and matrix calculations. Do not repair cardinality
in the config writer or normalize a merged project as an individual preset.

Mixed-filament metadata has a separate explicit cardinality contract:
`PrintConfig.cpp:11475-11488` resizes its seven metadata vectors, and
`OrcaSlicer.cpp:4243-4244` passes the loaded logical filament count for both
arguments. The existing Rust `resize_mixed_filament_metadata` port must run for
STL as well as 3MF loading. This does not justify resizing ordinary missing
filament defaults: the mixed-metadata normalizer, not palette growth or option
type alone, owns those seven vector shapes.

## Public comparisons

Keep these existing comparisons and their assertions unchanged in both Native
modes:

- `runner::profile_entry_3mf::profile_entry_3mf_load_settings`
- `runner::profile_entry_3mf::profile_settings_slice_matches_native_replacement_3mf`
- `runner::profile_entry_3mf::profile_settings_slice_matches_native_replacement_stl`
- `load_settings_3mf_partial::load_settings_3mf_machine_only_whole_file`
- `load_settings_3mf_partial::load_settings_3mf_process_only_whole_file`
- `runner::direct_stl_override_bookkeeping::flush_preparation::short_palette_grows_preserving_prefix`
- `runner::native_jerk_semm_migration::overhang_wall_jerk_matches_native`
- `runner::adjacent_color::processed_layer_templates_preserve_ordered_color_change_items`
- `runner::adjacent_color::sync_delays_survive_color_and_pause_time_flushes`
- `runner::direct_stl_override_bookkeeping::direct_stl_no_override_bookkeeping_control_complete_output`
- `adaptive_octree_spacing::adaptive_octree_spacing_two_region_control`
- `adaptive_octree_spacing::adaptive_octree_spacing_multiline_red`

Adjacent behavior deferred: changing option ownership, system-preset resolution
policy, printer geometry, motion planning, time estimation, and filament-count
validation. No legacy fallback or output-only key whitelist is part of this
slice.
