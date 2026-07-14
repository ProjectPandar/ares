# Task 19B.1B Export/Runtime Retract Views Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve the variant-materialized full/export project settings while deriving the fixed Orca runtime retract view and its 12-field effective `GCodeOptions` projection entirely from typed 3MF options.

**Architecture:** Add a crate-private `ProjectConfigViews` compatibility representation of fixed Orca's `m_full_print_config`, runtime `m_config`, and runtime `GCodeConfig`. An owned full view is cloned once; a small monomorphized nullable-vector helper explicitly overlays all sixteen retract fields into the runtime clone; `GCodeOptions::from_sources` then projects the twelve GCode-owned effective fields. Delete the dead dynamic JSON source-structure scaffold that this typed slice supersedes.

**Tech Stack:** Rust 1.91.0, existing `ares-core` typed option structs, Cargo Nextest, rustfmt, Clippy, wasm32, wasm-bindgen browser tests, PowerShell, independent Agent and OpenCode review gates.

---

## Reviewed specification and fixed source

- Reviewed spec:
  `docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task19b1b-export-runtime-retract-views.md`
- Frozen spec SHA-256:
  `d19db197eb2d302f536d41a8694aaf8b34f0806a1c49b92c689f91f72bc17647`
- Independent Agent verdict: `VERDICT: APPROVE`
- OpenCode verdict: `VERDICT: APPROVE`
- Fixed OrcaSlicer commit:
  `8500fcdccaa10b5099ac20d252af3a7c560046f1`
- Implementation base commit:
  `da896a98719a621ad87a2317c23f1d27f0a3c6e5`

Before dispatching any implementer, verify these exact bytes and base state:

```powershell
(Get-FileHash docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task19b1b-export-runtime-retract-views.md -Algorithm SHA256).Hash.ToLower()
git rev-parse HEAD
git status --short
```

Expected: the spec hash above, base SHA above, and only the approved untracked
spec/plan documents. Any spec edit invalidates both spec approvals and stops
implementation until fresh dual approval.

## Fixed upstream rewrite boundary

This plan ports one bounded slice from fixed OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintApply.cpp:222-263::print_config_diffs` and
  `PrintApply.cpp:1164-1191,1261-1283` define the sixteen retract-key diff,
  variant/full source order, runtime override application, and preserved full
  config.
- `src/libslic3r/PrintConfig.cpp:7374-7392` owns the sorted sixteen-key set;
  `PrintConfig.cpp:10300-10332::compute_filament_override_value` owns the
  nullable overlay and the two fixed long-retraction branches.
- `src/libslic3r/Config.hpp:713-751::ConfigOptionVector<T>::apply_override`
  owns empty-vector, cardinality, `Value`, one-based `Nil`, and invalid-map
  fallback semantics.
- `src/libslic3r/Print.cpp:3166-3195::update_filament_maps_to_config` is the
  raw-source rematerialization witness; this task tests the composition but does
  not add automatic map orchestration.
- `src/libslic3r/PrintConfig.hpp:1300-1478::GCodeConfig` owns the twelve runtime
  GCode fields; `PrintConfig.hpp:1481-1610::PrintConfig` owns the four adjacent
  print-only fields.
- `src/libslic3r/GCode.cpp:2532-2534,5552-5557` consumes the runtime config;
  `GCode.cpp:5591-5594::append_full_config` consumes the preserved full config.

The exact Rust destination is
`crates/ares-core/src/options/project_config_views.rs` and
`crates/ares-core/src/options/project_config_views/retract.rs`, reusing the
existing typed `ProjectSettings`, `FilamentRetractOverrideOptions`, and
`GCodeOptions` owners. Included behavior is the owned full/runtime split,
sixteen typed overlays, exact nullable/map semantics, fixed long-retraction
typo, 12/4 projection, and raw-source-only rematerialization proof. Deferred
behavior remains Task 19B.2 document association, Task 19B.3 full orchestration,
Task 19C serialization, Task 20E remaining dynamic `SliceOptions` removal, and
all geometry/slicing/G-code emission/golden parity.

The existing `options/filament_override.rs` JSON implementation is only a
temporary compatibility scaffold around this upstream concept. Once the typed
replacement is GREEN it and its four source-structure/pinning tests are deleted;
the already-shipped typed variant materializer remains the caller that supplies
the full view, not an Ares-owned replacement pipeline.

## Locked file structure

Create:

- `crates/ares-core/src/options/project_config_views.rs` — result type and
  owned full/runtime/GCode façade only.
- `crates/ares-core/src/options/project_config_views/retract.rs` — typed
  nullable-vector algorithm, long-retraction gate, and sixteen explicit field
  applications.
- `crates/ares-core/src/options/tests/project_config_views.rs` — test module
  root.
- `crates/ares-core/src/options/tests/project_config_views/support.rs` — small
  typed constructors and exhaustive expected-value assertions.
- `crates/ares-core/src/options/tests/project_config_views/vector.rs` — generic
  vector semantics through the production façade.
- `crates/ares-core/src/options/tests/project_config_views/fields.rs` — all
  sixteen fields, 12/4 split, unchanged full/non-family state.
- `crates/ares-core/src/options/tests/project_config_views/gate.rs` — fixed
  long-retraction mode and distance-typo behavior.
- `crates/ares-core/src/options/tests/project_config_views/fixture.rs` — real
  3MF and composed rematerialization behavior.

Modify:

- `crates/ares-core/src/options.rs` — replace the old dynamic module declaration
  with the new crate-private module and register the test root.
- `crates/ares-core/src/options/tests.rs` — register the new focused test root.
- `scripts/dynamic_value_baseline.txt` — remove only fingerprints whose path is
  `crates/ares-core/src/options/filament_override.rs`.

Delete after the typed behavior is GREEN:

- `crates/ares-core/src/options/filament_override.rs`
- `crates/ares-core/src/options/filament_override/tests.rs`
- `crates/ares-core/src/options/filament_override/key_loop_tests.rs`
- `crates/ares-core/src/options/filament_override/print_diff_tests.rs`
- `crates/ares-core/src/options/filament_override/full_print_diff_tests.rs`

Post-implementation docs, modified only after whole-implementation approval:

- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`
- ignored `.superpowers/sdd/progress.md`
- ignored `.superpowers/sdd/task-19b1b-release-evidence.md`

