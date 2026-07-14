# KSR FDM Test V4 Task 19B.1A: Typed Variant Materialization Spec

## Status and relationship to the persistent goal

This is the first independently releasable slice of Task 19B in
`2026-07-10-ksr-fdmtest-v4-gcode-parity.md`. It does not claim complete
`ksr_fdmtest_v4` slicing or G-code parity. It adds only the pure typed active
variant transform needed later by the effective `FullPrintConfig` resolver.

Task 19B is deliberately split further so each source boundary can be reviewed,
implemented, documented, committed, pushed, and proven on Tier 1 independently:

1. **19B.1A (this spec):** typed printer/process/filament variant
   materialization.
2. **19B.1B:** nullable filament retract overlay and the distinct runtime
   G-code view.
3. **19B.2:** retained model-option classification and bounded optional layer
   configuration import/association.
4. **19B.3:** typed FDM normalization and source-ordered effective
   `FullPrintConfig` orchestration.

This split preserves fixed Orca stage order: 19B.1A supplies a pure transform
but does not decide when slicing calls it.

## Goal

Add a crate-private, filesystem-free, fully typed transform:

```rust
pub(crate) fn materialize_project_variants(
    source: &ProjectSettings,
    filament_map: &OrcaInts,
) -> Result<ProjectSettings, SliceError>;
```

The function clones an unmaterialized typed `ProjectSettings` source, installs
the explicit logical-filament-to-physical-extruder map in the clone, and
selects the active values for the exact four fixed Orca variant families. Here
"unmaterialized" is relative only to variant selection: the source may already
contain the writes made by an earlier typed `normalize_fdm_1` / first
`normalize_fdm_2` stage. The input is never mutated. Callers must always
rematerialize from that unmaterialized source; a previously materialized
output is not a valid source.

For the committed fixture, printer variant 1 and process selection start from
raw base indices `[0, 2]`, printer variant 2 re-resolves base indices `[0, 1]`
after variant 1 materializes the shared printer selectors, and filament
selection starts from raw logical-filament indices `[0, 4]`. The resulting
silent-mode stride positions are `[0, 1, 2, 3]`. The selected
`printer_extruder_id` and `print_extruder_id` *values* are `[1, 2]`; these
values are not the zero-based source indices. Production must not use the
fixture path, name, hashes, or reference G-code.

## Fixed upstream rewrite boundary

