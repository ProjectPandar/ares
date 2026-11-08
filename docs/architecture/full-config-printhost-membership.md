# Full-config membership of the registered print-host option

## Decision and rewrite boundary

Port the default-membership distinction between OrcaSlicer's static
`FullPrintConfig` and the dynamic configuration snapshot serialized by
`GCode::append_full_config`. Registration in `PrintConfigDef` does not imply
membership in the static default config.

The upstream source revision is `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This slice owns only `bbl_use_printhost` default membership in the existing
registry/config-export projection. It is not a new Ares export pipeline or a
print-host feature.

## Source chain: what the exporter actually iterates

1. `PrintConfig.cpp:1177-1182` registers `bbl_use_printhost` as `coBool`, false
   by default, and `ConfigOptionDef::nocli`. This is a definition and parsing
   contract, not a static member declaration or an export ban.
2. `PrintConfig.hpp:1130-1273` defines static configuration classes and their
   caches. The member sequences populate the caches through `initialize`;
   static `keys()` returns that member cache, not all ConfigDef registrations.
3. `PrintConfig.hpp:2181-2184` composes `FullPrintConfig` from
   `PrintObjectConfig`, `PrintRegionConfig`, and `PrintConfig` (including their
   parents). There is no `bbl_use_printhost` member anywhere in this header.
   `PrintConfig.cpp:12716-12722` initializes these static caches. Thus the
   registered false default is not a `FullPrintConfig` member/default.
4. `OrcaSlicer.cpp:4250-4259` constructs `FullPrintConfig fff_print_config`,
   applies the received dynamic `m_print_config` into it with
   `ignore_nonexistent=true`, then applies those static defaults/values back
   into the original dynamic config. Existing dynamic-only keys are not erased.
   An absent `bbl_use_printhost` is not created by this static merge.
5. `Config.hpp:2802-2803` implements `apply` through `apply_to` or
   `apply_only(other, other.keys(), ...)`. `Config.cpp:484-522` skips missing
   destination members when requested; it does not erase unrelated destination
   keys. Static `FullPrintConfig::apply_to` returns false, preserving this
   key-iteration path. This also explains why a registered, explicitly loaded
   dynamic-only key survives the static round-trip rather than being banned.
6. `OrcaSlicer.cpp:6923-6925,6985` copies the completed dynamic config,
   overlays the plate and extra settings, and passes it to `Print::apply`.
   `PrintApply.cpp:1336,1675-1676` receives that dynamic config by value and
   saves it as `m_full_print_config`; the later normalized-state path at
   `2262` also assigns a dynamic config, not a replacement static key set.
7. `PrintBase.hpp:518` returns `const DynamicPrintConfig&` to this snapshot.
   `GCode.cpp:8244-8246` copies that snapshot into dynamic `cfg`.
   `GCode.cpp:8338-8352` iterates `cfg.keys()`, with banned-key and nil checks.
   `Config.cpp:1919-1926` defines those keys as the dynamic options map's actual
   keys. `bbl_use_printhost` is not in this exporter's banned-key set.

Consequently the export key set is the actual applied dynamic snapshot, with
static defaults contributing only their own members and preset construction
contributing its independently owned dynamic members. It is neither all
ConfigDef keys nor only the static members: explicit dynamic-only inputs may
still contribute keys.

## Conditional preset-owned membership

A missing authored key is not necessarily missing from the applied dynamic
configuration. `Preset.cpp:1059-1089` (`Preset::load_external_config`) first
copies the default preset, then applies project values over its member keys.
Only the seven connection-secret/address keys are excluded from that overlay;
`bbl_use_printhost` is not excluded. The printer preset member list at
`Preset.cpp:1592` includes it, and default preset construction at1787-1792
clones its registered false value. Thus an omitted project key becomes a
dynamic member with canonical wire value `0` on this path.

`OrcaSlicer.cpp:3167-3200` owns the condition: a nonempty current system
preset identity is required; printer refresh is called when
`new_printer_name.empty()` and `load_machine_config.empty()`. The current
identity is selected from the embedded printer identity or inheritance group
(`OrcaSlicer.cpp:1861-1889`). The refresh copies eligible preset members back
into `m_print_config`. Sparse STL settings replacement does not take that
embedded-project refresh path. If neither an input nor that preset path
creates this dynamic-only key, the later static merge does not create it.

The footer therefore includes `bbl_use_printhost` when the applied dynamic
config contains its non-nil value, whether explicitly authored or supplied by
preset construction. It is absent when no such producer created it. Keep the
static member flag false; do not add a blanket format, printer-model, false
value or footer exception. The existing supplied-profile core refresh already
implements the preset-owned producer, independently of static membership.

A project parity comparison must supply the same presets to all entry points.
The API project helper receives the catalog; its Ares CLI counterpart must
use `--project-preset-catalog` through the existing catalog-aware test helper.
Native runs with its system profile supply. Comparing that Native configuration
with an unsupplied Ares CLI is not the same-input experiment. Repair only the
command premise; retain the complete ordered-byte comparator and its existing
generated-by normalization. The sparse STL empty-override witness and the
Arachne project controls exercise the two distinct construction paths.

## Existing Rust mismatch and destination

`crates/ares-core/src/options/registry/definitions/table/early.rs` currently
marks `bbl_use_printhost` as `full_config_default=true`. The existing
`options/config_export/full_config.rs::write_entries` treats that registry
flag as default-membership authority for typed entries and absent defaults.
The typed printer model can contain a false value even when no dynamic input
provided the key; this must not manufacture static membership.

The membership delta is the existing registry member flag for this key. Keep its
registration, bool kind, false default, typed field and decoding behavior.

Preset default construction is independent of footer membership.
`PresetCollection::add_default_preset` (`Preset.cpp:1787-1792`) applies its
explicit option keys; `Config.cpp:489,1739` creates typed options from their
registered defaults. A bool false is encoded as canonical wire `0`, not the
presentation literal `false`. The existing Rust
`profiles/project_refresh/keys.rs::default_value` must therefore select
`default_wire_value` independently of `full_config_default`. Preserve its
nullable-filament override nil handling and the ordinary kind-owned
scalar/vector construction. Do not relax the bool decoder, catch errors,
substitute user values or couple constructor tokens to export membership. The existing projection
already distinguishes non-default keys that were actually applied; do not
add a key-specific raw-presence branch, footer blacklist, serializer omission,
or a second membership table. If the existing projection cannot preserve
explicit values after the member correction, stop and report that gap rather
than expanding this slice.

Ares's typed/full/raw projection remains a temporary compatibility shell for
the upstream dynamic snapshot. This slice corrects one source-cited membership
fact; it does not certify the remaining registry or reconstruct a new pipeline.

## Included and deferred behavior

Included:

- An absent `bbl_use_printhost` must not acquire membership merely because its
  registered default is false or its typed printer field exists.
- Explicit false and true remain valid registered values and must not be
  globally suppressed if present in the applied dynamic config.
- Preserve `bed_custom_model`, `bed_custom_texture`, key order, nil handling,
  every other registry flag, existing banned keys and all body bytes.

Deferred:

- Auditing other ConfigDef-only options or repairing an entire key family.
- Device/network/UI print-host behavior and CLI option availability.
- Dirty-key, preset normalization, inheritance, filament-map, motion, geometry,
  G-code processor, and template behavior.
- Native-reference availability/binding repairs or comparator changes.

## Failure modes and agreed public failing-test seam

The existing witness is
`runner::direct_stl_override_bookkeeping::direct_stl_no_override_bookkeeping_control_complete_output`.
It resolves real WEMAKE3D machine/process/filament settings, slices the original
solid-plate STL using runtime Native and `ares_core::slice_with_profiles`,
and compares complete ordered bytes with only the existing generated-by
identity normalization. Its empty override object is preserved. Do not modify
this test or add a duplicate witness just to create a test commit.

Before production, reproduce this unchanged test in official and seeded modes,
one test at a time. Record exact tested SHA, input/config hashes, actual Native
payload binding and the first difference. The expected failure is the extra Ares
`; bbl_use_printhost = 0` before Native's `; bed_custom_model = `; any different
failure needs investigation, not an assertion waiver.

Risks:

1. Confusing registration defaults with static snapshot membership.
2. Accidentally suppressing explicit false as though it were absent.
3. Accidentally banning explicit true or changing its registered decoding.
4. Dropping neighboring bed metadata or changing unrelated default flags.

The original full-output witness remains mandatory.
Explicit-value and absent-value Native premises must remain separate; do not
infer actual runtime explicit-input parity solely from the static source proof.
