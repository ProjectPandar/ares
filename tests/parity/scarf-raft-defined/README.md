# Conditional scarf over raft with a defined seam distance

The positive and control use the same taper model and two raft layers. Both
set `wall_loops=0` over object Z `[0, 0.2]`, with `layer_height=0.2`, through
`Metadata/layer_config_ranges.xml`. Its object index is 1, the one-based
ModelObject index, not package object ID 2 (`bbs_3mf.cpp:2235–2238`).

The first object layer has solid fill but no wall loops. Later object layers
retain walls, so their seam candidates have a nonempty prior object slice
(`SeamPlacer.cpp:1683–1727`). `place_seam:2384` therefore does not read an
uninitialized first-object-layer distance. Merely disabling conditional scarf
would not avoid that read: `GCode.cpp:8519` calls `place_seam` before the
conditional checks at 8534–8541. All source references use the manifest pin.

The positive enables external conditional scarf with a 40% overhang threshold.
The control differs only in scarf type `none`. Assertions cover Native
activation, object-layer and Outer wall placement, no first-object-layer
walls, later walls, and complete Native/API/CLI output equality. No slope
count, motion-coordinate or extrusion-value golden defines the contract.

Both inputs derive from `scarf-conditional-raft/taper-raft-40.3mf`. Only the
project settings member is rewritten and the range XML member added; all
other ZIP local records, including model geometry, are copied byte-for-byte.
The range XML contains both `layer_height` and `wall_loops`;
`Slicing.cpp:258` reads the former when building the height profile.

The former first-object-layer conditional Native comparison is historical,
as described in `docs/architecture/ard-scarf-raft-unset-distance.md`. These
fixtures do not claim coverage of scarf on that undefined first-layer seam.