No other production, test, fixture, CLI, WASM, project, or G-code file is in
scope. Every new/changed Rust file must remain below 400 physical lines. Do not
add to the 371-line
`crates/ares-core/src/options/tests/project_variants/support.rs`.

## Dispatch and review rules

- Execute slices in order. Slices 1-3 change shared production files and are
  not parallel-safe.
- Use a fresh bounded implementer Agent for each implementation slice (Slices
  1-3). Slice 4 is a verification/review slice and receives an implementer only
  if its reviewed evidence exposes a behavior defect. Give each implementer
  only the applicable plan slice, the approved spec, exact file ownership, and
  commands.
- After every slice, the primary agent inspects the diff and reruns the focused
  command. Then dispatch a different fresh read-only reviewer for literal
  `VERDICT: APPROVE | REVISE` against that slice.
- If a reviewer returns `REVISE`, use a fresh fix implementer, rerun the slice
  verification, and re-review. Do not carry an unresolved issue forward.
- Record a genuine RED run identifier and corresponding GREEN run identifier
  for every implemented behavior in the ignored evidence file. A compile/test
  RED must fail for the intended missing/wrong behavior, not for syntax,
  imports, environment, an unrelated test, or a missing test filter. Slice 2
  owns the real-fixture/rematerialization RED; Slice 4 re-verifies its GREEN
  result without pretending that verification itself is new behavior.
- Do not stage or commit any slice. The user requires one commit only after the
  frozen whole implementation, docs, and release gates approve.

---

### Slice 1: Owned views and typed vector semantics

**Files:**

- Create: `crates/ares-core/src/options/project_config_views.rs`
- Create: `crates/ares-core/src/options/project_config_views/retract.rs`
- Create: `crates/ares-core/src/options/tests/project_config_views.rs`
- Create: `crates/ares-core/src/options/tests/project_config_views/vector.rs`
- Create: `crates/ares-core/src/options/tests/project_config_views/support.rs`
- Modify: `crates/ares-core/src/options.rs`
- Modify: `crates/ares-core/src/options/tests.rs`

- [ ] **Step 1: Add the complete missing-API and vector-boundary RED**

Register `mod project_config_views;` in `options/tests.rs`. In the new test root,
register `mod support; mod vector;`. The first test calls the exact approved
API and observes both owned views:

```rust
use crate::options::{
    Nullable, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, ProjectSettings,
    project_config_views::resolve_project_config_views,
};
use super::support;

#[test]
fn project_config_views_apply_mixed_nullable_float_by_filament_map() {
    let mut full = ProjectSettings::default();
    // This test isolates one key. Empty nullable vectors are fixed no-ops, so
    // clear all sixteen defaults before installing the target vector.
    support::clear_retract_overrides(&mut full.filament.retract_overrides);
    full.project.gcode.filament_map =
        OrcaInts(vec![OrcaInt(2), OrcaInt(1), OrcaInt(2)]);
    full.project.gcode.deretraction_speed =
        OrcaFloats(vec![OrcaFloat(10.0), OrcaFloat(20.0)]);
    full.filament.retract_overrides.filament_deretraction_speed = vec![
        Nullable::Nil,
        Nullable::Value(OrcaFloat(99.0)),
        Nullable::Nil,
    ];
    let expected_full = full.clone();

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(views.full, expected_full);
    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        OrcaFloats(vec![OrcaFloat(20.0), OrcaFloat(99.0), OrcaFloat(20.0)])
    );
    assert_eq!(
        views.runtime_gcode.deretraction_speed,
        views.runtime.project.gcode.deretraction_speed
    );
}
```

