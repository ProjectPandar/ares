# Spec: re3D common travel boundary

Port OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1
`GCode/AvoidCrossingPerimeters.cpp:190-216` endpoint common-boundary selection
to `gcode_emit/motion/path/avoid_crossing/router/closest_lines.rs`. The existing
first-start-only selector is a temporary compatibility approximation. Replace
it with source end-first search over all start contours. No Ares-owned planner.

Include candidate membership, end candidate priority, first matching start
candidate and the existing two-intersection replacement. Defer other routing,
float-distance changes, contour construction, motion estimation and wipe logic.

Targets: GigabotX2 case-Qqf4Pr, GigabotX2 XLT case-qJQXNf, TerabotX2 case-Yl6xxq,
all1.75mm. On931691eb all3 are full-output DIVERGENT at
`${TMP_ROOT}/ares-re3d-red/replay-summary.json`. Immutable inputs copied into
`${TMP_ROOT}/ares-re3d-replay`. Existing cube10 fixture is a single minimal box; retain
printer/profile settings because they create the small final-layer inner hole.

Require3/3 target equality, fresh original CPU0 references, retained68+823
controls and extra-overhang stock/active-step controls, workspace Nextest, fmt,
default Clippy and real core+adapter WASM build. Coordinator review before
latest-main rebase, revalidation and ff-only integration. No comparison waiver.
