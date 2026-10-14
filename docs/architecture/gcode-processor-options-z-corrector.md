# GCodeProcessor: OptionsZCorrector vertex shift and M73 cache eligibility

Status: spec for review. Docs only. The RED (the `same-layer` case moved out
of slice C, `docs/architecture/plate-custom-gcode-emission.md`) is merged
together with this slice's GREEN. The RED needs slice C merged first,
because without slice C's emission the fixture has no `;CUSTOM_GCODE` tag.

## Finding

With the plate custom G-code emission (slice C), the `same-layer` fixture
emits the same G-code body as Native. Only two progress lines move:

| | `M73 P27` | `M73 P28` |
|---|---|---|
| Native, one `;CUSTOM_GCODE` at Z 2.2 | line 932 | line 960 |
| Ares, same input | line 936 | line 955 |
| Native by-layer control, no custom G-code | line 936 | line 955 |

Four variants on official Native reproduce it exactly: the original two
events, one event, one event with empty `extra`, and one event at 2.15.
So the trigger is the lone `;CUSTOM_GCODE` tag that the processor reads.
The emitted payload plays no part. The total time does not change.

## Upstream boundary

| Upstream | Behavior | Ares destination |
|---|---|---|
| `GCode/GCodeProcessor.cpp:3313-3320` (`Custom_Code` tag) | `store_move_vertex(Custom_GCode)`, a `custom_gcode_per_print_z` item, `m_options_z_corrector.set()`; no time call. Pause (`:3303-3310`) and ColorChange (`:3291-3296`) take the same path and then call `process_custom_gcode_time`, which flushes the planner | `processor/moves/tags.rs` (unchanged; it already records `pending_option`) |
| `GCode/GCodeProcessor.hpp:678-711` `OptionsZCorrector` | `set()` stores `m_move_id = moves.size() - 1`. `update(height)` (called from `process_G1`, `GCodeProcessor.cpp:3929`, before the G1's own block and vertex) copies `moves[m_move_id]` to the back with the current position and height, then **erases** `moves[m_move_id]` | `processor/moves.rs` `correct_options_z` |
| `GCode/GCodeProcessor.cpp:4004` | a queued `TimeBlock` stores `move_id = moves.size()` when it is created. Nothing renumbers it when `update` erases a vertex in front of it | new `processor/estimate/options_z.rs` |
| `GCode/GCodeProcessor.cpp:5951-6009` `GCodeProcessor::calculate_time` | each flush inserts the actual-speed vertices (`internal_only`) into `moves` and renumbers the queued blocks (`:6003-6008`). It does **not** renumber `OptionsZCorrector::m_move_id`, so after a flush between `set()` and `update()`, `update` relocates and erases the wrong vertex | same new module |
| `GCode/GCodeProcessor.cpp:419-580` `TimeMachine::calculate_time` | for each processed block, Normal mode reads `moves[block.move_id]`. If its **type** is not Extrude/Travel/Wipe, it `continue`s (`:484-488`) before `g1_times_cache.push_back` (`:574`) and the stop-time update (`:576-579`) | `processor/estimate.rs` `cache_eligible` (today derived from the block's own motion, not from the vertex `moves[move_id]` resolves to) |
| `GCode/GCodeProcessor.cpp:771-811` `ExportLines::update`, `:1236-1300` `process_line_move` | M73 is written only at G1 lines that have a `g1_times_cache` entry | `processor/estimate/elapsed.rs`, `export_cache` (unchanged) |

**Mechanism (to be confirmed by the GREEN probe below).** When `update`
erases vertex `k`, every block still queued with `move_id > k` now resolves
to the vertex one place later. `move_id == moves.size() - 1` resolves to the
relocated marker. A travel block can then read an `Unretract` vertex and
lose its cache entry, or a retract block can read a travel vertex and gain
one. A flush between `set()` and `update()` also makes `k` stale, so a
different vertex is erased. The diagnostic `procgcode-stable-options.patch`
already corrects that stale index; official Native does not. The `procgcode`
oracle run with and without the tag shows the side effect in
`moves`: an `internal_only` actual-speed vertex from G-code line 700 ends up
at the back with Z 2.2 (`/tmp/ares-estimation/diag-same-layer/proc/`).
Ares decides cache eligibility from each block's own motion and does not
model either shift, so the set of M73-capable lines around the tag
differs.

## Included and deferred

**Included:** M73 placement (`g1_times_cache` membership) and stop-time
anchors for blocks queued across an `OptionsZCorrector::update`, including
a stale `m_move_id` after a flush. Covers Custom, Pause and ColorChange
markers.

**Deferred:**
- The per-vertex side effects on `GCodeProcessorResult::moves`: which vertex
  receives `time` (`:472`), the actual-speed insertion point and
  `actual_feedrate`, and the relocated vertex's identity. These are viewer
  data, with their own oracle (`--dump-moves`); a later slice.
- Stealth mode (disabled on the fixtures).

## Probe first (GREEN step 1)

Before code, an instrumented `procgcode` build with the fixture machine
config and the official export (`finalize` + post-process) logs, for each
processed block, `g1_line_id`, `move_id`, and the type that
`moves[move_id]` resolves to. This shows the exact blocks whose cache
membership changes on `same-layer`. If the probe disproves the mechanism,
this spec is amended before any code.

## RED

`plate_custom_gcode_emission_same_layer_matches_native` (fixture
`tests/parity/plate-custom-gcode-emission/same-layer.3mf`, written by
`tools/chain-probe/plate-custom-gcode-emission-fixtures.mjs`), restored
from slice C. It runs official and seeded, and is expected to fail on the
ordered-bytes assertion at the first M73 line (line 932), in both modes.
A RED that fails anywhere else does not count.

**Ways it could fail:**
- Shifting only blocks queued at `update` time while ignoring an
  intervening flush (stale `k`), or the reverse.
- Applying the shift to blocks already flushed before `update`.
- Off by one at `move_id == moves.size() - 1` (the relocated marker).
- Changing the total time or stop-time values rather than only which lines
  carry them.
- Moving the existing Pause/ColorChange cases, which flush at the tag.

## Gates

| Stage | Gates |
|---|---|
| RED | `same-layer`, official + seeded, assertion failure at the M73 line |
| GREEN | `same-layer` passes in both modes; slice C's other cases and the processor/estimation parity tests do not regress; `cargo fmt`, strict clippy, wasm32; directed nextest `ares-core` + `ares-cli`; canonical exclusive full (official + seeded) with 0 new failures |

Each stage runs on the assigned CPUs inside a directed window, with
START/END in the inbox.
