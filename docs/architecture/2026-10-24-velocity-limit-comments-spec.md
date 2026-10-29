# SPEC: Klipper merged velocity-limit verbose suffix

Base `a6232a1dfa1735463e6be9088e9cc8bb673bb47b`; coordinator ACK authorizes
SPEC → Native-first public RED → minimal GREEN → review and hold, no FULL.

## Non-negotiable source boundary

Port `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:311-350::set_accel_and_jerk`,
only the nonempty command terminator at341-346: after unchanged admission/cache
logic, append ` ; adjust VELOCITY_LIMIT(accel/jerk)` when `full_gcode_comment`,
then newline. Rust destination
`crates/ares-core/src/project_slice/gcode_emit/motion/acceleration.rs`,
`set_accel_and_jerk` final `if !empty` arm (currently newline only).
Existing Ares writer code is a compatibility shell around this upstream writer,
not an independent command/comment policy. No new helper or separate pipeline.

Included: exact optional suffix on emitted merged Klipper velocity commands.
Deferred: admission/caches, clamps, numeric formatting, separate acceleration
and jerk writers, merged jerk1e-6 versus upstream EPSILON comparison, templates,
processor/fan/tower/skirt/brim/support behavior. Do not fold in the known merged
jerk tolerance issue; it requires a separate follow-up proposal.

## Evidence and GUI provenance

Stock Ender3 cube, tower off; common `gcode_flavor=klipper`,
`default_acceleration=500`, `travel_acceleration=700`,
`initial_layer_acceleration=500`. Target differs ONLY `gcode_comments=1` from
control0. GUI `Tab.cpp:2753` Verbose G-code, `PrintConfig.cpp:3965` boolean.
No geometry, templates, temperatures, lift, PA or jerk changes.

Control SHA `644dbe2a3e522c3f835ff854c5388ef546820a02cf31eeab442deb8fee3553e9`:
whole output agrees. Target SHA
`ce5dc71dc0a8e39933614cf0d7f752071a5fd3728cdc88a666bfb673560b938b`:
first normalized difference byte87481; Native has the exact suffix on the
ACCEL250/ACCEL_TO_DECEL125 command and restoration500/250, Ares omits it.
Four fresh Native slices replay-stable; actual exact-main CLI used. Verbose
byte-fidelity only, not changed motion. Full output, generator-line normalization
only; no masks. Discovery evidence
`/home/indexyz/ares-evidence/w4-next-velocity-comments/` including ownership audit.

## Test-first plan / failures / exits

Before production: add only two SHA-pinned archives and Native-first public
cases through existing `native_stream_case`, with no new helper. Native replay
and callback must qualify actual merged commands: comments enabled requires
source suffix, disabled requires absent suffix; both must exercise the command.
Then unchanged full Native/API/CLI comparator, all timing/config/footer included.
CPU17/PIN17/jobs1/threads1, official then seeded. RED target fails at missing
suffix while control passes; verify API==CLI. Commit RED before production.

Failure classes: comment appended to empty/no-op commands; suffix emitted with
flagoff; wrong spacing/spelling/placement; modifying cache/clamp/tolerance or
other writer paths; test callback observing only standalone velocity commands.
The nonempty nesting and source review must prove no synthetic no-op line.
No hardcoded Native coordinates, reference goldens, count masks or output filters.

GREEN changes only conditional suffix in existing nonempty arm. Tests/inputs
freeze unchanged. Run fmt, workspace all-target clippy and both directed cases
in both modes, preserve actual statuses/logs, issue REVIEW-REQUEST and hold.
No FULL, merge or push. Final files stay≤400LOC; group test registration without
squeezing root registries. Native outputs remain external artifacts only.
