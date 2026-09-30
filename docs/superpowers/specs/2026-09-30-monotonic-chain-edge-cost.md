# Monotonic chain edge-cost rewrite slice

Port the actual `Fill/FillRectilinear.cpp::AntPathMatrix::operator()`
edge-cost behavior into `fill/rectilinear/path_matrix.rs`, under ARD-0041.
This replaces a Rust-only reachable contour-distance branch with the
upstream behavior for ordered monotonic regions, not an Ares pipeline feature.

Include endpoint cost arithmetic and resulting monotonic path directions.
Defer unrelated chain heuristics, G-code progress reporting and broader
printer/option completion. Existing Rust scaffolding remains a temporary
compatibility shell around the cited upstream boundary.

The regression must enter through the CLI with model/config bytes and compare
complete output against the unpatched official Orca binary, with only the
existing generator identity/time normalization. A reduced concave slab must
exercise the direction difference before the fix. KSR evidence must confirm
the formerly split layer-46 path is restored; unrelated residual differences
must remain reported, not normalized away.

Exit requires red/green, fmt/clippy, workspace nextest, Tier-1 compilation,
independent six-axis review, and one code-plus-doc commit/push.
