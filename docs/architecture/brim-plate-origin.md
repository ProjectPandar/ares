# The per-object brim on plates after the first

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. Ares writes a per-object brim in global coordinates on every plate. On plate 1 (origin 0) that is invisible. On a later plate the brim lands a plate stride away from its object, and the toolpath check rejects the slice that Native completes. Found by the per-instance delete slice (`web-plater-delete-instance.md`, its cross-plate case); reproduced without it.

## Upstream boundary
- **The brim is emitted from the global origin.** `GCode::process_layer`'s `emit_brim` sets `set_origin(0., 0.)` before extruding a brim (`GCode.cpp:6053`), as the skirt does.
- **The writer makes every point plate-local.**
  - `Print::export_gcode` gives the writer the plate origin as its XY offset (`Print.cpp:4010`, `set_gcode_offset_with_imex_shift`, which is `set_gcode_offset` when the IMEX shift is zero, `GCode.hpp:264-273`).
  - `GCodeWriter` subtracts `m_x_offset`/`m_y_offset` from every written point (`GCodeWriter.cpp:1114`, and the other moves).
  - So the brim, emitted globally, is written relative to the plate like everything else.

## Ares destination
- **Where the offset lives.** Ares has no single writer offset: each emitter folds the plate origin into its `set_origin`. The skirt does so with `(-plate_origin)` (`gcode_emit/skirt.rs`, citing the same writer chain).
- **The bug.** The brim emitter (`gcode_emit/brim/emit.rs`) sets `(0, 0)` and leaves the plate origin out.
- **The fix.** The brim plan carries the plate origin, from `traversal.project.plate_origin()` as the skirt plan does, and its emitter sets `(-plate_origin)`: upstream's `set_origin(0, 0)` plus the writer offset. Nothing else in the brim changes.

## RED
**Fixture:** `brim_plate_origin.rs` under the plater session tests, derived in-test with no Native output committed. Load `combined-brim.3mf`, add a plate, move object 3 (one instance) by one plate stride onto plate 2, then slice plate 2.
- Native slicing the GUI export completes.
- Ares writes the brim at global X and fails with `ToolpathOutsidePrintableArea`.

**GREEN** is checked by the plate-2 slice equal to Native's (ordered bytes, generator line excepted), plus a paired FULL run (no PASS → FAIL).

## GREEN
- **`BrimPlan.plate_origin`** comes from `traversal.project.plate_origin()` for an owner's plan. A combined plan takes it from its template, the same plate.
- **`BrimPlan::emit`** sets `(-plate_origin)`: upstream's `set_origin(0, 0)` plus the writer's plate offset. On plate 1 the origin is 0, so its brim bytes are unchanged.
- **Result:** the plate-2 slice now matches Native through every move: brim, skirt, walls and every layer. The case still differs in the config block, where `wipe_tower_x`/`_y` echo plate 1's value. That is a separate literal fix (next commits).

## The plate's tower position in the config block
- **The bug.** The brim case's remaining difference was the config block's `; wipe_tower_x =` / `; wipe_tower_y =` line. Upstream writes `get_at(print.get_plate_index())` with three decimals, then the ordinary serialization (`GCode.cpp:8342-8346`). Ares's writer (`full_config.rs`) indexes by the plate it is given, but both callers passed 0.
- **The two callers:**
  - the state's config block (`project_slice/state.rs`);
  - its re-render after an automatic filament-map refresh (`tool_ordering/recommend.rs`). Upstream renders this block once, at export, from the final config. `ToolOrdering.cpp:1291-1303` only updates the filament map it reads, so the re-render is the same `GCode.cpp` block.
- **The fix.** Both callers pass `Project::current_plate_index()`, `Model::curr_plate_index` as plate selection sets it.
- **RED** `plater_added_plate_echoes_its_own_tower_position_like_native`: plate 2 of the cubes-on-added-plate session, compared Native-first on those lines only.
- **Result:** that case and the plate-2 brim case slice like Native.
