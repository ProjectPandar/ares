# Arrange separate STL inputs before CLI slicing

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f7`), `OrcaSlicer.cpp:1924–2002`, marks
ordinary imports for arrangement. The `arrange` transform (`:5051–5065`)
keeps the import decision unless the user explicitly disables (`0`) or
forces (`1`) arrangement. The fresh inputs participate in the combined model's
arrangement, not the saved project's selected-plate import (`:2042–2045`,
`:5619–6100`). The slice selector still controls what to export.

`libslic3r/ModelArrange.cpp:105–129,135–201` translates a model instance
into an arrangement polygon and applies the resulting bed XY and Z rotation
to the instance. `OrcaSlicer.cpp:5957–6012` supplies the loaded print config
to `libslic3r/Arrange.cpp` for inflation, axis alignment, and packing. The CLI
applies the placement before the final optional grounding (`:6058–6098`,
`:6273–6281`) and slicing.

## Scope

- For multiple separate STL inputs to the direct `--slice` / `--outputdir`
  command, omitted `--arrange` and `--arrange 1` arrange before slicing.
  `--arrange 0` keeps the existing coordinates and behavior.
- Resolve the model and presets through the existing STL profile entry.
  Reuse the same plater arrangement job and `libslic3r` packing port used
  for project models, applying the CLI's default parameters (no search for
  arbitrary rotations, printer-derived Y-axis alignment, no GUI skirt
  clearance or extrusion-calibration region). Feed the resulting instances
  to the normal project slice lifecycle. Do not derive placement from
  the requested output plate number or change object identity and order.
- Cover single-plate results with one material and valid loaded presets.
  Deferred: preset-free multi-STL error parity, changing single-STL
  placement, other integer modes, explicit rotation/alignment and
  calibration flags, multicolor and wipe tower policy, multi-plate overflow,
  foreign formats, and import geometry transforms.

## Verification

Slice separate synthetic meshes with the same loaded presets in both
executables. Exercise the omitted and forced flags and the selected-plate
and all-plates selectors. Compare exit status, stderr, output inventory,
`result.json` (only volatile timings normalized), and all of `plate_1.gcode`
except the generator banner. Keep the fixed-coordinate cases as controls.
