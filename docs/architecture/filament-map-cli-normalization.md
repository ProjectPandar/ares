# CLI filament-map normalization and cardinality

## Upstream rewrite boundary

This slice ports `OrcaSlicer.cpp:6027-6034` before `Print::apply`, together
with the overlong-map admission exercised by that entry path. Its Rust
normalization destination is `options/project_variants.rs::expanded_filament_map`.
The additional API admission boundary is
`project/effective_config/cardinality.rs::validate`.

`PrintApply.cpp:1162-1172` materializes variants, and `1184-1185` collects
`filament_map` without rejecting an overlong vector. The multiple-filament
materializer explicitly uses the map's size (`PrintConfig.cpp:9817-9837`).
An exact map-length-equals-diameter-count gate is therefore not this upstream
entry contract. The existing Ares equality gate is replaced by a minimum
length gate, not by input truncation.

## Non-negotiable decisions

- Grow a short map to `filament_count` using default `1` only when shorter.
- Never truncate an overlong map at this normalization boundary.
- For one physical extruder, set only entries `0..filament_count` to `1`.
  Do not overwrite the overlong tail.
- Require enough map entries for the logical filaments, and retain both the
  existing positive/in-range physical-ID gate and variant-index bounds check.
- Do not move validation, add a fallback at an error, change variant selection,
  or change the auto-map recommendation/export pass.
- `expanded_filament_map` remains a temporary compatibility-shell name around
  the cited CLI normalization; it is not a new Ares mapping policy.
- This platform-neutral logic remains within `ares-core`: no filesystem,
  process, terminal, or platform-dependent behavior is added.

## Included and deferred behavior

Included: CLI grow-only behavior, the single-extruder prefix reset, admission
of valid overlong vectors, and strict end-to-end Native/API/CLI verification.
Deferred: changes to physical-ID validation policy, malformed map recovery,
variant fallbacks, manual mapping semantics, and unrelated PrintApply behavior.
Those existing owners are unchanged by this slice.

## Verification

The frozen X1C cube has one consumable/one nozzle and plate map `1 1`.
The canonical `runner::native_stream_case::check` registration has no opt-in,
seeded-only skip, expected mismatch pin, or bespoke runner. Official and seeded
modes each qualify two Native outputs before strict complete generator-line-only
API/CLI comparisons and raw API/CLI equality. The richer KSR's known official
object-ID nondeterminism does not grant this distinct fixture an exception.

The failing public API was independently observed in GDB entering the
cardinality owner with map length 2, diameter count 1, and nozzle count 1;
that owner returns the reported `InvalidInput` before slicing. RED precedes
any production edit. Canonical FULL qualification remains coordinator-owned.
