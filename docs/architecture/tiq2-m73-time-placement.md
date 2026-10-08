# TiQ2-M73: time-machine placement divergence on the Dumbbell long-loop case

Status: spec for coordinator review (diagnosis + fix plan). No production
change, no RED, no GREEN in this document. Base: main 3cad8b4c3. Inputs and
dual streams are worker1's retained evidence, read-only:
`/tmp/ares-tiq2-loop-red/corrected-transport/03-long_loop_fragments/`
(input.3mf, native.gcode, ares.gcode, manifest) and the admitted record in
`/tmp/ares-pi-lint/docs/architecture/gcode-small-perimeter-loop-speed-red-controls.md`
("Separate Dumbbell time discrepancy / handoff ARD").

## Observed first difference (worker1's admitted complete-byte DIVERGENT)

native.gcode lines 80-82 vs ares.gcode lines 80-82 (both 849→851 lines total,
identical perimeter moves and E values):

- native: `G1 X119.69 Y106.31 E.22677` → **`M73 P67 R0`** → `G1 X119.69 Y113.69 E.22677`
- ares: `G1 X119.69 Y106.31 E.22677` → `G1 X119.69 Y113.69 E.22677` → **`M73 P67 R0`**

Same marker, same P/R values, attached one move later in Ares. Later chunks of
the diff (M73 P71 placement, a seam/travel coordinate) follow the same
one-marker-shift pattern.

## Upstream emission chain (source-cited)

1. Time simulation — `TimeMachine::calculate_time`
   (`GCode/GCodeProcessor.cpp`, trapezoid over `TimeBlock`
   `GCodeProcessor.hpp:447`): per-move block with acceleration/jerk
   constraints, feeding
   `machine.g1_times_cache` (`G1LinesCacheItem`: g1-line id → cumulative
   `elapsed_time`) and `machine.stop_times`.
2. Post-process M73 emission — `GCodeProcessor.cpp:1242-1252`
   `process_line_move(g1_lines_counter)`: advances
   `g1_times_cache_it[i]` while `it->id < g1_lines_counter`, probes
   `it->id == g1_lines_counter`, and when
   `{int(100·elapsed/machine.time), time_in_minutes(machine.time − elapsed)}`
   differs from `last_exported_main[i]` emits
   `format_line_M73_main(mask, P, R)` via
   `export_line.append_line(…, true)` (`:1249-1253`). The R (stop-time)
   emission follows with its own `is_last` logic (`:1255-1290`).
3. Placement — `ExportLines` (`:1056-1059`: backtrace-enabled → ByTime)
   buffers lines and re-orders them by simulated time, which is what places
   the M73 marker relative to the surrounding G1 moves.
4. Rounding — `time_in_minutes` (`:1013-1016`) and
   `int(100·elapsed/machine.time)` truncation decide when P/R change.

## Ares port (already mirrors 1-4)

`project_slice/gcode_emit/processor/estimate/elapsed.rs`
(`elapsed_snapshots`: pre-increment probe `it->id == exported_g1_lines`,
mirroring `:1246-1248`), `estimate/elapsed.rs::Snapshot {elapsed, next}`,
`estimate.rs` (`elapsed_at`/`next_elapsed_at`), `time.rs`
(`time_in_minutes`), `export_lines.rs` (ByTime/BySize). The M73 writer
consumes these snapshots; no placement-rule difference is known.

## Diagnosis hypothesis and plan

Both sides emit `M73 P67 R0` with identical values: the writer and rounding
agree. The one-move placement shift means native's per-move `elapsed` crossed
the 67 % threshold one move earlier than Ares' — a small per-move elapsed-time
difference in the trapezoid model, not a marker-rule difference.

Diagnosis plan (each step falsifiable):

1. Extract the complete M73 P/R sequences from both streams and find the
   FIRST differing marker (the divergence may start before P67).
2. Map the differing marker to its move index (the marker sits at a specific
   g1 counter) and dump Ares' block-level time components for the moves in
   that region (acceleration source per block type, trapezoid in/out
   velocities, cruise length).
3. Compare against upstream `TimeMachine::calculate_time`'s block
   construction for the same moves (acceleration/jerk selection per move
   type, `TimeBlock` fields) to identify the divergent component.
4. The fix ports only the divergent component; the whole-file comparison of
   this exact case is the RED (already admitted by worker1's retained
   receipt) and the GREEN proof.

## Explicit non-goals

- No change to small-loop caller selection (worker1's slice owns it).
- No comparator relaxation; the retained complete-byte DIVERGENT stays.
- No M73 mask/footer changes; `time_in_minutes`/truncation stay.
- CoolingBuffer interplay is out of scope: the processor reads the
  post-cooling gcode on both sides, so any cooling F difference is already
  embedded in the inputs and not a time-model concern.
