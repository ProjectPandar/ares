# Preserve contour anchors on a single cubic infill line

OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/Fill/FillAdaptive.cpp:1912–1947`, clips the cubic lattice to
one surface component. The single-width T-joint hook stage requires at least
two polylines, but that restriction does not apply to the final call to
`chain_or_connect_infill`.

`Fill/FillBase.cpp:2088–2098` skips empty inputs, chains multiple paths if
connection is disabled, and otherwise delegates every nonempty input to
`connect_infill`. Its open-end pass (`:2000–2078`) can add a limited anchor
along a contour even when no other infill path is available to join.

Port this distinction in `fill/adaptive/filler.rs::fill_surface`. Return
early only for empty clipped input; keep the existing T-joint guard and
reuse `fill/connect.rs::connect_infill` for the singleton. Both Adaptive
Cubic and Support Cubic share the filler, including when generating bridge
anchors. The connector, octree, path ordering, clipping, and float
arithmetic are unchanged.

Use a synthetic recessed prism with a thin web and a through-hole. Compare
exact undirected sparse extrusion edges against Native at the first affected
layer, retaining controls with zero anchor length and disabled connection.
