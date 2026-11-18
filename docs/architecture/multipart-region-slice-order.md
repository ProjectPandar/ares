# Multipart region slice order

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) `PrintApply.cpp:622–630` removes bed XY translation before composing the volume transform. `PrintObjectSlice.cpp:367–398` sends overlapping active records through clipping; for non-overlapping records it appends every active model part to its region in record order. The uncompensated first layer backup (`PrintObjectSlice.cpp:1469–1480,1561–1578`) subsequently chains these polygons using first contour points. Do not change chaining, compensation, import, or G-code tour production.

Port the two prerequisites in `project_slice/volume_bounds.rs` and `project_slice/region_slices.rs` together: retain local volume XY translation for overlap checks, then append every active model part in the simple path. Merely fixing the transform would drop the second part through Ares's existing first-part-only shortcut; remove that shortcut rather than compensating later. Leave complex clipping, bbox Z policy, and belt behavior unchanged by this slice.

Acceptance: `local-id-bfs.3mf` keeps both islands on every layer and matches strict Native/API/CLI ordered output; `nested-bfs.3mf` remains a strict matching single-part control. Use pinned official and seeded reference modes and compare the same relevant regression inventory at the exact base and candidate tip.
