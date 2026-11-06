# Nullable ramming, nozzle-change retract and preheat filament variants

## Upstream boundary

Pinned OrcaSlicer source: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This slice ports six registered nullable float members through the existing
filament variant layout and static gather, not ramming or heating commands.

All six definitions in `src/libslic3r/PrintConfig.cpp` use `coFloats` with
`nullable = true` and `ConfigOptionFloatsNullable` defaults:

| Key | Definition | Default |
| --- | --- | --- |
| `filament_ramming_travel_time` | 8869–8877 | `0` |
| `filament_ramming_volumetric_speed` | 8888–8896 | `-1` |
| `filament_ramming_travel_time_nc` | 8898–8906 | `0` |
| `filament_ramming_volumetric_speed_nc` | 8918–8926 | `-1` |
| `filament_retract_length_nc` | 8928–8937 | `10` |
| `filament_preheat_temperature_delta` | 8971–8977 | `0` |

The first five are `GCodeConfig` members in `PrintConfig.hpp:1925,1927–1931`;
`filament_preheat_temperature_delta` is a `PrintConfig` member at2129. Nil is
undefined, zero is defined, and volumetric speed `-1` means using the maximum
volumetric speed. The hotend-change `_nc` members are independent options.

`PrintConfig.cpp:10153–10161::filament_options_with_variant` includes all six
alongside the two pre-cooling integers. Load-time default completion and
singleton/per-filament layout use `PrintConfig.cpp:11431–11457` and
`OrcaSlicer.cpp:4250–4259`. Static selection uses
`DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments`
at `PrintConfig.cpp:11884–11994`: the coFloats dispatch at11973 calls
`gather_option_values` at11862–11882 with the existing source indices.

Each logical slot receives its selected original payload; a missing source
column uses the original front, and empty sources leave destinations unchanged
inside the gather helper. Registered default completion remains a separate
load/dispatch responsibility. `PrintApply.cpp:1436–1455` and
`Print.cpp:4723–4755` apply the same static gathering at initial apply and map
writeback. `GCode.cpp:8244–8246,8342–8353::append_full_config` uses ordinary
serialization and omits all-nil options.

## Destination

The six keys already have `FloatsNullable` definitions and defaults in
`options/registry/definitions/table/pre_middle_filament.rs` and
`pre_middle_tail.rs`. The current raw-settings/default full-config collector
is a temporary compatibility shell: it can serialize a singleton or authored
source vector but does not own a typed, selected logical-filament vector.

The Rust destination is the shared filament option boundary:

- `options/filament_options/gcode_source.rs::FilamentGCodeSourceOptions` holds
  the six members as `Vec<Nullable<OrcaFloat>>`, initialized with the existing
  `nullable_floats` helper and unchanged defaults. The common Rust filament
  group carries both the five upstream GCodeConfig members and the upstream
  PrintConfig preheat member; their distinct keys and meanings remain intact.
- The existing option-group decoder/builder, declaration order,
  `gcode_source/wire.rs`, `filament_options/wire.rs` nullable classification
  and `filament_options/wire/middle.rs` provide ordinary decode/profile/wire
  paths using `serialize_nullable_vector`.
- `options/registry/key_lists.rs::FILAMENT_OPTIONS_WITH_VARIANT_KEYS` includes
  all six in sorted order, connecting them to existing load-time layout.
- `options/project_variants/filament.rs::materialize` selects all six through
  `select_vec_field!` and `select_filament<T: Clone>` using existing checkpoint,
  index and static-refresh paths.

The six key-list entries and six gather assignments belong to one atomic
production change together with typed/wire promotion. Typed full-config
values take precedence over raw echoes; omission of an all-nil typed value
must not resurrect raw or default values. No footer expansion, token patch,
second index formula, nullable conversion or duplicate CLI algorithm is used.
Existing pre-cooling, multitool ramming, activation, default completion and
profile-refresh key boundaries remain unchanged. Core is platform-neutral for
browser WASM, Windows, macOS and Linux; touched Rust modules remain below400
lines, with splitting only where field wiring crosses that limit.

## Included and deferred

Included: the six members, registered defaults, decode/profile/wire paths,
load-time key registration, initial/static-refresh gathering, selected order,
original-front fallback, nil/zero/nonzero/negative-default identity and normal
serialization. Absent authored values materialize to four logical entries on
the Ender four-slot project and two entries on the MK4 two-slot project,
whereas the raw Ares path emits singleton values.

Deferred: physical ramming, travel, retraction, heating or preheat consumption;
multinozzle/Hybrid producers; activation or source-index algorithm changes;
IDEX extruder-count normalization; wipe-tower placement/clamping; other option
cohorts; GUI read-back; registry regeneration. This is a source-cited
libslic3r rewrite slice, not an independently designed cooling pipeline.

## Tests

`crates/ares-cli/tests/orca_parity/option_coverage/remaining_filament_variants.rs`
uses Native replay, public API and CLI. Each case asserts the complete six-key
Native cohort, successful slicing, raw API/CLI equality and labelled footer
equality, including key absence. Strict whole-stream comparisons remain
separate and unchanged.

1. `ender_four_slot_defaults`: the existing Ender PLA project produces four
   entries per key, preserving the `0`, `-1` and `10` defaults.
2. `mk4_two_slot_defaults`: the existing MK4 project produces two entries per
   key, without reusing the Ender slot count.
3. `distinct_source_columns`: independent fractional vectors select source
   columns1/3 into two logical entries, retaining decimal and negative values.
4. `mixed_nil_and_defined`: ragged three-entry vectors preserve nil versus
   zero or speed `-1`, with original-front fallback at missing column3.
5. `all_nil_omission`: five all-nil vectors omit their keys while the sixth
   explicit-zero vector remains defined and selected.

## Failure modes

1. Singleton raw/default values survive instead of logical slot vectors.
2. Source vectors leak into applied output, or source/logical order is wrong.
3. Nil becomes zero, a default replaces an explicit value, or speed `-1` is lost.
4. Nullable floats become integers or fractional payloads are truncated.
5. Missing columns use the last value instead of the original front.
6. Empty sources overwrite the destination inside the gather helper.
7. Unused logical slots are lost or static refresh uses another selector.
8. All-nil options are resurrected or typed/raw duplicate keys are emitted.
9. `_nc` and ordinary fields are aliased or cross-assigned.
10. Public API and CLI differ or neighboring pre-cooling/ramming keys regress.
