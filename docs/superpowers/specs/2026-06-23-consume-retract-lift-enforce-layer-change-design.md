# Consume Retract Lift Enforce For Layer Changes Design

## Goal

Consume OrcaSlicer `retract_lift_enforce` in Ares' existing layer-change retraction Z-hop path so the option changes concrete G-code instead of remaining metadata-only.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:390-395`: declares `RetractLiftEnforceType` variants `rletAllSurfaces`, `rletTopOnly`, `rletBottomOnly`, and `rletTopAndBottom`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:534-540`: maps serialized enum strings `"All Surfaces"`, `"Top Only"`, `"Bottom Only"`, and `"Top and Bottom"` to `RetractLiftEnforceType`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5187-5200`: defines `retract_lift_enforce` as an advanced enum option with default `rletAllSurfaces`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5622-5628`: layer changes call `retract(...)` when `retract_when_changing_layer` is active and Z will move.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7606-7634`: `GCode::retract` computes `can_lift` from `retract_lift_enforce`, the current layer index, and the last non-gap-fill extrusion role before delegating to `GCodeWriter` lift helpers.
- `OrcaSlicer/src/libslic3r/GCode.hpp:580-582`: stores `m_last_notgapfill_extrusion_role` explicitly so gap fill does not control `retract_lift_enforce`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-644`: `lazy_lift` still applies `retract_lift_above`, `retract_lift_below`, and `z_hop` after `retract_lift_enforce` allows a lift.

## Ares Destination Boundary

- Extend `crates/ares-core/src/options/layer_change_retraction.rs` so `LayerChangeRetraction` parses and stores the first single-extruder `retract_lift_enforce` value.
- Split parsing helpers currently at the bottom of `crates/ares-core/src/options/layer_change_retraction.rs` into `crates/ares-core/src/options/layer_change_retraction/parsing.rs` before adding the enum parser. The split keeps `layer_change_retraction.rs` below the 400 LOC limit while preserving the existing helper semantics.
- Update `crates/ares-core/src/gcode.rs` only at the existing layer-change retraction and print-move loop boundary so Z-hop is conditioned by the previous non-gap-fill print role.
- Move existing layer-change retract/unretract helper structs and functions from the full `crates/ares-core/src/gcode.rs` file into a focused `crates/ares-core/src/gcode_layer_change_retraction.rs` module. This split is required by the repository 400 LOC limit and must preserve behavior.
- Do not grow `crates/ares-core/src/pipeline/test_support.rs`; it is already at the 400 LOC limit. Synthetic role-sequence coverage must live in the focused layer-change G-code test module or in a new focused helper module if needed.
- Add focused runtime coverage under `crates/ares-core/src/tests/layer_change_retraction_gcode/`.

## Included Behavior

- `retract_lift_enforce` defaults to `All Surfaces`, preserving current layer-change Z-hop output.
- Accepted runtime values are the Orca enum strings:
  - `"All Surfaces"`: allow layer-change Z-hop whenever the existing `z_hop` and `retract_lift_above` / `retract_lift_below` gates allow it.
  - `"Top Only"`: allow layer-change Z-hop only when the previous non-gap-fill print role is `TopSolidInfill`.
  - `"Bottom Only"`: allow layer-change Z-hop only when leaving the first layer.
  - `"Top and Bottom"`: allow layer-change Z-hop when either the previous non-gap-fill print role is `TopSolidInfill` or the layer change is leaving the first layer.
- Scalar string values and non-empty string arrays are accepted; arrays use index `0` for this single-extruder slice.
- Array parsing validates every element as one of the accepted Orca enum strings even though this slice consumes only index `0`; this matches the existing option parsing style that rejects malformed per-extruder lists instead of silently ignoring later invalid entries.
- Invalid `retract_lift_enforce` values return `SliceError::InvalidInput` naming `retract_lift_enforce`.
- Gap-fill print moves must not become the remembered non-gap-fill role for lift enforcement, matching Orca's `m_last_notgapfill_extrusion_role` comment.
- `gcode.rs` initializes `last_non_gap_fill_print_role` to `None` before the layer loop. For every emitted `ToolpathMoveKind::Print`, update it immediately after `gcode_move_emit::move_gcode(...)` appends that move's G-code and before the move is placed in `buffered_move`, but only when `extrusion_move.role() != PrintPathRole::GapFill`. Travel moves and gap-fill print moves leave the remembered role unchanged. The next layer's `layer_change_z_hop` decision reads this state before layer-change retract/unretract work begins.
- The existing `retract_when_changing_layer`, positive `retraction_length`, `z_hop`, `retract_lift_above`, `retract_lift_below`, firmware retraction, E-axis mode, comments, fan, role-change, object-label, and pending-unretract behavior must continue to work.

## Deferred Behavior

- Ordinary travel retraction, toolchange retraction, wipe, wipe distance, minimum-travel retraction triggers, seam/scarf retraction, multi-extruder per-tool enforcement, and full Orca `GCode::retract` orchestration are deferred.
- `z_hop_types`, slope lift, spiral lift, `travel_slope`, auto overhang-dependent lift selection, and lazy XY/XYZ lift fusion remain deferred.
- `Ironing` as a top-surface lift trigger is deferred because Ares' current print path model does not emit an `Ironing` print-path role.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a concise source-cited note that `retract_lift_enforce` is now consumed in Ares' layer-change Z-hop path, including deferred adjacent Orca behavior.

## Acceptance Criteria

- A focused RED run of `cargo nextest run -p ares-core layer_change_retraction_gcode::z_hop` fails before implementation because `Top Only` still emits a layer-change lift after non-top-surface output.
- After implementation, the focused command passes.
- Tests prove `All Surfaces` preserves current default Z-hop behavior, `Top Only` suppresses non-top-surface layer-change Z-hop, `Top Only` allows layer-change Z-hop after a top-surface layer, `Bottom Only` allows only the first layer change, `Top and Bottom` allows bottom and top-surface cases, and invalid enum values name `retract_lift_enforce`.
- Tests prove scalar string parsing and non-empty string-array parsing both affect runtime output, and malformed arrays are rejected with an error naming `retract_lift_enforce`.
- Tests prove a gap-fill print move after a top-surface print move does not overwrite the remembered top-surface role before the following layer change.
- Related retraction coverage passes with `cargo nextest run -p ares-core layer_change_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and a touched Rust LOC guard.
- No new crates or dependencies are added.
- Touched Rust files remain at or below 400 LOC.
