# GCodeProcessor: OptionsZCorrector side effects on `GCodeProcessorResult::moves`

Status: SPEC approved (5c538e11e). COORD ruling (b): `custom`,
`mode-mismatch` and `pause` join `template` in the RED. If the GREEN does
not close one of them, work stops and the case is reported, not dropped.

This is the "slice C" deferral of
`docs/architecture/gcode-processor-options-z-corrector.md`: that slice
ported the effect of the stale `OptionsZCorrector` index on the time
blocks and the M73 cache. This slice ports its effect on the move list
the viewer reads.

## Finding

Discovery on main `57fea9498`. For each fixture, `ares slice --dump-moves`
was compared with the `procgcode` oracle (`tools/chain-probe/procgcode.nix`,
verbatim `GCodeProcessor` plus the dump hook). The oracle ran with
`--process-gcode --dump-moves` on the same G-code. The tolerances are the
ones in `crates/ares-cli/tests/gcode_moves_oracle.rs`. The evidence is in
`/tmp/ares-estimation/moves-discovery/r2/`.

| Fixture (`tests/parity/plate-custom-gcode-emission/`) | Rows oracle / Ares | Out-of-tolerance fields |
|---|---|---|
| bounds, by-object, color-change, tool-change, ender3, ksr_fdmtest_v4 | equal | 0 |
| **template** | 5564 / 5564 | 82 `gcode_id`, 156 `time0`, ... (first at row 1024) |
| custom | 5564 / 5564 | 128 `gcode_id` (first at row 659) |
| mode-mismatch (a pause) | 5564 / 5564 | 128 `gcode_id` (first at row 659) |
| pause | 5564 / 5564 | 141 `gcode_id` (first at row 253) |
| same-layer | **5560** / 5564 | 4304 `gcode_id` |

The `color-change` fixture shows no difference. Why it is unaffected while
`pause` (which also flushes in `process_custom_gcode_time`) is affected is
not established yet. The GREEN probe records it, and it stays a guard case.

**The oracle is consistent with Native.** The script
`/tmp/ares-estimation/moves-discovery/native-check.sh` re-ran the oracle
on the official-AppImage `plate_1.gcode` that the `7e6340dde` directed
parity run compared against.
- **G-code lines:** for all 9 fixtures, the non-comment lines of the Native
  file and of the Ares file are identical.
- **Oracle on Native vs oracle on Ares:** identical CSVs, 0 mismatches, the
  same row counts (same-layer 5560 on both).
- **Oracle on Native vs Ares:** the same mismatch counts as the table
  above.

So these are Ares divergences in `moves`, not artifacts of the dump format.
The Native G-code itself already matches (12/12 directed).

**`template` in detail.** The Native and Ares G-code is identical. Line 777
is `;CUSTOM_GCODE`, followed by `M117 template T0 H0.2` and `G4 P10`.

| Row | Oracle | Ares |
|---|---|---|
| 1024 | line 651, `internal_only` split, X109.406 | line 651, `internal_only` split, X113.285 |
| 1214 / 1221 | line 778 `Custom_GCode` left in place: X105.959 Y110.732 Z1.2, `time0` 0 | line 778 `Custom_GCode` relocated: X114.346 Y114.346 Z1.4, `time0` 0.130475 |
| 1221 (oracle) | the line-651 split vertex (X113.285, layer 4), moved to the back: X114.346 Z1.4, `time0` 0.130475 | — |

Native erased and relocated the first actual-speed split of line 651 on the
previous layer, not the marker. Ares relocates the marker itself.

## Upstream boundary

| Upstream | Behavior | Ares destination |
|---|---|---|
| `GCode/GCodeProcessor.cpp:3291-3296`, `:3303-3310`, `:3313-3320` | the ColorChange, Pause and Custom tags: `store_move_vertex`, a `custom_gcode_per_print_z` item, `m_options_z_corrector.set()` | `processor/moves/tags.rs` (unchanged) |
| `GCode/GCodeProcessor.hpp:688-691` `OptionsZCorrector::set` | `m_move_id = moves.size() - 1`, i.e. an index into the **split** list | `processor/moves.rs` `set_option` |
| `GCode/GCodeProcessor.hpp:693-705` `OptionsZCorrector::update`, called from `GCodeProcessor.cpp:3929` | `emplace_back(moves[*m_move_id])` with the back position and current `m_height`, then `erase(begin() + *m_move_id)`, and `print_z = position.z()` | `processor/moves.rs` `correct_options_z`: copy and erase the vertex that the stale index resolves to, which the `options_z::Log` already computes (`List::erase`) |
| `GCode/GCodeProcessor.cpp:5951-6009` `GCodeProcessor::calculate_time` | inserts the actual-speed vertices of every flush into `moves` and renumbers the queued blocks (`:6003-6008`), but not `m_move_id` | `processor/moves/actual_speed.rs` `apply` (the merged split list) |
| `GCode/GCodeProcessor.cpp:472` | `moves[block.move_id].time[mode] = block_time`, where `move_id` may now resolve one vertex later | `actual_speed::apply`: write `time` to the resolved vertex |
| `GCode/GCodeProcessor.cpp:480-573` | the actual-speed splits and `actual_feedrate` are computed from `moves[move_id]` and `moves[move_id - 1]` at the resolved position. If the resolved vertex is not Extrude/Travel/Wipe, there are no splits (`:484-488`) | `actual_speed::apply`: use the resolved current and previous vertices |

