# SPEC: Repetier travel-acceleration cache admission

## Source owner and architectural constraints

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:49-52::apply_print_config` initializes
`m_max_travel_acceleration` only when BOTH `use_mach_limits` and
`supports_separate_travel_acceleration` are true; otherwise it stores zero.
The `std::round` conversion remains part of that source operation.
The first predicate at lines 32-35 excludes Repetier; the separate-travel
predicate at lines 23-26 includes it. Therefore Repetier supports separate M202
travel commands but has no writer machine-limit cache for those commands.

`set_acceleration_internal`, lines 216-237, applies that cache before command
suppression and Repetier M202 X/Y serialization. It already behaves correctly
with a zero cache. Do not disable separate-travel command support to hide the
bad cache value.

Rust destination:
`gcode_emit/motion/options/build/fields/assemble.rs::assemble`, the
`max_travel_acceleration` initializer. It currently checks only the separate
flavor set, incorrectly admitting Repetier's raw machine cap. The fix is to
add ONLY the existing local `use_mach_limits` value as a conjunction to that
initializer. Preserve the current flavor set, rounding, u32 cache type and zero
branch. Do not extract helpers or alter the existing shared predicate.

The options assembly remains a temporary compatibility shell around the cited
upstream writer cache. This is not a new Ares travel/acceleration pipeline.
Included: the missing cache-admission conjunction. Deferred: travel execution,
processor limits and time estimates, process acceleration, print-cache ties-even,
jerk gates, raw headers and generic `rounded_acceleration`. No changes to
auto_lift/M73, the tokenizer/fan/zero-E or value.rs/coInts files. No masks or
processor compensation.

## Saved-project witness and controls

Use the genuine GUI-generated `tests/parity/orca_cli_ender3.3mf`. No BBL model
identity or cross-family BBL archive is needed. All three cases disable the
prime tower and set `default_acceleration=1000`,
`initial_layer_acceleration=500`, and `machine_max_acceleration_travel=9000`.
Only flavor or requested travel acceleration differs between cases. Geometry,
templates, jerk, temperature, PA and fan settings remain unchanged.

| Case | Flavor | Travel request | SHA256 |
|---|---|---|---|
| Over-limit target | Repetier | 10000 | a1bbe4c18e06b709f580a67fe8bec437ca1d080f1b70912075252229f0d53b65 |
| Below-limit control | Repetier | 6000 | 97495393518915bbd5285bdd90785745f03b1ef50cc625d5251daeaf0fd64d07 |
| Enabled-limit control | Marlin2 | 10000 | 2b232b6735ba52645223c1adbf9734320efccbbd5f5f347df7458cb0c1317134 |

The target's first normalized difference is byte 1211: Native emits
`M202 X10000 Y10000`, Ares emits `M202 X9000 Y9000`. The complete diff contains
exactly 50 such M202 differences, with equal line counts and all M73 records
identical. Both controls match complete streams. The below-limit case verifies
stable ordinary Repetier behavior; Marlin2 verifies that the proper machine
limit remains active. These are admitted saved settings, not a recommendation
for factory Ender3 firmware or physical acceleration tuning.

All six fresh Native discovery slices replay identically and all Native/CLI
exits are zero. The older deferred BBL travel=10000 observation is not needed
as a public input in this slice.

## RED seam and failure modes, before implementation

Freeze the three archives. Add direct calls to the existing
`native_stream_case` Native/API/CLI seam. Fresh Native replay qualifies first.
Callbacks derive requested travel acceleration, cap and flavor from the input,
then observe actual Native body travel commands. For Repetier, M202 X/Y must
follow the request without machine clamping; for Marlin2, M204 T must remain
capped. The initial envelope command is not the writer's body cache. Assertions
use saved values, not hardcoded Native coordinates, counts or output goldens.

Then compare complete Native/API/CLI streams in official and seeded modes using
only the established generator-line normalization. Keep timing, headers,
configuration and footers. Require raw API==CLI. Callback observations are not
filters. Commit no Native output references. Expected RED: the target fails at
the M202 clamp; both controls pass; all Native replay and callbacks qualify.
Commit RED before GREEN.

Failure modes: forgetting the `use_mach_limits` conjunction, removing separate
M202 support instead of disabling only its machine cache, disabling Marlin2/RRF
limits, changing rounding or other caches, inactive requests below the cap,
confusing raw envelope data with body commands, and masking downstream output.
No new Klipper, BBL, rounding or comment variants are introduced.

GREEN: only the initializer conjunction, RED tests and
inputs unchanged; fmt,
workspace all-target clippy and the same three cases in official then seeded
mode. Preserve normal group separators, root registries and the 400-line Rust
file limit.
