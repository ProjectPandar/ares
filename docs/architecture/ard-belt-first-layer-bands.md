# ARD: Classify and emit first-layer bands on a tilted belt

Source: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Citations are relative to `src/libslic3r`. This extends the single-instance
[belt slice and export decision](ard-belt-print-slice-and-export.md), replacing
its `first_layer_bands` rejection, not its other admission conditions.

## Carry the owning object's belt plane into export

`BeltTransform.cpp:89–118` derives the signed shear and the coordinate it
acts on. `PrintObject::update_slicing_parameters` (`PrintObject.cpp:4468–4499`)
stores those in `SlicingParameters` (`Slicing.hpp:117–122`). During slicing,
`PrintObjectSlice.cpp:910–921,995–1041` sets the floor intercept from the clamped
mesh lift, then applies the same placement correction as to `Layer::print_z`.
`GCode::belt_height_above_floor` (`GCode.cpp:11615–11640`) obtains that plane
from `Layer::object()` (`Layer.hpp:144`) and `PrintObject::slicing_parameters()`
(`Print.hpp:496`). It does not use `BeltFloorContext` or support offsets.

Carry the plane with `project_slice/belt.rs::BeltSlice` through the object plan
and layer export. Use the layer's placement-adjusted print Z with centered
slicing XY. Height is measured along slicing Z: subtract the plane's height
at XY and half the bead width times the absolute shear. Never evaluate against
machine axes or infer the floor from the first emitted layer. With no active
shear, retain Orca's layer-ID behavior (`GCode.hpp:991–1023`).

## Emit the consequences of the plane at the existing writer boundaries

- **Extrusion pieces.** Port `GCode::_extrude`, `belt_cut_at_band_edges` and
  `_extrude_piece` (`GCode.cpp:9226–9343`) in `gcode_emit/motion/path` and the
  extrusion writer. After loop and seam preparation, cut at each crossed
  height-band edge using the scaled-coordinate interpolation and truncation
  Orca uses. Emit pieces in traversal order, with the original role, width
  and height; do not reorder paths or add a retract at a shared cut endpoint.
  Use the maximum bead-adjusted height to classify a piece as first-layer,
  and the midpoint of its height range for its slowdown step. Keep the
  original segment lengths for small-area flow compensation
  (`GCode.cpp:10063–10082`). Apply the effective first-layer state to existing
  speed, flow, acceleration, jerk and overhang decisions
  (`9415–9458,9490–9515,9598–9618,9696+`). A zero `slow_down_layers` still
  leaves the first-layer band and its cut in place. Do not replace the slicing
  layer ID for geometry, layer templates or ordinary layer bookkeeping.
- **Travel.** `GCode.cpp:3483–3497,9373–9375` and
  `GCodeWriter.cpp:58–62,890,994,1181` select speeds at the travel destination
  in the object's frame, undoing writer origin and tool/plate offsets if
  operating on writer coordinates. Use the target bead width for the approach
  leading to a path, and zero width when no path is the target. Wire this into
  `gcode_emit/motion/path/start_travel` and the existing travel emitter, not the
  machine-axis transform.
- **Layer transition.** `GCode::belt_layer_past_first_layer_band`
  (`GCode.cpp:11575–11598`) reads `Layer::lslices_bboxes` (`Layer.hpp:172`,
  populated by `PrintObjectSlice.cpp:951–955`). It uses their min height
  without a bead width. At `GCode.cpp:6873–6900` the resulting predicate
  defers the one-time temperature, power-loss recovery and motion-setting
  transition until the whole layer is past the band. Replace only the
  predicate in `gcode_emit/layers/boundary.rs`; preserve the existing
  transition ordering and the fallback for an unknown plane.
- **Cooling.** `GCode.cpp:9582–9597,10082,10213` evaluates each emitted
  segment midpoint with its bead width. Emit a band transition marker only
  when the resulting band index crosses `close_fan_the_first_x_layers`.
  Replace the inactive-plane-only marker in `gcode_emit/motion/path.rs` with
  this classification for both active and inactive planes. Retain the cooling
  buffer's fan-off precedence and carry across layer boundaries
  (`GCode/CoolingBuffer.cpp:558–561,903–906,974–980,1065–1081`). Cooling bands
  do not introduce extrusion cuts of their own.

Brim aprons and their owner and Z overrides, skirt, support/raft, belt purge,
additional objects or instances, multiple materials, sloped or contoured
paths, arrangement and preview stay outside this change. The current
admission boundaries still reject those print modes; no specific printer or
filament name participates in classification.

## Verification

Compare the whole G-code against official and seeded Orca for the six stock
45° printer/filament combinations with brim disabled, using the same inputs
and argv and ignoring only the generator identifier. Use public CLI parity
for a synthetic 45° band crossing with `slow_down_layers=0`, replacing the old
rejection assertion, and preserve the other six belt cases unchanged. Test
behavior through the existing object-emission test grouping; use no committed
Native output. Check one-core and four-core output equality and compare
paired baseline and candidate results for belt, object emission and cooling
cases with no passing case becoming a failure. Run formatting, workspace
all-target Clippy and wasm32 checks.
