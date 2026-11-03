# GCodeProcessor same-layer custom G-code moves guard

This is a test-only follow-up to the
[OptionsZCorrector moves slice](gcode-processor-options-z-moves.md). That
slice deferred the `same-layer` fixture. This slice turns it into a moves-oracle
case. No production code changes.

## Upstream boundary

The ported behavior is the same as in the moves slice:

| Upstream | Behavior | Ares destination |
|---|---|---|
| `GCode/GCodeProcessor.cpp:3313-3320` | `;CUSTOM_GCODE`: `store_move_vertex(EMoveType::Custom_GCode)` and `m_options_z_corrector.set()` | `processor/moves/tags.rs` |
| `GCode/GCodeProcessor.hpp:688-705` `OptionsZCorrector::set` / `update` | the stale `m_move_id` into the split move list; the copy and the erase on the next extrusion | `processor/moves.rs` `correct_options_z` |
| `GCode/GCodeProcessor.cpp:5951-6009` `calculate_time` | actual-speed vertices inserted on every flush; the queued blocks are renumbered, `m_move_id` is not | `processor/moves/actual_speed.rs` `apply` |
| `GCode/GCodeProcessor.cpp:472`, `:480-573` | `time` and the actual-speed splits use the vertex that `move_id` resolves to; there are no splits when it is not Extrude/Travel/Wipe (`:484-488`) | `actual_speed::apply` |

`same-layer` (`tools/chain-probe/plate-custom-gcode-emission-fixtures.mjs:39`)
has two `;CUSTOM_GCODE` items close together, at 2.15 and 2.2. It is the only
emission fixture with two OptionsZCorrector events this close. Before the
moves slice, its row count differed from Native (5564 in Ares, 5560 in the
oracle). That fits the no-split path (`:484-488`), which the four existing
cases (`template`, `custom`, `pause`, `mode-mismatch`) do not cover.

## Evidence

- **Base:** at `57fea9498`, the moves discovery against the raw
  `procgcode` oracle (`${TMP_ROOT}/ares-estimation/moves-discovery/r2/summary.txt`)
  shows 5560 oracle rows against 5564 Ares rows, and 4304 `gcode_id`
  mismatches, the first at row 1140.
- **After the GREEN:** at `683d5706f` (the same code as the approved
  `13f628158`), a release `ares slice --dump-moves` of `same-layer.3mf`,
  compared with the raw oracle under the M853 rules
  (`${TMP_ROOT}/ares-estimation/same-layer-683/`), shows 5560/5560 rows,
  0 mismatches, and all columns bit-identical.

## Included and deferred

**Included:** in `crates/ares-cli/tests/gcode_moves_oracle.rs`, the test
`plate_custom_gcode_same_layer_moves_match_gcode_processor_oracle`, which calls
`check_emission_fixture("same-layer")` like the four existing cases (+5 LOC,
269 → 274). It needs `ARES_PROCGCODE_BIN` and fails under
`ARES_REQUIRE_ORCA=1` without it, the same policy as the other cases.

**Deferred:**
- Stealth `time[1]`: `silent_mode` is obsolete (ARD-0052).
- Fixtures with more than two OptionsZCorrector events per layer.
- Web display: the views that read `moves` are unchanged.

## TDD note

The guard is added after the GREEN it protects, so there is no failing RED on
this branch. Instead, the test runs and fails on base `29f4a49c1`, a scratch run with nothing
committed, and is shown to pass at the branch head. Both results are
attached to the REVIEW-REQUEST. The test, its assertions and the M853
tolerance rules are unchanged from the four existing cases.

## Gates

Before the REVIEW-REQUEST, at the exact head:
- `gcode_moves_oracle` 8/8, with the procgcode oracle.
- fmt, plus strict clippy and wasm32 clippy.
- The new test fails on `29f4a49c1` (scratch, CPUs 16,17).
