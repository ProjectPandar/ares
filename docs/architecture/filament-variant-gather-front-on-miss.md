# Filament variant gather front-on-miss

## Owning upstream boundary

Pinned OrcaSlicer revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This rewrite concerns libslic3r's gathering of existing filament option values,
not a new retraction policy or writer algorithm.

- `src/libslic3r/PrintConfig.cpp:11862–11882`,
  `gather_option_values<OptType, ValueType>`: reserve one output per selected
  source index; copy that source element when present; otherwise copy the
  original vector front. Preserve the actual payload, including nullable nil.
  Missing or empty options are skipped by this helper, not synthesized as zero.
- `PrintConfig.cpp:11884–11948`,
  `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments`:
  resolve each filament's variant source index from the filament map, active
  extruder/volume type and self-index metadata. The existing resolver remains
  the owner of source-index selection.
- `PrintConfig.cpp:11965–11981`: the caller prepares empty vectors from the
  registered default and dispatches supported string, integer, float, percent,
  float-or-percent, bool and enum vectors to the same gather helper. This slice
  addresses nonempty vectors; empty/default preparation is a separate boundary.
- `src/libslic3r/PrintApply.cpp:1436–1455`: materialize the static per-filament
  variant arrays before computing runtime print-config differences. Dynamic
  persisted-selector expansion in the neighboring branch remains separate.
- `PrintApply.cpp:284–318`, `print_config_diffs`: route filament retract
  overrides to `compute_filament_override_value`, using the mapped extruder
  index for each static filament.
- `PrintConfig.cpp:12385–12417`, `compute_filament_override_value`: clone the
  machine option and apply the nullable filament override. No special command
  insertion occurs at this stage.
- `src/libslic3r/Config.hpp:767–815`, `ConfigOptionVector::apply_override`:
  explicit values override the machine value; nil keeps the mapped machine
  default. A nil and an explicit numeric zero are different values.
- `src/libslic3r/GCodeWriter.cpp:1457–1465,1479–1503`: the normal retract path
  consumes the configured length and speed, emitting only for nonzero retract
  amount. `Extruder.cpp:109–118` tracks that amount. These consumers are not
  being rewritten by this slice.

## Exact Rust destination

Replace the zero-on-missing-index shell in
`crates/ares-core/src/options/project_variants/filament_select.rs::select_filament`
with the cited gather behavior for nonempty vectors. The current
`FilamentZero` implementation constructs `Nullable::Value(0)` on a miss, losing
nil and treating an absent source column as an explicit disabling override.
Remove only zero-construction machinery made unused by this replacement.

`project_variants/filament.rs::materialize` already routes filament values
through this shared gather helper. Keep the existing index resolver,
materialization ordering and selected-index list. Do not add a retraction-only
index formula, special-case the witness or independently reconstruct nil after
selection. Front-on-miss applies to all existing payload types dispatched
through this source-backed helper, not merely retraction length.

`options/project_config_views/retract.rs::apply_nullable` and
`project_slice/gcode_emit/motion/options/build/fields/retraction.rs::apply`
consume the resulting values. Their current machine-default/explicit-override
and indexed getter behavior remains unchanged. The existing motion/writer
retract implementation must not be patched to add a missing command.

## Included and deferred behavior

Included: nonempty filament-vector gathering through already resolved source
indices; exact source front cloning on a miss; preserving nil, explicit zero,
nonzero values, output count and selected-index order. Full and runtime views
continue to use the existing configuration lifecycle. No new platform-specific
behavior or dependency is introduced; core remains portable to all Tier 1
platforms.

Deferred: empty-vector/default initialization, unsupported option types, new
variant resolvers, dynamic/multi-slot selector producers, nullable overlay
policy changes, writer/retraction algorithms, wipe and lift behavior, command
formatting, GUI/plate read-back and registry regeneration. Existing gather code
is a temporary compatibility shell around the cited libslic3r helper, not an
Ares-owned option-selection design.

## Witnesses and failure modes

The unchanged authored input `tests/parity/nozzle-footer/variant-column.3mf`
has SHA256 `908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d`.
It selects source variant columns 1 and 3 for logical filaments 0 and 1. Its
source filament retraction length/speed overrides are `[nil, nil]`; machine
retraction length is 4 mm and speed is 60 mm/s. Gathering must yield two nils,
not `[nil, 0]`. The first relevant travel must retain `G1 E-4 F3600` in Native,
public API and CLI output. The previously recorded first difference is line 59:
Native has that retract, while Ares proceeds to the fan-layer marker.

Retain Native first/replay stability, actual positive tool-1 model activity,
successful API/CLI output and raw API/CLI equality. Compare the labelled
retract/travel section and preserve complete bytes and exact first differences
separately. Do not claim general complete-output parity from this section.

Before implementation, add public contrasts for short all-nil overrides,
explicit zero and an explicit nonzero front value, with the same out-of-range
variant selection. Nil must preserve the mapped machine default; explicit zero
must remain disabling; nonzero must repeat the authored front value. Existing
in-range variant/nozzle and bed/map witnesses retain their values. Failed
Native setup or absent positive model activity cannot establish this behavior.
