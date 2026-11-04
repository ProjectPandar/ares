# Timelapse traditional insertion: bounded repair PLAN extension

Finite GREEN: seeded 6/6 and official 6/6 PASS. Native/API/CLI whole output and all existing RED assertions match; active cases have100 blocks over50 layers. Test files remain byte-identical to RED.

Reset audit: `layers.rs:292-295,369` groups entries by physical Z and invokes `boundary::append` once before tool/object visits; `boundary.rs:232` calls `advance_for_layer`, whose existing line50 reset is source-cited to `GCode.cpp:4653`. No reset-point production edit is needed.

## Why the render-only repair is insufficient

The unchanged six-case Native/API/CLI suite admits every project. Native emits100 blocks over50 physical layers (one direct and one traditional each); Ares emits150. Explicit211/223 commands already render correctly in RED but still fail complete output at line212 because Ares emits an extra traditional block for the second cube/tool visit.

The render-only trial fixes implicit current-slot temperature selection while retaining the independently failing insertion count. This is not GREEN; there is no comparison relaxation or Native-variant exemption.

The original [SPEC](timelapse-filament-context-spec.md) says: “Do not expand production scope if this argument-only change cannot repair admitted cases.” Therefore the missing lifecycle rule requires this explicit extension before implementation. No option_coverage/workspace/full exit claim follows from the failed trial.

## Added upstream rule and exact destination

Source remains OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `libslic3r/GCode.cpp:4653` initializes `has_insert_timelapse_gcode` once in `GCode::process_layer`.
- `GCode.cpp:5455-5460` requires `!has_insert_timelapse_gcode` before the I3 perimeters/infill traditional interlude and sets the flag after insertion. This lifetime spans every object/tool visit in that physical layer.
- `GCode.cpp:5527-5546` uses the same flag to suppress the layer-end fallback.
- The parser invocation rule remains `GCode.cpp:4672-4674,5167`, current logical writer filament.

Rust already has `EmitState.has_insert_timelapse_gcode`, documented as the upstream per-process_layer local. `layers/boundary.rs` resets it at the layer boundary, and `tool_transition.rs`/layer-end consumers already consult it. However `motion/layer.rs::emit_layer` uses a local interlude boolean per invocation; the existing `timelapse.rs::append_traditional` only checks its enabled argument and does not record the per-layer flag.

Proposed production boundary stays in **`gcode_emit/timelapse.rs` only**:

1. Retain the `render_for_filament(..., state.extruders.active_id())` call.
2. In `append_traditional`, return false when disabled **or the existing per-layer insertion flag is already true**.
3. After the existing successful `append_and_track` call, set the existing flag true, then return true in the current order.

No new state, helper, public API, scheduling algorithm, defensive validation or fallback. Do not edit object-entry, motion/layer, layer-end, parser, position/Z handling or physical-ID mapping. Existing layer emission remains a compatibility shell around the cited upstream GCode owner, not an Ares-designed pipeline.

## Unchanged evidence and exits

Keep the committed six-case RED suite and all existing assertions/comparisons unchanged. Require both modes to pass complete Native/API/CLI output, raw API/CLI equality, executable implicit/explicit M104 context, both logical wall tools and current/upcoming distinction. The same source assertions require two phases per physical layer, exposing incorrect reset lifetime or premature suppression.

Reuse vendor/empty controls to prove non-trigger preservation. Add no fixture or assertion workaround to hide the second-tool callback. Stop on any further mismatch before expanding scope again.

After finite GREEN: whole option_coverage both modes with fresh caches/artifacts, strict workspace/all-targets/all-features Clippy, fmt, wasm32 core/vgcode/wasm, then workspace and full parity runs.

BBL-specific admission, wipe-tower/safe-position completeness, physical-ID corrections and by-object scheduling remain deferred. The shared rule is source-owned; this extension claims finite runtime evidence only for the admitted non-BBL I3 direct/traditional recipe.
