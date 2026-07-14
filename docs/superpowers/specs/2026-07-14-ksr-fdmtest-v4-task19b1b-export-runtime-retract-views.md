# Task 19B.1B: Export/Runtime Retract Views

## Status and objective

Task 19B.1A materializes the four fixed Orca variant families into an owned,
typed `ProjectSettings`. Task 19B.1B ports the immediately following fixed
Orca boundary: preserve that materialized configuration for full-config export,
derive the distinct runtime configuration by applying the nullable filament
retract overrides, and project the runtime `GCodeConfig` fields into the
existing typed `GCodeOptions`.

This is a bounded configuration rewrite slice. It does not wire project
slicing, normalize FDM configuration, resolve model/object/region settings, or
emit G-code. The public project path must still return
`ProjectSlicingIncomplete` after this task.

The persistent goal remains byte-for-byte `ksr_fdmtest_v4` G-code parity after
normalizing only the allowed generator name and timestamp metadata.

## Chosen design

The implementation will add one crate-private typed result representing the
fixed Orca state split:

```rust
pub(crate) struct ProjectConfigViews {
    pub(crate) full: ProjectSettings,
    pub(crate) runtime: ProjectSettings,
    pub(crate) runtime_gcode: GCodeOptions,
}

pub(crate) fn resolve_project_config_views(
    full: ProjectSettings,
) -> Result<ProjectConfigViews, SliceError>;
```

`full` is the variant-materialized, normalized-at-the-caller source equivalent
to fixed Orca's `m_full_print_config`. The function moves it into the result,
clones it once for the runtime view, applies all sixteen fixed retract
overrides only to the ordinary runtime fields, and builds `runtime_gcode` from
the four existing typed source owners in that runtime view. The nullable
`filament_*` source fields remain present in the runtime clone because
`ProjectSettings` is the current typed compatibility shell; they are not
runtime-effective fields and must never be used to export the runtime view.

This design is selected over two narrower alternatives:

1. Returning only a runtime clone would make the required full/runtime split
   implicit and easy for later orchestration to lose.
2. Overlaying only `GCodeOptions` would omit four fixed `PrintConfig` fields:
   `retract_when_changing_layer`, `retraction_minimum_travel`, `wipe`, and
   `wipe_distance`.

The result is not a new Ares slicing pipeline. It is a typed compatibility
representation of the fixed `PrintBase::m_full_print_config`,
`Print::m_config`, and `GCode::m_config` state boundary. Task 19B.3 owns its
first production orchestration call and may replace this compatibility shell
when it ports the complete fixed `FullPrintConfig` resolution boundary.

The input contract is deliberately narrow: callers supply the post-filament
variant result of Task 19B.1A, after any earlier normalization writes required
by the final fixed stage order. This function does not materialize variants or
reload the project. When a map changes, orchestration must rerun Task 19B.1A
from the original unmaterialized source and pass that new owned result here;
neither a previous `full` nor a previous `runtime` view is a valid remap source.

## Fixed upstream rewrite boundary

