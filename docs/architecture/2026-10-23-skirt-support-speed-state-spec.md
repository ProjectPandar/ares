# Skirt support-speed fallback: current object configuration

## Upstream rewrite boundary

Port OrcaSlicer `libslic3r/GCode.cpp:4434-4441::generate_skirt`'s explicit
`extrude_loop(..., m_config.support_speed.value)` argument, together with the
object-configuration applies that determine that value. Upstream revision:
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `2533-2534`: initialize default object and default region configuration.
- `4559-4594`: select the first available object layer in source order (or
  support layer when no object layer exists) and apply its OBJECT config.
- `5334-5335`: per-object skirt/brim emission precedes the upcoming instance's
  default-region and object applies at `5341-5342`; skirt therefore reads the
  previously active object config, not necessarily its owner.
- `4434-4441`: pass current support_speed as the explicit skirt loop speed.
- `6557-6597`: preserve first-layer/over-raft speed replacement and existing
  slow-down-layer behavior; `6598-6603` overrides with skirt_speed only if >0.
- `6136/6163`: REGION applies do not change the object-scoped support_speed.

GUI citations: `PrintConfig.hpp:974` declares object support_speed;
`slic3r/GUI/Tab.cpp:2526` exposes Support speed. GUI skirt_speed=0 selects the
upstream fallback instead of the positive explicit skirt speed override.

## Rust destination and replacement

Stack on ratio GREEN `93d9057636233b245795fec3ea926bb3d984c52b`,
which includes seam state43e10f4c8. Minimum-skirt017a9876b remains a
separate slice.

Extend the existing `gcode_emit/motion/state.rs::EmitState` current-object
state with only current support_speed. Initialize it beside existing object
ratio initialization and update it in the existing
`motion/state/config.rs::apply_object_config` method at its three source-owned
layer/instance apply sites. Preserve those sites' object selection/order and
existing seam-gap/brim-ratio state. No parallel config mechanism or generic
m_config framework.

Current compatibility shell: `motion/options/build/context.rs` selects the
first resolved object, `fields/assemble.rs:64-66` installs that object's
support_speed into MotionOptions, and `motion/features.rs` uses it as Skirt's
fallback. That eager speed must not substitute for current m_config at skirt
emission. Have the existing skirt emission boundary read the live object speed
for skirt only; retain all existing non-skirt consumers and support extrusion
paths unchanged. Audit exact speed-handoff and downstream caps before GREEN;
report necessary direct wiring before editing. Do not temporarily overwrite
shared MotionOptions if that would leak into support or wall paths.

## Included and deferred behavior

Included: default object initialization, current-object apply updates, live
support_speed as skirt's explicit base speed, and the existing positive
skirt_speed override/first-layer/slow-down/caps pipeline consuming that input.
Retain source arithmetic and existing cooling/time response rather than
altering downstream algorithms to force equality.

Deferred: support generation or support extrusion speeds, brim speed changes,
wall/infill speeds, seam-gap/brim-ratio semantics, min-skirt budgets, geometry,
travel, fan/cooling algorithms, processor/time/M73, scarf, validation and GUI
changes. Any residual needing those changes requires STOP. No fallback/flag,
legacy eager-vs-live branch, or Ares-owned pipeline policy.

`motion/state.rs` is already400LOC on the parent. Keep all source files<=400:
place methods in the existing config submodule; any necessary focused state
file split must be reported before production edits and limited to this change.

## Frozen GUI control/target pair

Seed `tests/parity/ordered-brim-owner/far-uncombined.3mf`, SHA256
`c9f9c760caedd4fc2118e491b0a3df667346c6b4d2435ef4f752c8c87c4929a6`.
Common GUI change skirt_speed=0; add object3 support_speed override40 for
control and10 for target. All geometry, transforms, placements and other
settings remain unchanged; only that one object XML value differs between
control and target. Supports disabled, per-object skirt2loops/2layers,
min_skirt_length0, uniform seam_gap10%, brim_flow_ratio1.

| case | object3 support_speed | SHA256 |
| --- | --- | --- |
| control | 40 | `931449cc699bac934bfe399a59e9447a91d2ce28a58c8cf674113035c7e812ef` |
| target | 10 | `8db2eaacfd6ad62ad1c0937f5fa2c37a7f2f88f08b8160545e07db5d9a1bf33d` |

Discovery on33afd7d57 found complete generator-only control equality in both
modes; target strict first difference is M73R4 versus R3. This is explicitly
accepted as M73-leading with independent feed evidence, not masked. Second
layer next-instance skirt NativeF600 versus AresF1842, all skirt XY/E sequences
identical. Native double replay/modes agree. These are external diagnostics,
not test constants.

The initial positive-skirt-speed brim hypothesis was rejected because both
cases matched; do not revive it or claim a first-layer brim defect. Frozen
seam/ratio/min-length slices do not address this speed field.

## Staged gates

1. Docs-only SPEC commit before tests/production.
2. Add exactly two input-only3MFs and two public cases through
   unchanged `runner::native_stream_case::check` with `(false, |_, _| {})`.
   Register in runner.rs (parent395LOC), not root-registry migration/include.
   Preserve all earlier tests/inputs/helpers. No hard-coded feed assertions,
   geometry callbacks, copied runner, guard, allow/expect or expected-error test.
3. Exact RED tip; serial official/seeded, independent fresh empty caches. Each mode must show
   one genuine strict target failure and one complete control pass, Native
   double replay, successful public API/actual CLI and raw API/CLI equality.
   Compare complete ordered bytes with generator-line-only handling, no M73,
   timing, metadata, travel or geometry mask. Attach independent feed proof.
   Run fmt and strict workspace all-targets clippy.
4. Carrier audit then minimal GREEN with tests/input/helper frozen;
   STOP for any broader state model or other residual.
5. One fresh dual FULL at the exact final tip.
