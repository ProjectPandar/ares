# Painted split: first object loop entry

## Source and comparison boundary

Upstream source is `345efec64c7b95a05ce5cf96df0dde66a158835c`. The public contract is the existing `paint_color::paint_color_split` through `native_stream_case::check` and `NativeCase::compare`. The Rust execution tree is `56885932c9ddf5181c9c239c0ff6bfe126d0e0d1`, whose production tree is unchanged from `8ba63961f0c42cea7b798b8d7e4082a88e334297`.

The complete ordered comparison remains failing. The findings below use the original input, test, helper, Native replay and generator-only comparator. They are observations, not new expected-output snapshots.

## First public difference

Official and seeded modes report the same API and CLI difference at normalized byte2781, line117, column14. Native has253478 normalized bytes and Ares253566. The entire normalized prefix through line116 is equal.

| Line | Native | Ares API and CLI |
|---|---|---|
|116|`G1 Z.6 F9000`|`G1 Z.6 F9000`|
|117|`G1 X129.27 Y129.27 Z.6`|`G1 X129.27 Y110.73 Z.6`|
|118|`G1 Z.2`|`G1 Z.2`|

This travel follows the matching skirt extrusion, object marker and wipe sequence. The following role is `Inner wall` on the first object layer. It is not a skirt-extrusion difference. The last visible wipe endpoint is the same, but equal visible commands do not establish equal internal cursor or loop inputs.

The following wall commands visit the rectangle corners with different cyclic starts. Native starts at the upper-right corner and Ares at the lower-right corner. Subsequent outer-wall starts also differ. This output observation does not establish equality of prepared loop geometry, region membership, seam candidates, cached seam data or all later paths.

Both output configurations contain `seam_position = aligned`, `spiral_mode = 0` and `staggered_inner_seams = 0`. These serialized values identify the applicable source path. They do not reveal its complete runtime inputs.

## Owning source API and Rust handoff

`GCode.cpp::GCode::extrude_loop` at8489 copies the loop and selects its start before emission. For a nonspiral perimeter, the call at8519 invokes `GCode/SeamPlacer.cpp::SeamPlacer::place_seam`.

`SeamPlacer::place_seam` at2321 finds the corresponding perimeter in the layer seam data, selects its finalized or candidate seam, projects an inner-perimeter seam onto the loop, applies inner staggering when enabled, and splits the loop at the selected point. This is the source API owning perimeter start placement consumed by the first differing travel. It is not proof that a particular scoring, cache, lookup, projection or split operation is defective.

Existing Rust counterparts are `project_slice/seam_placement.rs::apply_objects`, its `alignment` and `placement::place_layer` handoff, and `project_slice/gcode_emit/motion/perimeter.rs::emit_entity` to `loop_paths::emit`. Aligned placement is prepared before emission. The emit-time nearest branch is a separate mode and must not be substituted for aligned behavior.

The immediate public divergence is therefore an object-perimeter entry endpoint. The painted-side producer at `MultiMaterialSegmentation.cpp:2361–2437` is not demonstrated as its cause. An indirect upstream influence through prepared regions or perimeter inputs is also not excluded. No internal Native/Rust input equality or runtime call trace was obtained.

## Existing tests and limits

On the unchanged execution tree, in each mode:

- Original `paint_color_native_preflight`:1pass, exit0. Both modes finish before any Ares test leg. All original inputs, positive painted-material model activity and Native double replay checks remain intact.
- Original `paint_color_split`:1fail, exit100. Public API succeeds and CLI exits0, but both complete Native comparisons fail at the same point.
- Original `paint_color_unpainted` and `paint_color_leaves`:2passes, exit0. Their complete Native/API/CLI comparisons and raw API/CLI equality assertions pass.

For the failing split target, external inspection finds full raw API and CLI bytes equal. The original helper's later raw-equality assertion is not reached after its API comparison panic. These two facts are distinct.

There are24 retained Native command captures across preflight and public legs. Each reference/replay pair has equal complete output under the existing comparator allowance. The command/status schema does not record a Native PID, so a process identity is not invented from wrapper metadata.

No source, test, helper, fixture, comparator or production behavior changed during these executions. No private runtime probe, debugger, Native rebuild or instrumentation was used. Build checks and artifact consistency checks do not convert the failed target to parity success.

## Scope disposition

This record stops at the first divergent public endpoint and its owning source API. Its exact internal cause remains unresolved. The seam consumer is outside the painted side-producer analysis boundary. Further private seam-input analysis or a production replacement needs an exact source-owned boundary of its own. No speculative island, seam, formatter, identity or configuration patch is included.
