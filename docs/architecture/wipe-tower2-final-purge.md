# WipeTower2 final purge skip

Rewrite slice of the Type2 final-purge decision at OrcaSlicer v2.5.0-beta `345efec64`. It follows the tower-approach lift type (`wipe-tower2-tower-lift.md`).

## Problem
When the tower stops below the last print layer, Native prints no final purge. Ares always prints one. When the tower stops below the last layer, Ares raises it to the last print layer and purges there, in mid-air.

`mk4-layer-change/mk4base.3mf` plate 1 (Prusa MK4, Type2 rib tower, SEMM; the tower ends at Z 6.2, the print at Z 10):
- **Native:** after the last layer's wipe (`G1 X78.549 Y103.296 E-.24`, `;WIPE_END`, `M486 S-1`), it goes straight to `M106 S0` and the filament end G-code.
- **Ares:** prints a tower block there. It travels to the tower at Z 10.4 (`G1 X46.752 Y203.803 Z10.4`, `X39.275 Y203.803`), drops to Z 10, unretracts, then emits `; CP TOOLCHANGE START` with no `; toolchange #` and a full unload pattern.
- **Statistics:** the extra purge also shows in the footer. Filament 2's `filament used [mm]` is 2942.30 in Ares against Native's 2901.51, and the `[cm3]`, `[g]` and cost lines differ to match.

After the tower-approach lift slice, this is the only tower difference left in the file.

## Upstream boundary
- **`Print::_make_wipe_tower`**, Type2 branch (`Print.cpp:5573-5594`, inside the `WipeTower2 wipe_tower` block from `:5509`):
  - `generate_final_purge` starts `true` (`:5575`).
  - When `tool_ordering.back().wipe_tower_partitions > 0` (`:5576`), the tower reaches the last print layer. If `wipe_tower.layer_finished()`, it is lifted to `print_z + layer_height` with `set_layer(…, 0, false, true)` (`:5578-5582`); otherwise the purge stays on the current layer.
  - Otherwise `generate_final_purge = false` (`:5586-5591`), "to avoid generating purge lines in mid-air".
  - `final_purge` is `tool_change((unsigned int)(-1))` when generating, else an empty `ToolChangeResult` (`:5592-5594`).
  - `used_filament` and `number_of_toolchanges` are read after this (`:5596-5597`). A skipped purge therefore adds no filament. The unload-only `tool_change(-1)` never counts as a toolchange either way (`WipeTower2.cpp:1435-1437`).
- **The consumer** is `WipeTowerIntegration::finalize` (`GCode.cpp:2141-2151`). It emits only for Type2 with `!m_final_purge.gcode.empty()` (`:2144`). In that case it calls `change_layer(print_z)` when Z differs (`:2145-2146`), then `append_tcr2(m_final_purge, -1)` (`:2147`).

## Ares destination
- **`wipe_tower/data.rs`** (`Planned::generate`) follows the older revision. It calls `set_layer(last.print_z)` when `last.wipe_tower_partitions == 0` and always calls `tool_change(None)`.
  - **Change:** `Data::final_purge` becomes `Option<ToolChangeResult>`. It is `Some` only when `last.wipe_tower_partitions > 0`, and keeps the existing `layer_finished` lift to `print_z + normal_height`.
  - The `wipe_tower_partitions == 0` `set_layer` call is removed.
  - `used_filament` is still read after this point, so it drops the skipped purge as at `:5596`.
- **`wipe_tower/runtime.rs::finalize`** returns early when `final_purge` is `None`. This is the `!m_final_purge.gcode.empty()` gate.
- The stale `Print.cpp:3601–3690` citation in the `data.rs` module doc is updated to the beta range.

## Included and deferred
- **Included:** `Print.cpp:5573-5594` for Type2, and the `GCode.cpp:2144` empty-purge gate.
- **Not applicable:** the Type1 branch (`Print.cpp:5457-5478`, the BBL `WipeTower`). The Type1 `finalize` path (`GCode.cpp:2144` is Type2-only) already returns early in Ares (`data.type1`), and `type1/data.rs` is not touched.
- **Not in this slice:** the other `mk4-layer-change` differences (layer-change `M204`, processor `M73`/`M74`, config-footer vector lengths).

## Oracle and RED
A section oracle with no whole-output parity claim, in a new `wipe_tower2_final_purge.rs`, a `#[path]` child of `plater_tower_positions.rs`, reusing the `wipe_tower2_rib` slicing helper.

**Section definition.** The print tail: the lines from the last `; stop printing object` up to the first `;TYPE:Custom` after it (the end G-code), compared together with the `; filament used [mm]` footer line. Both fixtures label objects, so the marker is present in both.

**Fixture reuse.** `wipe_tower2_rib::slice_both` is hard-wired to `class-default.3mf`. It gains a fixture-path parameter, through a `slice_fixture(fixture, label, edits)` that `slice_both` calls with `class-default.3mf`; the existing callers are unchanged.

**Cases:**
1. **Short tower:** `mk4-layer-change/mk4base.3mf` as is. The tower ends at Z 6.2 and the print at Z 10. RED: Ares appends the purge that Native skips, and the footer `filament used [mm]` differs.
2. **Control:** `class-default.3mf` as is. The tower reaches the top (its last toolchange is on the Z 10 top layer), so Native prints the final purge there (an unload-only `; CP TOOLCHANGE START` at Z 10, after the last object stops). It must match Native now and after the slice.

If case 2 differs from Native today, the RED step stops and reports before committing.

**GREEN gate:**
- Paired FULL in the exact environment of the integration batch it is compared with, with no PASS→FAIL.
- FAIL→PASS is listed.
- The `mk4-layer-change` tail and its `filament used` line are reported.
