# Default-nozzle-volume-type membership in the applied full configuration

## Decision and rewrite boundary

`default_nozzle_volume_type` is a registered enum-vector printer option, not a
static `FullPrintConfig` member. Its registered `Standard` default must not
manufacture membership in an otherwise absent applied dynamic configuration.
An actual dynamic key, including one supplied by a completed preset, remains
eligible for ordinary export. This is not a blanket omission of the option.

This single-key Rust rewrite slice follows upstream revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. The destination is the existing
registry `full_config_default` flag and generic config-export projection.
No new pipeline, enum policy, raw-presence conditional or key blacklist is
introduced.

## Static-member proof first

`PrintConfig.cpp:6306-6324` registers `default_nozzle_volume_type` as `coEnums`,
uses the `NozzleVolumeType` enum map, and assigns a one-element default
`ConfigOptionEnumsGeneric{ NozzleVolumeType::nvtStandard }`. Registration
supplies type and creation defaults, not a member of every configuration.

The **entire `PrintConfig.hpp` has zero occurrences** of this key.
`PrintConfig.hpp:1130-1273` builds static caches from declared member sequences;
static `keys()` enumerates those cached members. `FullPrintConfig` composes
`PrintObjectConfig`, `PrintRegionConfig`, and `PrintConfig` at `2181-2184`,
without additional option members. `PrintConfig.cpp:12716-12722` initializes
the caches. Therefore there is no static FullPrintConfig member/default for
this key. The static-member stop condition does not apply to this source.

The extruder-option lists are different authorities: `PrintConfig.cpp:8994`
includes this key in `init_extruder_option_keys`, and `10227` includes it in
`printer_extruder_options`. Neither list declares a static configuration member.
Do not confuse vector resizing or parameter size with full-config membership.

## Dynamic snapshot and export authority

`OrcaSlicer.cpp:4250-4259` applies dynamic settings into a static FullPrintConfig
and then applies that static configuration back into the original dynamic
settings. `Config.hpp:2802-2803` applies source keys through `apply_to` or
`apply_only`; static `apply_to` returns false at `PrintConfig.hpp:1234-1242`.
`Config.cpp:484-522` skips unsupported static destination keys when requested
and does not erase unrelated dynamic destination entries.

Thus this round-trip does not invent an absent default-nozzle vector. An
explicit dynamic vector survives even though the static destination lacks
that member. `OrcaSlicer.cpp` has no occurrence of this key, so it supplies no
additional key-specific initialization step. This does not claim that other
preset/session routes cannot produce a real dynamic key.

`OrcaSlicer.cpp:6923-6925,6985` sends the dynamic configuration plus overlays
to `Print::apply`. `PrintApply.cpp:1336,1675-1676,2262` retains the dynamic
snapshot as `m_full_print_config`; `PrintBase.hpp:518` exposes it as a
`const DynamicPrintConfig&`.

`GCode.cpp:8244-8246` copies the actual snapshot. Its `8338-8352` loop visits
`cfg.keys()` with existing banned/nil checks. The key is not export-banned.
`Config.cpp:1919-1926` defines dynamic keys from the actual options map,
not all registered defaults. Preserve actual applied values and normal vector
serialization rather than filtering `Standard`, empty vectors or enum strings.

## Included behavior and minimal Rust correction

`options/registry/definitions/table/pre_middle_defaults.rs` currently declares
`default_nozzle_volume_type` as `Enums`, default `Standard`, with
`full_config_default=true`. `options/config_export/full_config.rs::write_entries`
uses that membership flag to admit typed/default-only values. The typed
`NozzleVolumeTypes([Standard])` default is not evidence of an actual dynamic key.

The proposed production change is only this existing flag from true to false.
Keep the registered enum/default/metadata, typed fields/serializers, and the
generic applied-key path through config-export collection and projection.
The typed/raw Ares projection is a temporary compatibility shell around
upstream's dynamic snapshot, not a replacement Ares-owned pipeline.

- Absent snapshot key: do not generate a default `Standard` footer entry.
- Actual applied key: retain the vector and ordinary enum serialization,
  including a supplied default-valued `Standard` vector.
- Preserve `NozzleVolumeType`/`NozzleVolumeTypes`, ordering, nil checks, all
  other option membership and every non-target output byte.
- Do not add a value predicate, raw-presence branch, blacklist/second table,
  serializer omission or vendor/nozzle-specific export condition.

## Adjacent behavior deliberately unchanged

`Preset.cpp:1573-1585` lists this as a printer-preset option. A completed preset
may therefore legitimately own the key even when FullPrintConfig does not.
`PresetBundle.cpp:4195-4218` can use the selected printer's default vector to
initialize project `nozzle_volume_type`; `5358-5362` resets that project vector
from the edited printer preset. These are real nozzle-selection semantics,
not footer membership. Do not change them or infer absence in all load routes.

`PrintConfig.cpp:9867-9874` maps legacy `Normal`/`Big Traffic` tokens for this
key, and `10669-10686` resizes present extruder vectors during
`DynamicPrintConfig::set_num_extruders`. Leave the corresponding Rust legacy
rules, extruder/parameter key lists, normalization and resizing intact. The
existing Rust `vector_resize::default_array_member` reads registration defaults
independently of the full-config membership flag; changing membership must not
change that registration default or the present-vector resizing contract.

Also deferred: nozzle mappings, extruder count/variants, preset normalization,
project loading, nozzle-dependent slicing/flow/temperature behavior, UI/session
work, and membership of every other key. This SPEC is not a key-family audit.

## Failure model and proposed public verification

Use the unchanged full-output witness
`runner::direct_stl_override_bookkeeping::direct_stl_no_override_bookkeeping_control_complete_output`:
real Native CLI versus public `ares_core::slice_with_profiles`, empty overrides,
original STL and selected real profiles. Preserve whole-byte comparison and
existing generated-by-only normalization. Never bypass an earlier/later diff.

Before a production edit, add an applied nondefault enum-vector control through
the public API, using the existing applied-machine-option pattern with an array
input appropriate to this option. Save full input/catalog/selection and complete
API output, require the exact key/value once, and retain existing applied-value
controls. Proposed single value: `High Flow`, distinct from registered Standard.
An enum-vector control must not silently use the scalar-string test shape.

Report exact tested before/after SHAs, official then seeded, matched control
statuses and complete output deltas. A later first difference is not a complete
PASS. Report any next key without repairing it. No runtime verification is
claimed by this source-only SPEC.

Failure modes: treating an extruder/preset list as a static member declaration,
confusing registration defaults with dynamic membership, dropping explicitly
applied Standard/High Flow vectors, serializing the vector as a scalar, changing
nozzle selection or normalization, and chaining unrelated membership edits.
