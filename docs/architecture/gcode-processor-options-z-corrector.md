# GCodeProcessor: OptionsZCorrector vertex shift and M73 cache eligibility

Status: implemented (see GREEN). The RED is the `same-layer` case moved out
of slice C, `docs/architecture/plate-custom-gcode-emission.md`. It depends on
slice C, because without slice C's emission the fixture has no
`;CUSTOM_GCODE` tag.

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
at the back with Z 2.2.
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

The probe adds `ORCA_OPTZ_PROBE` stderr tracing to `TimeMachine::calculate_time`
(each Normal block: `g1_line_id`, `move_id`, block type, the type of
`moves[move_id]`, cached or not), `GCodeProcessor::calculate_time` (actual-speed
insertions per flush) and `OptionsZCorrector::set`/`update` (the index and
type of the erased vertex). It runs `--slice 0` on `same-layer` and the
by-layer control. The patch and its output are not committed.

**What it must explain.** The two Native `same-layer` outputs share their
body bytes. Only two M73 lines move, both on layer Z 2, 27 to 55 lines
*before* the `;CUSTOM_GCODE` tag at line 987, and in opposite directions:

| | Native | Ares / Native control |
|---|---|---|
| `M73 P27` | after the wipe `G1 X105.959 Y110.732 E-1` (line 931) | 4 lines later (936) |
| `M73 P28` | after `G1 Z2.4` (line 959) | 5 lines earlier (955) |

A vertex erased while these blocks are still queued in the planner
predicts exactly this: effects before the tag only, and gains and losses
of cache entries. A mechanism that only touches blocks after the tag is
ruled out by these lines.

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

## GREEN

The probe confirmed the stale index on `same-layer`: `set()` stored
`move_id` 1140, a flush inserted 260 actual-speed vertices, and `update()`
erased a Wipe vertex while the Custom vertex stayed in place.

`processor/estimate/options_z.rs` replays this after the planner
schedule. It keeps a log of every stored vertex (type and position), the
first vertex of each block (`move_id`), and the queue state at each
`set()`/`update()`. For every update it:

1. counts the actual-speed vertices that flushes between `set()` and
   `update()` inserted in front of the marker (same rule as
   `moves/actual_speed.rs`), which is how far the stored index is stale;
2. walks back that many elements from the marker to find the erased
   element, which is either a vertex or an inserted actual-speed vertex;
3. moves every block still queued at `update()` whose element is at or
   after the erased one to the next element.

A block whose element moved takes its `g1_times_cache` membership, and
with it its stop-time anchor, from the new element's type. `moves.rs`
`correct_options_z` is unchanged. The log is built on every pass, since
the M73 lines depend on it even when `moves` is not requested.

Still deferred: the per-vertex `moves[]` side effects listed above, and
Stealth mode.

## Gates

| Stage | Gates |
|---|---|
| RED | `same-layer`, official + seeded, assertion failure at the M73 line |
| GREEN | `same-layer` passes in both modes; slice C's other cases and the processor/estimation parity tests do not regress; `cargo fmt`, strict clippy, wasm32; directed nextest `ares-core` + `ares-cli`; full parity (official + seeded) with 0 new failures |