Before creating either production file, add the remaining boundary tests
through `resolve_project_config_views`, with these exact assertions:

```rust
// Value entries do not consult map values; nil entries use one-based map.
// OrcaInt(0), OrcaInt(-1), and OrcaInt(99) all inherit machine[0].
// Empty machine + nonempty filament preserves an empty machine vector.
// Nonempty machine + empty filament preserves the machine vector/cardinality.
// Nonempty filament length 2 + map length 1 returns:
// "filament_deretraction_speed length must match filament_map".
```

In `support.rs`, implement `clear_retract_overrides` by explicitly clearing all
sixteen vectors on `FilamentRetractOverrideOptions`; do not loop over names or
serialize the group. Every one-key vector test must call it before installing
the target vector, so later slices cannot make an unrelated default-length
override fail against the test's custom map. Use distinct test names containing
`project_config_views_`; do not test a private helper and do not add a test-only
production API. At this point every vector test exists while the approved
façade/helper do not.

- [ ] **Step 2: Run the focused RED**

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_
```

Expected: nonzero exit because `options::project_config_views` and
`resolve_project_config_views` do not exist. Save the run ID and the exact
missing-item diagnostic. This single pre-production RED covers the mixed,
invalid-map, empty-vector, and length-mismatch tests already present in the
test module; do not implement the complete helper before this run.

- [ ] **Step 3: Add the façade and minimum typed helper**

In `options.rs`, remove `filament_override` from the compact
`option_modules!(...)` declaration only later in Slice 3; for now add:

```rust
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) mod project_config_views;
```

Implement the façade exactly as follows, with no public re-export:

```rust
mod retract;

use crate::SliceError;

use super::{GCodeOptions, ProjectSettings};

#[derive(Debug, PartialEq)]
pub(crate) struct ProjectConfigViews {
    pub(crate) full: ProjectSettings,
    pub(crate) runtime: ProjectSettings,
    pub(crate) runtime_gcode: GCodeOptions,
}

pub(crate) fn resolve_project_config_views(
    full: ProjectSettings,
) -> Result<ProjectConfigViews, SliceError> {
    let mut runtime = full.clone();
    retract::apply(&mut runtime, &full)?;
    let runtime_gcode = GCodeOptions::from_sources(
        &runtime.printer.gcode,
        &runtime.process.gcode,
        &runtime.filament.gcode,
        &runtime.project.gcode,
    );
    Ok(ProjectConfigViews {
        full,
        runtime,
        runtime_gcode,
    })
}
```

Implement the generic helper and only the first float application in
`retract.rs`. The invalid-index behavior is fixed source behavior, not a legacy
adapter:

```rust
use crate::{
    SliceError,
    options::{Nullable, OrcaInts, ProjectSettings},
};

fn apply_nullable<T: Clone>(
    machine: &mut Vec<T>,
    filament: &[Nullable<T>],
    filament_map: &OrcaInts,
    filament_key: &'static str,
) -> Result<(), SliceError> {
    if machine.is_empty() || filament.is_empty() {
        return Ok(());
    }
    if filament.len() != filament_map.0.len() {
        return Err(SliceError::InvalidInput(format!(
            "{filament_key} length must match filament_map"
        )));
    }

    let defaults = machine.clone();
    machine.resize(filament.len(), defaults[0].clone());
    for (index, value) in filament.iter().enumerate() {
        machine[index] = match value {
            Nullable::Value(value) => value.clone(),
            Nullable::Nil => filament_map.0[index]
                .0
                .checked_sub(1)
                .and_then(|index| usize::try_from(index).ok())
                .and_then(|index| defaults.get(index))
                .unwrap_or(&defaults[0])
                .clone(),
        };
    }
    Ok(())
}

pub(super) fn apply(
    runtime: &mut ProjectSettings,
    full: &ProjectSettings,
) -> Result<(), SliceError> {
    apply_nullable(
        &mut runtime.project.gcode.deretraction_speed.0,
        &full
            .filament
            .retract_overrides
            .filament_deretraction_speed,
        &full.project.gcode.filament_map,
        "filament_deretraction_speed",
    )
}
```

- [ ] **Step 4: Run the complete vector GREEN**

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_
```

