# Spec: Task 220.129 aligned seam strings

## Observable contract

For `seam_position=aligned`, Ares groups visibility-selected seams into nearby cross-layer strings and fits their final XY positions as deterministic cubic B-splines. Each fitted position is projected onto its associated extrusion loop before splitting; internal-wall depth projection follows the selected external perimeter candidate. The first KSR outer-wall seam-start travel and extrusion must match OrcaSlicer exactly after three-decimal formatting, while the preceding inner-wall travel must be within 0.03 mm per axis. Exact inner-wall coordinates remain a later parity slice.

All positions derive from generated perimeter candidates, their mesh visibility, layer Z coordinates, flow widths, and the typed seam option. No fixture identity or reference G-code enters production.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode/SeamPlacer.cpp:742-854,1058-1103,1107-1425,1500-1628` and `src/libslic3r/Geometry/Curves.hpp:61-200` with `Geometry/Bicubic.hpp:127-213`. Include nearby-layer string discovery, overhang and embedded-point preference, score tolerance, weighted cubic B-spline fitting, sharp-angle interpolation, finalized seam positions, and final loop projection. Global penalty-ordered string ownership, exact slice-contour distance fields, seam enforcers/blockers, negative volumes, and non-aligned modes remain deferred.
