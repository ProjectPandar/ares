# TiQ2-M73: time-machine placement divergence on the Dumbbell long-loop case

Status: spec for coordinator review (diagnosis + fix plan). No production
change, no RED, no GREEN in this document. Base: main 3cad8b4c3. Inputs and
dual streams are worker1's retained evidence, read-only:
`${TMP_ROOT}/ares-tiq2-loop-red/corrected-transport/03-long_loop_fragments/`
(input.3mf, native.gcode, ares.gcode, manifest) and the admitted record in
`${TMP_ROOT}/ares-pi-lint/docs/architecture/gcode-small-perimeter-loop-speed-red-controls.md`
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

## Root cause located: most_used_physical_extruder_id hardcodes logical 0

The M9711 line is the machine template's layer-change/timelapse gcode with
`{most_used_physical_extruder_id}` (H2D machine json,
`M9711 M{timelapse_type} E{most_used_physical_extruder_id} …`). Upstream
expands it as `physical_extruder_map.get_at(most_used_extruder)`
(`GCode.cpp:4679` in `change_layer`, `:5162/:5222` in tool-change paths):
`most_used_extruder` is the layer's most-used LOGICAL extruder, mapped
through `physical_extruder_map` to the physical extruder.

Ares (`gcode_placeholders.rs:70,97`) hardcodes logical extruder 0:
`physical_extruder_id_for_logical(0)` — with the cross-map
`filament_map = ["2","1"]`, native's layer-2 most-used logical extruder is
1 (filament 2), mapping to physical 1 (`E1`); Ares reads logical 0 →
physical 0 (`E0`). This is a real Ares port gap: per-layer most-used
extruder tracking is missing in the layer-change/timelapse placeholder
path.

Fix (after SPEC-REVIEW): track the layer's most-used logical extruder
(extrusion-per-extruder accounting in the layer-change path, mirroring
upstream `most_used_extruder`) and pass it to
`physical_extruder_id_for_logical` instead of the hardcoded 0, in BOTH the
layer-change gcode and timelapse gcode placeholders
(`gcode_placeholders.rs:70,97`).

## most_used_extruder computation (upstream ToolOrdering.cpp:955-976)

`ToolOrdering::cal_most_used_extruder(config)`: `extruder_count` sized to
`nozzle_diameter.size()`; for EVERY layer's tools, the layer's filaments
map through `config.filament_map.values[filament] − 1` into a per-layer
`std::set` (each distinct extruder counts once per layer);
`extruder_count[id]++` per set entry; `most_used_extruder` = argmax with
`>=` (the id-1-vs-0 comparison prefers the higher extruder on ties). The
count spans the WHOLE print, and the filament→extruder mapping is the
`filament_map` option — so with `filament_map = ["2","1"]` every layer
counts extruder 2 (filament 1) and extruder 1 (filament 2), making
extruder 2 the most-used (it appears first in the set scan and wins the
`>=` tie against itself... precisely: both extruders count 3 layers each;
extruder_id 1 vs 0: counts equal → `>=` keeps the later id 1 → E1).

Ares port: the same whole-print count over the tool-ordering layers
(each layer's extruder set via `filament_map`, deduped, mapped 0-based),
argmax `>=`; the result feeds `physical_extruder_id_for_logical` in the
layer-change/timelapse placeholders. Implementation requires plumbing the
count through the tool-ordering → placeholder path (next window).

Precision (coordinator-confirmed): `GCode.cpp:3342` calls
`cal_most_used_extruder` on the per-object-order ToolOrdering and `:3422`
on the global one; `:3690/:3791` pass each ordering's own
`get_most_used_extruder()` into `process_layer`. The Ares port must
count per print-ordering mode — both orderings carry their own most-used
extruder — never a single global count.

Coverage: the shipped `schedule.tool_ordering` (`tool_ordering::build`,
built over all objects) is the global print ordering — the non-sequential
path (`GCode.cpp:2769` `print.tool_ordering()`, recomputed at `:3422`).
The per-object mode (`:2742`/`:3290` rebuild a per-object ToolOrdering for
sequential printing, recomputed at `:3342`/`:3345`) is **deferred**: the
sequential print mode is not yet ported, so Ares has no per-object
ordering to count. When that slice lands, the sequential emit path must
recompute `most_used_extruder` from its own object ordering — not reuse
the schedule's global one.

## Diagnosis result (coordinator-ruled): seam-rooted, slice suspended

The wipe/retract position divergence is a DOWNSTREAM symptom of the layer-3
seam placement, not a wipe-algorithm difference. Measured loop endpoints
(the last extrusion move before the retract):

- native: `… X112.043 Y110.4 E.00548` → **`G1 X107.957 Y110.4 E.11889`**
  (the loop crosses the neck to the left end), then `G1 E-3.5 F3600` and the
  three wipe moves toward `X107.803 Y110.524`.
- ares: `… X112.043 Y110.4 E.00548` → `G1 E-3.5 F3600` directly (the loop
  STOPS at the right neck end), then a single `G1 X111.043 Y110.4 E-.5`.

The endpoints differ by exactly the neck width (X107.957 vs X112.043,
mirrored across the bed center): case (b) of the coordinator's split. Root
cause = worker6's seam-placement port bug (`seam_placement.rs:34` gauss
denominator e^(e−1) instead of e−1, distance penalty shrunk ≈8.24×), which
also explains the mirrored layer-3 travel target. The whole-file M73
placement divergence (and the ≈171 ms total-time delta from the different
wipe paths) sits downstream of it.

Slice SUSPENDED pending worker6's seam fix; ROOT CAUSE CONFIRMED AND FIXED
at main 117cd7edd (worker6's seam_placement.rs gauss denominator e^(e−1) →
e−1). Re-verified TWICE after the fix:
1. main 117cd7edd: whole-file PASS.
2. main 3d65ec24e (worker1 small-perimeter loop-only preselection + scarf
   caller speed merged): whole-file PASS again — the re-centering and M73
   placement remain byte-identical to native after the loop-speed change.
The whole-file RED stays unmerged (duplicate coverage of the seam fix's own
gates); this spec and the diagnosis record are the surviving docs.

## Explicit non-goals (unchanged)

- No change to small-loop caller selection (worker1's slice owns it).
- No comparator relaxation; the retained complete-byte DIVERGENT stays.
- No M73 mask/footer changes; `time_in_minutes`/truncation stay.
- No seam fix in this slice (worker6 owns SeamPlacer).
