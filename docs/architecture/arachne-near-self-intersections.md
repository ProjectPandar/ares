# Arachne near-self-intersection correction

## Source boundary

Port OrcaSlicer 2.5.0 release `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/Arachne/WallToolPaths.cpp:263–326`:
`createLocToLineGrid` and `fixSelfIntersections`. Its dependencies are
`utils/PolygonsPointIndex.hpp::PolygonsPointIndexSegmentLocator`,
`utils/SparseLineGrid.hpp::insert`, `utils/SparseGrid.hpp::{getNearby,
processNearby,processFromCell}`, `utils/SquareGrid.cpp::{processLineCells,
processNearby,toGridCoord,nonzeroSign}`, `utils/linearAlg2D.hpp::pointIsLeftOfLine`,
and `Line.hpp:44–71::line_alg::distance_to_squared` with the closest-point output.

The Rust destination is `ares-core::arachne::wall_toolpaths::outline::prepare`,
with private correction/grid modules under `arachne::wall_toolpaths::outline`.
The existing even-odd union is a temporary compatibility shell, not an
implementation of the upstream near-self-intersection correction. Replace that
substitution rather than retain it as a fallback or retune an epsilon.

## Non-negotiable behavior

- Epsilon below one returns without changing the polygons.
- Use scaled 2mm grid cells. Insert every original segment by polygon order and
  vertex order, including closing segments. Grid entries retain polygon/vertex
  indices: later distance checks resolve endpoints from the live polygons.
  Do not rebuild the index as vertices move.
- Port SquareGrid's inclusive fat-line cell traversal, including repeated cells,
  negative-coordinate correction, and truncating coordinate division. Nearby
  cells are traversed by increasing Y, then increasing X. Preserve the source
  multimap equivalent-key traversal order; do not sort or deduplicate candidate
  segments. Establish the pinned Native container ordering before implementing
  its Rust per-cell representation.
- Query each current vertex once to materialize its candidate list. Exclude only
  its own incident segments. Mutations during the candidate loop must not change
  the already-materialized list; candidate endpoints remain live.
- Use the source closest-point calculation, integer point conversion and squared
  distance test against `half_epsilon * half_epsilon`. Half epsilon is
  `(epsilon + 1) / 2`, move distance is `max(2, half_epsilon - 2)`.
- Direction comes from `pointIsLeftOfLine(other, segment.a, segment.b) > 0`,
  where `other` is the next vertex in the polygon being moved. Follow the source
  zero/sign branch. Truncate the vector norm to the source integer length;
  multiply before signed integer division and truncate toward zero. Apply each
  displacement immediately, in source visitation order.
- Preserve the release call chain in `WallToolPaths.cpp:494–509`: triple offset;
  outline simplify; correction; degenerate-vertex removal; angular-collinear
  removal; correction again; degenerate-vertex removal; small-area removal;
  final union. Both correction calls use `(maximum_deviation / 2) - 1`.
- The release correction function contains no Clipper union. The final
  `prepared_outline = union_(prepared_outline)` exists at line509 and stays.
  Remove the extra even-odd union substituted inside Rust's cleanup chain;
  do not remove the separately cited final union.

This slice must not silently broaden or rewrite unrelated simplify, angular
cleanup, small-area, or geometry-kernel policies. If their behavior prevents a
Native-matching result, localize that boundary rather than compensate here.

## Included and deferred

Included: the source grid/correction behavior, exact two integration positions,
removal of the falsely equivalent union substitution, and synthetic public
slicing coverage that reaches the source correction.

Deferred: beading, skeletal/Voronoi generation, wall ordering, toolpath stitch
and simplification, arc clipping, inward-wipe sampling, travel/emission,
processor timing, temperature, and configuration redesign. Core stays
platform-neutral on browser WASM, Windows, macOS and Linux; no new crate,
filesystem behavior, flags, dependencies, or fallback pipeline is needed.

## Public regression seam and verification

Use the existing public project byte/options slicing API and Native export
comparison. Create a deterministic, redistributable synthetic mesh/project with
a vertex within epsilon of a nonincident segment. Its generator must contain no
user-project geometry or settings. Establish its failure against unmodified
production code before implementing the correction. Keep normal startup,
shutdown, M73, exact time and extrusion bytes visible to the unchanged
complete-stream comparator; preserve repeatable Native outputs and API results
as verifiable artifacts. Add a nearby non-triggering control when needed to
separate the correction from unrelated slicing differences.

A synthetic mismatch is not sufficient by itself: show that the Native fixture
actually takes the correction branch and that the first changed outline vertex
belongs to this source boundary. Avoid unit tests of private internals or a new
public geometry API solely for testing. The original user project remains a
local witness, never a committed fixture or oracle reconstruction.

After the focused Native/API comparison matches, run all existing Arachne parity
guards in both reference modes, formatting, workspace all-target Clippy and the
browser-WASM compile check. Preserve failure identities rather than relax the
comparator or substitute another source version. Independently assess source
contract and test sensitivity before integration. Do not infer original-project
exact-time closure merely from fixing its first geometric witness.
