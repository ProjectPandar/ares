# Belt G-code back-transform for preview data

Source: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
Paths in the upstream citations are relative to `src/`.

## Observable difference

Ares emits belt machine coordinates and passes the same coordinates to the
rendering-neutral viewer. `GCodeProcessorResult` does not carry a belt preview
transform or the belt Z origin. `ProcessorConversion::extend` copies the
current and phantom vertex positions directly from `MoveVertex`
(`crates/ares-vgcode/src/convert.rs:67–83,103–125`). Orca's viewer can keep those
machine coordinates or return designed coordinates: it passes an affine
transform to `LibVGCodeWrapper::convert`, which applies it to both position
streams without changing the processor's moves
(`slic3r/GUI/GCodeViewer.cpp:1419–1436,1528–1529`,
`slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:238–253,283–322`). An X-axis belt
with a decoupled 30° frame and the signed/permuted remap in
`crates/ares-cli/tests/orca_parity/belt_frame_tilt_decouple.rs:17–21` has a
nonidentity affine inverse. The designed vertex stream therefore differs from
the machine stream; comparing exported G-code alone cannot observe this gap.

The processor's `m_belt_linear` is **not** the preview transform. The static
and dynamic `GCodeProcessor::apply_config` cache only the linear part for
mass bounds (`libslic3r/GCode/GCodeProcessor.cpp:3262–3266,3302–3326`).
`add_object_mass` maps the bead-center endpoints and takes the componentwise
absolute value of the mapped half-height before expanding the bounds. It
keeps ordinary bounds and mass moments in the machine frame and merges the
second box separately (`:7644–7682,7620–7626`). `ObjectMass::transform` then
uses that second box plus the preview translation, not an affine transform of
the ordinary box (`:7629–7642`). The GUI uses the resulting box for its mass
marker's bounding-box readout (`slic3r/GUI/GCodeViewer.cpp:1522–1526`,
`slic3r/GUI/GLCanvas3D.cpp:1174–1189,1328–1333`). Ares has no equivalent box
consumer; its `ObjectMass` currently holds only instance labels and scalar
part/support volumes and masses (`crates/ares-core/src/gcode_processor/statistics.rs:11–19`).
Caching `m_belt_linear` alone would not fix any existing Ares output.

| Output | Dependency at the pinned revision |
| --- | --- |
| G-code header/footer estimates and M73 | `GCodeProcessor::run_post_process` reads time-machine statistics (`libslic3r/GCode/GCodeProcessor.cpp:1280–1326,1446–1481`), not mass bounds or display coordinates. No change. |
| `result.json` | `OrcaSlicer.cpp:245–268,555–574,7196–7205` reports slice status, diagnostics and wall-clock times. Printable-area admission uses its own machine-frame inverse, not `m_belt_linear` (`libslic3r/GCode/GCodeProcessor.cpp:2958–2975`). No change. |
| Sliced 3MF and opt-in G-code statistics | `slic3r/GUI/PartPlate.cpp:8468–8497` takes prediction from Normal time and optional statistics from `gcode_statistics_json`. The 3MF writer emits them at `libslic3r/Format/bbs_3mf.cpp:8782,9067–9072`. The serializer reads raw move Z for layers and only volume and weight from object masses (`libslic3r/GCode/GCodeStatistics.cpp:84–108,159–182`). No designed coordinates or bounds are serialized. |
| Viewer data | Designed preview positions differ; raw machine vertices, times, flow, and G-code line IDs retain their meaning. The slice exposes that difference in the existing rendering-neutral conversion boundary. |

## Included rewrite and Rust destinations

1. **`BeltTransformPipeline::build_gcode_back_transform`**
   (`libslic3r/BeltTransform.cpp:48–84`). Extend
   `crates/ares-core/src/project_slice/belt/back_transform.rs` alongside
   `kinematics.rs`. An inactive belt returns identity. Otherwise form
   `inverse(axis_remap) * inverse(machine_frame)`: undo machine tilt first,
   then the signed axis permutation. A `rev_*` output row negates the
   selected input axis and translates by **that input axis's** build-volume
   maximum, from the configured bed XY maxima and printable height. Use the
   existing `MachineFrameTransform` and remap conventions; do not add a
   second tilt calculation or apply the pre-slice mesh rotation. The
   writer-side `BeltBackTransform` is a different inverse, and the
   machine-frame-only validator in
   `crates/ares-core/src/gcode_processor/printable_area.rs` stays unchanged.
