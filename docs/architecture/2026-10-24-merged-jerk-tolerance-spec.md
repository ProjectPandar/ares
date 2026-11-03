# SPEC: merged Klipper jerk approximate equality

Base frozen comment `062c0bedd7fdfc6911b824dba1ca0d9b997e8fe8`.
Coordinator ACK: SPEC → public Native-first RED → minimal GREEN → review/hold.
No FULL. No formatting-precision correction in this slice.

## Source owner and non-negotiable rewrite boundary

`OrcaSlicer/src/libslic3r/libslic3r.h:52`: EPSILON1e-4;
`:306-309::is_approx`: strict absolute difference below default EPSILON.
`GCodeWriter.cpp:330-338::set_accel_and_jerk`: clamp XY jerk, then admit only
`jerk > 0.01 && !is_approx(jerk,m_last_jerk)`; cache is updated only on admission.
Rust destination `gcode_emit/motion/acceleration.rs:114-120`, sole cached-jerk
comparison. Existing code is a compatibility shell for that upstream writer,
not an independent Ares tolerance/cache policy.

Included: replace1e-6 with the existing motion-visible `travel::EPSILON`, strict<,
without changing admission order or other behavior. Existing single definition
in `motion/travel/lift.rs:12` cites upstream. No helper/constant/visibility API.
Deferred: formatting, separate jerk writer, acceleration/clamps/comments/cache
policy, processor/templates/fan/tower and all other slices. In particular the
separate7.0002 serialization mismatch is NOT a control for this slice or fixed
here. It remains recorded in external evidence for the next proposal.

## Reachable evidence

Stock Ender3 cube/toweroff Klipper, default/initialaccel500/travel700, all role
jerks7, initial-layer travel jerk100%. Sole-key variants inner_wall_jerk:
7.00005 target;7 equal;7.0000001 withinboth;7.002 cleanabove. XYlimits8 do not
clamp these. Comments disabled. GUI Tab2566-2568, PrintConfig3317-3324 coFloat/min0.
No claim about GUI displayed decimals or physical need for such fine tuning.

Discovery target firstdiffbyte6161: Native proceeds to Innerwall, Ares emits
redundant `SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY=7` and later restores7.
Three cleancontrols fullPASS,10Native slices5inputs replaystable (the fifth,
7.0002, fails independently due precision and is explicitly excluded).
Evidence `${EVIDENCE_ROOT}/w4-merged-jerk-tolerance/PROPOSAL.md`,
results, clean-control and original streams. Exact062c existing CLI, CPU17.

SHA inputs:
- equal7:173b2cd1e4d549de65953512f9d8d3d853f3753f68ff0546b0be63368987a78b
- target7.00005:e407c7c7ea2f19cfd1436c7406b1d8f31b1fd72f1a776c60e814bfb20a276f60
- withinboth7.0000001:df9e6effe76a115996017892f4b488764add7073744619f7d69a4b7adabd9d4b
- above7.002:53ddea7da1ffd0262c4a04d9c83913adc0b46167349950732b4613099a584991

## Test-first plan / failure modes / exits

Freeze four archives and Native-first existing `native_stream_case` calls.
Fresh Native replay and callback must precede unmasked whole API/CLI comparison.
Callback observes whether body square-corner velocity values vary among actual
Native commands, not hardcoded reference values/counts. Target/equal/withinboth
suppress changed jerk; cleanabove admits it. No Native outputs committed.

Pin before implementation:1e-6 admission mistake; unconditional suppression;
cache updates on suppression; comparing unclamped requests or formatted text;
wrong relative/non-strict test. No decimal7.0001 exact boundary assumption.
Review source to retain clamp-before-comparison and suppress-before-cache;
public byte tests alone cannot prove private cache invariants.

CPU17/PIN17/jobs1/threads1 official then seeded. Commit RED when only target
fails; qualify Native replay/callback and raw API==CLI. GREEN modifies only the
strict comparison/import, leaving frozen tests/inputs and comment suffix intact.
fmt/workspaceall-targetclippy and fourcases bothmodes must pass, then
REVIEW-REQUEST/hold. No FULL/merge/push, no new helpers/lint suppressions.
