# Klipper TimeMachine estimator rewrite — ARD and RED plan

Status: proposed; specification and public RED only. No production port yet.
Upstream: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.
Rust baseline: `44fb3e6912efec80b2a12e01df6a44045daf3651`.
This is a source-cited `libslic3r` rewrite slice, not a new Ares planner design.

## Domain and observed boundary

A **TimeBlock** is the processor's attributed motion, not a slicer extrusion
path. A **lazy flush** commits only blocks whose cruise speed is independent of
future moves. A **minimum cruise ratio** constrains look-ahead cruising across
sequences; it is not a per-block acceleration multiplier. **First-layer time**,
**prepare time**, a vertex's **normal time**, and total estimated time are
separate upstream measurements, not interchangeable regression assertions.

The existing STL Plater session case uses the committed `cube10.stl`, the raw
LH Stinger 0.4 nozzle catalog and its compatible process/filament. Its complete
Native/Ares comparison reaches a timing difference. Independent diagnostic
observations report Native first-layer0.776319s versus Rust0.741911s and
firstG1 time0.015492s versus0.010625s, with Klipper and default cruise ratio0.5.
Those are diagnosis, not committed expected values or proof of total parity.
A repeatable RED must generate Native expectations at runtime from real inputs.

The current `processor/schedule.rs:38` unconditionally instantiates
`motion::RollingPlanner`; `motion/planner/mod.rs:109` runs the Marlin reverse/
forward trapezoid planner, and `prepare` always uses classic axis/jerk rules.
Changing just the final trapezoid integration would leave Marlin block rules.
Klipper block preparation/limits are part of this source slice, not an optional
optimization after copying `plan_klipper`. Diagnostic raw Rust MotionBlock and
post-cap Native TimeBlock snapshots are different representation stages; their
raw acceleration values alone do not establish a mismatched final acceleration.

## Upstream owner and exact Rust destination

All cpp citations below refer to `src/libslic3r/GCode/GCodeProcessor.cpp`.

| Upstream boundary | Included behavior | Rust destination |
|---|---|---|
| `TimeMachine::reset:383-387`; `GCodeProcessor.hpp:815-819` | Klipper flag, ratio0.5, requested legacy accel-to-decel sentinel, priming/window reset | existing processor motion/planner state; no external runtime |
| `process_G1:5185-5203,5402-5537` | Klipper precise decimal XYZE/F admission, then cruise/axis-limit selection; bypass Marlin centripetal and XY per-axis caps; limit E only for retract/no-XY; E-only acceleration with retract fallback; live toolhead acceleration; per-block junction and MCR budget | `processor/motion.rs`, `motion/segment.rs`, `motion/planner/` preparation, existing machine envelope |
| `get_junction_deviation:7787-7807`; Klipper branch of `calc_vmax_junction_deviation:7824-7857` | SCV/live-accel deviation, from-rest junction, XYZ double axis-velocity directions, extrusion-ratio corner limit and short-segment corner bounds | separate Klipper preparation/junction module under `motion/planner/`; retain Marlin module unchanged |
| `process_G1:5647-5671` | preceding acceleration/MCR entry bounds, time-window trigger instead of Marlin256/64 threshold | existing motion block data and planner enqueue boundary |
| `TimeMachine::plan_klipper:505-573` | backward v-squared reachability/MCR pass; lazy flush frontier; unresolved cruise chain and forward resolution; upstream f32 stores/order | new `motion/planner/klipper.rs` (split further if400LOC), no duplicate parser |
| `TimeMachine::calculate_time:576-637,780-791` | single-block Klipper eligibility, dispatch, full/lazy flush, double integration from planned speeds, no Marlin first-retained max-entry patch for Klipper | `processor/schedule.rs` and flavor-specific planner implementation; keep shared delay/attribution output |
| `process_SET_VELOCITY_LIMIT:6870-6916` | numeric parsing at input boundary, live ACCEL/VELOCITY/SCV, ratio and legacy accel-to-decel precedence, priming adjustment | existing `motion/config_commands.rs` and typed per-block command state |
| processor dispatch `:8052-8061`, synchronization `:8136-8146` | flavor selects Klipper; synchronization resets priming window to modern1s/legacy2s | existing `processor/estimate.rs`/schedule flush events; no Ares-only timing switch |

Existing `RollingPlanner` is an incomplete compatibility shell around the
upstream Marlin planner. Keep it for non-Klipper flavors; remove its ownership
of Klipper scheduling. The new private planner is selected from the existing
`GCodeFlavor`, not a feature flag or fallback. Reuse the existing `Trapezoid`,
`Schedule`, move IDs, time-cache, stop/prepare/layer accounting and actual-speed
vertex consumers. Do not grow a second G-code parser or scheduling API.

## VG1 dispatch amendment

`GCodeProcessor.cpp::process_VG1:5845-6049` also queues blocks consumed by
flavor-selected `calculate_time`. Preserve VG1's own centripetal/all-axis
preparation, zero-default junction deviation and MCR budget, and lack of an
automatic window flush. It is a prepared-block variant inside the Klipper
planner, not a Marlin-planner fallback. Share the literal centripetal operation
with non-Klipper G1 only where those source calculations coincide. VG1 is a
source-literal, unwitnessed part of this port; no VG1 parity claim is made.

