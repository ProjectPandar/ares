# Sparse-fill close-end joining — beta rewrite slice A

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`FillBase.cpp::Fill::fill_surface_extrusion:213–215` and
`FillBase.cpp::join_close_ends:3001–3117`.

## Required boundary

Port the complete source algorithm: endpoint spatial cells, prioritized gap
ordering, dominant-axis trailing classification, disjoint-set cycle rejection,
line/join crossing rejection, partner assignment and oriented chain walking.
Use the source maximum gap `scale_(2 * spacing)` and `can_reverse` policy.
Call in shared fill extrusion materialization only for the non-Arachne branch,
`erInternalInfill && !no_sort`. Do not special-case honeycomb. Rust's current
direct path materialization is a temporary compatibility shell missing this
upstream post-connector operation; it must gain this operation at the same
semantic boundary, preserving collection and flow boundaries.

## Acceptance

Reapply the existing honeycomb ordered layer-two sparse-motion RED,
demonstrate RED before production changes, then require
exact scoped ordered runtime-Native parity. Retain the complete-output
comparator unchanged. Check earlier hook, Hilbert and object-ID witnesses;
run rustfmt and all-target Clippy. No Native G-code or diagnostic binary is
committed.

## Deferred

Crosshatch contour-order selection in `FillBase.cpp:1880–1897` is separate
slice B. Do not change graph touching, trim flags, sorted thresholds, hook
arithmetic, cooling or travel emission. Do not claim whole-output parity from
undirected segment diagnostics. Arachne variable-width paths, non-sparse roles,
explicit no-sort collections, and calibration order retain their source-owned
behavior and do not enter this join operation.
