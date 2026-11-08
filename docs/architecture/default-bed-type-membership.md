# Default-bed-type membership in the applied full configuration

## Decision

Represent `default_bed_type` as a registered option that is not a static
`FullPrintConfig` default member. An absent machine/project value must not
acquire export membership from its registered empty-string default. An explicit
applied value, including an empty string, remains part of the dynamic snapshot.

This is a single-key, source-cited Rust rewrite slice of upstream revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. The destination is the existing
registry member flag used by the config-export projection, not a new pipeline,
footer blacklist, or special case for raw presence or empty string values.

## Upstream ownership and membership proof

### Registration does not declare a static member

`PrintConfig.cpp:1469-1475` registers `default_bed_type` with `coString` and a
`ConfigOptionString()` default. Its source comment identifies a machine-profile
maker's default-bed hint. A ConfigDef default defines a value for option
creation; it does not itself insert the key into every print configuration.

The entire `PrintConfig.hpp` has no occurrence of `default_bed_type`.
`PrintConfig.hpp:1130-1273` constructs static config caches from declared
member sequences; `initialize` adds those members to the cache and static
`keys()` returns that cache. `FullPrintConfig` composes `PrintObjectConfig`,
`PrintRegionConfig`, and `PrintConfig` at `2181-2184`, with no extra members.
`PrintConfig.cpp:12716-12722` initializes these caches. Therefore the registered
empty string is not a static `FullPrintConfig` member/default.

### CLI defaults do not create this absent key

`OrcaSlicer.cpp:4250-4259` constructs the static `FullPrintConfig`, applies the
received dynamic `m_print_config` into it with `ignore_nonexistent=true`, then
applies the resulting static defaults and values back into the original
`m_print_config`.

`Config.hpp:2802-2803` implements `apply` using `apply_to` or
`apply_only(other, other.keys(), ...)`. The derived static FullPrintConfig
returns false from `apply_to` (`PrintConfig.hpp:1234-1242`), retaining the
key-iteration path. `Config.cpp:484-522` skips unsupported static destinations
when requested; applying source keys does not erase unrelated dynamic keys.

Consequently:

- If no input supplied `default_bed_type`, the static config has no member to
  contribute it, and the dynamic config remains without that key.
- If an input supplied the registered key, applying into the static config
  skips its unsupported member, but the original dynamic destination still
  owns the key when static defaults are applied back. Explicit empty and
  explicit nonempty values are not equivalent to absence.

There is no `default_bed_type` occurrence in `OrcaSlicer.cpp`, so this CLI
source supplies no additional key-specific creation step.

### Export iterates the actual dynamic snapshot

`OrcaSlicer.cpp:6923-6925,6985` copies the dynamic config, overlays plate/extra
settings, and passes it into `Print::apply`. `PrintApply.cpp:1336,1675-1676`
saves the dynamic config as `m_full_print_config`; the normalized-state update
at `2262` also assigns a dynamic config. `PrintBase.hpp:518` returns its
`const DynamicPrintConfig&`.

`GCode.cpp:8244-8246` copies that actual snapshot into dynamic `cfg`.
`GCode.cpp:8338-8352` iterates `cfg.keys()` and filters banned/nil options;
`default_bed_type` is not in that banned set. `Config.cpp:1919-1926` collects
keys from the dynamic options map. It does not enumerate all registered
ConfigDef defaults or replace the key set with static class membership.

Thus export must omit an absent default-bed hint and retain a key actually
applied into the snapshot. This is a membership distinction, not a string-value
filter and not a change to bed-temperature or bed-selection logic.

## Existing Rust boundary and minimal correction

`options/registry/definitions/table/pre_middle_defaults.rs` currently declares
`default_bed_type` with kind `String`, default `""`, and
`full_config_default=true`. The existing
`options/config_export/full_config.rs::write_entries` uses this flag to admit
typed/default-only entries. A typed empty-string printer field cannot prove
that the dynamic snapshot actually contains the option.

The minimal production correction is this key's existing
`full_config_default` flag from true to false. Preserve kind, registered empty
default, metadata, `DefaultBedType` parsing/representation, typed fields and
serializers. Reuse the existing generic applied-key projection. Do not add a
raw-presence branch, empty-string predicate, blacklist entry, second key table,
or a conditional tied to printer vendor or selected physical bed.

The typed/raw Ares export projection remains a temporary compatibility shell
around the upstream applied dynamic config. Correcting this flag is not an
audit or parity certification of other registry members.

## Included behavior and non-negotiable constraints

- Absent key: do not manufacture an empty footer line from typed defaults.
- Explicit empty key: retain the applied empty string in the snapshot/export.
- Explicit nonempty key: preserve its ordinary serialization and application.
- Preserve all other registered members, nil handling, ordering, neighboring
  `default_filament_colour` and `default_acceleration`, and every body byte.
- Preserve `curr_bed_type`, bed-temperature lookup, profile interpretation,
  preset dirty keys and project/CLI loading order.
- No other key or key-family audit, even if a later full-output difference
  identifies another apparent membership issue.

## Verification boundary and failure modes

The existing public full-output seam is
`runner::direct_stl_override_bookkeeping::direct_stl_no_override_bookkeeping_control_complete_output`:
runtime Native CLI vs `ares_core::slice_with_profiles`, with empty overrides,
real machine/process/filament inputs and original STL. Keep its complete-byte
comparison and generated-by-only identity normalization unchanged. A first
difference hidden behind an earlier residual must not be bypassed to obtain a
PASS or relabeled as whole-output parity.

Public applied-value controls must distinguish an absent key, an explicit
empty string, and an explicit nonempty default-bed value. Check complete
artifacts and the exact emitted key line; an empty-value test alone cannot
prove absence handling. Source-level preservation is not a substitute for
public API runtime verification.

Failure modes are conflating ConfigDef with static membership, treating empty
as absent, suppressing explicit nonempty values, changing semantic bed choice,
or fixing unrelated registry members when the full comparator advances.
Any remaining complete-output discrepancy must retain its original assertion
and be reported separately rather than waived or repaired in this slice.
