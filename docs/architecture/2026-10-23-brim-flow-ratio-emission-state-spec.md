# Brim flow ratio: current object configuration at emission

## Source-owned boundary

This slice ports OrcaSlicer `libslic3r/GCode.cpp::_extrude`'s erBrim flow
multiplier and the object-configuration apply sequence that supplies it
(upstream revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`). It is not a new
brim flow policy or a correction to upstream behavior.

- `GCode.cpp:6470-6481`: form flow from `path.mm3_per_mm`, print/filament flow
  factors and role-specific multipliers; `6479-6480` multiplies erBrim by the
  **current `m_config.brim_flow_ratio`** at extrusion time.
- `2533-2534`: initialize default object and default region configuration.
  Brim flow ratio is an object-config field; initialize its current value from
  default object configuration, not from an arbitrary brim owner.
- `4593-4594::process_layer`: apply the first layer object's OBJECT config
  before combined/per-object skirt or brim emission. This updates brim ratio,
  unlike region-only seam_gap, which persists across this object apply.
- `5334-5335`: emit per-object skirt then object brim BEFORE the upcoming
  instance's configuration applies. Brim may therefore read a different
  object's ratio: the layer-selected object initially, or the previous
  instance's last applied object configuration afterward.
- `5341-5342`: reset default REGION config (does not reset object brim ratio),
  then apply the current printing instance's OBJECT config (updates ratio).
- `6136/6163`: actual REGION config applies do not override this object field.
  Preserve the seam-gap state updates already ported in the parent chain.

GUI citations: `PrintConfig.cpp:1723-1734` defines advanced `brim_flow_ratio`,
range0..2, default1; `slic3r/GUI/Tab.cpp:2717` exposes Brim flow ratio. Native
behavior must be matched even if a user might expect owner-local multiplication.

## Rust destination and existing scaffold replacement

Base this slice on the seam-state GREEN
`43e10f4c8ce0ba08c209c9cdf313a2459a43e99d`. Extend the existing current-config
state in `project_slice/gcode_emit/motion/state.rs::EmitState` with ONLY the
current object `brim_flow_ratio`; do not introduce a parallel configuration
state machine or general m_config framework. Retain the existing seam-gap
field and its region-apply semantics unchanged.

Current compatibility shell: `gcode_emit/brim.rs:360` pre-multiplies
`flow.mm3_per_mm * options.brim_flow_ratio.0` when preparing the owner's
`BrimPlan`; `brim/emit.rs` passes this value to `motion/skirt_brim.rs`, where
it becomes extrusion flow. This owner-based pre-multiply must be removed.
Keep geometric/base flow metadata unmultiplied, then apply the current object
ratio at the existing erBrim emission boundary, source-equivalent to `_extrude`.
Audit the existing first-layer-object selection and direct instance/raft
configuration-application carriers before GREEN; report required wiring before
editing. Do not substitute extrusion-nearest owner order for upstream layer
selection, or current brim owner options for the pre-apply state.

Potential direct destinations: existing initialization in `orchestration.rs`,
layer-object apply boundary in `layers/boundary.rs`/its caller, per-instance
object apply in `layers/tool_entry.rs`, base `brim.rs` flow storage and existing
`motion/skirt_brim.rs` emission. Choose only the necessary existing call sites
following the carrier audit; this list does not cover unrelated refactoring.
Keep files<=400LOC. If a broader state model is needed, STOP.

## Included and deferred behavior

Included: initialization, layer-first object apply, per-instance object apply,
region non-interference and erBrim emission-time ratio multiplication, extending
only the existing EmitState current-config mechanism. Preserve existing
print/filament flow factors, first-layer exclusions, path geometry, integer/f32
flow conversions and Native multiplication order after inspecting the emitter.

Deferred: seam-gap behavior changes, general object/region config engines,
wall/infill/skirt flow policy, min_skirt_length budgets, print-global brim width/
nozzle fallback, auto/painted brim geometry, contact grouping, ordering algorithm
changes, support/tower generation, travel, scarf, cooling/time/M73 and new GUI or
validation. No legacy owner-ratio fallback or dual mechanism. A strict residual
outside this source boundary requires STOP, not assertion weakening or scope
expansion. No changes to other workers' painted/infill/travel/fan/ironing slices.

## GUI pair and source-owned observable proof

Seed: `tests/parity/ordered-brim-owner/far-uncombined.3mf`, SHA256
`c9f9c760caedd4fc2118e491b0a3df667346c6b4d2435ef4f752c8c87c4929a6`.
Retain meshes, transforms, placements and all settings. Add source object2's
GUI brim_flow_ratio override: control1, target0.8. Target/control zipped archive
entries differ only at this one XML value.

| case | object2 brim_flow_ratio | SHA256 |
| --- | --- | --- |
| control | 1 | `534ea70d9a62b1e4e6d6787066941023a2fce314436d39306d12aeccaf16e913` |
| target | 0.8 | `726408c79aa4127f4bd3796c1b4277ca36c153f4e489722d5b45de89485f4d4f` |

Object2 is atX80Y110, object3 atX140Y110; existing print order emits object3's
brim first. Native target's first brim uses0.8 and second uses1, while Ares
owner-pre-multiply uses1 then0.8. Discovery on main33afd7d57: control complete
Native/CLI generator-only equality in both modes; target first strict difference
is actual erBrim filament E, Native `.01583` versus Ares `.01979`, sameXY/F.
Each brim has471 positiveXYmoves; only flow differs. Native double replay and
both Native modes agree. These are external diagnostics, not test constants.

This gap is not fixed by the parent seam-gap-only state or frozen skirt
minimum-length chain017a9876b. Inputs use uniform seam_gap10% and
min_skirt_length0; those source exclusions are recorded in discovery evidence.

## Staged contract

1. Commit this docs-only SPEC. No test/production changes in that commit.
2. Add exactly two input-only 3MFs and two public cases using
   unchanged `runner::native_stream_case::check` with `(false, |_, _| {})`.
   Register through existing `runner.rs` (currently393LOC), not root registry
   migration/include. Freeze original seam tests/inputs and all new SHA values.
3. Exact RED tip: serial official/seeded with independent fresh empty caches. Require each
   mode one strict genuine target failure and one complete control pass, Native
   double replay, successful public API/actual CLI, raw API/CLI equality. Retain
   artifacts; compare complete ordered bytes with only generator-line handling.
   No M73/time/travel/geometry/metadata masks, extra geometry callbacks or magic
   result constants. Run fmt and strict all-targets clippy.
4. After RED, implement minimal GREEN with tests/inputs/helper frozen.
   Report any residual immediately; never duplicate control runs.
