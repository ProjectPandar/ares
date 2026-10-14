# Frozen input-only ZAA sparse-anchoring fixtures

Inputs only; no Native G-code or comparison output is committed.
Source: the admitted `bridge_over_infill_zero_density` positive20% cube10
fixture at main c251aa616, with top vertices z5->5.2 (height10.2mm).
Bottom-5,10x10 XY, instance placement and other ZIP members are unchanged.
The two archives differ only in project-settings flag0 versus flag1.
ZIP timestamps are frozen at2026-10-15T00:00:00.

Both use classic walls,20% Rectilinear sparse infill,0.2mm layers,5 top shell
layers, ZAA enabled,minZ0.05,minimize_perimeter_height35. Native bridge output
is on one-based layer47; its supporting sparse layer has odd object index45.
The previous10mm cube's even support index44 had no bridge-geometry witness.

| Input | SHA256 |
| --- | --- |
| flag0.3mf | 9c724601a69ae4743e4caa9594baa6f7cef131823f84a5eb035592ab3b20c219 |
| flag1.3mf | 6ab88a9a3d5ae45bfadff52e6ff0e85e0b16f8b3b761255454f40134036e0a7e |

Owning source: Fill.cpp::generate_sparse_infill_polylines_for_anchoring:
1440-1447, FillBase.cpp::_infill_direction:311-313 / FillBase.hpp:211,
PrintObject.cpp::bridge_over_infill:2735-2754/2849/3244-3250.
The test first requires public layer47 Internal Bridge XY/E equality, then
requires unchanged strict whole-output comparison. Native activity/repeats
and API/CLI raw equality are independently checked. Source-derived
CoolingBuffer.cpp:389/442-484/261-288/669-686/913 timing explains why the
same-layer bridge mismatch also produces earlier M73 and perimeter-F drift;
none of those differences is filtered from the strict comparator.
