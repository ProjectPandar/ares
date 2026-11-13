# Arachne near-self-intersection ledge

Source: OrcaSlicer 2.5.0 `Arachne/WallToolPaths.cpp:282–326` and its two
`WallToolPaths::generate` correction calls after outline simplification and
angular cleanup. Rust destination: `arachne/wall_toolpaths/outline`.

`generate.py` extrudes an independently constructed seven-point polygon to
0.8mm: 14 vertices, 24 triangles. The shallow lower ledge contains a short
5.2um-X/2um-Y step. It uses the committed synthetic
`mesh-edge-pairing/cube-control.3mf` archive only as a settings/container
template; the generator replaces the complete mesh. No user model geometry or
settings are included. ZIP timestamps, ordering and permissions are fixed;
`manifest.json` pins the template, archive and every payload hash.

Ordinary project settings select one Arachne wall, 0.2mm layers, no sparse or
solid infill, 0.001mm slice resolution, 0.5mm maximum wall resolution and
0.05mm maximum wall deviation. The slice resolution preserves the short
feature; the accepted wall-deviation setting supplies epsilon24999 and
move-distance12498. These are fixture inputs, not comparator tolerances.

Native's correction moves one vertex away from a nearby nonincident segment
in each of the four layers. First-layer vertex `(-31872,-4821132)` becomes
`(-31872,-4833630)`; later-layer vertex `(-9570,-4919696)` becomes
`(-9570,-4932194)`. The first correction call moves it and the second leaves
it unchanged. These coordinates document Native reachability, not an oracle
reconstructed by the Rust test.

The test uses `runner::native_api_cli::NativeCase`: fresh Native export,
validated public project slicing API and bare CLI, complete ordered byte
comparisons with existing generator normalization, and raw API/CLI equality.
Startup, shutdown, widths, extrusion, M73, exact time and footer remain visible.
Artifacts include the input, all outputs and `comparison.json` under
`ARES_PARITY_ARTIFACT_ROOT`. No private geometry API is exposed for testing.

Regenerate with Python3: `python3 tests/parity/arachne-near-self-intersections/generate.py`.
The focused nextest filter is `test(arachne_near_self_intersection_)`.