Expected: every vector behavior that participated in Step 2's RED now passes.
Save the GREEN run ID and individual test names; a zero-match filter is a
failure.

- [ ] **Step 5: Run Slice 1 cumulative checks and inspect**

```powershell
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Expected: all new focused tests pass; formatting, Clippy, and diff checks pass.
Verify the full input is moved, only one full-to-runtime clone exists, no
`serde_json` appears in either production file, and every changed Rust file is
below 400 lines.

- [ ] **Step 6: Independent Slice 1 review**

Freeze the Slice 1 path/hash manifest. A fresh reviewer compares it to the
approved spec and returns:

```text
VERDICT: APPROVE | REVISE
ISSUES:
- [blocking issue or None]
REQUIRED_CHANGES:
- [change or None]
```

Do not start Slice 2 until the literal verdict is `VERDICT: APPROVE`.

---

### Slice 2: Complete sixteen-field overlay and 12/4 projection split

**Files:**

- Modify: `crates/ares-core/src/options/project_config_views/retract.rs`
- Create: `crates/ares-core/src/options/tests/project_config_views/fields.rs`
- Create: `crates/ares-core/src/options/tests/project_config_views/fixture.rs`
- Modify: `crates/ares-core/src/options/tests/project_config_views.rs`
- Modify: `crates/ares-core/src/options/tests/project_config_views/support.rs`

- [ ] **Step 1: Write the exhaustive field RED**

Create a typed full view with map `[2, 1, 2]`, two unique machine values per
ordinary field, and three nullable logical values per filament field. Use a
mix of `Nil` and `Value` for all supported types. Assert:

```rust
assert_eq!(views.full, expected_full);
assert_all_sixteen_runtime_fields(&views.runtime, &expected_runtime);
assert_all_twelve_runtime_gcode_fields(&views.runtime_gcode, &expected_runtime);
assert_eq!(
    views.runtime.printer.gcode.travel_slope,
    views.full.printer.gcode.travel_slope
);
```

The support assertions must explicitly name all sixteen fields. Do not loop
over registry keys, serialize either struct, or compare dynamic values. Add a
separate unchanged sentinel such as `machine_end_gcode` so the test proves
non-family preservation.

Explicitly set the exhaustive source gate before resolution so this normal
overlay test stays valid after Slice 3 adds the non-filament branches:

```rust
full.printer.gcode.enable_long_retraction_when_cut = OrcaInt(2);
```

In the same RED step, add `fixture.rs` and register `mod fixture;`. Load the
committed 3MF bytes only in tests, materialize from the original raw settings,
and assert the reviewed real-fixture full/runtime values. Also add the concrete
two-map rematerialization proof described below under Slice 4. These tests are
part of Slice 2's pre-implementation RED, not tests added after the production
behavior is already complete.

- [ ] **Step 2: Run the exhaustive RED**

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_all_sixteen_fields
cargo +1.91.0 nextest run -p ares-core project_config_views_fixture
cargo +1.91.0 nextest run -p ares-core project_config_views_rematerializes
```

Expected: nonzero exit because fifteen fields still retain their machine
values; the fixture and rematerialization filters must likewise fail on a
concrete missing overlay assertion. Save each run ID and at least the first
intended assertion difference. A filter with no matching test is not RED.

- [ ] **Step 3: Apply all sixteen fields explicitly**

Keep `apply_nullable` generic but add no runtime key loop. A literal-only macro
may remove wrapper boilerplate:

```rust
macro_rules! overlay {
    ($machine:expr, $filament:expr, $map:expr, $key:literal) => {
        apply_nullable(&mut $machine.0, $filament, $map, $key)?
    };
}
```

In `apply`, bind `map` and `overrides` from `full`, then explicitly overlay the
sixteen approved pairs in this exact fixed sorted-key order:

