# Narrow-fill traversal rewrite slice

Source boundaries and decisions: ARD-0073. Follow the upstream producer
`FillConcentricInternal::fill_surface_extrusion`, point-chain ordering and
`GCode::extrude_infill` collection ownership into the existing typed Rust
fill entities/shortest-path/emission modules. The shell is temporary until
these source contracts match; do not add an Ares-owned sorting pipeline.

Confirmed owner: the concentric constructor's short-line parameter consumed
by `WallToolPaths::removeSmallLines`; use the common initialized factor .5
from ARD-0069, remove the empirical internal-factor0 branch and its private
short-path assertion. Keep downstream sorting unchanged. Activate internal
fill with the existing short-branch model, wall_loops0 and top_shell_layers0;
require complete output equality, not a path-prefix check.

Exit: independently activated complete-output native RED/GREEN, a fixture
that catches the actual narrow-fill order rather than unrelated rounding,
source-owned counterfactual, original complete failure replay, workspace
nextest/fmt/Clippy/Tier-1 gates and independent six-axis review. Preserve
separate M73/bridge/cooling differences and the unfinished all-device/domain
goal. Commit code, tests, ARD/spec/plan and roadmap evidence together.
