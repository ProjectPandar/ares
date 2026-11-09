# Type2 skirt footprint cone eligibility prerequisite

## Source boundary

This is a bounded Rust rewrite of `Print::first_layer_wipe_tower_corners`,
OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`, not a new tower,
skirt policy or geometry pipeline. It precedes the accepted receiving-minimum
slice; the observed tree is `fccd90fb53b7e6d52d8aeaa424ebc2fdd7811882`
on landed main `8ba63961f0c42cea7b798b8d7e4082a88e334297`.

| Upstream owner under `src/libslic3r/` | Responsibility |
| --- | --- |
| `GCode/WipeTower2.cpp:2124-2129`; `GCode/WipeTower2.hpp:91-97` | Retain the actual outermost first-layer brim polygon's bounding box; `get_bbx` returns that generated box when defined. |
| `Print.cpp:5563-5571` | Copy generated Type2 depth, width, height, brim width, bounding box and rib offset into `WipeTowerData`. |
| `Print.cpp:4462-4495` | Build placed first-layer occupancy points from the generated box and a stabilization-cone sample; **only Type2 with wall type cone uses the configured cone angle**, otherwise the angle is zero (`4480-4483`). |
| `GCode/WipeTower2.cpp:2151-2165` | Convert the supplied angle to radius and x-scale; an angle of zero produces radius zero. This arithmetic is not changed. |
| `Print.cpp:4204-4207` | Add the tower points as occupied area for skirt grouping, not as an object that independently emits a skirt. |

The Rust destination is
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/occupancy.rs::Data::first_layer_corners`.
Its `Data` is the existing temporary compatibility shell around upstream
`WipeTowerData`; the upstream eligibility predicate belongs at this producer of
skirt footprint points, not in a skirt offset, comparator or emitted move.

## Read-only localization

The retained min-zero control uses rectangle wall, cone angle `30`, explicit
finishing filament T0, minima `[5,0]`, prime volume `5`, non-SEMM and purge disabled.
Native, public API and CLI have identical complete first-layer brim G-code blocks.
Their modal emitted XY bounds are all `[146.800,146.800]` to
`[213.200,154.200]` mm. These are observed G-code coordinates at 0.001 mm
precision, not a claim of debugger-observed internal bounding-box equality.

The first proven differing source operand is the angle passed to cone-base
calculation: upstream `Print.cpp:4480-4483` supplies **0 degrees** for this
rectangle, while Rust `occupancy.rs` unconditionally supplies **30 degrees**.
At height 10 mm the latter adds approximately 2.679492 mm to the sampled radius.
Using the emitted box's rounded center/brim illustrates the phantom top sample
at approximately Y156.379492, beyond the actual rectangle's Y154.200 extent;
this calculation is explicitly a reconstruction, not an internal-value trace.

The emitted Native skirt has a straight top edge, while Ares includes the
phantom centerward bulge. The original full comparison remains FAIL at byte
1591, line 67, column 9: Native `G1 X113.707 Y96.434`, Ares
`G1 X113.341 Y96.434`. No byte difference is normalized or masked.
The source-operand localization and identical brim block identify a missing
footprint-eligibility port; no change to a depth/rib/width producer is required
by this prerequisite. A successful whole-output result is not yet claimed.

## Included behavior and deferred boundaries

- In the shared footprint owner, use the configured angle only when
  `!self.type1` and wall type is `ProcessWipeTowerWallType::Cone`; otherwise
  supply zero, matching the upstream Type2-plus-cone predicate. `Data::type1`
  already records the generated tower identity; no new selector is needed.
  Type1 generation retains its separate owner.
- Preserve generated bounding box, depth, width, height, brim width, rib offset,
  cone-base arithmetic, sample iteration, rotation, placement and scaling.
- Do not delete the circular samples for non-cone walls: upstream still samples
  a zero-cone-radius circle enlarged by brim width.
- Do not alter skirt hull/group/offset/seam algorithms, tool-order or automatic
  finishing selection, ramming, tower generation, Type1, carousel or meshes.
- Depth/rib/width algorithms, including their existing deferred boundaries, stay
  deferred; mixed-filament `Print.cpp:2443-2447` remains deferred as well.
- The missing non-finishing floor in `WipeTower2::save_on_last_wipe` remains a
  separate following slice. This prerequisite must not absorb that change.

## Verification contract

Keep all six existing Native-first E2Es, real geometry, matched supplied catalog,
`ares_core::slice_with_profiles` and CLI complete ordered-byte comparisons with
generator-identity-only normalization. Do not change settings to disable skirts
or the configured cone angle, and do not adjust expected offsets or references.

First compare the zero/equality and existing finishing controls after this gate;
retain each original failure if another independent difference remains. Then
port the accepted minimum floor separately. Final completion requires complete
ordered-byte PASS for all six cases in both official and seeded modes; interim
footprint equality, partial output equality or a local volume witness is not
that completion. The core-only predicate must remain platform-neutral; native
execution or WASM compilation alone does not establish browser behavior.