2. **Processor preview inputs.** In
   `crates/ares-core/src/project_slice/gcode_emit/processor.rs` and its
   `processor/moves/` recorder, use the effective printer config for the
   affine transform. Record the absolute angle from the belt header outside
   the config block (`libslic3r/GCode/GCodeProcessor.cpp:4474–4482`) and the
   machine-Z origin from `G92` when the header angle is nonzero
   (`:6529–6556`). Use the real machine Z for belt start-custom moves rather
   than pinning Z to first-layer height (`:7557–7580`). Carry the preview
   inputs on `crates/ares-core/src/gcode_processor.rs::GCodeProcessorResult`
   without applying the affine transform to `moves`. For the public typed
   `process_gcode` entry point the caller already supplies the effective
   `ProjectSettings`; do not infer a printer from the rounded header angle
   or add an embedded-config loader. Non-belt processing and processor
   passes that do not record moves need no preview data.
3. **Display position conversion.** Extend
   `crates/ares-vgcode/src/convert.rs::ProcessorConversion` at the existing
   `LibVGCodeWrapper::convert` seam. Preserve the raw conversion and expose
   explicit designed conversion using the processor's affine transform,
   composed with the negative machine-Z origin translation **before** the
   inverse (`slic3r/GUI/GCodeViewer.cpp:1424–1436`). The same render transform
   applies to actual vertices and the previous-position phantom vertices
   (`slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:244–253,283–322`). Keep
   f64 affine evaluation and cast the final coordinates to f32. Do not
   change segmentation, motion statistics, or the underlying machine
   vertices. The Ares pass-through remains the machine-frame path, not a
   substitute for designed rendering.

## Deferred

- The `m_belt_linear` cache, `ObjectMass::belt_box`, per-body and per-layer
  mass moments, and GUI mass markers. They require a mass-box consumer;
  do not add unused state to the processor or new fields to statistics JSON.
- The GUI caller's model-aware min-corner anchoring and designed/raw toggle
  (`slic3r/GUI/GCodeViewer.cpp:1437–1521`, `GCodeViewer.hpp:369–371,524–527`).
  This slice exposes unanchored designed coordinates for a viewer without a
  model shell. It does not switch the WASM project preview to an unanchored
  view or change camera, shell, or web UI behavior.
- Standalone file-config extraction and printer overrides
  (`libslic3r/GCode/GCodeProcessor.cpp:3294–3327,3844–3856`,
  `slic3r/GUI/Plater.cpp:18289`). The standalone core API continues to
  receive typed settings.
- Unrelated viewer layer-ID compaction, belt admission expansion, support
  geometry, and PA calibration's `BeltKinematics` `world_coordinates`.

## Verification

- Use public `process_gcode`/project-slicing and viewer-conversion APIs with
  admitted X/Y belt fixtures: coupled and decoupled tilt, a signed permutation
  with a reversed axis and nonzero bed maximum, and startup motion followed
  by `G92 Z0` with a nonzero origin. Include a Cartesian control and a
  zero-tilt or identity control. Observe the complete designed XYZ stream,
  including phantom vertices, against the pinned conversion with the same
  unanchored transform and input. Capture the reference directly from the
  pinned source; do not commit captured output or hand-fit expected values.
  Raw conversion and move timing must remain in machine space.
- Compare whole exported G-code in Official and Seeded parity on the same
  belt fixture set; a G-code-only comparison is a regression check and does
  not establish preview parity. Verify unchanged `result.json` and sliced
  metadata with the existing comparators where those outputs are exported.
- Run workspace nonparity and exact-tip static checks, and verify that
  G-code bytes and both viewer position streams are invariant across core
  counts.