The baseline is OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/PrintApply.cpp:222-263::print_config_diffs` enumerates
  `extruder_retract_keys`, pairs each ordinary key with its `filament_` key,
  and computes a runtime override instead of treating the full-config value as
  the effective runtime value.
- `PrintApply.cpp:1164-1191` performs printer/process materialization, saves
  the pre-filament source, performs filament materialization, reads
  `filament_map`, and computes the runtime/full diffs.
- `PrintApply.cpp:1261-1283` applies `filament_overrides` to `m_config` but
  assigns the unoverlaid `new_full_config` to `m_full_print_config`.
- `PrintConfig.cpp:7374-7392` defines the sorted sixteen-key
  `filament_retract_keys` set.
- `PrintConfig.cpp:10300-10332::compute_filament_override_value` clones the
  ordinary machine vector, applies the nullable filament vector using
  `filament_map`, and contains the two long-retraction gate branches.
- `Config.hpp:713-751::ConfigOptionVector<T>::apply_override` defines vector
  sizing, non-nil replacement, nil fallback through the one-based default
  index, invalid-index fallback to the first machine value, and empty-vector
  behavior.
- `Print.cpp:3166-3195::update_filament_maps_to_config` restores the saved
  pre-filament source, rematerializes the full view, then recomputes and applies
  the runtime overrides. This is the rematerialization witness; this task does
  not add auto-map orchestration.
- `PrintConfig.hpp:1300-1478::GCodeConfig` owns twelve of the sixteen ordinary
  runtime fields. `PrintConfig.hpp:1481-1610::PrintConfig` owns the adjacent
  four print-only fields.
- `GCode.cpp:2532-2534,5552-5557` applies `Print::config()` to the writer and
  runtime `GCodeConfig`, while `GCode.cpp:5591-5594::append_full_config` starts
  from `print.full_print_config()`. These are the runtime/export consumers that
  make the split observable.

The Rust destination is
`crates/ares-core/src/options/project_config_views.rs` plus small sibling
modules when required by the 400-LOC limit. It reuses
`ProjectSettings`, `FilamentRetractOverrideOptions`, `GCodeOptions`, and their
existing concrete source-owner types. No generic option representation is
introduced.

## Exact included behavior

### State split and ownership

1. Consume the supplied `ProjectSettings` by value and preserve it exactly as
   `ProjectConfigViews::full`.
2. Clone the full view once to create `runtime`.
3. Apply the sixteen nullable filament retract vectors only to their matching
   ordinary fields in `runtime`.
4. Preserve all other ordinary fields and every nullable `filament_*` source
   field exactly in both views.
5. Build `runtime_gcode` only after all sixteen overlays have been applied,
   using the existing concrete `GCodeOptions::from_sources` projection.
6. `runtime_gcode` must therefore contain the twelve GCode-owned effective
   values, while the four print-only effective values remain observable in
   `runtime.project.print`.
7. The transform must not read a path, archive, fixture name/hash, reference
   G-code, clock, environment variable, or global mutable state.

### Nullable vector application

For each ordinary vector `machine`, nullable filament vector `filament`, and
the one-based `filament_map`:

1. If either `machine` or `filament` is empty, preserve `machine` unchanged.
   This matches the fixed early return before resize and index-count assertion.
2. Otherwise require `filament.len() == filament_map.len()`. At Ares' typed
   external-project boundary, replace the fixed C++ assertion with
   `SliceError::InvalidInput` naming both the concrete `filament_*` key and
   `filament_map`.
3. Clone the pre-overlay machine values as the default vector.
4. Resize the ordinary runtime vector to `filament.len()`, filling new entries
   from the first machine value.
5. A non-nil filament entry replaces the runtime value at the same logical
   filament index.
6. A nil filament entry selects the machine default at
   `filament_map[index] - 1` when that one-based entry is in range.
7. A zero, negative, or out-of-range map entry used by a nil entry falls back
   to the first machine value, matching fixed `apply_override`. No new map
   validation is added here; Task 19B.1A already validates active variant
   materialization, while the inactive branch intentionally remains lazy.
8. The operation is type-preserving for floats, percents, bools,
   `RetractLiftEnforce`, and `ZHopType`. It does not stringify, serialize, or
   dynamically dispatch by key.

The old/new comparison and diff-key collection in fixed `Print::apply` are
incremental invalidation mechanics. Because this pure transform constructs the
final state directly, it must reproduce the computed runtime values, not the
intermediate changed-key list.

### Long-retraction gate

`enable_long_retraction_when_cut == 2` is fixed
`LongRectrationLevel::EnableFilament` and applies both nullable filament values
normally.

For every other integer value, preserve the exact fixed-commit behavior:

1. `filament_long_retractions_when_cut` is replaced conceptually by an
   equally-sized all-nil vector, so `long_retractions_when_cut` is resized to
   logical filament count and selected from the machine defaults through
   `filament_map`.
2. The fixed source accidentally fills the bool temporary while passing an
   empty float temporary for `filament_retraction_distances_when_cut`.
   Therefore `retraction_distances_when_cut` takes the empty-vector early
   return and remains the materialized machine vector unchanged. This typo is
   observable when physical and logical cardinalities or mappings differ and
   is required for fixed-commit parity.

Production code must express these two typed branches directly. It must not
copy the old dynamic JSON helper, which normalized both special fields to
same-sized nil arrays and therefore does not match the fixed distance behavior.

### Exact sixteen-field mapping

| Ordinary runtime field | Nullable filament source | Ordinary typed owner | In `GCodeOptions` |
| --- | --- | --- | --- |
| `deretraction_speed` | `filament_deretraction_speed` | `project.gcode` | yes |
| `long_retractions_when_cut` | `filament_long_retractions_when_cut` | `printer.gcode` | yes |
| `retract_before_wipe` | `filament_retract_before_wipe` | `project.gcode` | yes |
| `retract_lift_above` | `filament_retract_lift_above` | `project.gcode` | yes |
| `retract_lift_below` | `filament_retract_lift_below` | `project.gcode` | yes |
| `retract_lift_enforce` | `filament_retract_lift_enforce` | `printer.gcode` | yes |
| `retract_restart_extra` | `filament_retract_restart_extra` | `project.gcode` | yes |
| `retract_when_changing_layer` | `filament_retract_when_changing_layer` | `project.print` | no |
| `retraction_distances_when_cut` | `filament_retraction_distances_when_cut` | `printer.gcode` | yes |
| `retraction_length` | `filament_retraction_length` | `project.gcode` | yes |
| `retraction_minimum_travel` | `filament_retraction_minimum_travel` | `project.print` | no |
| `retraction_speed` | `filament_retraction_speed` | `project.gcode` | yes |
| `wipe` | `filament_wipe` | `project.print` | no |
| `wipe_distance` | `filament_wipe_distance` | `project.print` | no |
| `z_hop` | `filament_z_hop` | `project.gcode` | yes |
| `z_hop_types` | `filament_z_hop_types` | `printer.gcode` | yes |

`travel_slope` remains a printer variant field but is not a nullable filament
override key. It must not be included in this transform.

## Required tests and TDD evidence

Implementation begins with a genuine compiler/test RED for the missing typed
API. Each subsequent slice records its own focused RED and GREEN command.

1. **Typed vector semantics**
   - mixed value/nil floats with a non-identity map prove direct override and
     map-indexed fallback;
   - bool, percent, `RetractLiftEnforce`, and `ZHopType` cases prove concrete
     type preservation;
   - output resizes to logical filament count;
   - zero, negative, and out-of-range nil fallback use machine index zero;
   - empty machine or empty filament vectors preserve the machine vector;
   - a non-empty filament/map length mismatch returns the required keyed
     `SliceError`.
2. **Complete sixteen-key split**
   - assign unique sentinels to every ordinary and nullable field;
   - prove all sixteen runtime fields change according to the same fixed
     semantics;
   - prove the full view remains byte-for-byte/field-for-field equal to the
     owned input;
   - prove fields outside the sixteen-key set are unchanged;
   - prove the four print-only results are not silently lost;
   - prove all twelve GCode-owned results in `runtime_gcode` equal the runtime
     view and differ from full sentinels when the override requires it.
3. **Long-retraction gate**
   - value `2` applies both special overrides;
   - another value maps all-nil bool entries through `filament_map`;
   - the same disabled/machine value leaves the distance machine vector and
     its physical cardinality unchanged, freezing the fixed typo;
   - unrelated override keys are unaffected by the gate.
4. **Committed fixture**
   - load the 3MF through `load_project`, call the already-approved variant
     materializer with its typed map, and then resolve the views;
   - prove the full view retains the reference config-block values, including
     `deretraction_speed = [30, 20]`,
     `retraction_distances_when_cut = [18, 18]`,
     `retraction_length = [0.8, 2]`,
     `retraction_speed = [30, 20]`, `wipe_distance = [2, 2]`, and
     `z_hop_types = [Auto Lift, Auto Lift]`;
   - prove the runtime view uses the fixture's solely typed inputs and map to
     produce `deretraction_speed = [30, 30]`,
     `retraction_distances_when_cut = [10, 10]`,
     `retraction_length = [0.4, 0.4]`,
     `retraction_speed = [30, 30]`, `wipe_distance = [1, 1]`, and
     `z_hop_types = [Spiral Lift, Spiral Lift]`;
   - prove `runtime_gcode` carries the twelve effective GCode-owned values and
     that public core/CLI/browser project slicing remains incomplete.
5. **Composed rematerialization**
   - resolve two different maps by rerunning Task 19B.1A from the same raw
     source before this transform;
   - prove the raw source remains unchanged and both results are deterministic;
   - prove feeding a previous `full` or `runtime` view is not used anywhere in
     the supported call path.

Fixture expectations belong only in tests. Production code must not branch on
these values or the fixture identity.

## Obsolete scaffold and source-pinning policy

This task deletes the existing dead dynamic
`options/filament_override.rs` scaffold, its four sibling test files, its
`options.rs` module declaration, and its fingerprints from
`scripts/dynamic_value_baseline.txt`. The module has no production caller; it
contains test-only staged JSON implementations of override and adjacent diff
mechanics, and its distance-gate behavior differs from the fixed source. If a
later fixed rewrite slice needs diff/invalidation mechanics, it must implement
the actually consumed typed boundary rather than restore this source-structure
pinning shell.

No committed test may read, grep, hash, or otherwise pin the OrcaSlicer source
tree or fixed source text. Upstream source inspection is review evidence only.
Behavior is frozen through typed synthetic tests and the committed 3MF/G-code
fixture. Existing obsolete source-text pinning tests encountered inside the
owned file set must be deleted rather than updated.

## Explicitly deferred

- `normalize_fdm`, `normalize_fdm_1`, `normalize_fdm_2`, active sizing,
  object/region resolution, and the complete `FullPrintConfig` orchestration
  order (Task 19B.3).
- Model option classification, material/layer-range document import, and
  association (Task 19B.2).
- Placeholder parser update/diff mechanics and automatic filament-map runtime
  mutation.
- Full-config/config-block serialization (Task 19C). This task preserves the
  source view but emits no config bytes.
- Remaining dynamic `SliceOptions` compatibility-path removal (Task 20E).
- Geometry, slicing, toolpaths, G-code emission, metadata, post-processing, and
  final golden parity.

## Architecture and platform constraints

- `ares-core` remains byte/in-memory only and portable across browser WASM,
  Windows, macOS, and Linux.
- Production code contains no `serde_json::Value`, JSON map, raw JSON,
  type-erased option, string-key runtime dispatch, filesystem, terminal,
  process, clock, OpenGL, or native-only API.
- The transform operates only on existing concrete typed option fields.
  Generic helpers may be monomorphized over `T: Clone`; all sixteen field
  applications remain explicit and reviewable.
- New or changed Rust modules stay below 400 physical lines and are split by
  source responsibility.
- No legacy fallback, fixture hardcoding, source-tree dependency, or new crate
  or dependency is allowed.
- Fixture bytes remain unchanged.

## Approval, documentation, and release gates

1. Freeze this spec and obtain literal `VERDICT: APPROVE` from a fresh
   independent Agent and OpenCode. Any spec edit invalidates both approvals.
2. Write the detailed Superpowers Subagent-Driven TDD plan and obtain the same
   two independent approvals. No production/test implementation begins before
   both plan approvals.
3. Execute bounded implementation slices with fresh implementer and reviewer
   agents. Verify every claimed RED/GREEN result in the shared workspace.
4. Freeze the implementation manifest. Obtain literal `VERDICT: APPROVE` from
   a fresh independent spec-compliance reviewer, a separate code-quality
   reviewer, and OpenCode. Any production/test edit invalidates all final
   implementation approvals.
5. Only after implementation approval, update
   `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, and the ignored
   SDD progress ledger. Freeze and independently approve the docs-only diff.
6. Run focused tests, adjacent typed project/G-code tests, full workspace
   nextest, rustfmt, warning-denying Clippy, native/WASM checks, release WASM,
   wasm-bindgen browser tests, the dynamic-value audit, fixture hash checks,
   no-hardcoding/source-pinning scans, per-file LOC checks, and frozen-manifest
   equality checks.
7. Stage only the frozen manifest, use a Conventional Commits message approved
   by the plan, push the branch, and require all five Tier 1 jobs green for the
   exact pushed SHA before declaring Task 19B.1B complete.

Task 19B.1B completion does not complete Task 19B or the persistent
`ksr_fdmtest_v4` slicing goal.