## Non-negotiable architectural decisions

1. Port from the pinned source, including source operand precision/order and
   live command state. Do not import current external Klipper algorithms,
   approximate ratio per segment, tune constants to the witness, or choose
   Marlin as a fallback for unhandled Klipper input.
2. Planner state stays in platform-neutral `ares-core`; no filesystem,
   process, terminal, OpenGL, OS affinity or Native dependency enters core.
   Browser WASM, Windows, macOS and Linux share the same pure math/state code.
3. Lazy flushing follows source queue ownership and dependency frontier, not
   an independently chosen fixed block count. A lazy return may retain
   squared-speed scratch state; do not interpret it as finalized speeds or
   double-commit cached delays. Synchronization and finalization are distinct.
4. Compute Klipper motion time using the upstream double formula from the
   planned f32 speeds, not reconstructed rounded deceleration distance.
   Preserve existing shared attribution of pending delays and move times.
5. Non-Klipper behavior is not a legacy fallback: it remains the appropriate
   source flavor branch and must not change as a side effect of this slice.
6. Keep existing strict public comparisons, missing-Native policy, input
   bytes/settings and test names. No ROOT registry edits, private test revival,
   Native rebuild, invented Native golden, M73 suppression, bit snapping or
   tolerance broadening. No claim all strict residuals will disappear.

## Public RED seam and failure inventory (before test code)

Two complementary public boundaries, using the real committed cube/profile
fixture and the already-provisioned pinned Native binaries:

- Existing `PlaterSession::load_with_profiles` then
  `slice_plater_with_profiles`, compared with actual Native preset/STL slicing
  by unchanged `plater_stl_session_centers_moves_and_slices_like_native`.
  Native is generated at runtime; original complete ordered comparison stays.
- `ares_core::gcode_processor::process_gcode` on that same runtime Native
  G-code/settings, compared to the existing Native `--process-gcode
  --dump-moves` adapter on identical bytes. This isolates the processor from
  later slicer geometry differences without weakening the Plater comparison.
  Assert actual normal motion times from the Native CSV, never a private
  planner/block call or cached temporary diagnostic JSON. Save both outputs,
  input/settings identity, full mismatch report and source/binary identities
  outside Git. Original `M73` records need not be removed for this read-only
  public processor seam (`post_process=false`).

Failure inventory:

- Default ratio omitted; plan formula copied but flavor dispatch absent.
- First E-only block gets Marlin acceleration/safe-entry instead of Klipper.
- Wrong Z/E limit or positive-XY-extrusion E-limit selection.
- SCV/live acceleration/junction direction quantization copied incorrectly.
- Lazy trigger/priming boundary differs, losing/duplicating tail blocks or
  falsely committing an unresolved MCR cruise chain.
- ACCEL_TO_DECEL and ratio simultaneous precedence/legacy windows differ.
- Single block, synchronization or buffered delay gets dropped or double counted.
- Move time is right but IDs, actual-speed splits or aggregate/footer/M73 remain
  wrong; do not hide these under a time-only whole-output GREEN claim.
- Profile resources are missing or source/binary mismatch: fail before parity,
  not accept a skip or derive fake expectations from source arithmetic.

First RED should expose the real first motion/time mismatch, retaining the
complete public output comparison for later verification. Existing test
harness may provide fixture setup but production private APIs must not be
imported. New test registration stays in an already-registered leaf module.

## Verification plan and exit gate

1. Pin source/main/fixture/preset and Native identities; record real generation
   command. Write this ARD before any test/production edit, then freeze a
   bounded public RED. No numeric literals from diagnostic JSON as assertions.
2. Submit SPEC/RED RR and wait before production changes. The RR must identify
   exact observed failure and any harness/environment prerequisite separately.
3. If implementation is approved: first port only the confirmed full Klipper
   source boundary above. Rerun the same processor and Plater public tests;
   inspect every downstream strict first difference, never manufacture GREEN.
4. Targeted Marlin/public processor controls, Klipper legacy/ratio/flush cases
   and the six original acceleration-literal cases must be run only within
   separately approved bounded runtime scope. The six Klipper acceleration-literal
   cases stay open until their strict results receive the timing-owner disposition;
   improving timing alone does not close them. Gap-fill and spiral-lift differences
   remain with their existing owners and are not fixed by this estimator slice.
5. Use nextest rather than cargo test. fmt and workspace/all-target/
   all-feature clippy `-D warnings` required. Keep all runtime dumps outside
   Git; no G-code, Native binaries, caches or computed Native goldens committed.

Deferred: new stealth-mode public data APIs, other firmware planner changes,
viewer/UI behavior, source slicer geometry/FillBase/spiral-lift emission,
multicore work, cache delivery/retention and any unrelated M73 approximation.
None of these deferrals permits a Klipper fallback or a false parity claim.