```rust
let map = &full.project.gcode.filament_map;
let overrides = &full.filament.retract_overrides;

overlay!(runtime.project.gcode.deretraction_speed,
    &overrides.filament_deretraction_speed, map, "filament_deretraction_speed");
overlay!(runtime.printer.gcode.long_retractions_when_cut,
    &overrides.filament_long_retractions_when_cut, map, "filament_long_retractions_when_cut");
overlay!(runtime.project.gcode.retract_before_wipe,
    &overrides.filament_retract_before_wipe, map, "filament_retract_before_wipe");
overlay!(runtime.project.gcode.retract_lift_above,
    &overrides.filament_retract_lift_above, map, "filament_retract_lift_above");
overlay!(runtime.project.gcode.retract_lift_below,
    &overrides.filament_retract_lift_below, map, "filament_retract_lift_below");
overlay!(runtime.printer.gcode.retract_lift_enforce,
    &overrides.filament_retract_lift_enforce, map, "filament_retract_lift_enforce");
overlay!(runtime.project.gcode.retract_restart_extra,
    &overrides.filament_retract_restart_extra, map, "filament_retract_restart_extra");
overlay!(runtime.project.print.retract_when_changing_layer,
    &overrides.filament_retract_when_changing_layer, map, "filament_retract_when_changing_layer");
overlay!(runtime.printer.gcode.retraction_distances_when_cut,
    &overrides.filament_retraction_distances_when_cut, map, "filament_retraction_distances_when_cut");
overlay!(runtime.project.gcode.retraction_length,
    &overrides.filament_retraction_length, map, "filament_retraction_length");
overlay!(runtime.project.print.retraction_minimum_travel,
    &overrides.filament_retraction_minimum_travel, map, "filament_retraction_minimum_travel");
overlay!(runtime.project.gcode.retraction_speed,
    &overrides.filament_retraction_speed, map, "filament_retraction_speed");
overlay!(runtime.project.print.wipe,
    &overrides.filament_wipe, map, "filament_wipe");
overlay!(runtime.project.print.wipe_distance,
    &overrides.filament_wipe_distance, map, "filament_wipe_distance");
overlay!(runtime.project.gcode.z_hop,
    &overrides.filament_z_hop, map, "filament_z_hop");
overlay!(runtime.printer.gcode.z_hop_types,
    &overrides.filament_z_hop_types, map, "filament_z_hop_types");
```

This step initially applies both special long-retraction fields normally with
gate value `2`; Slice 3 adds the fixed non-filament-mode branches.

- [ ] **Step 4: Freeze the 12/4 split without source pinning**

Add a typed test that asserts the twelve runtime GCode fields equal their
ordinary runtime owners and separately asserts the four print-only fields on
`runtime.project.print`. Use `GCodeOptions::FIELD_METADATA` only to prove the
existing typed inventory contains the twelve ordinary keys and omits the four;
do not read Orca source or build a production runtime key set.

- [ ] **Step 5: Run Slice 2 GREEN and adjacent projections**

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_config_views|gcode_options|project_variants)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Expected: focused and adjacent suites pass. Audit exactly sixteen literal
`overlay!` calls, exactly twelve equality assertions against `runtime_gcode`,
four print-only assertions, and no `travel_slope` overlay.

- [ ] **Step 6: Independent Slice 2 review**

Freeze the cumulative manifest. Require literal `VERDICT: APPROVE` before
Slice 3.

---

### Slice 3: Fixed long-retraction modes and obsolete scaffold deletion

**Files:**

- Modify: `crates/ares-core/src/options/project_config_views/retract.rs`
- Create: `crates/ares-core/src/options/tests/project_config_views/gate.rs`
- Modify: `crates/ares-core/src/options/tests/project_config_views.rs`
- Modify: `crates/ares-core/src/options.rs`
- Modify: `scripts/dynamic_value_baseline.txt`
- Delete: `crates/ares-core/src/options/filament_override.rs`
- Delete: all four files under
  `crates/ares-core/src/options/filament_override/`

- [ ] **Step 1: Write the fixed-typo RED**

Use machine bool `[false, true]`, machine distance `[18, 17]`, map `[2, 1, 2]`,
concrete filament bool/distance overrides, and gate modes `0`, `1`, and `2`.
Start from `clear_retract_overrides`, then install only the bool, distance, and
unrelated `retraction_length` vectors at logical length three. Assert:

```rust
// gate 2: normal nullable overrides apply to both fields.
// gate 0/1 bool: ignore concrete filament bools, remap machine bools through
// an equally-sized all-Nil logical vector, producing [true, false, true].
// gate 0/1 distance: preserve [18, 17] and physical cardinality exactly,
// freezing the fixed empty-float-temporary typo.
```

Also assert an unrelated field such as `retraction_length` still applies its
normal nullable override under every gate mode.

- [ ] **Step 2: Run the gate RED**

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_long_retraction_gate
```

Expected: gate `0`/`1` fail because Slice 2 applies both concrete filament
values normally.

- [ ] **Step 3: Implement the two exact gate branches**

In `apply`, compare the typed scalar directly. Keep the bool branch at the
second sorted-key position and the distance branch at its ninth sorted-key
position; do not group the two writes together:

```rust
const ENABLE_FILAMENT_LONG_RETRACTION: i32 = 2;

let filament_long_enabled = full
    .printer
    .gcode
    .enable_long_retraction_when_cut
    .0
    == ENABLE_FILAMENT_LONG_RETRACTION;

