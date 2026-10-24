# Spiral perimeter hole projected split

Source port: upstream8500fcdcc `GCode.cpp::extrude_loop:5754–5775` and
`ExtrusionEntity.cpp:230–320`. Orientation excludes holes; spiral projection
includes them. See docs/architecture/spiral-hole-projected-split-spec.md.

`control.3mf` and `spiral.3mf` are complete official-Native exported projects:
stock Prusa MK4 0.4 nozzle, actual `0.20mm Standard @MK4` and
`Prusa Generic PLA @MK4`, unchanged upstream `tests/data/cube_with_hole.obj`.
`manifest.json` binds their hashes, effective settings and actual Native
normalization deltas. They contain no frozen reference G-code. CW rectangle
reuses the existing `spiral-placement-gate/cw-rectangle.3mf` input/test.

Reproduction recipe: use existing `orca_parity/presets.rs::VendorProfiles`
for stock preset inheritance flattening; load machine/process with Native
`--load-settings`, filament with `--load-filaments`, unchanged OBJ, then
`--arrange 1 --export-3mf`. For target request only process spiral_mode=1
before export. Native normalization changes wall/top/infill/retraction too;
do not describe exported archives as differing in just one key. Full original
argv/presets/inputs and streams are retained in worker1 mk4-{default,spiral}-
hole-discovery-1a57f677f evidence directories. GUI ConfigManipulation.cpp:
345–378 supplies source-constructive reachability only, not a live GUI save.

Run `orca_parity_spiral_hole_native_preflight` in both official/seeded modes
before public tests; it double-replays all three inputs at the existing
native_stream_case seam, with no added helper or private oracle. Then run
the two new public tests plus existing
`spiral_placement_gate_cw_rectangle_regression`. Complete output comparison
allows only the repository's generator line; audit raw API==CLI additionally.
Set CPU18, CARGO_BUILD_JOBS=1, ARES_PARITY_THREADS=1 and test-threads1.
This is bounded nextest selection, not authorization for FULL.
