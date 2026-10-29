# Overlapping spiral entry/final state RED

SPEC374f1c602 ports GCode/SpiralVase.cpp122–123/153–166/206–215 into
`gcode_spiral_vase/project.rs::process_layer`: independent states,
extrusion if/else-if priority and final passthrough chunk. Header line3
3vs2 follows missing chunk, not a third physical slice. No header,
processor, height/layer/shape special case belongs to this scope.

Both3MFs are INPUTS only: actual official exports copied byte-for-byte,
not expected G-code snapshots. manifest.json pins complete archive and
per-ZIP hashes, settings and provenance. Only project_settings.config
payload differs, with only spiral_mode0→1. No post-export edits.

Each test directly calls existing main `native_stream_case::check`:
Native double replay then full ordered validated public API+bare CLI.
No duplicated runner/normalization/JSON logic, extra Native content
assertion, copied helper, private seam, borrowed gate, ignored test or
expected-failure conversion. SHA freezes effective settings. Shared
native-replay.json links both Native roots; comparison.json retains both
public attempts even on actual RED failure. Complete bytes including
M73/comments/E/timing/line endings are compared with only existing
canonical generator identity/date normalization.

Filter `test(spiral_overlap_transition_)`: official and seeded must each
show actual positiveFAIL/controlPASS. Positive discovery first mismatch
is line3 total layer number Native3/Ares2; body input is two layers on
both sides. Native's final259byte nonextruding passthrough chunk is
omitted by old Rust state gate. No expected output is embedded in tests.

Original exports/launches/source snapshots:
`/home/indexyz/ares-coord/evidence/w5-spiral-overlap-transition/`.
Reproduction: `$ORCA --slice 0 --outputdir /tmp/spiral-short positive.3mf`.
Formal RED and SPEC need review before any GREEN production edit.
No FULL is authorized; later FULL waits for an explicit merge window.