The baseline is OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/PrintConfig.cpp:8344-8473`: the four semantic variant key
  sets. The repeated `nozzle_volume` insertion is one `std::set` member, so
  the unique counts are 2 / 37 / 24 / 15.
- `PrintConfig.cpp:8981-9002`: physical extruder count and supported-variant
  detection from `nozzle_diameter` and comma-separated
  `extruder_variant_list`.
- `PrintConfig.cpp:588-606::get_extruder_variant_string`: canonical typed
  extruder/nozzle-volume spelling used by index lookup.
- `PrintConfig.cpp:9004-9054`: complete and generated ID-map lookup and first
  exact variant match.
- `PrintConfig.cpp:9634-9803`: physical printer and process materialization,
  including stride-two machine-limit pairs.
- `PrintConfig.cpp:9805-10023`: logical filament materialization through
  `filament_map`, `filament_self_index`, and `filament_extruder_variant`.
- `PrintApply.cpp:1164-1173`: runtime family order: printer stride one,
  printer stride two, process, then filament.
- `Print.cpp:3166-3175`: a changed filament map restores the saved
  pre-filament state before rerunning only filament materialization; it proves
  the restore-before-rematerialize rule. `PrintApply.cpp:1165-1173` is the
  source witness for this slice's complete four-family transform.

The Rust destination is
`crates/ares-core/src/options/project_variants.rs` plus small sibling modules
only when required by the 400-LOC limit. The existing typed
`ProjectSettings`, `PrinterOptions`, `ProcessOptions`, `FilamentOptions`, and
`ProjectRuntimeOptions` remain the owning data structures. This slice does not
introduce an Ares-owned pipeline abstraction.

## Exact included behavior

### Raw and effective state

1. Clone `source` once and mutate only the clone.
2. Write the explicit `filament_map` argument to the clone's canonical typed
   field before filament selection.
3. Preserve every field outside the four variant families and
   `filament_map` exactly.
4. Preserve raw selector metadata that upstream does not materialize,
   especially `filament_self_index` and `extruder_variant_list`.
5. Materialize selector fields that are themselves family members:
   `printer_extruder_id`, `printer_extruder_variant`, `print_extruder_id`,
   `print_extruder_variant`, and `filament_extruder_variant`.
6. Return a variant-materialized `ProjectSettings`, not a final
   `FullPrintConfig`, runtime G-code view, or serialized config block.

### Sequential family evaluation

The four fixed calls are not independent projections from the original
`source`. Matching `PrintApply.cpp:1164-1173`, each call resolves indices from
the current, already-mutated clone and then writes its complete family before
the next call:

1. Printer variant 1 resolves from the unmaterialized
   `printer_extruder_id` / `printer_extruder_variant` selectors and writes all
   24 members, including those two selectors. For the fixture its base indices
   are `[0, 2]`, and the selectors become `[1, 2]` and
   `[Direct Drive Standard, Bowden Standard]`.
2. Printer variant 2 invokes index resolution again against those now-shortened
   printer selectors. For the fixture its base indices are therefore `[0, 1]`,
   and each still-unmaterialized stride-two payload selects positions
   `[0, 1, 2, 3]`. Reusing variant-1 base indices here would incorrectly select
   `[0, 1, 4, 5]`.
3. Process selection resolves from its separate, still-unmaterialized
   `print_extruder_id` / `print_extruder_variant` selectors. For the fixture its
   base indices remain `[0, 2]` before the two selector fields are written.
4. Filament selection resolves from its separate, still-unmaterialized
   `filament_self_index` / `filament_extruder_variant` selector state and the
   installed `filament_map`. For the fixture its raw logical-filament indices
   are `[0, 4]`.

This sequential mutation is observable even though the transform remains pure
to its caller: only the clone is mutated, while `source` remains unchanged.

The input contract matches the later fixed stage order. Task 19B.3 must create
the unmaterialized typed source, run `normalize_fdm_1`, run the cold-start first
`normalize_fdm_2`, and only then call this transform. Fixed
`normalize_fdm_1` does not change the selector/ID metadata, but it does rewrite
two payload members in these families when spiral mode is active:
`retract_when_changing_layer` and
`filament_retract_when_changing_layer`. Therefore this function must select
the values present in the supplied source and must not reload the original 3MF
settings internally. The fixture-only direct test below proves this pure
transform, not final orchestration order.

The complete `ProjectSettings` output is intentional: the 24-key printer
family crosses Ares typed owners. Machine retract fields such as
`retraction_length` live in `ProjectRuntimeOptions`, while nozzle and selector
fields live in `PrinterOptions`.

### Selection guard and index resolution

1. The physical extruder count is the `nozzle_diameter` length. An empty vector
   is rejected as `nozzle_diameter` at this typed external boundary.
2. Reproduce the fixed `support_different_extruders` scan exactly. Inspect only
   group positions `0..nozzle_diameter.len()`. A short non-empty
   `extruder_variant_list` repeats its first group through Orca `get_at`
   semantics; trailing groups beyond the physical count are ignored; an empty
   group vector is rejected as `extruder_variant_list` instead of reproducing
   the C++ assertion. Split each scanned group on runs of one or more commas,
   preserve leading/trailing empty tokens, do **not** trim whitespace, and add
   those exact tokens to the distinct-token set. This guard intentionally
   differs from generated ID-map token handling below, which trims and skips
   empty tokens.
3. Run materialization when the physical count exceeds one or that exact guard
   finds more than one distinct token. Otherwise return the clone with only
   the explicit `filament_map` replacement. Matching fixed `Print::apply`, the
   no-op branch does not inspect or validate extruder/nozzle selector vectors,
   family payloads, or `filament_map`; Task 19B.3 owns general project
   validation. A focused test must freeze this behavior with an invalid map on
   the no-op branch.
4. An active physical extruder's exact variant token is the typed
   `extruder_type` plus typed `nozzle_volume_type`, using Orca's canonical
   spelling such as `Direct Drive Standard`.
5. For each active physical position, preserve Orca `get_at` semantics for the
   typed `extruder_type` and `nozzle_volume_type` control vectors: a short
   non-empty vector repeats its first element; an empty vector is rejected with
   its exact key rather than reproducing a C++ assertion.
6. When an ID vector covers the complete variant vector, match the requested
   one-based physical/logical ID from that vector. Otherwise generate physical
   IDs by scanning **all stored** `extruder_variant_list` groups, splitting on
   compressed comma runs, trimming every token, and skipping empty trimmed
   tokens. This is deliberately different from the guard scan.
7. Choose the first exact `(ID, variant)` match. Unrelated trailing source
   entries are allowed.
8. For physical printer/process selection, resolve one source base index for
   each physical extruder ID `1..=nozzle_diameter.len()`.
9. For filament selection, every one-based `filament_map` entry names a valid
   physical extruder. Resolve one source index per logical filament ID against
   raw `filament_self_index` and raw `filament_extruder_variant`, using the
   mapped physical extruder's type and nozzle-volume type.
10. Once the guard selects the active branch, an empty required variant
    selector, missing exact match, zero/out-of-range filament map entry, or
    selected payload position outside its concrete vector is an
    external-project error naming the offending Orca key. Complete ID vectors
    may contain trailing entries; a shorter ID vector uses the generated map
    branch exactly as above.
11. This slice intentionally does **not** reproduce two fixed C++ recovery
    behaviors at the untrusted project boundary: printer/process payload
    `ConfigOptionVector::get_at` would repeat the first value for an
    out-of-range selected position, and filament materialization would log and
    leave a default-constructed output element. Ares rejects both with the
    payload key. This also replaces the transient UI missing-match
    assert/index-zero behavior with a keyed error. Tests must cover all three
    stricter divergences. The source fallback definition is
    `Config.hpp:624-630`; filament bounds checks are in
    `PrintConfig.cpp:9869-10011`.

The implementation may use a small monomorphized `select_stride<T: Clone>`
helper over concrete vectors. It must explicitly cover ordinary typed vectors
and the existing Vec-backed special wrappers used by the four families,
including `NullableInts` (`nozzle_flush_dataset`), `VariantStride`
(`filament_extruder_variant`), and `SpaceTuple`
(`volumetric_speed_coefficients`). It may not use a generic option value,
runtime key/value map, type erasure, or JSON reserialization.

### Exact materialized families

The existing Ares registry key lists are semantic ledgers used to review the
concrete field implementations; they are not runtime dynamic dispatch tables.

| Family | Unique fields | Stride | Selected output for N physical/logical entries |
| --- | ---: | ---: | ---: |
| Process | 2 | 1 | N physical entries |
| Filament | 37 | 1 | N logical filament entries |
| Printer variant 1 | 24 | 1 | N physical entries |
| Printer variant 2 | 15 | 2 | 2N physical entries |

The process family is `print_extruder_id` and `print_extruder_variant`.

The printer variant-1 family is:
`deretraction_speed`, `long_retractions_when_cut`, `nozzle_flush_dataset`,
`nozzle_type`, `nozzle_volume`, `printer_extruder_id`,
`printer_extruder_variant`, `retract_before_wipe`,
`retract_length_toolchange`, `retract_lift_above`, `retract_lift_below`,
`retract_lift_enforce`, `retract_restart_extra`,
`retract_restart_extra_toolchange`, `retract_when_changing_layer`,
`retraction_distances_when_cut`, `retraction_length`,
`retraction_minimum_travel`, `retraction_speed`, `travel_slope`, `wipe`,
`wipe_distance`, `z_hop`, and `z_hop_types`.

The printer variant-2 family is:
`machine_max_acceleration_e`, `machine_max_acceleration_extruding`,
`machine_max_acceleration_retracting`, `machine_max_acceleration_travel`,
`machine_max_acceleration_x`, `machine_max_acceleration_y`,
`machine_max_acceleration_z`, `machine_max_jerk_e`, `machine_max_jerk_x`,
`machine_max_jerk_y`, `machine_max_jerk_z`, `machine_max_speed_e`,
`machine_max_speed_x`, `machine_max_speed_y`, and `machine_max_speed_z`.

The filament family is:
`activate_air_filtration`, `activate_air_filtration_during_print`,
`activate_air_filtration_on_completion`, `complete_print_exhaust_fan_speed`,
`during_print_exhaust_fan_speed`, `filament_adaptive_volumetric_speed`,
`filament_cooling_before_tower`, `filament_deretraction_speed`,
`filament_extruder_variant`, `filament_flow_ratio`, `filament_flush_temp`,
`filament_flush_volumetric_speed`, `filament_ironing_flow`,
`filament_ironing_inset`, `filament_ironing_spacing`,
`filament_ironing_speed`, `filament_long_retractions_when_cut`,
`filament_max_volumetric_speed`, `filament_retract_before_wipe`,
`filament_retract_lift_above`, `filament_retract_lift_below`,
`filament_retract_lift_enforce`, `filament_retract_restart_extra`,
`filament_retract_when_changing_layer`,
`filament_retraction_distances_when_cut`, `filament_retraction_length`,
`filament_retraction_minimum_travel`, `filament_retraction_speed`,
`filament_wipe`, `filament_wipe_distance`, `filament_z_hop`,
`filament_z_hop_types`, `long_retractions_when_ec`, `nozzle_temperature`,
`nozzle_temperature_initial_layer`, `retraction_distances_when_ec`, and
`volumetric_speed_coefficients`.

## Required tests and TDD evidence

Implementation begins with a genuine RED caused by the missing production API,
then proceeds in independently reviewable RED/GREEN slices.

1. **Index resolver RED/GREEN**
   - complete ID maps;
   - generated ID maps with compressed/trimmed comma tokens;
   - first exact match;
   - guard scans only active groups, repeats a short non-empty first group,
     ignores trailing groups, preserves untrimmed/edge-empty tokens, and stays
     observably distinct from generated-map token handling;
   - one physical extruder with multiple supported variants triggers selection;
   - one extruder/one supported variant is a no-op;
   - invalid selector/payload/map data is not inspected on that no-op branch;
   - short non-empty `extruder_type` and `nozzle_volume_type` vectors repeat
     their first typed value, while empty vectors name their key;
   - key-specific invalid selector and filament-map errors.
2. **Printer/process RED/GREEN**
   - unique sentinels choose base indices `[0, 2]`;
   - after variant 1 shortens the shared selectors, variant 2 re-resolves base
     indices `[0, 1]`; a raw stride-two sentinel vector
     `[10, 11, 20, 21, 30, 31]` must produce `[10, 11, 20, 21]`, distinguishing
     the required positions `[0, 1, 2, 3]` from stale-base reuse at
     `[0, 1, 4, 5]`;
   - every 24/15/2 concrete field is covered and its output cardinality is
     frozen;
   - a selected out-of-range position names the payload key.
   - strict selected-payload errors are distinguished from fixed C++
     first-element/default-output recovery.
3. **Filament RED/GREEN**
   - a synthetic `filament_map = [1, 2]` with distinct data selects indices
     that cannot pass through a `len / count` shortcut, including the second
     logical filament at raw index 6;
   - all 37 concrete fields are covered, including nullable and enum vectors;
   - rerunning from the same raw source is deterministic;
   - rematerializing a new map from the same raw source changes only the
     filament family plus `filament_map`.
   - already-normalized payload values supplied by the caller, including the
     two spiral-mode retract members, are selected without reloading raw 3MF
     values.
4. **Real fixture RED/GREEN**
   - load the committed 3MF through `load_project` and materialize its typed
     settings;
   - assert raw input remains unchanged;
   - assert printer/process IDs become `[1, 2]`, variants become
     `[Direct Drive Standard, Bowden Standard]`, printer
     `retraction_length` becomes `[0.8, 2]`,
     `machine_max_acceleration_e` becomes `[30000, 5000, 30000, 5000]`, and
     `machine_max_speed_e` becomes `[30, 30, 30, 30]`;
   - assert filament variants become two `Direct Drive Standard` values,
     `filament_max_volumetric_speed` becomes `[21, 21]`, and all 37 filament
     family fields have two entries;
   - assert `filament_self_index` remains its raw eight entries;
   - keep the public project-slicing boundary at the existing
     `ProjectSlicingIncomplete` result because orchestration is deferred.

Tests may use the fixture to prove behavior, but production code may not read
its path/name/hash/reference output or branch on any of its values. Tests must
not read mutable Orca source or pin source line text. Fixed-source inspection
is review evidence only.

## Explicitly deferred

- `normalize_fdm`, `normalize_fdm_1`, and `normalize_fdm_2`; 19B.3 owns the
  typed calls and stage order. The monolithic `normalize_fdm` is not called by
  fixed `Print::apply`.
- `get_parameter_size`, `extend_extruder_variant`, `set_num_extruders`, and
  `set_num_filaments`; these are preset/UI sizing behavior and must not truncate
  the raw 4-/8-entry project vectors before selection.
- Nullable filament retract overlay, export/full-config versus runtime G-code
  separation, and final `GCodeOptions` projection (19B.1B/19B.3).
- `FullPrintConfig`, used-filament discovery, model/volume/layer association,
  second-pass normalization, and wiring into `slice_project` (19B.2/19B.3).
- Config-block serialization (19C), dynamic `SliceOptions` removal (20E), and
  all geometry, toolpath, G-code emission, and final golden parity work.
- Auto filament-map orchestration. This slice only provides source-faithful
  rematerialization from the original raw settings with an explicit new map.

No existing dynamic `SliceOptions` helper may be called or expanded. This
slice's typed `project_variants.rs` is the project-path replacement for the
same upstream boundary currently represented by the temporary dynamic
compatibility shell in `options/update_printer_extruders.rs` and
`options/update_printer_extruders/multiple_filament.rs`, including
`SliceOptions::update_values_to_printer_extruders_string_int_keys`,
`SliceOptions::update_values_to_printer_extruders_for_multiple_filaments_string_int_keys`,
and `ExtruderIndexIdMapLookup`. Those dynamic APIs remain only for the deferred
STL/legacy path until Task 20E; this slice neither calls, expands, nor deletes
them.

## Architecture and platform constraints

- `ares-core` remains byte/in-memory only: no filesystem, terminal, process,
  clock, OpenGL, or native-only API.
- Production code contains no `serde_json::Value`, JSON `Map`, `RawValue`,
  `BTreeMap<String, _>`, erased option value, or string-key runtime dispatch.
- Values remain in concrete typed option fields; helper generics are
  monomorphized over those fields.
- New Rust files remain below 400 physical lines and are split by source
  responsibility when needed.
- Existing fixture bytes remain unchanged.
- The behavior must compile and test on native Tier 1 and browser WASM.

## Approval, documentation, and release gates

1. Freeze this spec and obtain literal `VERDICT: APPROVE` from both a fresh
   independent Agent and OpenCode. Any edit invalidates both approvals.
2. Write a detailed Superpowers implementation plan and obtain the same two
   independent approvals. No production implementation starts before both
   plan approvals.
3. Use Subagent-Driven TDD for the implementation slices. Each implementer
   receives only its bounded task and the approved spec/plan.
4. Freeze the implementation manifest. A fresh independent spec-compliance
   reviewer and a separate code-quality reviewer must approve it; OpenCode
   must independently approve the whole implementation. Any production/test
   edit invalidates those approvals.
5. Only after implementation approval, update
   `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, and the ignored
   SDD progress ledger. Freeze and independently approve the docs-only diff.
6. Run focused tests, adjacent typed project/G-code tests, full workspace
   nextest, rustfmt, warning-denying Clippy, native/WASM checks, release WASM,
   wasm-bindgen browser tests, dynamic-value audit, fixture-hash checks,
   no-hardcoding scans, per-file LOC checks, and diff/manifest equality checks.
7. Stage only the frozen manifest, use Conventional Commits message
   `feat(config): materialize active variant options`, push the branch, and
   require all five Tier 1 jobs green for that exact pushed SHA before 19B.1B.

Task 19B.1A is complete only after every gate above succeeds. The persistent
`ksr_fdmtest_v4` goal remains active afterward.
