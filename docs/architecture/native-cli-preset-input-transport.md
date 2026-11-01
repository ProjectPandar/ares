# Native CLI preset input transport

## Upstream boundary

This is test-harness adaptation to OrcaSlicer beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`, not an Ares slicing feature.
`src/OrcaSlicer.cpp:2118-2157::resolve_preset` accepts an explicitly typed
standalone config without inheritance, but otherwise resolves a canonical
preset. `libslic3r/PresetBundle.cpp:556-650::resolve_preset_config` identifies
that preset by filesystem-equivalent source file, not by JSON name.

The previous v2.4.2 CLI parsed these flattened inputs without this resolver.
Its harness inheritance reattachment is not a compatibility fallback.

## Non-negotiable input contracts

### Independently flattened system snapshots

The 28 stock-profile cases already construct Native inputs through
`VendorProfiles::{machine,process,filament}` and its parent-chain flattener.
Do not reattach the raw leaf's `inherits` to these complete standalone
snapshots. Preserve explicit type/name/from, every slicer field and case
mode override, raw Ares catalog inheritance, nine source-profile pins,
geometry, Native activation controls and exact output comparisons.

The seven case constructors change locally. The shared JSON writer,
direct-STL runner and unrelated inherited-preset tests do not strip fields.
The existing cache key includes the three input JSON hashes.

### Genuine user process presets

The four embedded-script/inherited-group/project-export cases retain their
user process name, `from=user`, parent and settings. They cannot be converted
into system snapshots or have their inheritance assertions removed.

`PresetBundle.cpp:3313-3345` loads system bundles from `data_dir()/system`;
`:1411-1429,1505-1519` loads user presets from `user/default/process`.
`Preset.cpp:1825-1917,2038-2046` requires a valid version and existing parent,
and records the scanned file's identity and filename-derived name.

A small harness leaf stages an isolated datadir inside the case work dir at
test runtime. It copies the unchanged BBL and OrcaFilamentLibrary manifests
and directories from the pinned beta resource tree used by the test catalog.
It writes the same user process under its name, supplying version metadata
from that BBL manifest, and passes this registered path plus the supported
`--datadir` to Native. No staged resource, user JSON, cache or Native output
is committed. Do not use host user directories or inject inherited/diff
results; the canonical merge and comparison remain Native-owned.

## Verification boundary

The existing 32 public cases are the before/after boundary, with unchanged
case identities and assertions. Use the same production overlay on both
sides. An Ares admission failure before Native remains a failure and does
not qualify the user-preset staging branch. New Native prerequisites or
output differences are reported, not bypassed. Resolving preset lookup is
not whole-file parity or a family-ledger closure.

Adjacent Ares profile resolution, Native implementation, oracle selection,
root test registration, production changes and additional test families are
outside this adaptation. The independent classic-layer-id provenance copy
remains outside the 28-case data-cutover scope.
