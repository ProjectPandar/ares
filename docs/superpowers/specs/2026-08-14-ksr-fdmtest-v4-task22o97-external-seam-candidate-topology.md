# Task 22O.97 — external seam candidate topology

Port pinned `SeamPlacer.hpp:42-108`, `SeamPlacer.cpp:229-273,406-592,
1014-1038`, and `ExtrusionEntity.hpp:507-512` as a pure project-slice seam.

Requirements:

- extract external loops from retained source-native perimeter collections;
- recognize mixed external/overhang loops by any external constituent path;
- preserve repeated closing/join points from source `collect_points`;
- normalize clockwise polygons to counter-clockwise while preserving signed
  local angles relative to original winding;
- calculate source vertex angles with a 0.4 mm arm and retain each polygon's
  corresponding region external flow width;
- freeze exact deterministic KSR candidate/perimeter inventory and checksum;
- do not invoke the seam from `slice_project` or legacy perimeter code.

Focused tests live in a separate module. Files remain below 400 LOC and use
ordinary modules, never source-splitting macros.

Deferred: visibility/occlusion, painting, overhang/embedding, selection,
alignment, placement/clipping, runtime cursor, and O96 activation.
