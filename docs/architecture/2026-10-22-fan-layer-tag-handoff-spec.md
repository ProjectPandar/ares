# FanMover per-layer reserved-tag handoff — independent SPEC

## Authorization and upstream owner

Docs-first SPEC → test-only RED on exact main
`afcb6aa7c90f8eb363cba82aaaac9e23717d0cfb`; no production implementation,
GREEN, FULL or integration authorized. This is independent of paused signed
FanMover SPEC/RED a741 and rejected candidate740; neither is resumed or changed.

Pinned upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/GCode.cpp:3727–3743` and`:3830–3841` define serial per-layer
  FanMover filters: each incoming layer string is processed with
  `fan_mover->process_gcode(in, true)` (whole-layer flush).
- `GCode.cpp:4619` emits the reserved `ETags::Layer_Change` marker.
- `GCode/GCodeProcessor.hpp:375` selects printer-family tag vocabulary;
  `GCodeProcessor.cpp:64` BBL ` CHANGE_LAYER`,`:87` compatible `LAYER_CHANGE`.
  G-code comment form includes semicolon: `; CHANGE_LAYER` vs `;LAYER_CHANGE`.
- `GCode/FanMover.cpp:24–44` recomputes buffered time and drains remaining
  items when flush=true. Its state machine is not redesigned in this slice.

The owner is the libslic3r **per-layer FanMover input/flush handoff**, not a new
Ares cooling pipeline, output-reordering patch or generic G-code parser.

## Rust destination and included semantics

`crates/ares-core/src/project_slice/gcode_emit/orchestration.rs:326` forwards
aggregate layer bytes into `fan_chunks.rs::process_layer_chunks`.
`fan_chunks.rs:16` recognizes only compatible `;LAYER_CHANGE`, although the
existing `tags.rs:9–10,25–41::Tags::of/layer_change` correctly chooses/emits BBL
`; CHANGE_LAYER` for this project. The aggregate compatibility shell currently
misses those boundaries and calls FanMover once for all50 layers.

Port the cited upstream handoff into that existing orchestration/fan_chunks/tags
boundary only:

1. Partition by the **actual emitted reserved layer-change tag**, selected by
   existing printer-family vocabulary. Recognize the BBL and compatible forms
   as that existing vocabulary, not a new flavor heuristic or new marker text.
2. Give FanMover each owning layer chunk separately and use **flush=true for
   every chunk**, retaining FanMover state across calls as upstream does.
3. Preserve layer-leading M106/M107 ownership: upstream rebuilt layer text can
   prepend fan commands before the reserved marker (`change_extruder_set_fan`).
   Keep the existing backward fan-prefix handling, not an output-only fan move.
4. Preserve leading/trailing bytes and layer order; no duplicate/drop/rewrite of
   markers, geometry, custom G-code, fan commands or footer. Retain no-FanMover
   gate semantics and existing control behavior.
5. Reuse the existing typed tags vocabulary. No general-purpose parser,
   additional scheduler/state counter, compatibility fallback or synthetic tag
   injection. Existing aggregate-layer scaffold is only a temporary adapter
   around the cited upstream per-layer concept, not an Ares-owned pipeline.

Core remains pure byte-oriented/platform-neutral: browser WASM,Windows,macOS,
Linux; no filesystem,terminal,UI,Native oracle or debugger dependency in core.

## Deferred / hard stop boundary

Defer all other FanMover behavior: delay thresholds, float/double accumulation,
`_print_in_middle_G1` interpolation/rounding, kickstart scheduling, remove-slow-
fan semantics, role classification, arc handling, custom-code barriers, general
flush implementation, PWM/flavor writer policy and option admission. No timing/
M73 repair, numerical tolerance, output masking or downstream split patch.
Do not incorporate negative-time abs conversion/with_D_option representation;
a741/740 remains paused until this independent positive slice is resolved.

Any eventual residual requires STOP/report and separate source-owned approval,
not adjacent changes or broader FanMover fixes. This SPEC does not assert an
untested tag change will close all possible fan cases.

## Positive GUI witness and frozen inputs

Accepted read-only diagnosis:
`${EVIDENCE_ROOT}/w3-fan-positive-delay-readonly-ad5111f0b/README.md`,
manifest `87bf617358a0be11ad085fa6d95efb6b8205e2f8a19416a9a9ad1a0d297f4b71`.
Copy the existing two input bytes exactly; no regeneration or output golden:

| Input | fan_speedup_time | SHA256 |
|---|---|---|
|control.3mf|0|17896c6fdf4d5d570c604843fafd049cbc1ac6cde86a21c07dd20ebfdd8a13b2|
|target.3mf|+3|a7ed84d65c690a3e50b7d8623e545fde6537f15ffdf43db209d940d3285361c3|

Origin tracked modifier-bottom-shell/classic-four.3mf SHA438bf4c61895a7a8d51e3c61a70fa8df120a9f3a607d63a4291d359984154a7c.
Stored Bambu Standard-Mine profile is customized, not pristine factory.
Both share fan_min/max20 and Only overhangs0. Geometry,placement,modifier
bottom-shell4,all other original settings and non-settings ZIP payloads stay
exact. Pair differs only in fan_speedup_time0→3.
GUI `Tab.cpp:4552–4556` exposes Cooling Fan speed-up time;
`PrintConfig.cpp:3830–3845` defines coFloat seconds; positive3 is ordinary GUI
input, not private-key activation. Both public and Native slices succeed.

Discovery confirmed control whole PASS/target whole FAIL both modes on ad511,
Native stable/raw API==CLI. Read-only canonical observers decoded Native50
single-layer strings vs Rust1 string containing50BBL markers. First20% fan
increase sees Native buffer0 but Rust3.093228s and a previous-layer front;
Rust takes the existing delay split at Y125.939. Invalid empty-list sentinel
reads and unknown breakpoint locations were excluded. Those are diagnosis
facts, not expected state/counter/output constants in the formal tests.

## Formal RED seam and stage exit

Exactly two direct unchanged `runner::native_stream_case::check` calls with
`(false, |_, _| {})`: live Native double replay before public validated API and
actual CLI, then complete ordered equality/raw API==CLI. Commit only two inputs,
SHA README and thin wiring; no Native G-code/golden,private observers,ignored/
inverted checks,unsupported-as-success,extra helper/comparator or tolerances.
Keep fan/M73/time/E/F/wipe/start/end/footer bytes in the original comparison.

Separate docs-only SPEC and test-only RED commits. On the exact clean RED tip,
CPU12/jobs1/threads1/incremental0,fresh sequential official/seeded roots: expect
one control PASS and one genuine whole-output target FAIL each mode. Native
must succeed/replay stably; API/CLI must succeed/raw-equal. Target panic can
precede wrapper-tail; audit raw bytes without claiming unexecuted assertions.
Run fmt and locked all-target Clippy,retain full logs/exits/source/input/runtime
provenance,submit SPEC/RED RR. **Stop before GREEN/production/FULL**.
