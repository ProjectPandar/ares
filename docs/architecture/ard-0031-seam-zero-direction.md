# ARD-0031: Preserve zero seam directions

`SeamPlacer.cpp:1584-1587` delegates normalization to Eigen. Preserve its
squared-norm-positive branch, including returning an unchanged zero vector.
This is reached geometry semantics, not defensive validation or a fallback.
Retain duplicate closing vertices from upstream `collect_points`; changing
topology to avoid zero directions changes candidate and neighbor ownership.

Apply this correction to aligned inner-seam placement. The runtime-nearest
control already passes and its separate implementation is unchanged.
Do not broaden this fix to unrelated 3D vectors. Complete public slicing
output is the regression boundary; no source-token assertions are needed.