**Ares today.** `MoveRecorder` stores `moves` without splits.
`correct_options_z` erases the marker at a fresh index (`moves.rs:282-294`).
`actual_speed::apply` inserts all splits at the end, keyed by each block's
own `move_id`. `options_z::Log` (`estimate/options_z.rs`) already resolves,
in creation order including splits, which vertex Native erases and which
vertex each shifted block reads, but only `cache_eligible` uses that
result.

**Destination.** The Log's resolution becomes the single source for:
- the identity of the erased and copied vertex, which can be a split vertex
  as in `template`;
- the vertex each block writes `time` and `actual_feedrate` to, and whose
  type and position drive its splits.

`apply` then builds the split list in Native's order. Files stay below 400
LOC: `options_z.rs` is 264 LOC, `actual_speed.rs` 165 and `moves.rs` 394.
Part of `moves.rs` moves into a new `moves/options_z.rs` if the edit would
take it past 400. No new public API, and the wasm and vgcode consumers are
unchanged.

**GREEN as built.** `options_z::Log` no longer reconstructs the resolution
from kinds and positions. It records the block queue and the corrector
calls with their clocks, and `Log::replay` rebuilds Native's move list:
recorded vertices are stored, each `set()`/`update()` first runs the
`calculate_time` flushes due by its clock (`moves/actual_speed.rs` `Pass`,
`:472-573` detection and `:5962-6008` insertion and renumbering), and
`update()` copies and erases the stale index exactly as
`GCodeProcessor.hpp:693-705` does. Queued blocks at or after the erased
index are marked shifted; their `cache_eligible` follows the type of the
element they resolve to, as before. `VG1` blocks keep writing `moves[0]`.
The per-block `move_id` list (`block_moves`) in `estimate.rs` and the
`record_motions`/`tool_change`/`motion` return values it fed became unused
and were removed (`estimate.rs`, `estimate/commands.rs`, beyond the
allowlist below).

## Included and deferred

**Included:**
- `OptionsZCorrector::update` erases and copies the stale-index vertex in
  `moves`.
- `time[Normal]` is placed on the resolved vertex (`:472`).
- Splits and `actual_feedrate` use the resolved current and previous
  vertices (`:480-573`). This includes no splits when the resolved vertex
  is untimed.
- RED and GREEN on `template`, the smallest case.

**Deferred:**
- **`custom`, `mode-mismatch`, `pause`:** they have the same mechanism and
  are expected to pass with this GREEN. They become tests in the same RED
  only if COORD agrees; otherwise they are a follow-up test-only slice.
- **`same-layer`:** two events, and the row count differs by 4. That fits
  the "resolved vertex untimed means no splits" path (`:484-488`), which
  `template` does not exercise. If it is not fixed by this GREEN, it gets
  its own slice.
- **Stealth `time[1]`:** `silent_mode` is off on every fixture, and Ares
  computes no Stealth machine (`:6026`).
- **Web:** the actual speed and flow views and the ToolPosition read
  `moves`. There is no web code change, and the browser E2E is not
  re-baselined in this slice.

## Tests (RED first, separate commit)

`crates/ares-cli/tests/gcode_moves_oracle.rs` gains
`plate_custom_gcode_template_moves_match_gcode_processor_oracle`. It calls
`check_fixture` on `tests/parity/plate-custom-gcode-emission/template.3mf`
with plate `None`. The file grows by about 12 LOC, to about 244.
- It needs `ARES_PROCGCODE_BIN` (the `procgcode` oracle) and skips without
  it, like the existing two tests.
- The RED is the existing `report.mismatches == 0` assertion; there are 82
  `gcode_id` mismatches today. No tolerance change.
- The artifact is `target/gcode-moves-oracle/template/diff.csv`.

**GREEN gates:**
- the RED test, `ender3`/`ksr_fdmtest_v4` moves, and the existing processor
  M73 tests;
- directed `-E test(plate_custom_gcode_emission)`, official and seeded
  (12/12, since the G-code bytes must not change);
- fmt, `nextest -p ares-core`, wasm32 clippy, and `clippy --workspace`;
- the canonical full that COORD assigns, because this is an ares-core
  production change.

## Ways it could fail

- **Erased split vertex has no recorder entry.** The erased vertex is a
  split vertex, which `MoveRecorder` never stores. The copy must come from
  the computed split, not from the recorder list.
- **Several updates.** Each one shifts the next one's resolution. `Log`
  already applies them in order (`copies`).
- **Index 0.** A resolved vertex at index 0 would make `move_id - 1`
  underflow. This is not reached in any fixture, and upstream has no guard
  either (`:491`), so none is added.
- **G-code bytes change.** The export remap (`finish(line_map)`) must still
  see the same `gcode_id` per vertex. The directed 12/12 run guards this.
- **Performance.** The Log is already built on every pass, so the resolved
  index costs one lookup per block.

## Allowlist (needs COORD OK)

| File | Edit |
|---|---|
| `crates/ares-cli/tests/gcode_moves_oracle.rs` | RED test |
| `crates/ares-core/src/project_slice/gcode_emit/processor/moves.rs` (+ optional `moves/options_z.rs`) | `correct_options_z` |
| `.../processor/moves/actual_speed.rs` | resolved vertex in `apply` |
| `.../processor/estimate/options_z.rs` | expose the resolution |
| `docs/architecture/gcode-processor-options-z-moves.md`, `gcode-processor-options-z-corrector.md` (link) | docs |
