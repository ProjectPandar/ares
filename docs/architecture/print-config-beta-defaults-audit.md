# PrintConfig beta defaults and initial public-config RED boundary

## Non-negotiable boundary

This is a source-cited rewrite audit, not a new Ares configuration pipeline.
Compare upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1` with the sole current
source `345efec64c7b95a05ce5cf96df0dde66a158835c` (v2.5.0-beta).
The initial change is audit plus runtime-Native RED tests only. No production
implementation, producer fallback, comparator tolerance or Native golden is
included. Existing historical inputs remain unchanged.

The Rust destination is `crates/ares-core/src/options/registry/{definitions,
print_config_def}.rs`, its `OptionDefinition` / `OptionMetadata` tables and the
public `ares_core::print_config_def_json()` API. Existing 2.4.2-derived tables
are a partial upstream rewrite, not an independent source of new semantics.
Key lists and typed option/normalization consumers remain separately identified
port boundaries; adding metadata does not qualify their execution behavior.

## Initial included declarations

All line references below are in the pinned beta `src/libslic3r/` sources:

| Key | Owning declaration | Native constructor default | Included observation |
|---|---|---|---|
| `prime_volume_mode` | `PrintConfig.cpp:8310-8321`; enum `PrintConfig.hpp:656-660`, member2134 | `PrimeVolumeMode::pvmDefault`; serialized enum `Default` | public metadata key, enum kind and default |
| `filament_is_mixed` | `PrintConfig.cpp:3545-3549`; member `PrintConfig.hpp:1737` | one false Boolean | public metadata key, Boolean-vector kind and default |
| `filament_mixed_components` | `PrintConfig.cpp:3551-3555`; member `PrintConfig.hpp:1738` | one empty string | public metadata key, string-vector kind and default |

`PrintConfig.cpp:11481-11482` also resizes the two mixed-filament vectors.
That normalization and the execution of virtual/mixed filaments are adjacent
behavior, not implemented by this RED. `FilamentMixer` execution, active
Saving/Fast wipe-tower policy, nozzle-change grouping, preheat and belt behavior
remain deferred. No inactive-default observation admits those active features.

## Runtime authority and public seam

Use the qualified Native CLI `--export-settings` action on the existing
`tests/parity/orca_cli_ender3.3mf` input. Its three target keys are absent from
authored project settings: values observed after import are compiled defaults,
not values authored into a new test fixture. Record the fixture digest.

Upstream `src/OrcaSlicer.cpp:6326-6335` delegates that action to
`DynamicPrintConfig::save_to_json`. This public CLI export exposes the enum even
when `GCode::append_full_config` intentionally bans it from G-code comments.
Therefore testing only CONFIG_BLOCK inclusion would miss `prime_volume_mode`;
forcing that key into G-code would be a wrong port.

Compare each Native exported default to the corresponding default in public
`ares_core::print_config_def_json()` output. Native settings encode the two
single-element vectors as arrays of strings; compare the one element, not a
joined/vector-normalized value. Assert the Ares public kind separately against
the cited upstream constructor. No production-private table is accessed.
This is a configuration CLI/API end-to-end boundary, not a geometry comparison
and not a replacement for any existing whole-G-code comparator.

Register three behavioral tests as a child of the existing config-roundtrip
group. Do not change parity ROOT registration or rename existing tests.

Before tests were written, two official and two seeded Native settings exports
were checked: exact2.5.0-beta producer version, all three defaults present,
and all four complete settings JSON files byte-identical. This qualification
is only for the selected historical input and these constructor observations;
it is not a FULL or active mixed/prime execution qualification.

## Wider audit obligations

Literal common/FFF declarations and their source defaults are compared
independently of line movement/whitespace. The candidate inventory also tracks
nonliteral plugin/remap registrations, generated nullable filament overrides,
header member deltas, key-list changes and enum/type changes. A constructor
expression spelling change is not automatically a behavioral default change.
See `print-config-beta-option-inventory.md` for the concrete new-key/type groups.
At the RED base, the Ares public export's675 keys exactly match the old common/FFF source keyset;
the beta keyset is805 (131 additions, one removal). This is not an intersection
that silently discards unknown beta keys.

The major existing-key migration is scalar speed/acceleration/jerk/flow values
becoming nullable per-filament vectors. It belongs to configuration definition,
wire types and source-backed typed-consumer ports; silently using the old
scalar as a beta fallback is not acceptable. `enable_arc_fitting` becomes an
enum, not a Boolean with broadened truthiness. `silent_mode` is removed.

## Failure modes fixed before adding tests

- An absent/foreign Native producer must fail required-oracle initialization or
  the exact exported version check, not produce a skipped/pass result.
- Native failure, timeout, absent output, malformed JSON or a missing target
  field is infrastructure failure, not an Ares content RED.
- Fixture-authored target values would not prove defaults; audit the original
  archive, retain its bytes and record its digest.
- G-code ban-list behavior must not be mistaken for a missing definition.
- Scalar/vector or nullable/type mismatches must not be hidden by conversion.
- A missing Ares exported definition must fail with its key and durable work
  directory after successful Native production, not disappear in an
  intersection of Ares/Native key sets.
- Preserve Native settings, producer identity, command capture, input digest
  and Ares public JSON in the external runner artifact root. Never commit them.

## Verification sequence

1. Audit declarations, generated keys and Rust ownership; source-cite findings.
2. Add only the three targeted runtime-Native tests, then verify attributable
   missing/default/type REDs in required mode against official and seeded beta.
3. Run rustfmt, strict Clippy and scope/diff audits. Do not implement GREEN here.
4. Keep the separate dual-FULL ledger gated on explicit `B11-FULL-DONE`.

## Approved definition-only GREEN

The follow-up ports only the three declarations above into the default and
metadata tables, growing the public export675 ->678 without changing any old
record. `filament_is_mixed` uses the Native Boolean wire spelling `0`
for the cited `ConfigOptionBools{false}` constructor. `prime_volume_mode` carries
its three source enum names and integer codes0/1/2. Labels, tooltips, develop
mode and all other metadata fields use the in-repository generator's rendering.
The pre-existing675 metadata records must remain byte-identical.

The full generator currently rejects beta's unrelated `printer_agent.plugin_type`.
Do not fix that parser or regenerate other beta records in this slice. An
external selected-input invocation keeps the existing parser/renderer intact,
feeds only the three exact source blocks and produces the formatted records;
insert only those records at their sorted positions. Retain the failed full
attempt and selected-renderer receipt externally. The existing generated metadata
table is exempt from the module-length split; handwritten modules remain below
400 LOC.

Registering a key also makes authored raw settings eligible for config-block
write-back. Therefore add `prime_volume_mode` to the existing writer ban list,
matching beta `GCode.cpp:8287`. Do not merely rely on its absence from the static
CONFIG_BLOCK key list. No typed mixed-filament execution, normalization, active
Saving/Fast behavior or other beta definition is included. The approved three
public REDs are unchanged; verify only those cases plus fmt and strict Clippy.

## Tower-estimate prime-volume definition follow-up

At the same beta source revision, `GCode/WipeTowerEstimate.cpp:190` reads
`filament_prime_volume` through `float_at` for each Type1 purge estimate.
It does not read `filament_prime_volume_nc`; that separate hotend-change key
is deferred rather than registered speculatively.

The included definition is `PrintConfig.cpp:8955-8961`: `coFloats`, one
`ConfigOptionFloats{45.}` value, label `Filament change`, minimum1.0, simple
mode and `mm³` sidetext. Its destination is the existing public definition and
metadata tables, not an estimate/planner implementation. Native public settings
export on the unchanged fixture provides the default; the fixture does not
author this key. Add one behavior-named RED in the existing definition-defaults
group before porting it. Neither prime-volume vector is in the beta
`GCode::append_full_config` ban list, so do not introduce a ban. This slice does
not add typed emission fields, alter the static CONFIG_BLOCK list or qualify
full G-code/planner parity. Preserve all existing metadata records and use the
same generator-format, source-cited insertion as the prior three definitions.
