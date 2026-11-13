# Empty float JSON values at raw config boundaries

## Upstream boundary

This slice ports `ConfigBase::load_from_json` and
`ConfigOptionFloatsTempl<NULLABLE>::deserialize` from OrcaSlicer
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. It does not define a new
configuration or device-option policy.

`libslic3r/Config.cpp:983` initializes `value_str` empty for each JSON option.
The array visitor `parse_str_arr` (`:887–925`) joins string elements, so an
empty JSON array leaves that string empty. The array branch calls the visitor
at `:1052` and passes its result to `set_deserialize` at `:1058`. The JSON
string branch calls `set_deserialize` directly (`:985–987`).

`libslic3r/Config.hpp:933–940` clears a float vector and appends one zero for an
empty serialized value. This is the shared `ConfigOptionFloatsTempl` rule for
both ordinary and nullable float vectors, not a key-specific default. A
nullable `nil` remains `nil`; an empty float vector is not implicitly `nil`.
Other option kinds retain their own deserialization rules.

The upstream loader serves:

- System vendor documents: `PresetBundle.cpp:8434`, before resolving presets.
- Preset imports and reloads: `Preset.cpp:887,1883,4900` and
  `PresetBundle.cpp:2575`.
- Embedded 3MF project/settings JSON: `Format/bbs_3mf.cpp:2806,2846`.
- Explicit CLI settings JSON: `OrcaSlicer.cpp:2217`.

## Rust destination

The owner is `crates/ares-core/src/options/json_deserialize.rs`, specifically
raw normalization shared by `normalize_empty_vector_strings` and
`normalize_scalar_vector_strings`. Route empty float values through the
existing `decode`/wire decoder by `OptionValueKind::{Floats,FloatsNullable}`.
`options/model_config_deserialize/wire.rs` already implements empty float
wire to `["0"]`; do not duplicate that parser or change its semantics.

The current raw configuration is a temporary compatibility shell. It must
present the same interpreted value as the installed upstream configuration,
not expose a raw empty shape after the typed decoder interpreted zero.

Affected raw callers include catalog-document insertion, CLI system-preset
refresh, Always collection loading, embedded project loading and applying
refreshed project configs. The typed `decode_preset_values` entry already
routes string arrays through the wire decoder. Keep raw and typed boundaries
consistent without changing unrelated option kinds or nonempty inputs.

Do not add a device-key whitelist, mutate device assembly, or change an
option's registered default to implement the shared float rule.

## Included and deferred behavior

Included:

- Empty JSON arrays and bare empty JSON strings on ordinary or nullable
  float-vector kinds become the one-element zero vector.
- Existing nonempty values, nullable slots, vector order, identities,
  dirty-key policy and optional exact-name system lookup misses stay intact.
- Existing device-option append, partial-lookup and variant-restoration
  comparisons remain strict controls around this parser boundary.

Deferred: other JSON type conversions, nonempty float stream-prefix changes,
other vector-kind parsers, resolver/merge changes, replacement policy and
slicing/G-code/time/cost parity. An internally empty device vector is not the
same input as a float JSON array loaded through this parser; do not claim the
internal empty-vector skip branch from this case.

## Test-first checks

Potential failures to distinguish before implementation:

1. Native vendor loading fails: reject this as an input-premise failure,
   never an API regression.
2. A float JSON array remains empty in raw config even though the typed
   decoder produces zero.
3. Only device keys are corrected while a non-device float key still fails.
4. Nullable empty vectors become `nil`, or an existing unrelated `nil` is
   lost.
5. A nonempty vector or a non-float option changes accidentally.
6. A machine-normalization path already produces zero and is incorrectly
   described as a new failing case.
7. A generic parser change modifies existing parity output without recording
   the transition.

Use source-directed synthetic two-slot projects and actual upstream CLI
exports before comparing the public `profile_settings` result. The original
empty device-array mismatch is Native `["0","51"]` versus API `["51"]` on
unchanged `1dc0f6d7a71d68e20debb99d607272d696e85460` core. Retain that baseline
and the same complete archive, catalog and Native output for the candidate.
Add non-device float and nullable/nonempty controls before the shared fix.

The pinned resource `BBL/machine/fdm_bbl_3dp_001_common.json` contains
`extruder_printable_height: []`. Exercise a machine inheriting that source
value. If the existing typed machine normalizer already consumes it as zero,
retain it as an unchanged control, not a failing-baseline claim.

After the separate specification, test and fix commits, run all A1/A1b
directed comparisons, all non-parity `ares-core` tests, existing actual CLI
controls and a bounded BBL-machine parity sample in both environments.
Preserve inherited failures and every setup/verifier failure. Report existing
output changes before proceeding; do not run a full parity inventory or
relax comparators, tolerances, masks or golden files.