// Second sorted key: long_retractions_when_cut.
if filament_long_enabled {
    overlay!(runtime.printer.gcode.long_retractions_when_cut,
        &overrides.filament_long_retractions_when_cut, map,
        "filament_long_retractions_when_cut");
} else {
    let nil_long = vec![Nullable::Nil;
        overrides.filament_long_retractions_when_cut.len()];
    overlay!(runtime.printer.gcode.long_retractions_when_cut,
        &nil_long, map, "filament_long_retractions_when_cut");
}

// Ninth sorted key: retraction_distances_when_cut.
if filament_long_enabled {
    overlay!(runtime.printer.gcode.retraction_distances_when_cut,
        &overrides.filament_retraction_distances_when_cut, map,
        "filament_retraction_distances_when_cut");
}
// Otherwise the fixed source passes an empty float nullable vector, so the
// distance field intentionally remains unchanged.
```

Remove the two unconditional special-field calls from the common list. Do not
generalize this into an enum or correct the fixed distance behavior.

- [ ] **Step 4: Delete the dead dynamic scaffold**

Use `apply_patch` to remove the five old files and remove `filament_override`
from the `option_modules!(...)` declaration. Remove from
`scripts/dynamic_value_baseline.txt` every and only line matching:

```text
crates/ares-core/src/options/filament_override.rs#
```

Do not remove or edit any other dynamic fingerprint. Verify no reference
remains:

```powershell
rg -n "mod filament_override|filament_override::|collect_filament_override_updates|compute_filament_override_value" crates/ares-core/src scripts/dynamic_value_baseline.txt
Test-Path crates/ares-core/src/options/filament_override.rs
Test-Path crates/ares-core/src/options/filament_override
```

Expected: `rg` has no result and both `Test-Path` calls print `False`. The
unrelated `painted_region_config_copies_parent_before_filament_overrides` test
name may remain because it is neither an import nor a symbol from the deleted
module. The new typed module name may contain `retract`, but never the old
dynamic symbol names.

- [ ] **Step 5: Run gate, cumulative, and ratchet GREEN**

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Expected: all pass. Confirm the dynamic audit reports no new fingerprint and
that the working baseline contains zero old-path rows.

- [ ] **Step 6: Independent Slice 3 review**

The reviewer must explicitly validate the fixed typo, the typed gate source,
and exact old-scaffold deletion before returning literal
`VERDICT: APPROVE`.

---

### Slice 4: Fixture/rematerialization and unchanged cross-surface boundary verification

Slice 4 adds no late test behavior. Its fixture and rematerialization tests
were created before Slice 2 production work, produced genuine assertion REDs,
and became GREEN when the sixteen overlays landed. This slice independently
verifies that evidence and the still-incomplete public slicing boundary.

**Files:**

- Verify: `crates/ares-core/src/options/tests/project_config_views/fixture.rs`
- Modify only if a reviewed behavior bug is found:
  `crates/ares-core/src/options/project_config_views.rs`
  or `crates/ares-core/src/options/project_config_views/retract.rs`

- [ ] **Step 1: Audit the committed-fixture assertions created in Slice 2**

The test loads the real bytes only in tests and resolves the full view from the
original raw settings:

```rust
let project = crate::load_project(include_bytes!(
    "../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"
))
.unwrap();
let raw = project.settings().clone();
let full = super::super::super::project_variants::materialize_project_variants(
    &raw,
    &raw.project.gcode.filament_map,
)
.unwrap();
let views = super::super::super::project_config_views::resolve_project_config_views(full)
    .unwrap();
```

Require the exact typed full/runtime differences from the reviewed spec:

```text
full:    deretraction_speed [30,20]
runtime: deretraction_speed [30,30]
full:    retraction_distances_when_cut [18,18]
runtime: retraction_distances_when_cut [10,10]
full:    retraction_length [0.8,2]
runtime: retraction_length [0.4,0.4]
full:    retraction_speed [30,20]
runtime: retraction_speed [30,30]
full:    wipe_distance [2,2]
runtime: wipe_distance [1,1]
full:    z_hop_types [Auto,Auto]
runtime: z_hop_types [Spiral,Spiral]
```

The test also names all twelve `runtime_gcode` fields and compares each with
its runtime owner, while preserving the original raw settings exactly.

- [ ] **Step 2: Audit concrete raw-source rematerialization values**

Use the same committed raw fixture with map A `[1, 1]` and map B `[2, 1]`.
The test-only `resolve_from_raw` must call
`materialize_project_variants(&raw, &map)` on every invocation before passing
the owned result to `resolve_project_config_views`; there is no production
combined or cache API.

Assert raw immutability, same-map whole-view determinism, and these concrete
map-sensitive `retraction_length` values for both results:

```text
map A full machine:       [0.8, 2.0]
map A nullable source:    [Value(0.4), Value(0.4)]
map A runtime:            [0.4, 0.4]
map A runtime_gcode:      [0.4, 0.4]

