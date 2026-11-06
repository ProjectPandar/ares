# Multi-tool ramming filament variant gathering

## Owning upstream boundary

Pinned OrcaSlicer revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This is a Rust rewrite slice of libslic3r filament configuration materialization,
not a new ramming policy or an Ares-owned slicing pipeline.

- `src/libslic3r/PrintConfig.hpp:1899–1901` declares
  `filament_multitool_ramming` as `ConfigOptionBools`, and
  `filament_multitool_ramming_volume` / `filament_multitool_ramming_flow` as
  `ConfigOptionFloats`. `PrintConfig.cpp:3475–3498` defines their ordinary,
  nonnullable singleton defaults: false, 10 mm³ and 10 mm³/s.
- `PrintConfig.cpp:10211–10213` includes all three in
  `filament_options_with_variant`.
- `PrintConfig.cpp:11884–11994`,
  `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments`,
  resolves one source index per logical filament from the existing filament
  map, self-index and extruder/volume variant metadata. Its dispatch at
  `11970–11977` gathers floats and bools through the same helper.
- `PrintConfig.cpp:11862–11882`, `gather_option_values`, clones the selected
  source payload, or the original vector front when the index is out of range.
  An empty source is skipped by this helper; registered-default preparation
  before gathering is a separate responsibility.
- `PrintApply.cpp:1436–1456` applies the static filament-key materialization.
  `Print.cpp:4723–4745`, `Print::update_filament_maps_to_config`, rebuilds the
  same filament-key set and publishes materialized values on map writeback.
- `PrintConfig.cpp:10836–10857`, `support_different_extruders`, admits a single
  physical nozzle when its declared group contains multiple variant tokens.
  Physical nozzle count does not determine the number of logical filament
  payloads gathered.

`GCode/WipeTower2.cpp:1192–1217` consumes the triplet for non-SEMM ramming,
while SEMM uses `filament_ramming_parameters`. `GCode.cpp:1698–1700` reads the
bool for tower travel and `10810–10812` remaps the triplet into placeholders.
These consumers are context, not additional algorithms to port in this slice.

## Exact Rust destination and decisions

- Add the three members to
  `crates/ares-core/src/options/registry/key_lists.rs::FILAMENT_OPTIONS_WITH_VARIANT_KEYS`.
  They are absent from the current list.
- Gather the three existing `settings.filament.gcode` vectors in
  `crates/ares-core/src/options/project_variants/filament.rs::materialize`,
  using its wrapped-field path through the existing shared
  `filament_select.rs::select_filament` helper.
- **All three list entries and all three gathers are one atomic production
  change.** List-only registration changes load normalization without completing
  runtime materialization; gather-only registration leaves the upstream key-set
  ownership incomplete. Neither intermediate state is a valid rewrite slice.
- Both `materialize_project_variants` and `refresh_filament_map` already route
  through this materializer. Preserve their order, checkpoints and selected
  index sequence. Do not introduce a ramming-specific index formula or change
  the activation guard.
- Keep ordinary bool/float storage and defaults in
  `options/filament_options/gcode_source.rs`, existing decode/profile/wire
  bindings and footer exports unchanged. The current singleton-valued storage
  is only a temporary compatibility shell until this upstream materialization
  boundary is complete, not a new Ares option-selection design.
- Core owns the transformation for all API consumers on WASM, Windows, macOS
  and Linux. No CLI duplicate algorithm, footer token patch, physical-nozzle
  resize, false/zero synthesis or payload coercion is permitted.

## Included and deferred behavior

Included: the three ordinary filament keys, existing source-index selection,
selected/front-fallback payload gathering and existing initial/refresh dispatch.
Source order and logical filament cardinality are preserved. The shared helper's
empty-source no-op and pre-existing default preparation remain unchanged.

Deferred: SEMM and non-SEMM motion algorithms, positive ramming enablement policy,
FlushMode processing, dynamic/multiple-slot nozzle producers, registered-default
preparation changes, nullable behavior, single-variant activation/load normalization,
other filament keys and multi-nozzle map production. Adjacent cooling-fan and
recommended-temperature gathers and static-volume-map activation retain their
separate ownership.

## Public witnesses and failure modes

Use two existing members of the seven-ID first-difference cluster without
changing their fixtures, overrides, labels or original complete-stream checks:

- `runner::plate_custom_gcode_emission::plate_custom_gcode_emission_multi_as_single_matches_native`:
  two logical filaments with model activity on filament 1; distinguish logical
  filament cardinality from active printing tools.
- `vg1_flush_state::vg1_x1c_direct_command_matches_native`:
  retain the existing explicit tool-command Native witnesses and exercise the
  same configuration boundary without changing FlushMode semantics.

The public seams are Native output and the existing byte-input core slicing
application adapter. Persist the input archive, full Native/API output and
labelled three-field comparison before assertions. First require Native's exact
lines `; filament_multitool_ramming = 0,0`,
`; filament_multitool_ramming_flow = 10,10` and
`; filament_multitool_ramming_volume = 10,10`; then require each complete API
line to equal Native. Keep the original ordered-byte comparator and assertion;
labelled equality is not a substitute for complete-stream parity.

Failures include: singleton output despite two logical filaments; resizing by
physical nozzle or active tool count; correcting only the bool; list-only or
gather-only registration; losing source values/order or replacing front fallback
with false/zero. Failed Native setup is not evidence of this materialization gap.
The selected witnesses establish singleton/front-fallback cardinality for all
three fields, not distinct nondefault variant payloads or enabled ramming-motion
parity. They do not establish whole-stream parity for the other five IDs.
