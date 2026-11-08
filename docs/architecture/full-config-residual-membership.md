# Remaining applied-full-config membership in the direct-STL witness

## Decision and source boundary

The remaining extra keys in the direct-STL full-output witness are registered
ConfigDef options without static `FullPrintConfig` members. Their typed/default
values must not independently create export membership. Real applied dynamic
values remain eligible for export. The aligned `deretraction_speed` row is not
missing; its static membership and value are already correct in this witness.

This is one consolidated source-cited Rust rewrite boundary for upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`, not an export-pipeline redesign or a
registry-wide audit. It covers the closed extra/missing key set of
`runner::direct_stl_override_bookkeeping::direct_stl_no_override_bookkeeping_control_complete_output`,
using its real WEMAKE3D profiles and original solid-plate STL.

The Rust destination is the existing `full_config_default` authority in
`options/registry/definitions/table/`, consumed by
`options/config_export/full_config.rs::write_entries`. The typed/raw projection
remains a temporary compatibility shell for upstream's actual dynamic snapshot.
A second membership table, footer blacklist or serializer omission is not an
alternative implementation of this boundary.

## Complete key-set classification

The Native footer has 721 rows; Ares has 735. Ordered occurrence accounting
finds **14 extra keys and no missing keys**, with matching duplicate occurrences
for `wipe_tower_x` and `wipe_tower_y`. Each listed extra occurs once. This is
not a classification inferred from the first differing text line.

All rows below have the same source classification: **registered dynamic-only,
not a static FullPrintConfig member, not export-banned**. The final column names
the existing Rust table file beneath `options/registry/definitions/table/`;
these are existing authorities, not a new runtime key list.

| Extra key | Upstream registration in PrintConfig.cpp | Kind; creation default | Rust table |
|---|---|---|---|
| `default_print_profile` | 2356-2361 | coString; empty | pre_middle_defaults.rs |
| `extruder_variant_list` | 6326-6331 | coStrings; `["Direct Drive Standard"]` | pre_middle_process.rs |
| `flashforge_serial_number` | 1229-1234 | coString; empty | middle.rs |
| `group_algo_with_time` | 8829-8831 | coBool; false | middle.rs |
| `host_type` | 5759-5800 | coEnum; octoprint | middle.rs |
| `ironing_expansion` | 5248-5256 | coFloat; 0 | middle_tail.rs |
| `pellet_modded_printer` | 4681-4685 | coBool; false | late_tail_after_pad.rs |
| `plugins` | 6007-6011 | coStrings; empty vector | late_tail_final.rs |
| `printer_agent` | 1192-1200 | coString; empty | late_tail_final.rs |
| `printer_plugin_config_overrides` | 1304-1313 | coString; empty | late_tail_final.rs |
| `printhost_authorization_type` | 1282-1292 | coEnum; key | late_tail_final.rs |
| `printhost_ssl_ignore_revoke` | 1269-1274 | coBool; false | late_tail_final.rs |
| `thumbnails_format` | 8602-8617 | coEnum; PNG | tail_terminal_suffix.rs |
| `upward_compatible_machine` | 2220-2224 | coStrings; empty vector | tail_terminal_suffix.rs |

`printer_plugin_config_overrides` is registered inside a loop over preset-kind
keys; the cited span includes `this->add(key, coString)` and its empty default.
Other keys in that loop are outside this witness's closed extra-key set.

The entire `PrintConfig.hpp` has no static member declaration for any of these
14 keys. Thirteen have zero textual occurrences; `extruder_variant_list` occurs
only in a comment at line694. A comment is not a declaration or a parent member.

`PrintConfig.hpp:1130-1273` constructs static caches from declared member
sequences and inherited caches. `PrintConfig.hpp:1950-1951` declares PrintConfig
with `MachineEnvelopeConfig` and `GCodeConfig` parents; `PrintConfig.hpp:2181-2184`
composes FullPrintConfig from `PrintObjectConfig`, `PrintRegionConfig`, and
`PrintConfig`.
`PrintConfig.cpp:12716-12722` initializes those caches. No registration row or
preset/extruder key list adds a member to these static classes.

The actual machine/process/filament input maps in this witness contain none of
the 14 keys. Native exports none; Ares exports their registered/default-shaped
values. Every corresponding existing Rust definition marks
`full_config_default=true`. Those flags conflate registration with static
membership for this closed set; the kind/default/metadata are separate contracts.

## Dynamic ownership and exporter semantics

`OrcaSlicer.cpp:4250-4259` applies dynamic settings into static FullPrintConfig,
then applies that static configuration back to the original dynamic settings.
`Config.hpp:2802-2803` uses `apply_to` or source-key `apply_only`; static
`apply_to` returns false at `PrintConfig.hpp:1234-1242`.
`Config.cpp:484-522` skips unsupported static destinations when requested,
without deleting unrelated dynamic entries. No absent member is created merely
because ConfigDef registers a default. An already applied dynamic-only key is
not erased by this round-trip.

`OrcaSlicer.cpp:6923-6925,6985` sends the actual dynamic configuration plus
overlays to `Print::apply`. `PrintApply.cpp:1336,1675-1676,2262` retains it as
`m_full_print_config`; `PrintBase.hpp:518` returns a DynamicPrintConfig reference.
`GCode.cpp:8244-8246` copies that snapshot and `8338-8352` iterates `cfg.keys()`
with existing nil and banned-key checks. `Config.cpp:1919-1926` enumerates the
actual map, not the entire ConfigDef. None of the 14 keys is in the exporter ban.

The bounded membership correction belongs in the existing 14 member flags,
not in registration, typed field presence or an export-value predicate.
`full_config_default=false` must leave the existing generic applied-key path
intact. Preserve applied empty/default values as well as valid nondefault values:
empty strings and vectors, false and zero are not absence tests. Preserve each
option's scalar/vector kind and ordinary wire serialization.

## The deretraction row is already correct

`PrintConfig.hpp:1700-1704` declares `GCodeConfig::deretraction_speed` as
ConfigOptionFloats. Through PrintConfig and FullPrintConfig inheritance it is a
real static member. `PrintConfig.cpp:6470-6477` registers coFloats with `[0]`
default. The Rust `pre_middle_process.rs` definition's membership remains true.

Both outputs contain exactly `; deretraction_speed = 0`. Native reaches it
before Ares only because an extra `default_print_profile` row precedes it.
Do not add/remove/rename the deretraction key, change its default, resize it or
modify retraction motion. Zero missing keys means this row supplies no evidence
for a missing-option or retraction-behavior port.

## Separate shared-value residual: flush_volumes_matrix

Across all shared keys, the sole differing ordered value is
`flush_volumes_matrix`: Native `0`; Ares
`0,280,280,280,280,0,280,280,280,280,0,280,280,280,280,0`.

This is **not membership**. `PrintConfig.hpp:2108` declares it as a static
ConfigOptionFloats member; `PrintConfig.cpp:8274` registers coFloats.
`GCode.cpp:8247-8272` handles the actual flush matrix and multiplier shape before
serialization. Matrix construction/application and value shape belong to that
upstream boundary, not this registry-membership correction. Its flag, values,
resize rules, templates and slicing behavior remain unchanged in this slice.
No guessed matrix repair or serializer substitution belongs here.

The complete current residual is accounted for by 407 bytes of extra key lines
and 54 bytes of shared matrix-value length difference. Diagnostic in-memory
attribution of those two categories equals the Native complete payload under
existing generated-by exclusion. This is not permission to filter keys or
substitute values in a comparator, and is not whole-output runtime parity.

## Included invariants and adjacent behavior deferred

- Only the closed 14-key absent-default membership discrepancy is included.
  Preserve every other flag, including already-correct static deretraction and
  matrix membership and the three earlier corrected dynamic-only keys.
- Preserve generic applied-key export, sorted order, duplicate wipe-tower rows,
  nil policy, registration defaults, metadata, typed fields and serializers.
- Preserve every body, timing, M73 and other footer value byte. Removing absent
  default-only lines must not silently change default selection or slicing.
- Keep actual preset ownership and normalization distinct from static defaults.
  `extruder_variant_list` can legitimately be created by upstream
  `PrintConfig.cpp:10630-10668` and used by preset/extruder variant resolution.
  Do not suppress it globally or change count/variant normalization.
- Default process selection, compatibility matching, host/authentication/TLS,
  printer-agent/plugin configuration, grouping policy, ironing geometry, pellet
  behavior and thumbnail production are not being ported or redesigned here.
  Their registered values remain available when genuinely applied.
- Plugin/host options do not authorize external execution/network behavior in
  the rendering-neutral core. This slice changes no platform boundary.
- Missing-key work is empty for this witness. Other witnesses, printers and
  registry families are outside this classification. The flush-matrix value
  residual is explicitly separate rather than hidden behind membership flags.

## Verification boundary and failure model

The public seam remains the original complete-output Native CLI versus
`ares_core::slice_with_profiles` witness. Keep its full-byte comparator and
existing generated-by-only normalization; no footer mask or assertion waiver.
The existing explicit print-host, bed and nozzle-vector controls remain intact.

Applied-value checks must use valid public inputs of the original option kind:
strings, coStrings arrays, enum tokens, booleans and floats. A scalar string
cannot stand in for a vector. Present default/empty values must be distinguished
from absence; unsupported inputs do not justify changing validation or adding
fallbacks. Preserve complete input/catalog/selection and output artifacts.

Failure modes are treating a header comment or preset list as a static member,
inferring missing keys from line displacement, dropping explicitly applied
values, changing a shared-value discrepancy through membership, losing duplicate
occurrences, or fixing unrelated behavior while the first difference moves.