map B full machine:       [0.8, 2.0]
map B nullable source:    [Value(3.0), Value(0.4)]
map B runtime:            [3.0, 0.4]
map B runtime_gcode:      [3.0, 0.4]
```

Do not use a whole-result `assert_ne!` as the remap proof: `filament_map`
itself differs and would make that assertion pass even if no map-sensitive
field were rematerialized.

- [ ] **Step 3: Re-run the fixture and rematerialization GREEN evidence**

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_fixture
cargo +1.91.0 nextest run -p ares-core project_config_views_rematerializes
cargo +1.91.0 nextest run -p ares-core project_config_views_
```

Expected: all pass. If a reviewed behavior defect is exposed, dispatch a fresh
bounded fix implementer, change only the two approved production files, rerun
all prior focused tests, and re-review.

- [ ] **Step 4: Verify the exact existing cross-surface expectations**

These surfaces do not share one assertion:

- Core `fixture_project_slicing_boundary_remains_incomplete` and
  `project_import_slice_project_loads_before_typed_incomplete_error` expect the
  typed `SliceError::ProjectSlicingIncomplete` after valid project loading.
- CLI `slice_3mf_rejects_the_legacy_explicit_options_pipeline` expects the
  legacy `--options` route to fail with
  `3MF project input must be loaded with load_project`; it does not assert the
  typed incomplete error.
- WASM `project_incomplete_error_has_stable_javascript_mapping` only verifies
  the stable string mapping for an already-constructed incomplete error; it
  does not slice the fixture.
- Browser `project-slice.spec.mjs` loads the generated WASM binding, sends the
  real committed 3MF through `sliceProject`, and expects
  `{ resolved: false, error: "ProjectSlicingIncomplete" }`.
- The CLI `project_matches_orca_242_except_generator_line` golden remains
  explicitly ignored as `full project parity incomplete`.

Run each current expectation directly:

```powershell
cargo +1.91.0 nextest run -p ares-core fixture_project_slicing_boundary_remains_incomplete
cargo +1.91.0 nextest run -p ares-core project_import_slice_project_loads_before_typed_incomplete_error
cargo +1.91.0 nextest run -p ares-cli --test cli slice_3mf_rejects_the_legacy_explicit_options_pipeline
cargo +1.91.0 nextest run -p ares-wasm project_incomplete_error_has_stable_javascript_mapping
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser test -- project-slice.spec.mjs
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Expected: all selected tests pass, the focused browser test reaches the real
fixture through the freshly generated binding and returns the typed incomplete
error, and the ignored full-golden test remains ignored. This task must not
make a false G-code-parity claim.

- [ ] **Step 5: Independent Slice 4 review**

Freeze the cumulative implementation paths and require literal
`VERDICT: APPROVE` for the fixture expectations, concrete raw-source-only remap
proof, no hardcoding, and exact unchanged public incomplete boundaries.

---

## Whole implementation approval gate

- [ ] Freeze an implementation manifest containing every production, test,
  deletion marker, and baseline path, but excluding the spec, plan, and later
  docs-only paths. Compute deterministic per-path hashes; represent deleted
  files explicitly as `DELETED`.
- [ ] Run and record fresh pre-review evidence:

```powershell
cargo +1.91.0 nextest run -p ares-core project_config_views_
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_config_views|project_variants|gcode_options|project_fixture|project_import)/)'
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

- [ ] Dispatch a fresh independent spec-compliance reviewer against the frozen
  spec, reviewed plan, manifest/diff, and verification. Require exactly:

```text
VERDICT: APPROVE | REVISE
SPEC_COVERAGE:
- [implemented requirement or missing requirement]
BLOCKERS:
- [blocking gap or None]
REQUIRED_CHANGES:
- [change or None]
ROLE: SPEC COMPLIANCE
```

- [ ] Dispatch a different fresh code-quality reviewer against the same frozen
  bytes. It must check correctness, source-faithful semantics, ownership,
  performance, Rust idioms, LOC, obsolete removal, and tests. Require literal
  `VERDICT: APPROVE` and `ROLE: CODE QUALITY`.
- [ ] Run the same bounded whole-implementation review through OpenCode's
  default model. Require literal `VERDICT: APPROVE`.
- [ ] If any reviewer returns `REVISE`, unfreeze, fix with a fresh bounded
  implementer, rerun focused verification, freeze a new manifest, and rerun all
  three reviewers. Do not update architecture/roadmap docs until all three
  approvals apply to identical implementation bytes.

## Post-approval documentation gate

