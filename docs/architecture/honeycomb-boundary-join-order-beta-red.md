# Honeycomb boundary join ordering: beta rewrite slice

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`Fill/FillBase.cpp::Fill::connect_infill:1809–1897`: open-end initialization,
DSU/consumed-state join-count simulation, half-spacing buckets, and forward
and reverse contour orders chosen only when they increase complete joins.
Rust destination: private `fill/connect` arc selection before mutation.
The length-only connector ordering is an incomplete compatibility shell,
not an independently specified Ares algorithm.

The public RED observes ordered raw XY/E Sparse extrusion lines on layer 2
(Z=0.4), with runtime Native double replay and `slice_project_plate_validated`.
Input SHA-256:
`d88cb59ce2a810545e8de2a1e5ba890227305485f4bdd5501a54bda049636f58`.
It is the existing disabled-ZAA input, not a new geometry search.
Native has 5629 extrusion segments and Ares 5633. Diagnostic undirected
comparison finds 5625 common segments; four complete Native boundary joins
are replaced by eight Ares hooks. For example, Native joins X146.332,
Y92.765–95.351 while Ares leaves a gap Y93.080–93.723.
Undirected comparison is diagnostic only: acceptance preserves ordered raw
records, with no sorting, numerical normalization, or assertion weakening.

Included: upstream arc-order scoring and selection. Deferred: independent
pattern geometry/smoothing, cooling, footer order, other layers, complete
output acceptance, and general connector parity. The earlier hook and
Hilbert witnesses must stay passing. Existing complete-output comparators
are unchanged. Pending processor/geometry production changes do not alter this Sparse
residual. External evidence retained outside the repository.

OPEN residual: the experimental join-count ordering and beta join threshold
leave the same four missing joins. Collision-touching intervals set both
endpoint trim flags before arc admission. Beta collision arithmetic matches
on the captured Ares inputs; the unresolved boundary is which boundary/infill
pairs EdgeGrid visits or whether clipped input geometry differs. The earlier
hook and Hilbert witnesses still pass. No GREEN port is retained.
