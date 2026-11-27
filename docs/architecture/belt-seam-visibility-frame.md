# Seam visibility in the belt slice frame

## Observed difference

On a synthetic two-hole project with an X-axis belt tilt of 1°, aligned seams, and brim off, the first non-generator difference is `M73 P12 R4` in OrcaSlicer versus `M73 P11 R4` in Ares. The first motion difference is the travel to the seam of the first overhang wall: `G1 X112.325 Y10.869 Z116.652 F9000` versus `G1 X98.325 Y10.869 Z116.652 F9000`. The wall follows the same directed contour from a different start. Changing only the X remap from `rev_x` to `pos_x` reflects both coordinates about X110 and leaves the 14 mm separation. The remap does not account for the different choice of seam.

Changing aligned to rear (`seam_position=back`) or disabling belt mode separately produces byte-identical exports after normalizing only the generator line.

## Source boundary

Source reference: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. Paths are relative to `src/libslic3r`.

- `GCode/SeamPlacer.cpp:986–1027`, `compute_global_occlusion`, applies each volume matrix in object space, decimates and merges the positive and negative meshes, then applies `PrintObject::trafo_sliced` while correcting mirrored winding. It samples the resulting mesh at lines 1031–1038.
- `PrintObjectSlice.cpp:1738–1743`, `PrintObject::trafo_sliced`, starts from `trafo_centered` and calls `BeltSliceStrategy::apply_preslice_transforms`. `BeltSliceStrategy.cpp:12–31,53–75` composes the belt rotation and the lift derived from the rotated mesh and belt floor. The placement-dependent shift later applied to `Layer::print_z` (`PrintObjectSlice.cpp:1032–1041`) is not part of this transform.
- `GCode/SeamPlacer.cpp:1635–1666` builds perimeter candidates at `Layer::slice_z`; lines 1668–1681 read visibility at those 3D candidate positions. Lines 2149–2167 use the visibility mesh for aligned, aligned-back, and nearest, but not rear. `SeamComparator::is_first_better` at 1180–1221 compares these penalties only after its priority checks and rear-Y comparison. `pick_seam_point` at 1339–1350 retains the first strict minimum. There is no belt-specific comparator or rear direction to add.

## Rust destination and scope

In `crates/ares-core/src/project_slice`, `raw_intersections.rs:105–133` already intersects the tilted geometry using `BeltSlice::centered_transform`. `seam_placement.rs:67–99` builds a separate occlusion mesh with `seam_placement/mesh.rs:156–179`, which uses the flat `centered_transform` even in belt mode. Candidates still come from the belt walls and their physical `slice_z` (`seam_placement.rs:156–174`). The candidate and visibility mesh are therefore in different frames.

Use the existing per-print-object belt preslice transform (`belt.rs:187–230,254–264`) at the seam visibility mesh boundary. Obtain it from the same planned print object already used for physical layers and belt floor (`perimeters/classic/traversal.rs:138–150`). Use the existing flat centered transform when there is no belt plan. Apply the selected object transform at the existing merged-mesh step in `seam_placement/mesh.rs:221–250`, before visibility sampling. Preserve the volume transform order, decimation and sampled triangle identities, mirror winding correction, and visibility units. Do not apply the XY integer scale, instance placement, print-Z offset, or final machine remap to the visibility mesh.

This replaces a flat-frame assumption at the existing visibility input; it does not introduce a seam strategy. Use the same object frame for all existing consumers of that field without changing the mode-specific decisions. Leave perimeter geometry, candidate enumeration, score weights and tie order, cross-layer alignment, split/gap, placement, and motion emission unchanged. Seam painting, precise-seam modifiers, unsupported belt topologies, and overhang feed-rate differences are outside this slice. In particular, do not change the shared flat transform helper used independently by seam paint.

## Public parity witness

Derive the synthetic input from `tests/parity/inner-only-brim/two-holes.3mf`, not a saved export. In `3D/3dmodel.model`, scale the component's Y by 0.8 and Z by 2. Keep the existing aligned preference; enable the X-axis 1° belt, set X/Y/Z remaps to `rev_x`/`pos_z`/`pos_y`, set both layer heights to 0.3 mm, disable arc fitting, and select `no_brim`. Preserve the original skirt setting and remaining project payloads. Derive the rear and flat controls by changing one setting each.

Slice plate 1 through the public CLI and the pinned OrcaSlicer reference. Require ordered byte equality of the whole G-code export, normalizing only the generator line, for the aligned belt witness and the rear and flat controls. Do not mask progress commands, rotate loops before comparison, or accept coordinates without the rest of the toolpath. Capture the paired exports in Official and Seeded modes, and retain the existing surrounding seam and belt parity coverage. The observed coordinates diagnose the mismatch; the reference output is obtained at runtime, not committed or hard-coded.
