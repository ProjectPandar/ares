# ARD: MM flush_into_infill object-label envelope (wiping two-pass)

Status: proposed. SPEC + RED.
Base commit: main `78d642702b9c1497d658b04d5478ba0d5dd86483`.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1` (seed-replay source).
GREEN is a separate slice.

## Witness

Stock multi-material P1S project: machine `Bambu Lab P1S 0.4 nozzle`, process
`0.20mm Standard @BBL X1C`, filaments `Bambu PLA Basic @BBL X1C` +
`Bambu PLA Matte @BBL X1C` (AMS two slots, flush matrix 2×2 at stock 280),
two cube objects assigned to extruders 1/2, type2 wipe tower, ByLayer.
Single GUI knob `flush_into_infill` 0→1 (`PrintConfig.cpp:7013`).

- Base control: generator-only PASS in official and seeded dual mode.
- `flush_into_infill=1`: stable dual-mode FAIL, 170 raw diff lines, all four
  producers exit 0. **Zero motion/E/tower bytes differ.**
- Native double replay is byte-stable modulo the generator line; Ares is
  byte-deterministic; the residual reproduces exactly across reruns.

Residual is entirely the BBL object-label envelope:

1. 84 sites (one per `; PURGING FINISHED`) where Ares emits an extra
   `; stop printing object, unique label id: N\nM625` immediately before
   `; start printing object, unique label id: N\nM624` — Native keeps the
   envelope from splitting across an empty wiping/normal half-pass.
2. 2 sites where the layer-end stop label id is swapped (Native 45, Ares 56):
   Native closes the envelope of the last instance that actually flushed its
   start label; Ares closes the label of the last `queue_stop` caller.
3. Envelope balance: Native 142 starts / 142 stops; Ares 142 / 205.

## Owning upstream boundary

Seed-replay source `GCode.cpp::process_layer` object-label lifecycle:

1. **5326–5329** — `for (print_wipe_extrusions = is_anything_overridden; ...)`
   runs the wiping/normal two-pass when flushing is overridden;
   `; PURGING FINISHED` heads the normal pass (84× in the witness).
2. **5352–5359** — per instance per pass, `m_writer.set_object_start_str`
   arms the pending `; start printing object, unique label id: N\nM624 ...`
   (single writer slot, `GCodeWriter.hpp:111`; overwrite semantics).
3. **5477–5481** — `; stop printing object <name>` comment per instance per
   pass, gated by `gcode_label_objects` only.
4. **5482–5493 — the defect boundary**: `if (!m_writer.is_object_start_str_empty())
   set_object_start_str("");` — a start label still pending (never flushed by
   motion) is **dropped and the end label is NOT armed**; `else if
   (m_enable_exclude_object)` arms `; stop printing object, unique label id:
   N\nM625` only when the start was actually consumed.
5. `GCodeWriter.cpp:1173–1201` — `add_object_change_labels` emits end then
   start at object-change points (`GCode.cpp:5274/5296/5544/5699/7467`),
   including the layer-end emission that closes the last live envelope.

`ToolOrdering.cpp:1593, 1672–1690, 1785` (flush_into_infill wipe selection)
and `PrintConfig.cpp:7013` (option definition) are context only: the witness
motion bytes already match byte-for-byte, so the wiping decision itself is
not the defect and is not in scope.

## Ares destination and localization

- `project_slice/gcode_emit/object.rs::ObjectLabels::queue_stop` — the BBL
  branch calls `motion::queue_object_stop_label(state, self.label_id)`
  **unconditionally**: it arms the pending stop even when the start armed by
  the same half-pass (`queue_start`, mirroring 5352–5359) was never flushed
  by motion. This inverts 5482–5493.
- `project_slice/gcode_emit/motion/state/object_labels.rs::queue_object_stop_label`
  — arms `pending_object_stop_label`; the drop-then-conditionally-arm
  semantics of 5482–5493 exist here only in the non-BBL `queue_exclude_end`
  ("Mirrors `GCode.cpp:5478-5494`: an unflushed start label is dropped …
  only then does the end label get armed"). The BBL arm lacks this guard.
- Caller context (unchanged by this slice): `layers/tool_entry.rs:186–196`
  queues start+stop for empty half-pass bodies (comments plus arming, matching
  5348–5372); `layers/boundary.rs:207` / `layers/transition.rs:73` /
  `tool_transition.rs:55` append pending labels at layer end / layer content
  start / tool transitions.

Localization evidence: with `flush_into_infill=1`, `is_anything_overridden`
is true (84 `; PURGING FINISHED`), so every instance is visited in two
half-passes; halves with no overridden extrusions take the empty-body path.
Native drops their pending start and arms nothing (comments only); Ares arms
stop labels from empty halves, which then surface as extra stop/M625 pairs
at the next append and as the stale layer-end stop id. The base control has
`is_anything_overridden=false` (0 `; PURGING FINISHED`, single pass) and
passes, confirming the single-pass path is already correct.

## Included rewrite (proposed GREEN)

Mirror `GCode.cpp:5482–5493` in the BBL stop path: when arming the stop
label, if the pending object start has not been flushed, drop it and do not
arm the stop. The change lands in `object.rs::queue_stop` (BBL branch) or
`motion/state/object_labels.rs::queue_object_stop_label`, matching the
existing `queue_exclude_end` structure. `layers/tool_entry.rs` and the append
callers stay unchanged.

Invariants: `; printing object` / `; stop printing object` comments remain
emitted for every visited instance half (5348–5352, 5477–5481); the non-BBL
`EXCLUDE_OBJECT_*` / `M486` path is unchanged; single-pass projects
(`is_anything_overridden=false`) are byte-unchanged; M624 base64 encoding,
timelapse layer envelopes, toolchange motion and wiping selection are not
touched. No fixture, printer, filament-count or print-sequence special cases.

## Deferred

Deferred upstream behavior: `ToolOrdering.cpp` wipe selection and flush
volume math (already byte-identical), non-BBL exclude-object flavors,
`add_object_change_labels` call sites beyond the envelope state machine
(gcode emission order is already byte-identical), multi-copy (>1 instance
per object) label sequencing beyond what the frozen witness exercises, and
`flush_into_objects`/`flush_into_support` two-pass witnesses (their knobs
were generator-only PASS in discovery but are not frozen here).

## RED definition and exit gate

Tests-only RED, `crates/ares-cli/tests/orca_parity/mm_flush_into_infill_labels.rs`
registered in `object_emission_cases.rs`, using `native_stream_case::check`:

- target `project.3mf` (SHA `c80f9ebb…`, flush_into_infill=1): requires
  Native (missing Native fails, no silent pass), qualifies two Native
  replays, asserts the two-pass marker `; PURGING FINISHED` is present and
  the Native envelope is balanced (starts == stops), then compares complete
  Native/API/CLI streams with the existing comparator and raw API==CLI.
  Expected to FAIL now (Ares emits 63 extra stop envelopes + 2 stale ids).
- control `base.3mf` (SHA `8e09d0d1…`, flush_into_infill=0): same machinery,
  asserts no `; PURGING FINISHED`; expected PASS now.

Selector `test(mm_flush_into_infill_labels)`, fresh
cache/artifact directories, official and seeded modes. No Native output is
committed; input 3MFs are committed byte-for-byte from the discovery
evidence. After RED, GREEN must keep this selector fully green in
both modes with fresh caches, plus fmt/clippy/wasm32 gates; any other
residual (including M73/timing) → STOP and report, never mask or broaden.
