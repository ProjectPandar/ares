# Fully clipped loop motion-state parity

## Source owner and destination

Rewrite `libslic3r/GCode.cpp:5796-5805::GCode::extrude_loop`'s empty result
after `ExtrusionLoop::clip_end` at
`ares-core/src/project_slice/gcode_emit/motion/loop_paths.rs::emit`.
Native returns before wipe-before-external, wipe-path replacement and inward
travel when the seam gap consumes the entire loop. Preserve that behavior,
including scarf and active spiral-vase loop handling, without introducing
any new rejection or changing support generation.
The same input also exercises `GCode.cpp:320-358,435-497,7659-7670`'s
stored-path wipe eligibility when the current substituted path has no moving
segments. Port this boundary in `motion/travel/wipe_sequence.rs`: an eligible
stored path still runs before-wipe retraction, wipe tags/speed/reset and final
top-up. Do not substitute nonempty movement segments for `Wipe::has_path`.

## Included and deferred

Include clipped-empty loop state/output handling and real 3MF complete-output
regression through the existing core byte-input/byte-output seam. Use a
single cube with public `seam_gap=50` mm, larger than its closed wall loops,
and enable wipe-before-external. Verify exported settings, native no-wall
emission and positive native infill, then compare the entire ordered stream.
Only the generated-by line may be normalized. Retain native and Rust streams,
input and manifest even on failure; do not filter movement or M73 commands.

Defer Tree/Normal support, modifiers, regional config and multi-tool work to
their existing source-cited slices. This independent change must not carry
the held support/region batch into a commit. All printer/options/runtime
coverage remains part of the full goal, not established by this one tracer.

## Acceptance

The same public input must fail before the code change and match complete
native output afterward. Preserve existing passing behavior, original KSR
and spacing assertions; run fmt, Clippy, nextest workspace, existing emission
controls and Tier-1 checks. Independently review all six dimensions, fix and
re-review before a code-plus-docs Conventional Commit and push.