- [ ] After whole implementation approval, update only:

  - `docs/architecture/option-parity-v4.md` with the fixed full/runtime split,
    16-key typed overlay, 12/4 ownership, exact long-distance typo behavior,
    map/cardinality policy, dynamic scaffold deletion, platform boundary, and
    explicit 19B.2/19B.3/19C/20E deferrals;
  - `docs/roadmap.md` with Task 19B.1B completion evidence and Task 19B.2/19B.3
    remaining;
  - ignored SDD progress/evidence ledgers with approvals and run IDs.

- [ ] Freeze the two tracked docs paths separately. Dispatch a fresh read-only
  documentation reviewer to validate every claim against the approved
  implementation and fixed Orca citations. Require literal
  `VERDICT: APPROVE` with `ROLE: DOCUMENTATION`. Any tracked docs edit
  invalidates that docs approval only.

## Fresh release matrix

After all implementation and docs approvals, rerun every command from the
frozen working tree:

```powershell
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 nextest run -p ares-core project_config_views_
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_config_views|project_variants|gcode_options|project_fixture|project_import)/)'
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser test
git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
```

Mandatory audits:

```powershell
# Fixture hashes must remain exact.
Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf -Algorithm SHA256
# 698F40F13C9075B818ABEDD3D10F022FBB5D8200AED48FBDDE651F6BFB21B8A9
Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode -Algorithm SHA256
# 10AEC9A156849F59929B578429A764A61453996A5834056F600C0ADBB5D6A1B3

# No old scaffold, runtime JSON, fixture/source pinning, or hardcoding.
rg -n "mod filament_override|filament_override::|collect_filament_override_updates|compute_filament_override_value" crates/ares-core/src scripts/dynamic_value_baseline.txt
Test-Path crates/ares-core/src/options/filament_override.rs
Test-Path crates/ares-core/src/options/filament_override
rg -n "serde_json::Value|serde_json::Map|RawValue|BTreeMap<String" crates/ares-core/src/options/project_config_views.rs crates/ares-core/src/options/project_config_views
rg -n 'OrcaSlicer|8500fcdc|ksr_fdmtest_v4|include_(str|bytes)!|"[^"]*\.gcode' crates/ares-core/src/options/project_config_views.rs crates/ares-core/src/options/project_config_views

# Every intended, tracked-changed, and untracked Rust file remains below 400
# physical lines even though staging is intentionally deferred.
$rustPaths = @(
    'crates/ares-core/src/options/project_config_views.rs',
    'crates/ares-core/src/options/project_config_views/retract.rs',
    'crates/ares-core/src/options/tests/project_config_views.rs',
    'crates/ares-core/src/options/tests/project_config_views/support.rs',
    'crates/ares-core/src/options/tests/project_config_views/vector.rs',
    'crates/ares-core/src/options/tests/project_config_views/fields.rs',
    'crates/ares-core/src/options/tests/project_config_views/gate.rs',
    'crates/ares-core/src/options/tests/project_config_views/fixture.rs',
    'crates/ares-core/src/options.rs',
    'crates/ares-core/src/options/tests.rs'
)
$rustPaths += git diff --name-only --diff-filter=ACMR -- '*.rs'
$rustPaths += git ls-files --others --exclude-standard -- '*.rs'
$rustPaths | Sort-Object -Unique | Where-Object { Test-Path $_ } | ForEach-Object {
    $lines = (Get-Content $_).Count
    if ($lines -ge 400) { throw "$_ has $lines lines" }
}
```

The first and third `rg` commands are expected to return no production result,
and both old-path `Test-Path` calls must print `False`; fixture paths are allowed
only in the approved fixture test file. Source inspection commands against
fixed Orca remain release notes, never committed tests.

## Commit, push, and exact-SHA Tier 1

- [ ] Recompute the final manifest including spec, plan, approved
  implementation, baseline, and approved tracked docs. Confirm `git status`
  contains only those intended paths and that index/worktree bytes will match
  the frozen manifest.
- [ ] Read the Conventional Commits skill, stage only the frozen manifest, and
  commit once:

```powershell
git commit -m "feat(config): resolve runtime retract views"
```

- [ ] Push the current branch:

```powershell
git push origin codex/ksr-fdmtest-v4-parity
```

- [ ] Verify local/remote equality and clean state:

```powershell
git rev-parse HEAD
git rev-parse origin/codex/ksr-fdmtest-v4-parity
git status --short
```

- [ ] Wait for the exact pushed SHA's Tier 1 workflow and require all five jobs
  green: `format`, `ubuntu-latest`, `wasm`, `macos-latest`, and
  `windows-latest`. Do not start Task 19B.2/19B.3 or declare Task 19B.1B
  released while that exact-SHA gate is pending.

Task 19B.1B completion leaves Task 19B, Task 19C, consumer migration, geometry,
slicing, G-code, metadata, post-processing, and complete normalized
`ksr_fdmtest_v4` byte parity open.
