# Pre-slice clearance for a compacted prime tower

## Upstream boundary

At OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/Print.cpp:2267–2275` calls `Print::compacted_wipe_tower_clearance_valid`
after the layered placement checks. It applies only to a by-layer print with a
prime tower (`:5189–5207`) and effective sparse skipping
(`GCode/WipeTower.cpp:53–57`: enabled, neither Smooth nor wrapping detection).
Ares will reuse those predicates and the existing `Print::wipe_tower_data`
estimate in `print/validate/estimated_tower.rs`. Keep the earlier containment
check unchanged and validate the clearance after it, in the same applied-plate
validation lifecycle, before slicing.

Port the fresh estimate path of `estimated_wipe_tower_footprint`
(`Print.cpp:1380–1425,5211–5236`): no footprint with no slots or negligible
depth, rib width equal to depth, otherwise the configured prime-tower width.
Pad by nonnegative estimated brim plus twice nozzle 0's diameter. Rotate the
local rectangle first, then add the active plate's tower XY and origin. Do not
reuse the already brim-offset printable-region polygon as the clearance hull.

`print/validate/compacted_tower.rs` owns `compacted_wipe_tower_zone` and the
per-instance `compacted_wipe_tower_clearance` verdict (`Print.cpp:1220–1350,
1360–1376,1431–1478`). Its inputs are the estimated tower polygon, applied
settings, and the selected print instances. Grow the tower by the widest
per-filament spiral reach, bounding each lift to 5 mm; use round offsets at
0.1 mm resolution and re-hull the grown tower. The rod spans X: compare Y
bounds only, with half the configured nozzle-to-rod distance on each side of
each hull and strict positive overlap. Choose rod or lid height from that
band test. Split the near distances between the tower and object using
`0.5 * (diameter_or_radius - 0.2)` (`Print.hpp:1617`), using 4 mm for the bare
nozzle (`Print.hpp:72`) and the configured clearance radius for the head body.
A near-nozzle intersection allows zero rise. A body intersection only
constrains an object taller than the nozzle cone and caps the allowed rise at
nozzle height. Test transformed model-part hulls only, with the height based
on the print object's instance-zero top (`PrintObject.cpp:161–162`). Apply the
upstream `1e-4` tolerance when comparing height to the allowed rise.

Reuse `SliceError::LayeredPrintCollision` for the `:1352–1357` diagnostic.
Ares CLI already maps this error to `-64` (192 on Linux), retaining plate,
layer height, wall count and infill density. As at `OrcaSlicer.cpp:7010–7039`,
the detailed “No sparse layers” reason is on stderr; normal CLI `result.json`
uses the existing generic layered collision message and contains no success
G-code. Do not add a new CLI classifier.

## Deferred boundaries

`Print.cpp:1499–1584,3639` checks **actual** deposited tool-change extrusions,
emitter-specific rib-offset order and maximum rise above the compacted tower
on each non-sparse layer after planning. That separate slicing-error path
stays deferred: the witnessed plate already fails before planning, and the
same estimate is not a substitute for actual extrusion geometry and heights.
Require a distinct pre-slice-pass/post-plan-fail witness before porting it.
Cached generated tower bounds on GUI revalidation (`:1388–1395`) and plater
collision polygons/height overlays (`:1441–1478`) are not part of the fresh
CLI validation lifecycle. Sparse-layer combination and tower planning and
emission remain unchanged.

## Public verification

Slice committed `tests/parity/semm-ramming/R1.3mf` through Native and Ares in
both modes. With only `wipe_tower_no_sparse_layers` enabled, require 192,
signed report code -64, the Native report fields and diagnostic, and no G-code.
The unmodified off control must retain complete ordered G-code parity with
only the generator identity normalized. A synthetic edit enabling sparse
skipping and moving the tower farther from the same objects must succeed and
emit nonempty G-code on both sides. Produce the edited 3MF bytes only at test
runtime by patching the committed input's settings. Retain commands, streams
and reports, reproduce the failure before changing production code, and prove
no pass-to-fail in the scoped inventory in each mode.
