# Sequential origin-hop writer-frame parity SPEC

## Upstream source boundary

This is a source-cited rewrite slice of OrcaSlicer `GCode.cpp`, pinned at `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `_do_export:3298–3322` sets the sequential object origin and travels to local `(0,0)`.
- `travel_to:7480–7506` converts the local endpoint before emitting writer motion.
- `point_to_gcode:8124–8128` computes `unscale(point) + m_origin - extruder_offset`.
- `set_origin:5659–5669` translates the local cursor/wipe frame.

Rust destination: existing private `crates/ares-core/src/project_slice/gcode_emit/layers/transition.rs::emit`, where the transition hop currently emits raw `origin` and caches raw `origin` as writer XY. Existing `motion/origin.rs::set_origin` already computes `state.offset = origin - extruder_offset`.

## RED contract

The formal RED target is the stock-offset P1S ByObject two-object scene. It must use the unchanged complete comparator, including M73 ordering. The RED controls are:

1. the same scene with only `extruder_offset` changed from `0x2` to `0x0`;
2. the existing A1 ByObject control;
3. existing sequential ByLayer and auto-lift adjacent controls.

The stock-offset target is expected to fail as a compound whole-stream comparison: the 2mm Y origin hop and the earlier M73 relocation are both retained. A complete comparator failure is required; no masked or partial comparison is acceptable. The zero-offset same-scene control is expected to pass and is diagnostic, not an alternate acceptance target.

## GREEN contract

Only the transition hop's emitted X/Y and writer X/Y cache may change, using existing `state.offset`. Preserve local `(0,0)` cursor state, `last_scaled_position`, origin/wipe translation, labels, lift/retract, formatting, timing, and all adjacent controls. Do not change generic writer, processor, templates, M73 logic, or introduce a helper/counter/fallback.

If GREEN leaves any M73 residual, STOP immediately: do not weaken comparison or expand the slice. GREEN acceptance is complete parity, including M73.

## Deferred behavior

M73 ownership beyond the hop consequence, broader skirt behavior, generic travel planning, multi-tool/tower behavior, and other adjacent owner boundaries are deferred. This SPEC is not an independently designed Ares pipeline.
