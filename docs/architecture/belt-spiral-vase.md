# Belt spiral-vase export

## OrcaSlicer boundary

The reference is OrcaSlicer v2.5.0 at
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. Paths below are relative to
`src/libslic3r`.

- `Print.cpp:2138–2162` validates belt-only incompatibilities. An active belt
  brim is rejected when `spiral_mode` is enabled with the generic print
  validation diagnostic `Brim is not compatible with spiral vase mode on a
  belt printer. Disable one of them.` The rejection remains before export and
  uses the existing generic return code.
- `Print.cpp:2318–2347` retains spiral-vase admission checks for copies,
  print sequence, and mixed materials. This slice remains limited to one
  printable object, one instance, one material, and by-layer printing.
- `PrintObject.cpp:1369–1391` defines an active belt brim, including its
  brim-type and width rules. A purge prism is not a model brim. The brim
  predicate is the source of the rejection above.
- `BeltPurge.cpp:47–59` gates belt purge behavior with
  `!m_config.spiral_mode.value`; this slice does not add belt purge to spiral
  vase. The admitted shape therefore remains a single-material print with no
  belt purge object.
- `BeltSliceStrategy.cpp:12–80` applies the belt rotation and the belt-floor
  lift before slicing. Spiral vase reuses the resulting ordinary contour and
  shell pipeline rather than introducing a belt-specific contour generator.
- `BeltGCode.cpp:13–19,34–47` installs belt kinematics, forces normal lift,
  and adjusts the origin for the belt transform. `GCode/BeltKinematics.cpp:17–22`
  maps logical positions to machine coordinates, while
  `GCode/BeltKinematics.hpp:38` requires all machine axis words, including
  under an identity remap.
- `GCode.cpp:5178–5199,5235–5244` places the serial spiral filter after the
  layer generator and before cooling. `GCode.cpp:6655–6671` enables that
  filter only for a single-object layer without support, after the brim and
  configured `skirt_height` checks, and only for one perimeter with no fill.
  The `skirt_height` comparison is retained even when the belt skirt generator
  emits no ordinary skirt.
- `GCode.cpp:8458–8485` emits the immediate layer transition through the
  writer before the spiral filter rewrites body-layer moves.
  `GCode/SpiralVase.cpp:89–115,143–173` measures XY extrusion length from
  formatted G-code, derives the ramp from parsed motion, and writes literal
  machine-space Z values. The belt writer must therefore remain in the path
  seen by the spiral filter.

## Rust destination

`crates/ares-core/src/project_slice/belt.rs::admit` will remove only the
blanket `spiral_mode` unsupported result. It will retain the existing
single-object, single-instance, by-layer, one-material, support, purge,
draft-shield, contoured-toolpath, rotation, and remap admission rules. Before
accepting spiral mode, it will reject an active belt brim through the existing
`SliceError::Validate` and `PrintValidationKind::Generic` channel, preserving
plate and layer context and the upstream diagnostic.

The existing ordinary vase modules remain the slicing boundary:

- `project_slice/gcode_emit/spiral_vase.rs` keeps the layer admission filter,
  including the brim, `skirt_height`, bottom-shell, single-perimeter, and
  no-fill conditions.
- `gcode_spiral_vase/project.rs::ProjectSpiralVaseRunState::process_layer`
  remains the serial post-processing boundary. Its layer transition and
  literal-Z handling will follow the upstream machine-space behavior instead
  of assuming scalar planned Z values when belt kinematics are active.
- `project_slice/gcode_emit/layers/boundary.rs` continues to pass layer
  transitions through the spiral filter before cooling.
- `project_slice/gcode_emit/motion/belt_writer.rs` and
  `project_slice/belt/kinematics.rs` remain responsible for coupled machine
  axis words. Spiral-generated layer moves must use this writer, including
  the required X/Y/Z words for identity remapping.

No new belt spiral geometry, purge planner, or independent vase pipeline is
introduced. The existing belt slice transform, shell handling, belt brim
planner, cooling order, and motion formatting remain the owners of their
upstream behavior.

## Included behavior

- Admit a single-object, single-instance, single-material, by-layer belt
  project with spiral vase mode and no belt purge.
- Produce the ordinary spiral vase ramp through the belt writer for X90 and
  X45 belt rotations.
- Preserve machine-axis remapping, with swapped-axis and identity-remap
  controls. Identity remapping must still emit the complete machine-axis
  words required by the belt writer.
- Delay spiralization according to `skirt_height`, including zero skirt loops;
  `skirt_height = 0` and `skirt_height = 4` are public witnesses.
- Reject an active outer belt brim with the generic validation result and the
  upstream diagnostic, without producing G-code.
- Keep belt purge disabled for the supported single-material spiral project.

The public parity witnesses are the X90 no-brim project, the X45 no-brim
project, X90 with `skirt_height` set to zero and four, an active-brim
rejection, and an identity-remap control. Official and seeded Native modes
must agree on the accepted output and rejection channel.

## Deferred behavior

Multi-object or multi-instance spiral printing, by-object sequencing, mixed
or multiple physical materials, supports, raft layers, draft shields,
contoured toolpaths, belt purge objects, and non-belt spiral behavior are
unchanged. Belt-specific spiral geometry beyond the existing tilted slicing
transform is deferred. Ordinary skirt emission remains disabled in belt mode;
only its configured height participates in the spiral layer admission check.
