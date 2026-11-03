# SPEC: merged Klipper square-corner velocity ostream serialization

Base frozen `917ec39dde89d5f439d96b769f61b978b7018dc0`; coordinator ACK authorizes
SPEC → Native-first public RED → one-field GREEN → review/hold, no FULL.

## Source boundary / non-negotiable decisions

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:315-337::set_accel_and_jerk` creates a
plain `std::ostringstream gcode` then streams admitted double jerk without
`fixed`/`setprecision`, hence defaultfloat six significant digits. Target owner
at336 is the `SQUARE_CORNER_VELOCITY` word only.
Rust destination `gcode_emit/motion/acceleration.rs` same merged writer field,
currently `format::axis(jerk)` (three fixed places). Existing `motion/format.rs:78`
`ostream` already ports six-significant formatting and is used by `jerk.rs`.
Use it, not a new formatter or changed formatting policy. Existing Ares command
writer is a compatibility shell for this upstream writer boundary.

Included: only replace that field's `axis` call with `ostream`. Deferred:
ACCEL_TO_DECEL and other numeric fields, formatter implementation, machine
clamps, command admission/EPSILON/cache ordering, comment suffix, separate jerk
writers, and all other owned regions. No new helper, precision option, mask,
legacy fallback or lint suppression. Prior comment/EPSILON slices remain intact.

## Evidence / reachability

Stock Ender3/toweroff Klipper; allrolejerks7/initialtravel100%, default/initial
accel500/travel700. Sole-key inner_wall_jerk variants7,7.0002,7.002; belowmachine
limits8. GUI Tab2568/PrintConfig3317-3324 coFloat/min0. Saved numerical-boundary
settings, not a GUI display precision or physical tuning claim.

Target7.0002 Native `SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY=7.0002`, Ares=7,
first normalized difference byte6204. Equal7/exactmilli7.002 controls fullPASS.
Six fresh Native slices stable on exact917ec actual CLI after EPSILON fix; all
commands exit0. No API/seeded/FULL qualification claimed from discovery.
Evidence `${EVIDENCE_ROOT}/w4-merged-jerk-precision/PROPOSAL.md` and
original complete Native/CLI streams/diffs, CPU17/PIN17/threads1.

InputSHA:
- equal7:173b2cd1e4d549de65953512f9d8d3d853f3753f68ff0546b0be63368987a78b
- target7.0002:8b1a2d4927db6ce127d8f8392fab228a04485922a0db741de43d5a93d79fdee3
- milli7.002:53ddea7da1ffd0262c4a04d9c83913adc0b46167349950732b4613099a584991

## RED-first failures and exits

Before code add three SHA-pinned inputs and direct existing native_stream_case
calls. Fresh Native replay then callback observes that actual serialized velocity
words round-trip to the real saved input's inner jerk value (parsed from input,
not a hardcoded Native reference). Only then full unmasked API/CLI comparison.
No Native outputs, coordinate/count constants or new helpers committed.

Failures: fixed3 truncation instead of significant digits; formatting before
admission; changing command count/cache; replacing global formatter or adjacent
fields; losing exactmilli/equal outputs. Preserve field/word shape and existing
clamp/EPSILON/source order; no new validation policy.

CPU17/PIN17/jobs1/threads1 serial official then seeded: target expected RED,
two controls fullPASS, Native replay/callback/rawAPI==CLI qualify. Commit RED
before production. GREEN only one formatter call; inputs/tests frozen unchanged.
fmt/workspaceall-targetclippy and three cases bothmodes must pass; submit
REVIEW-REQUEST and hold, no FULL/merge/push. Group registration without squeeze,
all touched Rust files≤400LOC.
