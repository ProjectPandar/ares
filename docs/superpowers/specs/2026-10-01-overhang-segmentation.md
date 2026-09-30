# Overhang segmentation rewrite slice

Investigate the first remaining non-progress KSR movement difference through
`libslic3r/GCode/ExtrusionProcessor.hpp::estimate_points_properties` and
`ExtrusionQualityEstimator::estimate_extrusion_quality`, as called by
`GCode.cpp::GCode::_extrude`, including `AABBTreeLines.hpp` floating
intersection traversal and `Line.hpp::line_alg::intersection` dependencies.
Rust destination is the existing
`ares-core/src/project_slice/gcode_emit/motion/overhang.rs` and its `sections`
module and `geometry::line_distance_tree::intersections_mm`. This boundary must change if input capture demonstrates
an earlier producer divergence; existing emission scaffolding is an upstream
compatibility shell, not an independently designed Ares pipeline.

Include only the proven premature intersection quantization in the XY
boundary query. Preserve integer intersection APIs and tree pruning. Defer
general 3D/scarf intersection changes, M73 timing,
later movement ordering, RNG unification, and unrelated perimeter behavior.
Fixed seeds do not establish complete deterministic parity by themselves.

Before repair, minimize a public closed-mesh project and demonstrate a RED
with the existing complete ordered byte comparator and verified effective
settings/reference identity. Native and Ares complete inputs and outputs
must be retained. Diagnostic stage captures are not permanent source pins.
Do not suppress movement differences with tolerance or output sorting.

Exit requires a sensitive public RED/GREEN, original KSR rerun and honest
residual inventory, affected consumer tests, workspace Nextest without
fail-fast, fmt/clippy, four Tier-1 builds, and independent six-axis review
with repair/re-review. Use separate test modules and Rust files below400LOC;
no include macros for source splitting. Commit code and documents together
after these finite submission gates pass. All-printer/default and all-option
coverage remain separate outstanding requirements of the global goal.
